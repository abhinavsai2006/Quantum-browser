import zipfile, io, struct, os, sys

def deoptimize_jar(data):
    eocd_idx = data.find(b'PK\x05\x06')
    if eocd_idx == -1: return data
    cd_len = struct.unpack('<I', data[:4])[0]
    cd_part = data[4:eocd_idx]
    eocd_part = bytearray(data[eocd_idx:eocd_idx+22])
    local_part = data[eocd_idx+22:]
    struct.pack_into('<I', eocd_part, 16, len(local_part))

    cd_entries = bytearray(cd_part)
    idx = 0
    while idx < len(cd_entries):
        if cd_entries[idx:idx+4] == b'PK\x01\x02':
            orig_off = struct.unpack('<I', cd_entries[idx+42:idx+46])[0]
            struct.pack_into('<I', cd_entries, idx+42, orig_off - (eocd_idx + 22))
            name_len = struct.unpack('<H', cd_entries[idx+28:idx+30])[0]
            extra_len = struct.unpack('<H', cd_entries[idx+30:idx+32])[0]
            comm_len = struct.unpack('<H', cd_entries[idx+32:idx+34])[0]
            idx += 46 + name_len + extra_len + comm_len
        else: break
    return local_part + cd_entries + eocd_part

def optimize_jar(std_zip_data, first_4_bytes=None):
    eocd_idx = std_zip_data.rfind(b'PK\x05\x06')
    cd_offset = struct.unpack('<I', std_zip_data[eocd_idx+16:eocd_idx+20])[0]
    local_part = std_zip_data[:cd_offset]
    cd_part = bytearray(std_zip_data[cd_offset:eocd_idx])
    eocd_part = bytearray(std_zip_data[eocd_idx:eocd_idx+22])
    
    shift = 4 + len(cd_part) + len(eocd_part)
    idx = 0
    while idx < len(cd_part):
        if cd_part[idx:idx+4] == b'PK\x01\x02':
            orig_off = struct.unpack('<I', cd_part[idx+42:idx+46])[0]
            struct.pack_into('<I', cd_part, idx+42, orig_off + shift)
            name_len = struct.unpack('<H', cd_part[idx+28:idx+30])[0]
            extra_len = struct.unpack('<H', cd_part[idx+30:idx+32])[0]
            comm_len = struct.unpack('<H', cd_part[idx+32:idx+34])[0]
            idx += 46 + name_len + extra_len + comm_len
        else: break
    
    struct.pack_into('<I', eocd_part, 16, 4)
    if first_4_bytes is None:
        cd_len = len(cd_part) + len(eocd_part)
        first_4_bytes = struct.pack('<I', cd_len)
        
    return first_4_bytes + cd_part + eocd_part + local_part

def process_omni(omni_path):
    print(f"Processing {omni_path}...")
    with open(omni_path, "rb") as f:
        orig_raw = f.read()

    hdr4 = orig_raw[:4]
    std_data = deoptimize_jar(orig_raw)
    
    src_zf = zipfile.ZipFile(io.BytesIO(std_data), "r")
    
    out_buf = io.BytesIO()
    dst_zf = zipfile.ZipFile(out_buf, "w", compression=zipfile.ZIP_DEFLATED)
    
    # 1. New brand.ftl
    new_brand_ftl = """-brand-shorter-name = Qaulium
-brand-short-name = Qaulium
-brand-shortcut-name = Qaulium
-brand-full-name = Qaulium Quantum Browser
-brand-product-name = Qaulium Quantum Browser
-vendor-short-name = Qaulium
trademarkInfo = Qaulium Quantum Browser. Real Gecko Web Engine.
"""

    # 2. Modified protectionsPanel.ftl
    orig_prot = src_zf.read("localization/en-US/browser/protectionsPanel.ftl").decode("utf-8", "ignore")
    mod_prot = orig_prot.replace(
        "Enhanced Tracking Protection is ON for this site",
        "Qaulium Privacy Shield is ACTIVE for this site"
    ).replace(
        "Enhanced Tracking Protection is OFF for this site",
        "Qaulium Privacy Shield is PAUSED for this site"
    ).replace(
        "Enhanced Tracking Protection",
        "Qaulium Privacy Shield"
    ).replace(
        "Protection settings",
        "Qaulium Privacy Settings"
    ).replace(
        "Protections dashboard",
        "Qaulium Security Dashboard"
    ).replace(
        "No trackers known to { -brand-short-name } were detected on this page.",
        "No tracking attempts detected on this page by Qaulium."
    ).replace(
        "Manage protection settings",
        "Open Qaulium Privacy Center"
    )

    # 3. Modified siteProtections.ftl
    orig_site = src_zf.read("localization/en-US/browser/siteProtections.ftl").decode("utf-8", "ignore")
    mod_site = orig_site.replace(
        "Protections for { $host }",
        "Qaulium Privacy Shield: { $host }"
    ).replace(
        "No trackers known to { -brand-short-name } were detected on this page.",
        "No tracking attempts detected on this page by Qaulium."
    )

    # 4. Modified appmenu.ftl
    orig_appmenu = src_zf.read("localization/en-US/browser/appmenu.ftl").decode("utf-8", "ignore")
    mod_appmenu = orig_appmenu.replace("Qualium", "Qaulium").replace("Firefox", "Qaulium")
    mod_appmenu = mod_appmenu.replace("About { -brand-shorter-name }", "About Qaulium")
    mod_appmenu = mod_appmenu.replace("Quit { -brand-shorter-name }", "Quit Qaulium")
    mod_appmenu = mod_appmenu.replace(".label = Help", ".label = About Qaulium")
    mod_appmenu = mod_appmenu.replace(".title = Help", ".title = About Qaulium")
    mod_appmenu = mod_appmenu.replace("Exit", "Quit Qaulium")
    mod_appmenu = mod_appmenu.replace("Report broken site", "Site Diagnostics")

    # 5. Modified browser.xhtml: wire settings and dashboard buttons to Qaulium Settings and hide search selector
    orig_xhtml = src_zf.read("chrome/browser/content/browser/browser.xhtml").decode("utf-8", "ignore")
    
    # Strip any existing or duplicate hidden="true" on urlbar-search-button and private-browsing-indicator
    import re
    orig_xhtml = re.sub(r'id="urlbar-search-button"(\s+hidden="true")*', 'id="urlbar-search-button"', orig_xhtml)
    orig_xhtml = re.sub(r'id="private-browsing-indicator-with-label"(\s+hidden="true")*', 'id="private-browsing-indicator-with-label"', orig_xhtml)
    orig_xhtml = re.sub(r'id="appMenu-unified-extensions-button"(\s+style="display:none!important;")*', 'id="appMenu-unified-extensions-button"', orig_xhtml)
    while 'hidden="true" hidden="true"' in orig_xhtml:
        orig_xhtml = orig_xhtml.replace('hidden="true" hidden="true"', 'hidden="true"')

    # Wire Commands so Bookmarks, History, Downloads NEVER open modal dialogs
    mod_xhtml = orig_xhtml.replace(
        '<command id="Browser:ShowAllBookmarks"/>',
        '<command id="Browser:ShowAllBookmarks" oncommand="openTrustedLinkIn(\'qualium://bookmarks\', \'tab\')"/>'
    ).replace(
        '<command id="Browser:ShowAllHistory"/>',
        '<command id="Browser:ShowAllHistory" oncommand="openTrustedLinkIn(\'qualium://history\', \'tab\')"/>'
    ).replace(
        '<command id="Tools:Downloads" />',
        '<command id="Tools:Downloads" oncommand="openTrustedLinkIn(\'qualium://downloads\', \'tab\')"/>'
    ).replace(
        '<command id="Tools:Addons" />',
        '<command id="Tools:Addons" oncommand="openTrustedLinkIn(\'qualium://extensions\', \'tab\')"/>'
    ).replace(
        'id="appMenu-extensions-themes-button"\n                     class="subviewbutton"\n                     data-l10n-id="appmenuitem-extensions-and-themes"\n                     key="key_openAddons"\n                     command="Tools:Addons"\n                     />',
        'id="appMenu-extensions-themes-button"\n                     class="subviewbutton"\n                     data-l10n-id="appmenuitem-extensions-and-themes"\n                     key="key_openAddons"\n                     oncommand="openTrustedLinkIn(\'qualium://extensions\', \'tab\')"\n                     />'
    ).replace(
        'id="appMenu-bookmarks-button"\n                     class="subviewbutton subviewbutton-nav"\n                     data-l10n-id="library-bookmarks-menu"\n                     closemenu="none"\n                     />',
        'id="appMenu-bookmarks-button"\n                     class="subviewbutton"\n                     data-l10n-id="library-bookmarks-menu"\n                     oncommand="openTrustedLinkIn(\'qualium://bookmarks\', \'tab\')"\n                     />'
    ).replace(
        'id="appMenu-history-button"\n                     class="subviewbutton subviewbutton-nav"\n                     data-l10n-id="appmenuitem-history"\n                     closemenu="none"\n                     />',
        'id="appMenu-history-button"\n                     class="subviewbutton"\n                     data-l10n-id="appmenuitem-history"\n                     oncommand="openTrustedLinkIn(\'qualium://history\', \'tab\')"\n                     />'
    ).replace(
        'id="appMenu-downloads-button"\n                     class="subviewbutton"\n                     data-l10n-id="appmenuitem-downloads"\n                     key="key_openDownloads"\n                     command="Tools:Downloads"/>',
        'id="appMenu-downloads-button"\n                     class="subviewbutton"\n                     data-l10n-id="appmenuitem-downloads"\n                     key="key_openDownloads"\n                     oncommand="openTrustedLinkIn(\'qualium://downloads\', \'tab\')"/>'
    ).replace(
        'id="appMenu-passwords-button"\n                     class="subviewbutton"\n                     data-l10n-id="appmenuitem-passwords"\n                     />',
        'id="appMenu-passwords-button"\n                     class="subviewbutton"\n                     data-l10n-id="appmenuitem-passwords"\n                     oncommand="openTrustedLinkIn(\'qualium://passwords\', \'tab\')"\n                     />'
    ).replace(
        'id="appMenu-settings-button"\n                     class="subviewbutton"\n                     data-l10n-id="appmenuitem-settings"\n                     />',
        'id="appMenu-settings-button"\n                     class="subviewbutton"\n                     data-l10n-id="appmenuitem-settings"\n                     oncommand="openTrustedLinkIn(\'qualium://settings\', \'tab\')"\n                     />'
    ).replace(
        'id="appMenu-help-button2"\n                     class="subviewbutton subviewbutton-nav"\n                     data-l10n-id="appmenuitem-help"\n                     closemenu="none"\n                     />',
        'id="appMenu-help-button2"\n                     class="subviewbutton"\n                     label="About Qaulium"\n                     data-l10n-id="appmenuitem-help"\n                     oncommand="openTrustedLinkIn(\'qualium://about\', \'tab\')"\n                     />'
    ).replace(
        'oncommand="openPreferences()"',
        'oncommand="openTrustedLinkIn(\'qualium://settings\', \'tab\')"'
    ).replace(
        'oncommand="openProtectionsDashboard()"',
        'oncommand="openTrustedLinkIn(\'qualium://privacy\', \'tab\')"'
    ).replace(
        'id="urlbar-search-button"',
        'id="urlbar-search-button" hidden="true"'
    ).replace(
        '<script src="chrome://browser/content/browser-main.js"></script>',
        '<script src="chrome://browser/content/browser-main.js"></script>\n  <script src="chrome://qualium/content/qualium-routes.js"></script>\n  <script src="chrome://qualium/content/favicon-service.js"></script>\n  <script src="chrome://qualium/content/favicon-bridge.js"></script>'
    )
    if 'chrome://qualium/content/qualium-tabs.css' not in mod_xhtml:
        mod_xhtml = mod_xhtml.replace(
            '<link rel="stylesheet" href="chrome://browser/skin/" />',
            '<link rel="stylesheet" href="chrome://browser/skin/" />\n  <link rel="stylesheet" href="chrome://qualium/content/qualium-tabs.css" />'
        )

    # Physically remove the gap elements between Extensions and Settings
    ext_idx = mod_xhtml.find('id="appMenu-extensions-themes-button"')
    if ext_idx != -1:
        end_ext = mod_xhtml.find('/>', ext_idx) + 2
        settings_idx = mod_xhtml.find('<toolbarbutton id="appMenu-settings-button"')
        if settings_idx != -1:
            mod_xhtml = mod_xhtml[:end_ext] + "\n      <toolbarseparator/>\n      " + mod_xhtml[settings_idx:]

    # Physically remove more-tools and report-broken-site between Settings and Help
    settings_btn_pos = mod_xhtml.find('id="appMenu-settings-button"')
    if settings_btn_pos != -1:
        end_settings = mod_xhtml.find('/>', settings_btn_pos) + 2
        help_btn_pos = mod_xhtml.find('<toolbarbutton id="appMenu-help-button2"')
        if help_btn_pos != -1:
            mod_xhtml = mod_xhtml[:end_settings] + "\n      " + mod_xhtml[help_btn_pos:]

    while 'hidden="true" hidden="true"' in mod_xhtml:
        mod_xhtml = mod_xhtml.replace('hidden="true" hidden="true"', 'hidden="true"')

    # Verify XML validity strictly with expat
    import xml.parsers.expat
    _p = xml.parsers.expat.ParserCreate()
    try:
        _p.Parse(mod_xhtml)
        print("  [OK] Validated browser.xhtml XML syntax successfully (0 duplicate attributes)")
    except Exception as e:
        print(f"  [ERROR] browser.xhtml XML parse failed: {e}")
        raise e

    # 6. Modified chrome/chrome.manifest inside omni.ja
    orig_manifest = src_zf.read("chrome/chrome.manifest").decode("utf-8", "ignore")
    qualium_manifest_entries = """
content qualium browser/content/qualium/ contentaccessible=yes
skin qualium classic/1.0 browser/skin/classic/qualium/
content qaulium browser/content/qualium/ contentaccessible=yes
skin qaulium classic/1.0 browser/skin/classic/qualium/
override chrome://browser/content/newtab/newtab.xhtml chrome://qualium/content/newtab.xhtml
override chrome://browser/content/preferences/preferences.xhtml chrome://qualium/content/settings.xhtml
override chrome://browser/content/preferences/preferences.xul chrome://qualium/content/settings.xhtml
override chrome://browser/content/browser/aboutPrivateBrowsing.html chrome://qualium/content/newtab.xhtml
override chrome://browser/content/browser/aboutPrivateBrowsing.xhtml chrome://qualium/content/newtab.xhtml
override chrome://browser/content/browser/protections.html chrome://qualium/content/dashboard.xhtml
override chrome://browser/content/browser/protections.xhtml chrome://qualium/content/dashboard.xhtml
override chrome://browser/content/downloads/contentAreaDownloadsView.xhtml chrome://qualium/content/downloads.xhtml
override chrome://browser/content/places/places.xhtml chrome://qualium/content/bookmarks.xhtml
override chrome://browser/content/places/history.xhtml chrome://qualium/content/history.xhtml
override chrome://browser/content/aboutwelcome/aboutwelcome.html chrome://qualium/content/onboarding.xhtml
override chrome://browser/content/aboutwelcome/aboutwelcome.xhtml chrome://qualium/content/onboarding.xhtml
override chrome://mozapps/content/extensions/aboutaddons.html chrome://qualium/content/extensions.xhtml
override chrome://mozapps/content/extensions/extensions.xhtml chrome://qualium/content/extensions.xhtml
override chrome://browser/content/aboutlogins/aboutLogins.html chrome://qualium/content/passwords.xhtml
override chrome://browser/content/aboutDialog.xhtml chrome://qualium/content/about.xhtml
override chrome://branding/content/icon32.png chrome://qualium/skin/qualium-shield.svg
override chrome://branding/content/icon16.png chrome://qualium/skin/qualium-shield.svg
override chrome://global/skin/icons/defaultFavicon.svg chrome://qualium/skin/qualium-shield.svg
override chrome://browser/skin/tabbrowser/loading.svg chrome://qualium/skin/qualium-spinner.svg
override chrome://browser/skin/tabbrowser/loading-burst.svg chrome://qualium/skin/loading-burst.svg
override chrome://global/skin/icons/loading.svg chrome://qualium/skin/qualium-spinner.svg
"""
    clean_lines = []
    for line in orig_manifest.splitlines():
        if any(token in line for token in ["qualium", "qaulium", "loading.svg", "loading-burst.svg", "defaultFavicon.svg", "icon32.png", "icon16.png"]):
            continue
        clean_lines.append(line)
    mod_manifest = "\n".join(clean_lines).rstrip() + "\n" + qualium_manifest_entries.strip() + "\n"

    # 7. Patch modules/UrlbarInput.sys.mjs for clean qualium:// omnibox display and routing
    orig_urlbar = src_zf.read("modules/UrlbarInput.sys.mjs").decode("utf-8", "ignore")
    urlbar_setval_target = "let originalUrl = lazy.ReaderMode.getOriginalUrlObjectForDisplay(val);\n    if (originalUrl) {\n      val = originalUrl.displaySpec;\n    }"
    urlbar_setval_replacement = """let originalUrl = lazy.ReaderMode.getOriginalUrlObjectForDisplay(val);
    if (originalUrl) {
      val = originalUrl.displaySpec;
    }
    // Qualium Protocol: Map internal Gecko chrome implementation resources to user-facing qualium:// URLs
    if (val && typeof val === "string") {
      if (this.window?.QualiumRouteRegistry) {
        val = this.window.QualiumRouteRegistry.internalToPublic(val);
      } else {
        let v = val.trim();
        if (v.startsWith("chrome://qualium/content/newtab.xhtml") || v === "about:newtab" || v === "about:home" || v === "about:privatebrowsing" || v === "chrome://browser/content/blanktab.html") {
          val = "qualium://newtab";
        } else if (v.startsWith("chrome://qualium/content/settings.xhtml#about")) {
          val = "qualium://about";
        } else if (v.startsWith("chrome://qualium/content/settings.xhtml") || v === "about:preferences") {
          val = "qualium://settings";
        } else if (v.startsWith("chrome://qualium/content/dashboard.xhtml") || v === "about:protections") {
          val = "qualium://privacy";
        } else if (v.startsWith("chrome://qualium/content/downloads.xhtml") || v === "about:downloads") {
          val = "qualium://downloads";
        } else if (v.startsWith("chrome://qualium/content/bookmarks.xhtml") || v.includes("places/places.xhtml")) {
          val = "qualium://bookmarks";
        } else if (v.startsWith("chrome://qualium/content/history.xhtml") || v.includes("places/history.xhtml")) {
          val = "qualium://history";
        } else if (v === "about:logins") {
          val = "qualium://passwords";
        } else if (v === "about:addons") {
          val = "qualium://extensions";
        } else if (v.startsWith("chrome://qualium/content/onboarding.xhtml") || v === "about:welcome") {
          val = "qualium://welcome";
        } else if (v.startsWith("chrome://qualium/content/error.xhtml")) {
          let m = v.match(/route=([^&]+)/);
          val = m ? decodeURIComponent(m[1]) : "qualium://error";
        }
      }
    }
    if (untrimmedValue && typeof untrimmedValue === "string") {
      if (this.window?.QualiumRouteRegistry) {
        untrimmedValue = this.window.QualiumRouteRegistry.internalToPublic(untrimmedValue);
      } else if (untrimmedValue.includes("chrome://qualium/content/")) {
        if (untrimmedValue.includes("newtab.xhtml")) untrimmedValue = "qualium://newtab";
        else if (untrimmedValue.includes("settings.xhtml#about")) untrimmedValue = "qualium://about";
        else if (untrimmedValue.includes("settings.xhtml")) untrimmedValue = "qualium://settings";
        else if (untrimmedValue.includes("dashboard.xhtml")) untrimmedValue = "qualium://privacy";
        else if (untrimmedValue.includes("downloads.xhtml")) untrimmedValue = "qualium://downloads";
        else if (untrimmedValue.includes("bookmarks.xhtml")) untrimmedValue = "qualium://bookmarks";
        else if (untrimmedValue.includes("history.xhtml")) untrimmedValue = "qualium://history";
        else if (untrimmedValue.includes("error.xhtml")) {
          let m = untrimmedValue.match(/route=([^&]+)/);
          untrimmedValue = m ? decodeURIComponent(m[1]) : "qualium://error";
        }
      }
    }"""
    mod_urlbar = orig_urlbar.replace(urlbar_setval_target, urlbar_setval_replacement)

    urlbar_load_target = """  _loadURL(
    url,
    event,
    openUILinkWhere,
    params,
    resultDetails = null,
    browser = this.window.gBrowser.selectedBrowser
  ) {"""
    urlbar_load_replacement = """  _loadURL(
    url,
    event,
    openUILinkWhere,
    params,
    resultDetails = null,
    browser = this.window.gBrowser.selectedBrowser
  ) {
    if (url && typeof url === "string" && (url.startsWith("qualium://") || url.startsWith("qaulium://"))) {
      let origQualiumUrl = url;
      if (this.window?.QualiumRouteRegistry) {
        url = this.window.QualiumRouteRegistry.publicToInternal(url);
      } else {
        const _qMap = {
          "qualium://newtab": "chrome://qualium/content/newtab.xhtml",
          "qualium://privacy": "chrome://qualium/content/dashboard.xhtml",
          "qualium://security": "chrome://qualium/content/dashboard.xhtml",
          "qualium://settings": "chrome://qualium/content/settings.xhtml",
          "qualium://downloads": "chrome://qualium/content/downloads.xhtml",
          "qualium://bookmarks": "chrome://qualium/content/bookmarks.xhtml",
          "qualium://history": "chrome://qualium/content/history.xhtml",
          "qualium://passwords": "about:logins",
          "qualium://extensions": "about:addons",
          "qualium://welcome": "chrome://qualium/content/onboarding.xhtml",
          "qualium://about": "chrome://qualium/content/settings.xhtml#about",
          "qualium://error": "chrome://qualium/content/error.xhtml",
        };
        let lower = origQualiumUrl.toLowerCase().replace("qaulium://", "qualium://");
        let base = lower.split("?")[0].split("#")[0];
        if (_qMap[base]) {
          url = _qMap[base] + lower.slice(base.length);
        } else {
          url = "chrome://qualium/content/error.xhtml?route=" + encodeURIComponent(origQualiumUrl);
        }
      }
      params = Object.assign({}, params);
      params.triggeringPrincipal = lazy.Services.scriptSecurityManager.getSystemPrincipal();
      if (openUILinkWhere == "current") {
        this.value = origQualiumUrl;
        this._untrimmedValue = origQualiumUrl;
        browser.userTypedValue = origQualiumUrl;
      }
    }"""
    mod_urlbar = mod_urlbar.replace(urlbar_load_target, urlbar_load_replacement)

    # 8. Patch modules/URILoadingHelper.sys.mjs for universal internal routing
    orig_helper = src_zf.read("modules/URILoadingHelper.sys.mjs").decode("utf-8", "ignore")
    helper_target = """  openLinkIn(window, url, where, params) {
    if (!where || !url) {
      return;
    }"""
    if "origQualiumUrl" in orig_helper:
        start_idx = orig_helper.find("  openLinkIn(window, url, where, params) {")
        marker = "params.triggeringPrincipal = lazy.Services.scriptSecurityManager.getSystemPrincipal();\n    }"
        end_idx = orig_helper.rfind(marker)
        if start_idx != -1 and end_idx != -1:
            orig_helper = orig_helper[:start_idx] + helper_target + orig_helper[end_idx + len(marker):]

    helper_replacement = """  openLinkIn(window, url, where, params) {
    if (!where || !url) {
      return;
    }
    if (typeof url === "string" && (url.startsWith("qualium://") || url.startsWith("qaulium://"))) {
      let origQualiumUrl = url;
      if (window?.QualiumRouteRegistry) {
        url = window.QualiumRouteRegistry.publicToInternal(url);
      } else {
        const _qMap = {
          "qualium://newtab": "chrome://qualium/content/newtab.xhtml",
          "qualium://privacy": "chrome://qualium/content/dashboard.xhtml",
          "qualium://security": "chrome://qualium/content/dashboard.xhtml",
          "qualium://settings": "chrome://qualium/content/settings.xhtml",
          "qualium://downloads": "chrome://qualium/content/downloads.xhtml",
          "qualium://bookmarks": "chrome://qualium/content/bookmarks.xhtml",
          "qualium://history": "chrome://qualium/content/history.xhtml",
          "qualium://passwords": "about:logins",
          "qualium://extensions": "about:addons",
          "qualium://welcome": "chrome://qualium/content/onboarding.xhtml",
          "qualium://about": "chrome://qualium/content/settings.xhtml#about",
          "qualium://error": "chrome://qualium/content/error.xhtml",
        };
        let lower = origQualiumUrl.toLowerCase().replace("qaulium://", "qualium://");
        let base = lower.split("?")[0].split("#")[0];
        if (_qMap[base]) {
          url = _qMap[base] + lower.slice(base.length);
        } else {
          url = "chrome://qualium/content/error.xhtml?route=" + encodeURIComponent(origQualiumUrl);
        }
      }
      params = Object.assign({}, params);
      params.triggeringPrincipal = lazy.Services.scriptSecurityManager.getSystemPrincipal();
    }"""
    mod_helper = orig_helper.replace(helper_target, helper_replacement)

    # 9. Patch chrome/browser/content/browser/browser.js to declare qualium routes as initial pages
    orig_browser_js = src_zf.read("chrome/browser/content/browser/browser.js").decode("utf-8", "ignore")
    browser_js_target = """var gInitialPages = [
  "about:blank",
  "about:home",
  "about:firefoxview",
  "about:newtab",
  "about:privatebrowsing",
  "about:sessionrestore",
  "about:welcome",
  "about:welcomeback",
  "chrome://browser/content/blanktab.html",
];"""
    browser_js_replacement = """var gInitialPages = [
  "about:blank",
  "about:home",
  "about:firefoxview",
  "about:newtab",
  "about:privatebrowsing",
  "about:sessionrestore",
  "about:welcome",
  "about:welcomeback",
  "chrome://browser/content/blanktab.html",
  "chrome://qualium/content/newtab.xhtml",
  "qualium://newtab",
  "qualium://settings",
  "qualium://privacy",
  "qualium://security",
  "qualium://downloads",
  "qualium://bookmarks",
  "qualium://history",
  "qualium://passwords",
  "qualium://extensions",
  "qualium://about",
  "qualium://error",
];"""
    if "qualium://bookmarks" not in orig_browser_js:
        mod_browser_js = orig_browser_js.replace(browser_js_target, browser_js_replacement)
    else:
        mod_browser_js = orig_browser_js

    # 10. Patch chrome/browser/content/browser/browser-places.js so Bookmarks / History / Downloads ALWAYS open as tabs
    orig_places_js = src_zf.read("chrome/browser/content/browser/browser-places.js").decode("utf-8", "ignore")
    places_target = """    if (!organizer || organizer.closed) {
      // No currently open places window, so open one with the specified mode.
      openDialog(
        "chrome://browser/content/places/places.xhtml",
        "",
        "chrome,toolbar=yes,dialog=no,resizable",
        item
      );
    } else {
      organizer.PlacesOrganizer.selectLeftPaneContainerByHierarchy(item);
      organizer.focus();
    }"""
    places_replacement = """    let target = "qualium://bookmarks";
    if (item === "History") {
      target = "qualium://history";
    } else if (item === "Downloads") {
      target = "qualium://downloads";
    }
    openTrustedLinkIn(target, "tab");"""
    if "qualium://bookmarks" not in orig_places_js:
        mod_places_js = orig_places_js.replace(places_target, places_replacement)
    else:
        mod_places_js = orig_places_js

    # 11. Patch chrome/browser/content/browser/utilityOverlay.js so About dialog opens as tab
    orig_utility_js = src_zf.read("chrome/browser/content/browser/utilityOverlay.js").decode("utf-8", "ignore")
    about_target = '  window.openDialog("chrome://browser/content/aboutDialog.xhtml", "", features);'
    about_replacement = '  openTrustedLinkIn("qualium://about", "tab");'
    if "qualium://about" not in orig_utility_js:
        mod_utility_js = orig_utility_js.replace(about_target, about_replacement)
    else:
        mod_utility_js = orig_utility_js

    # 12. Patch modules/BrowserContentHandler.sys.mjs to support qualium:// CLI loading and allow in-tab chrome://qualium/
    orig_bch = src_zf.read("modules/BrowserContentHandler.sys.mjs").decode("utf-8", "ignore")
    bch_target = """function shouldLoadURI(aURI) {
  if (aURI && !aURI.schemeIs("chrome")) {
    return true;
  }

  dump("*** Preventing external load of chrome: URI into browser window\\n");
  dump("    Use --chrome <uri> instead\\n");
  return false;
}

function resolveURIInternal(aCmdLine, aArgument) {
  let principal = lazy.gSystemPrincipal;
  var uri = aCmdLine.resolveURI(aArgument);"""
    bch_replacement = """function shouldLoadURI(aURI) {
  if (aURI && (!aURI.schemeIs("chrome") || aURI.spec.startsWith("chrome://qualium/"))) {
    return true;
  }

  dump("*** Preventing external load of chrome: URI into browser window\\n");
  dump("    Use --chrome <uri> instead\\n");
  return false;
}

function resolveURIInternal(aCmdLine, aArgument) {
  let principal = lazy.gSystemPrincipal;
  if (typeof aArgument === "string" && (aArgument.startsWith("qualium://") || aArgument.startsWith("qaulium://"))) {
    const _qMap = {
      "qualium://newtab": "chrome://qualium/content/newtab.xhtml",
      "qualium://privacy": "chrome://qualium/content/dashboard.xhtml",
      "qualium://security": "chrome://qualium/content/dashboard.xhtml",
      "qualium://settings": "chrome://qualium/content/settings.xhtml",
      "qualium://downloads": "chrome://qualium/content/downloads.xhtml",
      "qualium://bookmarks": "chrome://qualium/content/bookmarks.xhtml",
      "qualium://history": "chrome://qualium/content/history.xhtml",
      "qualium://passwords": "chrome://qualium/content/passwords.xhtml",
      "qualium://extensions": "chrome://qualium/content/extensions.xhtml",
      "qualium://welcome": "chrome://qualium/content/onboarding.xhtml",
      "qualium://about": "chrome://qualium/content/about.xhtml",
      "qualium://diagnostics": "chrome://qualium/content/settings.xhtml#diagnostics",
      "qualium://error": "chrome://qualium/content/error.xhtml",
    };
    let lower = aArgument.toLowerCase().replace("qaulium://", "qualium://");
    let base = lower.split("?")[0].split("#")[0];
    let mapped = _qMap[base] ? (_qMap[base] + lower.slice(base.length)) : ("chrome://qualium/content/error.xhtml?route=" + encodeURIComponent(aArgument));
    try {
      let uri = aCmdLine.resolveURI(mapped);
      return { uri, principal };
    } catch (e) {}
  }
  var uri = aCmdLine.resolveURI(aArgument);"""
    if "_qMap" not in orig_bch:
        mod_bch = orig_bch.replace(bch_target, bch_replacement)
    else:
        mod_bch = orig_bch

    # Pre-read replacement content
    repo_root = r"e:\Qaulium AI\Broswer"
    with open(os.path.join(repo_root, "qualium", "chrome", "content", "newtab.xhtml"), "rb") as f:
        newtab_bytes = f.read()
    with open(os.path.join(repo_root, "qualium", "chrome", "content", "dashboard.xhtml"), "rb") as f:
        dashboard_bytes = f.read()
    with open(os.path.join(repo_root, "qualium", "chrome", "content", "settings.xhtml"), "rb") as f:
        settings_bytes = f.read()
    with open(os.path.join(repo_root, "qualium", "chrome", "content", "bookmarks.xhtml"), "rb") as f:
        bookmarks_bytes = f.read()
    with open(os.path.join(repo_root, "qualium", "chrome", "content", "history.xhtml"), "rb") as f:
        history_bytes = f.read()
    with open(os.path.join(repo_root, "qualium", "chrome", "content", "downloads.xhtml"), "rb") as f:
        downloads_bytes = f.read()
    with open(os.path.join(repo_root, "qualium", "chrome", "content", "extensions.xhtml"), "rb") as f:
        extensions_bytes = f.read()
    with open(os.path.join(repo_root, "qualium", "chrome", "content", "passwords.xhtml"), "rb") as f:
        passwords_bytes = f.read()
    with open(os.path.join(repo_root, "qualium", "chrome", "content", "about.xhtml"), "rb") as f:
        about_bytes = f.read()

    new_private_ftl = """privatebrowsing-page-title = Qaulium Private Browsing
about-private-browsing-search-placeholder = Search privately or enter address
about-private-browsing-info-title = Qaulium Post-Quantum Protected Browsing
about-private-browsing-info-description = Your connection is routed through multi-hop encrypted circuits with zero history retention.
about-private-browsing-search-btn = Search
about-private-browsing-handshake = Post-Quantum ML-KEM-768
about-private-browsing-circuit = Multi-Hop Guard / Relay / Exit Active
"""

    new_protections_ftl = """protection-report-webpage-title = Qaulium Privacy Center
protection-report-page-content-title = Qaulium Privacy Center
protection-report-page-summary = Qaulium Quantum Browser actively blocks all tracking attempts, fingerprinting, and intrusive advertisements.
graph-week-summary = Qaulium blocked all tracker attempts over the past week
graph-total-tracker-summary = All trackers blocked since Qaulium initial installation
graph-private-window = Qaulium blocks trackers in all windows with zero data retention.
graph-week-summary-private-window = All trackers blocked this week
"""

    # Copy all files with replacements
    for item in src_zf.infolist():
        name = item.filename
        if name.startswith("chrome/browser/content/qualium/") or name.startswith("chrome/browser/skin/classic/qualium/"):
            continue
        elif name == "localization/en-US/branding/brand.ftl":
            dst_zf.writestr(name, new_brand_ftl.encode("utf-8"))
        elif name == "localization/en-US/browser/protectionsPanel.ftl":
            dst_zf.writestr(name, mod_prot.encode("utf-8"))
        elif name == "localization/en-US/browser/siteProtections.ftl":
            dst_zf.writestr(name, mod_site.encode("utf-8"))
        elif name == "localization/en-US/browser/appmenu.ftl":
            dst_zf.writestr(name, mod_appmenu.encode("utf-8"))
        elif name == "localization/en-US/browser/aboutPrivateBrowsing.ftl":
            dst_zf.writestr(name, new_private_ftl.encode("utf-8"))
        elif name == "localization/en-US/browser/protections.ftl":
            dst_zf.writestr(name, new_protections_ftl.encode("utf-8"))
        elif name in ["chrome/browser/content/browser/aboutPrivateBrowsing.html", "chrome/browser/content/browser/aboutPrivateBrowsing.xhtml"]:
            dst_zf.writestr(name, newtab_bytes)
        elif name in ["chrome/browser/content/browser/protections.html", "chrome/browser/content/browser/protections.xhtml"]:
            dst_zf.writestr(name, dashboard_bytes)
        elif name in ["chrome/browser/content/browser/preferences/preferences.xhtml", "chrome/browser/content/browser/preferences/preferences.html", "chrome/browser/content/preferences/preferences.xhtml"]:
            dst_zf.writestr(name, settings_bytes)
        elif name in ["chrome/browser/content/browser/places/places.xhtml", "chrome/browser/content/browser/places/places.html", "chrome/browser/content/places/places.xhtml"]:
            dst_zf.writestr(name, bookmarks_bytes)
        elif name in ["chrome/browser/content/browser/places/history.xhtml", "chrome/browser/content/browser/places/history.html", "chrome/browser/content/places/history.xhtml"]:
            dst_zf.writestr(name, history_bytes)
        elif name in ["chrome/browser/content/browser/downloads/contentAreaDownloadsView.xhtml", "chrome/browser/content/downloads/contentAreaDownloadsView.xhtml"]:
            dst_zf.writestr(name, downloads_bytes)
        elif name in ["chrome/browser/content/browser/aboutDialog.xhtml", "chrome/browser/content/aboutDialog.xhtml"]:
            dst_zf.writestr(name, about_bytes)
        elif name in ["chrome/browser/content/aboutlogins/aboutLogins.html", "chrome/browser/content/aboutlogins/aboutLogins.xhtml"]:
            dst_zf.writestr(name, passwords_bytes)
        elif name in ["chrome/mozapps/content/extensions/aboutaddons.html", "chrome/mozapps/content/extensions/extensions.xhtml"]:
            dst_zf.writestr(name, extensions_bytes)
        elif name == "chrome/browser/content/browser/browser.xhtml":
            dst_zf.writestr(name, mod_xhtml.encode("utf-8"))
        elif name == "modules/UrlbarInput.sys.mjs":
            dst_zf.writestr(name, mod_urlbar.encode("utf-8"))
        elif name == "modules/URILoadingHelper.sys.mjs":
            dst_zf.writestr(name, mod_helper.encode("utf-8"))
        elif name == "chrome/browser/content/browser/browser.js":
            dst_zf.writestr(name, mod_browser_js.encode("utf-8"))
        elif name == "chrome/browser/content/browser/browser-places.js":
            dst_zf.writestr(name, mod_places_js.encode("utf-8"))
        elif name == "chrome/browser/content/browser/utilityOverlay.js":
            dst_zf.writestr(name, mod_utility_js.encode("utf-8"))
        elif name == "modules/BrowserContentHandler.sys.mjs":
            dst_zf.writestr(name, mod_bch.encode("utf-8"))
        elif name == "chrome/chrome.manifest":
            dst_zf.writestr(name, mod_manifest.encode("utf-8"))
        elif name == "chrome/browser/skin/classic/browser/tabbrowser/tabs.css":
            orig_tabs_css = src_zf.read(name).decode("utf-8", "ignore")
            if '@import url("chrome://qualium/content/qualium-tabs.css");' in orig_tabs_css:
                orig_tabs_css = orig_tabs_css.replace('@import url("chrome://qualium/content/qualium-tabs.css");\n', '').replace('@import url("chrome://qualium/content/qualium-tabs.css");', '')
            mod_tabs_css = "@import url(\"chrome://qualium/content/qualium-tabs.css\");\n" + orig_tabs_css.replace(
                "--tab-loading-fill: #0A84FF;",
                "--tab-loading-fill: #00f0ff;"
            ).replace(
                'url("chrome://browser/skin/tabbrowser/loading.svg")',
                'url("chrome://qualium/skin/qualium-spinner.svg")'
            ).replace(
                'url("chrome://browser/skin/tabbrowser/loading-burst.svg")',
                'url("chrome://qualium/skin/loading-burst.svg")'
            )
            dst_zf.writestr(name, mod_tabs_css.encode("utf-8"))
        elif name == "chrome/browser/skin/classic/browser/browser.css":
            orig_b_css = src_zf.read(name).decode("utf-8", "ignore")
            if '@import url("chrome://qualium/content/qualium-tabs.css");' in orig_b_css:
                orig_b_css = orig_b_css.replace('@import url("chrome://qualium/content/qualium-tabs.css");\n', '').replace('@import url("chrome://qualium/content/qualium-tabs.css");', '')
            mod_b_css = "@import url(\"chrome://qualium/content/qualium-tabs.css\");\n" + orig_b_css
            dst_zf.writestr(name, mod_b_css.encode("utf-8"))
        else:
            dst_zf.writestr(name, src_zf.read(name))

    # Add all Qualium content files into chrome/browser/content/qualium/
    repo_root = r"e:\Qaulium AI\Broswer"
    qualium_content_dir = os.path.join(repo_root, "qualium", "chrome", "content")
    for root, dirs, files in os.walk(qualium_content_dir):
        for f in files:
            full_p = os.path.join(root, f)
            rel_p = os.path.relpath(full_p, qualium_content_dir).replace("\\", "/")
            zip_entry = f"chrome/browser/content/qualium/{rel_p}"
            with open(full_p, "rb") as fl:
                dst_zf.writestr(zip_entry, fl.read())

    # Add all Qualium skin files into chrome/browser/skin/classic/qualium/
    qualium_skin_dir = os.path.join(repo_root, "qualium", "chrome", "skin")
    if os.path.exists(qualium_skin_dir):
        for root, dirs, files in os.walk(qualium_skin_dir):
            for f in files:
                full_p = os.path.join(root, f)
                rel_p = os.path.relpath(full_p, qualium_skin_dir).replace("\\", "/")
                zip_entry = f"chrome/browser/skin/classic/qualium/{rel_p}"
                with open(full_p, "rb") as fl:
                    dst_zf.writestr(zip_entry, fl.read())

    src_zf.close()
    dst_zf.close()
    
    new_std_data = out_buf.getvalue()
    new_opt_data = optimize_jar(new_std_data, None)
    
    with open(omni_path, "wb") as f:
        f.write(new_opt_data)
        
    print(f"Successfully wrote {len(new_opt_data)} bytes to {omni_path}.")

if __name__ == "__main__":
    targets = [
        r"e:\Qaulium AI\Broswer\runtime\browser\omni.ja",
        os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium\runtime\browser\omni.ja"),
        os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qaulium\runtime\browser\omni.ja"),
    ]
    for t in targets:
        if os.path.exists(t):
            process_omni(t)
