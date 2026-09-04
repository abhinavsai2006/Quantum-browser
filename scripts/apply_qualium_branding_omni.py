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

    mod_xhtml = orig_xhtml.replace(
        'id="appMenu-settings-button"\n                     class="subviewbutton"\n                     data-l10n-id="appmenuitem-settings"\n                     />',
        'id="appMenu-settings-button"\n                     class="subviewbutton"\n                     data-l10n-id="appmenuitem-settings"\n                     oncommand="openTrustedLinkIn(\'chrome://qualium/content/settings.xhtml\', \'tab\')"\n                     />'
    ).replace(
        'id="appMenu-help-button2"\n                     class="subviewbutton subviewbutton-nav"\n                     data-l10n-id="appmenuitem-help"\n                     closemenu="none"\n                     />',
        'id="appMenu-help-button2"\n                     class="subviewbutton"\n                     label="About Qaulium"\n                     data-l10n-id="appmenuitem-help"\n                     oncommand="openTrustedLinkIn(\'chrome://qualium/content/settings.xhtml#about\', \'tab\')"\n                     />'
    ).replace(
        'oncommand="openPreferences()"',
        'oncommand="openTrustedLinkIn(\'chrome://qualium/content/settings.xhtml\', \'tab\')"'
    ).replace(
        'oncommand="openProtectionsDashboard()"',
        'oncommand="openTrustedLinkIn(\'chrome://qualium/content/dashboard.xhtml\', \'tab\')"'
    ).replace(
        'id="urlbar-search-button"',
        'id="urlbar-search-button" hidden="true"'
    ).replace(
        'id="appMenu-unified-extensions-button"',
        'id="appMenu-unified-extensions-button" style="display:none!important;"'
    )

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
override chrome://branding/content/icon32.png chrome://qualium/skin/qualium-shield.svg
override chrome://branding/content/icon16.png chrome://qualium/skin/qualium-shield.svg
override chrome://global/skin/icons/defaultFavicon.svg chrome://qualium/skin/qualium-shield.svg
"""
    mod_manifest = orig_manifest + "\n" + qualium_manifest_entries

    # Pre-read replacement content
    repo_root = r"e:\Qaulium AI\Broswer"
    with open(os.path.join(repo_root, "qualium", "chrome", "content", "newtab.xhtml"), "rb") as f:
        newtab_bytes = f.read()
    with open(os.path.join(repo_root, "qualium", "chrome", "content", "dashboard.xhtml"), "rb") as f:
        dashboard_bytes = f.read()

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
        elif name == "chrome/browser/content/browser/browser.xhtml":
            dst_zf.writestr(name, mod_xhtml.encode("utf-8"))
        elif name == "chrome/chrome.manifest":
            dst_zf.writestr(name, mod_manifest.encode("utf-8"))
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
