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
          if (!url || url.startsWith("about:") || url.startsWith("chrome://qualium/")) {
            return;
          }

          // Check if tab already has an icon
          const tab = gBrowser.getTabForBrowser(aBrowser);
          if (tab) {
            const iconUrl = tab.getAttribute("image") || aBrowser.mIconURL;
            if (iconUrl) {
              persistGeckoFavicon(url, iconUrl);
            }
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

    // 3. Hook Star Button & Shortcut for Instant Bookmark Creation
    function hookBookmarkAction() {
      async function createBookmarkFromCurrentTab() {
        try {
          const currentTab = gBrowser.selectedTab;
          const currentBrowser = gBrowser.selectedBrowser;
          const url = currentBrowser && currentBrowser.currentURI ? currentBrowser.currentURI.spec : "";
          if (!url || url.startsWith("about:") || url.startsWith("chrome://")) return;

          const title = currentTab ? currentTab.getAttribute("label") : (currentBrowser ? currentBrowser.contentTitle : "");
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
    console.log("[QUALIUM:FAVICON_BRIDGE] Native Gecko favicon bridge attached successfully ✓");
  }

  if (document.readyState === "complete" || document.readyState === "interactive") {
    initBridge();
  } else {
    window.addEventListener("DOMContentLoaded", initBridge, false);
  }

})();
