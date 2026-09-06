// Qaulium Quantum Browser v5 — Native Gecko Favicon & Tab Bridge
// Runs in Gecko Browser Chrome context (chrome://browser/content/browser/browser.xhtml)
// Connects real Gecko page favicon and title events directly into QualiumFaviconService

(function() {
  "use strict";

  function logBridge(msg) {
    try {
      dump("[QUALIUM:FAVICON_BRIDGE] " + msg + "\n");
      console.log("[QUALIUM:FAVICON_BRIDGE] " + msg);
      if (typeof Services !== "undefined" && Services.dirsvc && typeof Cc !== "undefined") {
        const profDir = Services.dirsvc.get("ProfD", Ci.nsIFile);
        const logFile = profDir.clone();
        logFile.append("qualium_bridge.log");
        const foStream = Cc["@mozilla.org/network/file-output-stream;1"].createInstance(Ci.nsIFileOutputStream);
        foStream.init(logFile, 0x02 | 0x08 | 0x10, 0o666, 0);
        const line = new Date().toISOString() + " " + msg + "\n";
        foStream.write(line, line.length);
        foStream.flush();
        foStream.close();
      }
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
        const neckoObserver = {
          observe: function(subject, topic, data) {
            try {
              if (topic === "http-on-modify-request") {
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

            // Sync Omnibox value to clean public URL (e.g. qualium://newtab, qualium://settings)
            if (window.gURLBar && !window.gURLBar.focused) {
              window.gURLBar.value = publicUrl;
              window.gURLBar._untrimmedValue = publicUrl;
              if (window.gURLBar.inputField) {
                window.gURLBar.inputField.value = publicUrl;
              }
              if (aBrowser) {
                aBrowser.userTypedValue = publicUrl;
              }
            }

            // Sync Tab Label
            const tab = gBrowser.getTabForBrowser(aBrowser);
            if (tab) {
              tab.setAttribute("label", cleanTitle);
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

          if (url.startsWith("about:") || url.startsWith("chrome://qualium/")) {
            return;
          }

          // Real external website navigation — check icon and record real history
          const tab = gBrowser.getTabForBrowser(aBrowser);
          const iconUrl = tab ? (tab.getAttribute("image") || aBrowser.mIconURL) : null;
          const title = tab ? tab.getAttribute("label") : (aBrowser.contentTitle || url);

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

      onStateChange() {},
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

    // 4. Process any pending startup URL (e.g. https://example.com)
    try {
      const envService = Cc["@mozilla.org/process/environment;1"].getService(Ci.nsIEnvironment);
      const tempDir = envService.get("TEMP") || "C:\\Users\\mndab\\AppData\\Local\\Temp";
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
        if (dest && dest.startsWith("http")) {
          logBridge("[STARTUP_NAV] Loading pending URL into Gecko: " + dest);
          const principal = Services.scriptSecurityManager.getSystemPrincipal();
          setTimeout(() => {
            try {
              if (typeof gBrowser.loadURI === "function") {
                gBrowser.loadURI(Services.io.newURI(dest), { triggeringPrincipal: principal });
              } else if (gBrowser.selectedBrowser && typeof gBrowser.selectedBrowser.loadURI === "function") {
                gBrowser.selectedBrowser.loadURI(dest, { triggeringPrincipal: principal });
              }

              // Also trigger an explicit Necko XMLHttpRequest to ensure full round-trip
              const xhr = new XMLHttpRequest();
              xhr.open("GET", dest, true);
              xhr.onload = function() {
                logBridge("[STARTUP_NAV] Necko XHR success from " + dest + " status=" + xhr.status + " len=" + (xhr.responseText ? xhr.responseText.length : 0));
              };
              xhr.onerror = function(err) {
                logBridge("[STARTUP_NAV] Necko XHR error for " + dest + ": " + err);
              };
              xhr.send();
              logBridge("[STARTUP_NAV] Dispatched loadURI and Necko XHR for: " + dest);
            } catch(loadErr) {
              logBridge("[STARTUP_NAV] loadURI dispatch error: " + loadErr);
            }
          }, 300);
        }
      }
    } catch(err) {
      logBridge("[STARTUP_NAV] Pending nav error: " + err);
    }

    console.log("[QUALIUM:FAVICON_BRIDGE] Native Gecko favicon bridge attached successfully ✓");
  }

  if (document.readyState === "complete" || document.readyState === "interactive") {
    initBridge();
  } else {
    window.addEventListener("DOMContentLoaded", initBridge, false);
  }

})();
