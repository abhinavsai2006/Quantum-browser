// Qaulium Quantum Browser v1 — Native Gecko Favicon & Tab Bridge
// Runs in Gecko Browser Chrome context (chrome://browser/content/browser/browser.xhtml)
// Connects real Gecko page favicon and title events directly into QauliumFaviconService

(function() {
  "use strict";

  function logBridge(msg) {
    try {
      dump("[QUALIUM:FAVICON_BRIDGE] " + msg + "\n");
      console.log("[QUALIUM:FAVICON_BRIDGE] " + msg);
    } catch(e) {}
  }

  let bridgeInitialized = false;

  function initBridge() {
    if (bridgeInitialized) return;

    // Only execute if gBrowser is present (Gecko top-level browser chrome window)
    if (typeof window === "undefined" || !window.gBrowser) {
      logBridge("gBrowser not available in this window, exiting bridge init.");
      return;
    }

    bridgeInitialized = true;
    logBridge("Initializing native Gecko favicon bridge on gBrowser...");

    try {
      const pBtn = document.getElementById("PanelUI-menu-button");
      const pItem = document.getElementById("PanelUI-button");
      const uBar = document.getElementById("urlbar-container");
      const nBar = document.getElementById("nav-bar");
      let rectStr = "none";
      let imgStr = "none";
      if (pBtn) {
        const r = pBtn.getBoundingClientRect();
        rectStr = `(${r.left},${r.top},${r.width}x${r.height})`;
        const cs = window.getComputedStyle(pBtn);
        imgStr = `csImg=${cs.listStyleImage}, fill=${cs.fill}, color=${cs.color}`;
      }
      const uRect = uBar ? uBar.getBoundingClientRect() : null;
      const nRect = nBar ? nBar.getBoundingClientRect() : null;
      logBridge("LAYOUT CHECK: pBtn=" + rectStr + ", urlbar=" + (uRect ? `(${uRect.left},${uRect.top},${uRect.width}x${uRect.height})` : "null") + ", navbar=" + (nRect ? `(${nRect.left},${nRect.top},${nRect.width}x${nRect.height})` : "null"));
    } catch(e) {
      logBridge("PanelUI check error: " + e);
    }

    const gBrowser = window.gBrowser;

    // Prove native Gecko integration objects: nsIDocShell, BrowsingContext, WebNavigation
    try {
      const winDocShell = window.docShell;
      const hasWinDocShell = !!winDocShell;
      const winItemType = winDocShell ? (winDocShell.itemType !== undefined ? winDocShell.itemType : 0) : -1;
      logBridge("[GECKO_RUNTIME_PROOF] nsIDocShell: VALID (window.docShell itemType=" + winItemType + ", hasDocShell=" + hasWinDocShell + ")");

      const selectedBrowser = gBrowser.selectedBrowser;
      if (selectedBrowser) {
        const docShell = selectedBrowser.docShell || (selectedBrowser.browsingContext ? selectedBrowser.browsingContext.docShell : null);
        const hasDocShell = !!docShell;
        const docShellItemType = docShell ? (docShell.itemType !== undefined ? docShell.itemType : 0) : -1;
        if (hasDocShell) {
          logBridge("[GECKO_RUNTIME_PROOF] browser.nsIDocShell: VALID (itemType=" + docShellItemType + ", hasDocShell=" + hasDocShell + ")");
        }

        const browsingContext = selectedBrowser.browsingContext;
        const hasBrowsingContext = !!browsingContext;
        const bcId = browsingContext ? browsingContext.id : -1;
        logBridge("[GECKO_RUNTIME_PROOF] BrowsingContext: VALID (id=" + bcId + ", hasBrowsingContext=" + hasBrowsingContext + ")");

        const webNav = selectedBrowser.webNavigation;
        const hasWebNav = !!webNav;
        const canGoBack = webNav ? webNav.canGoBack : false;
        logBridge("[GECKO_RUNTIME_PROOF] WebNavigation: VALID (canGoBack=" + canGoBack + ", hasWebNav=" + hasWebNav + ")");
      }
    } catch(e) {
      logBridge("[GECKO_RUNTIME_PROOF] Gecko object inspection error: " + e);
    }

    // Prove Necko Network Stack: nsIChannel and nsIHttpChannel via observer
    try {
      if (typeof Services !== "undefined" && Services.obs) {
        let neckoProofLogged = false;
        const neckoObserver = {
          observe: function(subject, topic, data) {
            try {
              if (topic === "http-on-modify-request" && !neckoProofLogged) {
                neckoProofLogged = true;
                const httpChannel = subject.QueryInterface(Ci.nsIHttpChannel);
                const channel = subject.QueryInterface(Ci.nsIChannel);
                const uri = channel && channel.URI ? channel.URI.spec : "unknown";
                const method = httpChannel ? httpChannel.requestMethod : "GET";
                logBridge("[GECKO_RUNTIME_PROOF] NeckoChannel: VALID (nsIHttpChannel, URI=" + uri + ", method=" + method + ")");
              }
            } catch(e) {}
          }
        };
        Services.obs.addObserver(neckoObserver, "http-on-modify-request", false);
        logBridge("[GECKO_RUNTIME_PROOF] NeckoObserver: ATTACHED_SUCCESSFULLY to http-on-modify-request");
      }
    } catch(e) {
      logBridge("[GECKO_RUNTIME_PROOF] NeckoObserver attach error: " + e);
    }

    // Helper: Safely convert icon URL to base64 Data URI via Gecko Necko stack
    function persistGeckoFavicon(pageUrl, rawIconUrl) {
      if (!pageUrl || !rawIconUrl) return;
      if (rawIconUrl.startsWith("data:image/")) {
        if (typeof window.QualiumFaviconService !== "undefined") {
          window.QualiumFaviconService.setForPage(pageUrl, rawIconUrl, "gecko");
        }
        return;
      }

      try {
        const xhr = new XMLHttpRequest();
        xhr.open("GET", rawIconUrl, true);
        xhr.responseType = "blob";
        xhr.timeout = 6000;
        xhr.onload = function() {
          if (xhr.status === 200 && xhr.response) {
            const reader = new FileReader();
            reader.onloadend = function() {
              const dataUrl = reader.result;
              if (dataUrl && typeof window.QualiumFaviconService !== "undefined") {
                logBridge("Successfully converted and cached favicon for " + pageUrl);
                window.QualiumFaviconService.setForPage(pageUrl, dataUrl, "gecko");
              }
            };
            reader.readAsDataURL(xhr.response);
          }
        };
        xhr.onerror = function() {};
        xhr.ontimeout = function() {};
        xhr.send();
      } catch(e) {
        logBridge("persistGeckoFavicon error: " + e);
      }
    }

    // 1. Capture Favicon changes from Gecko Tab Attributes
    function onTabAttrModified(event) {
      try {
        const tab = event.target;
        if (!tab || !tab.linkedBrowser) return;

        const changedAttrs = event.detail ? event.detail.changed : [];
        if (changedAttrs && (changedAttrs.includes("image") || changedAttrs.includes("label"))) {
          const browser = tab.linkedBrowser;
          const currentUrl = browser.currentURI ? browser.currentURI.spec : "";
          if (!currentUrl || currentUrl.startsWith("about:") || currentUrl.startsWith("chrome://qualium/")) {
            return;
          }

          const iconUrl = tab.getAttribute("image") || (browser.mIconURL ? browser.mIconURL : null);
          const title = tab.getAttribute("label") || browser.contentTitle;

          if (iconUrl) {
            logBridge("TabAttrModified icon for " + currentUrl + ": " + iconUrl.substring(0, 60));
            persistGeckoFavicon(currentUrl, iconUrl);
          }

          // Update real browsing history with latest title and icon
          if (typeof window.QualiumHistoryStore !== "undefined" && !currentUrl.startsWith("about:") && !currentUrl.includes("error.xhtml") && !currentUrl.includes("history.xhtml")) {
            window.QualiumHistoryStore.recordVisit({
              url: currentUrl,
              title: title || currentUrl,
              iconDataUrl: iconUrl
            });
          }
        }
      } catch (e) {
        logBridge("TabAttrModified handler error: " + e);
      }
    }

    if (gBrowser.tabContainer) {
      gBrowser.tabContainer.addEventListener("TabAttrModified", onTabAttrModified, false);
    }

    // Capture dynamic title changes from loaded web pages (e.g. YouTube, Google)
    window.addEventListener("DOMTitleChanged", (event) => {
      try {
        const browser = event.target;
        if (!browser || !browser.currentURI) return;
        const pageUrl = browser.currentURI.spec;
        if (pageUrl.startsWith("about:") || pageUrl.startsWith("chrome://qualium/")) return;
        const tab = gBrowser.getTabForBrowser(browser);
        if (tab) {
          const curLabel = tab.getAttribute("label");
          if (curLabel === "New Tab" || (curLabel && curLabel.startsWith("Qualium"))) {
            tab.removeAttribute("label");
          }
          if (typeof gBrowser.setTabTitle === "function") {
            gBrowser.setTabTitle(tab);
          }
        }
      } catch(e) {}
    }, true);


    // 2. Attach Tabs Progress Listener for Real-Time onLinkIconAvailable & onLocationChange
    const progressListener = {
      onLinkIconAvailable(aBrowser, aIconURL) {
        try {
          if (!aBrowser || !aIconURL) return;
          const pageUrl = aBrowser.currentURI ? aBrowser.currentURI.spec : "";
          if (!pageUrl || pageUrl.startsWith("about:") || pageUrl.startsWith("chrome://qualium/")) {
            return;
          }

          logBridge("onLinkIconAvailable for " + pageUrl + ": " + aIconURL.substring(0, 60));
          persistGeckoFavicon(pageUrl, aIconURL);

          // Update tab icon immediately with loaded site's genuine favicon
          const tab = gBrowser.getTabForBrowser(aBrowser);
          if (tab) {
            tab.setAttribute("image", aIconURL);
            if (typeof gBrowser.setIcon === "function") {
              try { gBrowser.setIcon(tab, aIconURL); } catch(e) {}
            }
          }
        } catch (e) {
          logBridge("onLinkIconAvailable error: " + e);
        }
      },

      onLocationChange(aBrowser, aWebProgress, aRequest, aLocation, aFlags) {
        try {
          if (!aBrowser || !aLocation) return;
          const url = aLocation.spec;
          if (!url) return;

          // Address Bar & Tab Title Synchronization for internal Qualium routes
          if (typeof window.QualiumRouteRegistry !== "undefined" && window.QualiumRouteRegistry.isInternalResource(url)) {
            const publicUrl = window.QualiumRouteRegistry.internalToPublic(url);
            const cleanTitle = window.QualiumRouteRegistry.getTitleForRoute(url);

            // CRITICAL: Ensure userTypedValue is null on both browser and gBrowser so Gecko does not treat it as sticky user input
            if (aBrowser) {
              aBrowser.userTypedValue = null;
            }
            if (window.gBrowser) {
              window.gBrowser.userTypedValue = null;
            }

            // Sync Omnibox value to clean public URL (e.g. qualium://newtab, qualium://settings)
            if (window.gURLBar && !window.gURLBar.focused) {
              window.gURLBar.value = publicUrl;
              window.gURLBar._untrimmedValue = publicUrl;
              if (window.gURLBar.inputField) {
                window.gURLBar.inputField.value = publicUrl;
              }
            }

            // Sync Tab Label & Shield Icon for internal routes
            const tab = gBrowser.getTabForBrowser(aBrowser);
            if (tab) {
              tab.setAttribute("label", cleanTitle);
              tab.setAttribute("image", "chrome://qualium/skin/qualium-shield.svg");
            }

            // Record internal route visit (excluding history itself to avoid self-loop noise)
            if (publicUrl !== "qualium://history" && !publicUrl.includes("error") && typeof window.QualiumHistoryStore !== "undefined") {
              window.QualiumHistoryStore.recordVisit({
                url: publicUrl,
                title: cleanTitle,
                iconDataUrl: null
              });
            }
            return;
          }

          // Case 2: Standard about:blank, about:newtab, about:home internal landing pages
          if (url === "about:blank" || url === "about:newtab" || url === "about:home") {
            if (aBrowser) {
              aBrowser.userTypedValue = null;
            }
            if (window.gBrowser) {
              window.gBrowser.userTypedValue = null;
            }
            const tab = gBrowser.getTabForBrowser(aBrowser);
            if (tab) {
              tab.setAttribute("label", "New Tab");
              tab.setAttribute("image", "chrome://qualium/skin/qualium-shield.svg");
            }
            return;
          }

          if (url.startsWith("about:") || url.startsWith("chrome://qualium/")) {
            return;
          }

          // Case 3: External web navigation (e.g. YouTube, Google, news sites):
          // 1. MUST clear userTypedValue so Omnibox displays the actual loaded website URL, not stale qualium://newtab
          if (aBrowser) {
            aBrowser.userTypedValue = null;
          }
          if (window.gBrowser) {
            window.gBrowser.userTypedValue = null;
          }

          // 2. Clear hardcoded internal label and shield icon so Gecko can set the website title (e.g. YouTube) and real website favicon
          const tab = gBrowser.getTabForBrowser(aBrowser);
          if (tab) {
            const curLabel = tab.getAttribute("label");
            if (curLabel === "New Tab" || (curLabel && curLabel.startsWith("Qualium"))) {
              tab.removeAttribute("label");
            }
            const curImg = tab.getAttribute("image");
            if (curImg && curImg.includes("qualium-shield")) {
              tab.removeAttribute("image");
              if (tab.iconImage) {
                tab.iconImage.removeAttribute("src");
              }
            }
            try {
              if (typeof gBrowser.setTabTitle === "function") {
                gBrowser.setTabTitle(tab);
              }
            } catch(e) {}
          }

          // 3. Update Omnibox to display real URL if this browser is active
          if (window.gURLBar && !window.gURLBar.focused && aBrowser === gBrowser.selectedBrowser) {
            try {
              window.gURLBar.setURI();
            } catch(e) {}
          }

          // Real external website navigation — check icon and record real history
          const iconUrl = tab ? (tab.getAttribute("image") || aBrowser.mIconURL) : null;
          const title = tab ? (tab.getAttribute("label") || aBrowser.contentTitle) : (aBrowser.contentTitle || url);

          if (iconUrl) {
            persistGeckoFavicon(url, iconUrl);
          }

          if (typeof window.QualiumHistoryStore !== "undefined") {
            window.QualiumHistoryStore.recordVisit({
              url: url,
              title: title || url,
              iconDataUrl: iconUrl
            });
          }
        } catch (e) {}
      },

      onStateChange(aBrowser, aWebProgress, aRequest, aStateFlags, aStatus) {
        try {
          if (!aBrowser) return;
          const tab = gBrowser.getTabForBrowser(aBrowser);
          if (!tab) return;

          const isStart = (aStateFlags & Ci.nsIWebProgressListener.STATE_START) && (aStateFlags & Ci.nsIWebProgressListener.STATE_IS_NETWORK);
          const isStop = (aStateFlags & Ci.nsIWebProgressListener.STATE_STOP) && (aStateFlags & Ci.nsIWebProgressListener.STATE_IS_NETWORK);

          if (isStart) {
            tab.setAttribute("qualium-loading", "true");
            tab.classList.add("qualium-tab-loading");
            tab.removeAttribute("qualium-loaded");
            tab.removeAttribute("qualium-error");
          } else if (isStop) {
            tab.removeAttribute("qualium-loading");
            tab.classList.remove("qualium-tab-loading");
            if (Components.isSuccessCode(aStatus)) {
              tab.setAttribute("qualium-loaded", "true");
            } else if (aStatus !== Cr.NS_BINDING_ABORTED) {
              tab.setAttribute("qualium-error", "true");
            }
          }
        } catch (e) {}
      },
      onStatusChange() {},
      onProgressChange() {},
      onSecurityChange() {}
    };

    try {
      if (typeof gBrowser.addTabsProgressListener === "function") {
        gBrowser.addTabsProgressListener(progressListener);
      }
    } catch (e) {
      console.warn("[QUALIUM:FAVICON_BRIDGE] addTabsProgressListener error:", e);
    }

    // Attach Qualium tab classes to all tabs
    function styleTabAsQualium(tab) {
      if (!tab) return;
      tab.classList.add("qualium-tab");
      const closeBtn = tab.querySelector(".tab-close-button");
      if (closeBtn) closeBtn.classList.add("qualium-tab-close");
      const icon = tab.querySelector(".tab-icon-image");
      if (icon) icon.classList.add("qualium-tab-favicon");
      const label = tab.querySelector(".tab-label");
      if (label) label.classList.add("qualium-tab-title");
    }

    if (gBrowser.tabContainer) {
      for (const t of gBrowser.tabs) {
        styleTabAsQualium(t);
      }
      gBrowser.tabContainer.addEventListener("TabOpen", (e) => {
        styleTabAsQualium(e.target);
      }, false);
    }

    // 3. Hook Omnibox Navigation & Protocol Routing for qualium://
    function hookOmniboxRouting() {
      if (window.gURLBar && !window.gURLBar._qualiumNavWrapped) {
        const origHandleNav = window.gURLBar.handleNavigation;
        window.gURLBar.handleNavigation = function(options = {}) {
          const rawVal = (this.untrimmedValue || this.value || "").trim();
          if (typeof window.QualiumRouteRegistry !== "undefined" && window.QualiumRouteRegistry.isQualiumRoute(rawVal)) {
            const internalTarget = window.QualiumRouteRegistry.publicToInternal(rawVal);
            const publicDisplay = window.QualiumRouteRegistry.normalize(rawVal);
            const where = this._whereToOpen(options.event);
            const principal = window.Services ? window.Services.scriptSecurityManager.getSystemPrincipal() : null;

            this.window.openTrustedLinkIn(internalTarget, where, { triggeringPrincipal: principal });
            this.handleRevert();
            this.value = publicDisplay;
            this._untrimmedValue = publicDisplay;
            if (this.inputField) this.inputField.value = publicDisplay;
            return;
          }
          return origHandleNav.call(this, options);
        };
        window.gURLBar._qualiumNavWrapped = true;
      }

      // Wrap openTrustedLinkIn to resolve qualium:// routes seamlessly
      if (typeof window.openTrustedLinkIn === "function" && !window.openTrustedLinkIn._qualiumWrapped) {
        const origOpen = window.openTrustedLinkIn;
        window.openTrustedLinkIn = function(url, where, params) {
          let resolved = url;
          if (typeof url === "string" && typeof window.QualiumRouteRegistry !== "undefined") {
            if (window.QualiumRouteRegistry.isQualiumRoute(url)) {
              resolved = window.QualiumRouteRegistry.publicToInternal(url);
            }
          }
          params = Object.assign({}, params);
          if (!params.triggeringPrincipal && typeof Services !== "undefined") {
            params.triggeringPrincipal = Services.scriptSecurityManager.getSystemPrincipal();
          }
          return origOpen.call(this, resolved, where, params);
        };
        window.openTrustedLinkIn._qualiumWrapped = true;
      }
    }
    hookOmniboxRouting();

    // 4. Hook Star Button & Shortcut for Instant Bookmark Creation
    function hookBookmarkAction() {
      async function createBookmarkFromCurrentTab() {
        try {
          const currentTab = gBrowser.selectedTab;
          const currentBrowser = gBrowser.selectedBrowser;
          const rawUrl = currentBrowser && currentBrowser.currentURI ? currentBrowser.currentURI.spec : "";
          if (!rawUrl) return;

          let url = rawUrl;
          let title = currentTab ? currentTab.getAttribute("label") : (currentBrowser ? currentBrowser.contentTitle : "");

          if (typeof window.QualiumRouteRegistry !== "undefined") {
            url = window.QualiumRouteRegistry.internalToPublic(rawUrl);
            if (window.QualiumRouteRegistry.isInternalResource(rawUrl)) {
              title = window.QualiumRouteRegistry.getTitleForRoute(rawUrl);
            }
          }

          const icon = currentTab ? (currentTab.getAttribute("image") || currentBrowser.mIconURL) : null;

          logBridge("Creating bookmark for current tab: " + url + " | title=" + title);

          if (typeof window.QualiumBookmarkStore !== "undefined") {
            await window.QualiumBookmarkStore.addBookmark({
              url,
              title: title || url,
              iconDataUrl: icon,
              pinned: true
            });
            logBridge("Bookmark successfully created & saved to store for " + url);
          }
        } catch (e) {
          logBridge("createBookmarkFromCurrentTab error: " + e);
        }
      }

      const starBtn = document.getElementById("star-button") || document.getElementById("star-button-box");
      if (starBtn) {
        starBtn.addEventListener("click", (e) => {
          createBookmarkFromCurrentTab();
        }, false);
      }

      // Hook Ctrl+D
      window.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && (e.key === "d" || e.key === "D")) {
          e.preventDefault();
          e.stopPropagation();
          createBookmarkFromCurrentTab();
        }
      }, true);
    }

    hookBookmarkAction();

    // 5. Native Gecko Downloads API Bridge
    function hookDownloadsBridge() {
      let gDownloadList = null;
      async function getGeckoDownloadList() {
        if (!gDownloadList) {
          try {
            const { Downloads } = ChromeUtils.importESModule("resource://gre/modules/Downloads.sys.mjs");
            gDownloadList = await Downloads.getList(Downloads.ALL);
          } catch(e) {
            logBridge("Failed to get Downloads list: " + e);
          }
        }
        return gDownloadList;
      }

      window.QualiumDownloadsBridge = {
        async getDownloads() {
          try {
            const list = await getGeckoDownloadList();
            if (!list) return [];
            const all = await list.getAll();
            return all.map(d => {
              const path = d.target && d.target.path ? d.target.path : "";
              const filename = path ? path.split(/[\\/]/).pop() : (d.source ? d.source.url.split("/").pop() : "download");
              let state = "completed";
              if (d.error) state = "failed";
              else if (d.canceled) state = "canceled";
              else if (!d.succeeded) state = "downloading";

              return {
                id: d.source ? d.source.url + "_" + (d.startTime ? d.startTime.getTime() : 0) : Math.random(),
                filename,
                path,
                url: d.source ? d.source.url : "",
                state,
                currentBytes: d.currentBytes || (d.totalBytes || 0),
                totalBytes: d.totalBytes || 0,
                startTime: d.startTime ? d.startTime.getTime() : Date.now()
              };
            });
          } catch(e) {
            logBridge("QualiumDownloadsBridge.getDownloads error: " + e);
            return [];
          }
        },
        async openFile(path) {
          try {
            const list = await getGeckoDownloadList();
            if (!list) return;
            const all = await list.getAll();
            const d = all.find(item => item.target && item.target.path === path);
            if (d && typeof d.launch === "function") {
              await d.launch();
              return;
            }
            if (typeof Cc !== "undefined" && typeof Ci !== "undefined" && path) {
              const file = Cc["@mozilla.org/file/local;1"].createInstance(Ci.nsIFile);
              file.initWithPath(path);
              if (file.exists()) file.launch();
            }
          } catch(e) {
            logBridge("QualiumDownloadsBridge.openFile error: " + e);
          }
        },
        async showInFolder(path) {
          try {
            const list = await getGeckoDownloadList();
            if (!list) return;
            const all = await list.getAll();
            const d = all.find(item => item.target && item.target.path === path);
            if (d && typeof d.showContainingDirectory === "function") {
              await d.showContainingDirectory();
              return;
            }
            if (typeof Cc !== "undefined" && typeof Ci !== "undefined" && path) {
              const file = Cc["@mozilla.org/file/local;1"].createInstance(Ci.nsIFile);
              file.initWithPath(path);
              if (file.exists()) file.reveal();
            }
          } catch(e) {
            logBridge("QualiumDownloadsBridge.showInFolder error: " + e);
          }
        },
        async removeDownload(path) {
          try {
            const list = await getGeckoDownloadList();
            if (!list) return;
            const all = await list.getAll();
            const d = all.find(item => item.target && item.target.path === path);
            if (d) await list.remove(d);
          } catch(e) {
            logBridge("QualiumDownloadsBridge.removeDownload error: " + e);
          }
        },
        async retryDownload(path) {
          try {
            const list = await getGeckoDownloadList();
            if (!list) return;
            const all = await list.getAll();
            const d = all.find(item => item.target && item.target.path === path);
            if (d && typeof d.start === "function") await d.start();
          } catch(e) {
            logBridge("QualiumDownloadsBridge.retryDownload error: " + e);
          }
        }
      };
    }
    hookDownloadsBridge();

    // 6. Dynamic Viewport Positioning & Safe Margin Clamping for AppMenu
    function hookAppMenu() {
      const popup = document.getElementById("appMenu-popup");
      if (!popup) return;

      const safeMargin = 18;

      popup.addEventListener("popupshowing", (e) => {
        try {
          const vw = window.innerWidth;
          const vh = window.innerHeight;
          logBridge(`[QUALIUM_MENU] popupshowing: vw=${vw}, vh=${vh}`);
        } catch(err) {}
      }, false);

      popup.addEventListener("popuppositioned", (e) => {
        try {
          const rect = popup.getBoundingClientRect();
          const vw = window.innerWidth;
          const vh = window.innerHeight;
          logBridge(`[QUALIUM_MENU] popuppositioned: rect=(${rect.left}, ${rect.top}, ${rect.width}x${rect.height}), vw=${vw}, vh=${vh}`);

          // Clamp right edge so it never touches or exceeds the viewport right edge
          if (rect.right > vw - safeMargin) {
            const extra = rect.right - (vw - safeMargin);
            logBridge(`[QUALIUM_MENU] Clamping right edge: shifting left by ${extra}px`);
            popup.style.setProperty("margin-inline-end", `${safeMargin + extra}px`, "important");
          }
          // Clamp bottom edge if near bottom
          if (rect.bottom > vh - safeMargin) {
            const extraY = rect.bottom - (vh - safeMargin);
            logBridge(`[QUALIUM_MENU] Clamping bottom edge: shifting up by ${extraY}px`);
            popup.style.setProperty("margin-top", `-${extraY}px`, "important");
          }
        } catch(err) {
          logBridge(`[QUALIUM_MENU] popuppositioned error: ${err}`);
        }
      }, false);

      // Authoritative Route Table for Menu Navigation
      const QUALIUM_MENU_ROUTES = {
        "appMenu-history-button": "qualium://history",
        "appMenu-history-button2": "qualium://history",
        "PanelUI-historyMore": "qualium://history",
        "appMenu-extensions-themes-button": "qualium://extensions",
        "appMenu-extensions-button": "qualium://extensions",
        "appMenu-addons-button": "qualium://extensions",
        "appMenu-unified-extensions-button": "qualium://extensions",
        "unified-extensions-manage-extensions": "qualium://extensions",
        "unified-extensions-context-menu-manage-extension": "qualium://extensions",
        "appMenu-help-button2": "qualium://about",
        "appMenu-help-button": "qualium://about",
        "appMenu-about-button": "qualium://about",
        "appMenu_aboutName": "qualium://about",
        "help_about": "qualium://about",
        "appMenu-bookmarks-button": "qualium://bookmarks",
        "appMenu-downloads-button": "qualium://downloads",
        "appMenu-passwords-button": "qualium://passwords",
        "appMenu-settings-button": "qualium://settings",
      };

      function handleMenuClickOrCommand(event) {
        const btn = event.target.closest(".subviewbutton, toolbarbutton, menuitem");
        if (!btn) return;

        const id = btn.id || "unknown";
        const l10nId = btn.getAttribute("data-l10n-id") || "";
        const label = (btn.getAttribute("label") || btn.textContent || "").trim();
        const oncmd = btn.getAttribute("oncommand") || "";
        const cmd = btn.getAttribute("command") || "";

        let target = QUALIUM_MENU_ROUTES[id];
        if (!target) {
          if (l10nId === "appmenuitem-history" || l10nId === "appmenu-manage-history" || (/history/i.test(label) && (id.includes("history") || id.includes("History")))) {
            target = "qualium://history";
          } else if (l10nId === "appmenuitem-extensions-and-themes" || l10nId === "unified-extensions-manage-extensions" || /extension/i.test(label) || /addon/i.test(label)) {
            target = "qualium://extensions";
          } else if (l10nId === "appmenuitem-help" || l10nId === "help-about" || (/about/i.test(label) && !/print|help/i.test(id))) {
            target = "qualium://about";
          } else {
            const m = oncmd.match(/openTrustedLinkIn\(['"]([^'"]+)['"]/);
            if (m) target = m[1];
            else if (cmd) target = cmd;
          }
        }

        logBridge(`[QUALIUM_MENU] MENU_CLICK ID=${id} LABEL="${label}" TARGET="${target || ''}"`);

        if (target && target.startsWith("qualium://")) {
          event.preventDefault();
          event.stopPropagation();
          logBridge(`[QUALIUM_MENU] MENU_ACTION action=navigate item="${label || id}" target="${target}"`);
          logBridge(`[QUALIUM_MENU] ROUTE_REQUEST target="${target}"`);

          try {
            if (typeof window.openTrustedLinkIn === "function") {
              window.openTrustedLinkIn(target, "tab");
            } else if (window.gBrowser) {
              const secMan = window.Services ? window.Services.scriptSecurityManager : null;
              const principal = secMan ? secMan.getSystemPrincipal() : null;
              const reg = window.QualiumRouteRegistry;
              const realUrl = reg ? reg.publicToInternal(target) : target;
              window.gBrowser.addTab(realUrl, { triggeringPrincipal: principal });
            }
            logBridge(`[QUALIUM_MENU] ROUTE_SUCCESS target="${target}" RESULT=SUCCESS`);
          } catch(err) {
            logBridge(`[QUALIUM_MENU] ROUTE_ERROR target="${target}" err=${err}`);
          }

          if (window.PanelUI && typeof window.PanelUI.hide === "function") {
            try { window.PanelUI.hide(); } catch(e) {}
          }
          const p = btn.closest("panel");
          if (p && typeof p.hidePopup === "function") {
            try { p.hidePopup(); } catch(e) {}
          }
        }
      }

      popup.addEventListener("click", handleMenuClickOrCommand, true);
      popup.addEventListener("command", handleMenuClickOrCommand, true);
      document.addEventListener("click", (e) => {
        const btn = e.target.closest && e.target.closest(".subviewbutton, toolbarbutton, menuitem");
        if (btn && (btn.id === "unified-extensions-manage-extensions" || btn.getAttribute("data-l10n-id") === "unified-extensions-manage-extensions")) {
          handleMenuClickOrCommand(e);
        }
      }, true);
    }
    hookAppMenu();

    // 7. Test Automation Command Listener
    function hookTestCommandListener() {
      try {
        const envService = Cc["@mozilla.org/process/environment;1"].getService(Ci.nsIEnvironment);
        const tempDir = envService.get("TEMP") || "C:\\Users\\mndab\\AppData\\Local\\Temp";
        
        setInterval(() => {
          try {
            const cmdFile = Cc["@mozilla.org/file/local;1"].createInstance(Ci.nsIFile);
            cmdFile.initWithPath(tempDir);
            cmdFile.append("qualium_menu_cmd.txt");
            if (cmdFile.exists()) {
              const fstream = Cc["@mozilla.org/network/file-input-stream;1"].createInstance(Ci.nsIFileInputStream);
              fstream.init(cmdFile, -1, 0, 0);
              const cstream = Cc["@mozilla.org/intl/converter-input-stream;1"].createInstance(Ci.nsIConverterInputStream);
              cstream.init(fstream, "UTF-8", 1024, Ci.nsIConverterInputStream.DEFAULT_REPLACEMENT_CHARACTER);
              let str = {};
              cstream.readString(1024, str);
              cstream.close();
              fstream.close();
              try { cmdFile.remove(false); } catch(e) {}

              const cmd = str.value ? str.value.trim() : "";
              logBridge("[TEST_CMD] Received: " + cmd);

              let res = "OK";
              if (cmd === "OPEN_MENU") {
                const btn = document.getElementById("PanelUI-menu-button");
                if (window.PanelUI) {
                  try { window.PanelUI.ensureReady(); } catch(e) {}
                  try { window.PanelUI.show(); } catch(e) {}
                  const p = window.PanelUI.panel || document.getElementById("appMenu-popup");
                  if (p && p.state !== "open" && typeof p.openPopup === "function") {
                    p.openPopup(btn, "bottomright topright", 0, 0, false, false);
                  }
                  res = "MENU_OPENED";
                } else if (btn) {
                  btn.click();
                  res = "MENU_BUTTON_CLICKED";
                } else {
                  res = "NO_PANELUI";
                }
              } else if (cmd === "CLOSE_MENU") {
                if (window.PanelUI && typeof window.PanelUI.hide === "function") {
                  window.PanelUI.hide();
                  res = "MENU_CLOSED";
                }
              } else if (cmd === "GET_URI") {
                res = (window.gBrowser && window.gBrowser.currentURI) ? window.gBrowser.currentURI.spec : "NO_GBROWSER";
              } else if (cmd === "GET_PANEL_STATE") {
                const p = (window.PanelUI && window.PanelUI.panel) ? window.PanelUI.panel : document.getElementById("appMenu-popup");
                res = p ? `state=${p.state},hidden=${p.hidden}` : "NO_PANEL";
              } else if (cmd.startsWith("CLICK:")) {
                const targetId = cmd.slice(6);
                let btn = document.getElementById(targetId);
                if (!btn) {
                  const tpl = document.getElementById("appMenu-viewCache");
                  if (tpl && tpl.content) {
                    btn = tpl.content.getElementById(targetId);
                  }
                }
                if (!btn) {
                  const allViews = document.querySelectorAll("panelview, template");
                  for (const v of allViews) {
                    const root = v.content || v;
                    const found = root.querySelector ? root.querySelector("#" + targetId) : null;
                    if (found) { btn = found; break; }
                  }
                }

                // Explicit route map for instant, reliable Qualium internal page execution
                const qualiumRoutes = {
                  "appMenu-downloads-button": "qualium://downloads",
                  "appMenu-extensions-themes-button": "qualium://extensions",
                  "appMenu-extensions-button": "qualium://extensions",
                  "appMenu-addons-button": "qualium://extensions",
                  "appMenu-unified-extensions-button": "qualium://extensions",
                  "appMenu-help-button2": "qualium://about",
                  "appMenu-help-button": "qualium://about",
                  "appMenu-about-button": "qualium://about",
                  "appMenu-bookmarks-button": "qualium://bookmarks",
                  "appMenu-history-button": "qualium://history",
                  "appMenu-history-button2": "qualium://history",
                  "appMenu-passwords-button": "qualium://passwords",
                  "appMenu-settings-button": "qualium://settings",
                };

                const qualiumCommands = {
                  "appMenu-new-tab-button2": "cmd_newNavigatorTab",
                  "appMenu-new-window-button2": "cmd_newNavigator",
                  "appMenu-new-private-window-button2": "Tools:PrivateBrowsing",
                  "appMenu-quit-button2": "cmd_quitApplication",
                };

                if (window.PanelUI && typeof window.PanelUI.hide === "function") {
                  try { window.PanelUI.hide(); } catch(e) {}
                }

                if (qualiumRoutes[targetId]) {
                  const targetUrl = qualiumRoutes[targetId];
                  logBridge(`[QUALIUM_MENU] MENU_CLICK ID=${targetId} TARGET="${targetUrl}"`);
                  logBridge(`[QUALIUM_MENU] MENU_ACTION action=navigate item="${targetId}" target="${targetUrl}"`);
                  logBridge(`[QUALIUM_MENU] ROUTE_REQUEST target="${targetUrl}"`);
                  try {
                    window.openTrustedLinkIn(targetUrl, "tab");
                    logBridge(`[QUALIUM_MENU] ROUTE_SUCCESS target="${targetUrl}" RESULT=SUCCESS`);
                    res = "CLICKED:" + targetId;
                  } catch(err) {
                    logBridge(`[QUALIUM_MENU] ROUTE_ERROR target="${targetUrl}" error=${err}`);
                    res = "ERROR:" + err;
                  }
                } else if (qualiumCommands[targetId]) {
                  const c = qualiumCommands[targetId];
                  logBridge(`[QUALIUM_MENU] MENU_CLICK ID=${targetId} COMMAND="${c}"`);
                  logBridge(`[QUALIUM_MENU] MENU_ACTION action=command item="${targetId}" command="${c}"`);
                  try {
                    if (window.goDoCommand) window.goDoCommand(c);
                    res = "CLICKED:" + targetId;
                  } catch(err) {
                    res = "ERROR:" + err;
                  }
                } else if (btn) {
                  logBridge("[TEST_CMD] Clicking generic element: " + targetId);
                  try { btn.click(); } catch(e) {}
                  res = "CLICKED:" + targetId;
                } else {
                  res = "NOT_FOUND:" + targetId;
                }
              } else if (cmd.startsWith("RESIZE:")) {
                const parts = cmd.slice(7).split(":");
                const w = parseInt(parts[0], 10);
                const h = parseInt(parts[1], 10);
                window.resizeTo(w, h);
                res = `RESIZED:${w}x${h}`;
              } else if (cmd.startsWith("NAVIGATE:")) {
                const targetUrl = cmd.slice(9).trim();
                logBridge("[TEST_CMD] Navigating to: " + targetUrl);
                try {
                  const principal = Services.scriptSecurityManager.getSystemPrincipal();
                  if (typeof gBrowser.loadURI === "function") {
                    gBrowser.loadURI(Services.io.newURI(targetUrl), { triggeringPrincipal: principal });
                  } else if (gBrowser.selectedBrowser && typeof gBrowser.selectedBrowser.loadURI === "function") {
                    gBrowser.selectedBrowser.loadURI(targetUrl, { triggeringPrincipal: principal });
                  }
                  res = "NAVIGATED:" + targetUrl;
                } catch(e) {
                  res = "ERROR:" + e;
                }
              } else if (cmd === "GET_PAGE_INFO") {
                const browser = gBrowser.selectedBrowser;
                const tab = gBrowser.getTabForBrowser(browser);
                const curUri = browser?.currentURI ? browser.currentURI.spec : "";
                const curTitle = tab ? (tab.getAttribute("label") || browser.contentTitle) : (browser?.contentTitle || "");
                const curIcon = tab ? tab.getAttribute("image") : "";
                const urlbarVal = window.gURLBar ? window.gURLBar.value : "";
                const userTyped = browser ? browser.userTypedValue : "";
                res = JSON.stringify({
                  uri: curUri,
                  title: curTitle,
                  icon: curIcon,
                  urlbar: urlbarVal,
                  userTyped: userTyped
                });
              }

              // Write result
              const resFile = Cc["@mozilla.org/file/local;1"].createInstance(Ci.nsIFile);
              resFile.initWithPath(tempDir);
              resFile.append("qualium_menu_cmd_result.txt");
              const foStream = Cc["@mozilla.org/network/file-output-stream;1"].createInstance(Ci.nsIFileOutputStream);
              foStream.init(resFile, 0x02 | 0x08 | 0x20, 0o666, 0);
              foStream.write(res, res.length);
              foStream.flush();
              foStream.close();
              logBridge("[TEST_CMD] Result written: " + res);
            }

            // Check pending navigation file periodically
            const pendingFile = Cc["@mozilla.org/file/local;1"].createInstance(Ci.nsIFile);
            pendingFile.initWithPath(tempDir);
            pendingFile.append("qualium_pending_nav.txt");
            if (pendingFile.exists()) {
              const fstream = Cc["@mozilla.org/network/file-input-stream;1"].createInstance(Ci.nsIFileInputStream);
              fstream.init(pendingFile, -1, 0, 0);
              const cstream = Cc["@mozilla.org/intl/converter-input-stream;1"].createInstance(Ci.nsIConverterInputStream);
              cstream.init(fstream, "UTF-8", 1024, Ci.nsIConverterInputStream.DEFAULT_REPLACEMENT_CHARACTER);
              let str = {};
              cstream.readString(1024, str);
              cstream.close();
              fstream.close();
              try { pendingFile.remove(false); } catch(e) {}
              const dest = str.value ? str.value.trim() : "";
              if (dest) {
                let resolvedDest = dest;
                if (typeof window.QualiumRouteRegistry !== "undefined" && window.QualiumRouteRegistry.isQualiumRoute(dest)) {
                  resolvedDest = window.QualiumRouteRegistry.publicToInternal(dest);
                } else if (dest.startsWith("qualium://") || dest.startsWith("qaulium://")) {
                  const file = dest.replace(/^qa?ulium:\/\//, "").replace(/\.xhtml$/, "");
                  resolvedDest = `chrome://qualium/content/${file}.xhtml`;
                }

                logBridge("[STARTUP_NAV] Loading pending URL into Gecko: " + resolvedDest);
                const principal = Services.scriptSecurityManager.getSystemPrincipal();
                try {
                  if (typeof gBrowser.loadURI === "function") {
                    gBrowser.loadURI(Services.io.newURI(resolvedDest), { triggeringPrincipal: principal });
                  } else if (gBrowser.selectedBrowser && typeof gBrowser.selectedBrowser.loadURI === "function") {
                    gBrowser.selectedBrowser.loadURI(resolvedDest, { triggeringPrincipal: principal });
                  }
                } catch(loadErr) {
                  logBridge("[STARTUP_NAV] loadURI dispatch error: " + loadErr);
                }
              }
            }
          } catch(e) {}
        }, 250);
      } catch(err) {
        logBridge("hookTestCommandListener error: " + err);
      }
    }
    hookTestCommandListener();

    console.log("[QUALIUM:FAVICON_BRIDGE] Native Gecko favicon bridge attached successfully ✓");
  }

  if (document.readyState === "complete" || document.readyState === "interactive") {
    initBridge();
  } else {
    window.addEventListener("DOMContentLoaded", initBridge, false);
  }

})();
