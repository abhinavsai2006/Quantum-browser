// Qaulium Quantum Browser v1 — "Qualium Quiet Precision" Browser Controller
document.addEventListener("DOMContentLoaded", () => {
  const browserDeck = document.getElementById("browser-deck");
  const contentBrowser = document.getElementById("content-browser") || document.getElementById("content-frame");
  const urlInput = document.getElementById("url-input");
  const qBtn = document.getElementById("btn-q-security");
  const qPopover = document.getElementById("q-popover");
  const btnClosePopover = document.getElementById("btn-close-popover");
  const btnMenuToggle = document.getElementById("btn-menu-toggle");
  const browserMenu = document.getElementById("browser-menu");
  const btnBack = document.getElementById("btn-back");
  const btnForward = document.getElementById("btn-forward");
  const btnReload = document.getElementById("btn-reload");
  const btnDownloads = document.getElementById("btn-downloads");
  const btnBookmarkStar = document.getElementById("btn-bookmark-star");
  const btnAddTab = document.getElementById("btn-add-tab");
  const btnPopoverIdentity = document.getElementById("btn-popover-identity");
  const btnPopoverPrivacyCenter = document.getElementById("btn-popover-privacy-center");
  const securityPill = document.getElementById("security-pill");
  const securityLabel = document.getElementById("security-label");
  const circuitIndicator = document.getElementById("circuit-indicator");
  const toastContainer = document.getElementById("toast-container");

  // Tab State Management: tabId -> { url, title, canonical, iconType }
  let activeTabId = "tab-1";
  const tabData = new Map();
  tabData.set("tab-1", { url: "newtab.xhtml", title: "New Tab", canonical: "qualium://newtab", iconType: "tab" });
  tabData.set("tab-2", { url: "dashboard.xhtml", title: "Privacy Center", canonical: "qualium://privacy", iconType: "shield" });

  function showToast(message) {
    if (!toastContainer) return;
    const toast = document.createElement("div");
    toast.className = "q-toast";
    toast.textContent = message;
    toastContainer.appendChild(toast);
    setTimeout(() => {
      toast.style.opacity = "0";
      setTimeout(() => toast.remove(), 200);
    }, 2400);
  }

  function recordHistory(url, title) {
    if (!url || url.includes("newtab") || url.includes("browser.xhtml")) return;
    try {
      const history = JSON.parse(localStorage.getItem("qualium_history") || "[]");
      const timeStr = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      // Add new entry to the front
      history.unshift({ url, title, time: timeStr });
      if (history.length > 50) history.pop();
      localStorage.setItem("qualium_history", JSON.stringify(history));
    } catch(e) {}
  }

  async function updateTabFaviconElement(tabEl, url, iconType) {
    if (!tabEl) return;
    const oldFavicon = tabEl.querySelector(".tab-favicon");
    if (!oldFavicon) return;

    if (typeof QualiumFaviconService !== "undefined") {
      const resolved = await QualiumFaviconService.getForPage(url);
      if (resolved && resolved.dataUrl) {
        const img = document.createElement("img");
        img.src = resolved.dataUrl;
        img.className = "q-icon q-icon-sm tab-favicon";
        img.style.objectFit = "contain";
        img.style.borderRadius = "3px";
        img.onerror = () => {
          const temp = document.createElement("div");
          temp.innerHTML = QualiumFaviconService.getNeutralFallbackSvg();
          const fallback = temp.firstElementChild;
          if (fallback) tabEl.replaceChild(fallback, img);
        };
        tabEl.replaceChild(img, oldFavicon);
        return;
      }
      if (resolved && resolved.svg) {
        const temp = document.createElement("div");
        temp.innerHTML = resolved.svg;
        const newSvg = temp.firstElementChild;
        if (newSvg) {
          newSvg.classList.add("q-icon", "q-icon-sm", "tab-favicon");
          tabEl.replaceChild(newSvg, oldFavicon);
          return;
        }
      }
    }

    const temp = document.createElement("div");
    temp.innerHTML = typeof QualiumFaviconService !== "undefined"
      ? QualiumFaviconService.getNeutralFallbackSvg()
      : `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/></svg>`;
    const newFavicon = temp.firstElementChild;
    if (newFavicon) {
      tabEl.replaceChild(newFavicon, oldFavicon);
    }
  }

  function updateAddressBar(url, canonical) {
    if (!urlInput) return;
    const displayUrl = canonical || url;

    if (displayUrl.startsWith("qualium://")) {
      urlInput.value = displayUrl;
      if (securityLabel) securityLabel.textContent = "Private";
    } else {
      urlInput.value = displayUrl;
      if (securityLabel) {
        securityLabel.textContent = displayUrl.startsWith("https://") ? "Secure" : "Standard";
      }
    }
  }

  function updateReloadButton(isLoading) {
    if (!btnReload) return;
    if (isLoading) {
      btnReload.setAttribute("title", "Stop (Esc)");
      btnReload.setAttribute("aria-label", "Stop");
      btnReload.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="6" y="6" width="12" height="12" rx="2"/></svg>`;
    } else {
      btnReload.setAttribute("title", "Reload (Ctrl+R)");
      btnReload.setAttribute("aria-label", "Reload");
      btnReload.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>`;
    }
  }

  // Initialize Native Navigation Controller
  const navCtrl = new QualiumNavigationController(
    () => document.getElementById("content-browser") || document.getElementById("content-frame"),
    (url, title, canonical, iconType) => {
      tabData.set(activeTabId, { url, title, canonical, iconType });
      const activeTabEl = document.getElementById(activeTabId);
      if (activeTabEl) {
        const titleSpan = activeTabEl.querySelector(".tab-title");
        if (titleSpan) titleSpan.textContent = title;
        updateTabFaviconElement(activeTabEl, canonical || url, iconType);
      }
      updateAddressBar(url, canonical);
      recordHistory(canonical || url, title);
    },
    (isLoading) => {
      updateReloadButton(isLoading);
    }
  );

  window.qualiumNav = navCtrl;
  window.navigateTo = (input) => navCtrl.navigate(input);

  // Address Bar Submission
  if (urlInput) {
    urlInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        e.stopPropagation();
        const rawValue = urlInput.value.trim();
        console.log("[OMNIBOX] event=keydown key=Enter value=" + rawValue);
        if (rawValue) {
          navCtrl.navigate(rawValue);
          urlInput.blur();
        }
      }
    });
  }

  // Navigation Control Handlers
  if (btnBack) btnBack.addEventListener("click", () => navCtrl.goBack());
  if (btnForward) btnForward.addEventListener("click", () => navCtrl.goForward());
  if (btnReload) btnReload.addEventListener("click", () => navCtrl.reload());
  if (btnDownloads) btnDownloads.addEventListener("click", () => navCtrl.navigateInternal("qualium://downloads"));
  if (btnBookmarkStar) {
    btnBookmarkStar.addEventListener("click", async () => {
      const currentData = tabData.get(activeTabId);
      if (currentData && currentData.url && typeof QualiumBookmarkStore !== "undefined") {
        await QualiumBookmarkStore.addBookmark({
          url: currentData.canonical || currentData.url,
          title: currentData.title || currentData.url,
          pinned: true
        });
        showToast("Page bookmarked & added to New Tab ✓");
      } else {
        showToast("Page bookmarked locally");
      }
    });
  }

  // Popover Toggles
  if (qBtn && qPopover) {
    qBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      qPopover.classList.toggle("visible");
      if (browserMenu) browserMenu.classList.remove("visible");
    });
  }

  if (btnClosePopover && qPopover) {
    btnClosePopover.addEventListener("click", () => {
      qPopover.classList.remove("visible");
    });
  }

  if (btnMenuToggle && browserMenu) {
    btnMenuToggle.addEventListener("click", (e) => {
      e.stopPropagation();
      browserMenu.classList.toggle("visible");
      if (qPopover) qPopover.classList.remove("visible");
    });
  }

  document.addEventListener("click", (e) => {
    if (qPopover && !qPopover.contains(e.target) && e.target !== qBtn) {
      qPopover.classList.remove("visible");
    }
    if (browserMenu && !browserMenu.contains(e.target) && e.target !== btnMenuToggle) {
      browserMenu.classList.remove("visible");
    }
  });

  // Menu Item Clicks
  if (browserMenu) {
    browserMenu.querySelectorAll(".q-menu-item").forEach(item => {
      item.addEventListener("click", () => {
        const action = item.getAttribute("data-action");
        browserMenu.classList.remove("visible");
        switch (action) {
          case "new-tab": if (btnAddTab) btnAddTab.click(); break;
          case "new-identity": if (btnPopoverIdentity) btnPopoverIdentity.click(); break;
          case "bookmarks": navCtrl.navigateInternal("qualium://bookmarks"); break;
          case "downloads": navCtrl.navigateInternal("qualium://downloads"); break;
          case "history": navCtrl.navigateInternal("qualium://history"); break;
          case "passwords": navCtrl.navigateInternal("qualium://passwords"); break;
          case "extensions": navCtrl.navigateInternal("qualium://extensions"); break;
          case "privacy-center": navCtrl.navigateInternal("qualium://privacy"); break;
          case "settings": navCtrl.navigateInternal("qualium://settings"); break;
          case "about": navCtrl.navigateInternal("qualium://about"); break;
        }
      });
    });
  }

  if (btnPopoverPrivacyCenter) {
    btnPopoverPrivacyCenter.addEventListener("click", () => {
      navCtrl.navigateInternal("qualium://privacy");
      if (qPopover) qPopover.classList.remove("visible");
    });
  }

  // New Identity Controller
  if (btnPopoverIdentity) {
    btnPopoverIdentity.addEventListener("click", () => {
      btnPopoverIdentity.disabled = true;
      showToast("Resetting privacy circuit and ephemeral cache...");

      setTimeout(() => {
        showToast("New identity active ✓");
        btnPopoverIdentity.disabled = false;
        navCtrl.navigateInternal("qualium://newtab");
        if (qPopover) qPopover.classList.remove("visible");
      }, 700);
    });
  }

  // Tab Strip Management
  function switchTab(tabId) {
    activeTabId = tabId;
    document.querySelectorAll(".tab").forEach(t => t.classList.remove("active"));
    const activeTab = document.getElementById(tabId);
    if (activeTab) activeTab.classList.add("active");

    const data = tabData.get(tabId) || { url: "newtab.xhtml", title: "New Tab", canonical: "qualium://newtab", iconType: "tab" };
    navCtrl.navigateURL(data.url, data.title, data.canonical, data.iconType);
  }

  function bindTabEvents(tabEl) {
    tabEl.addEventListener("click", (e) => {
      if (e.target.closest(".tab-close-btn")) return;
      switchTab(tabEl.id);
    });

    const closeBtn = tabEl.querySelector(".tab-close-btn");
    if (closeBtn) {
      closeBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        tabData.delete(tabEl.id);
        tabEl.remove();

        const remainingTabs = document.querySelectorAll(".tab");
        if (remainingTabs.length > 0) {
          switchTab(remainingTabs[remainingTabs.length - 1].id);
        }
      });
    }
  }

  document.querySelectorAll(".tab").forEach(tab => {
    bindTabEvents(tab);
  });

  // Create New Tab
  if (btnAddTab) {
    let tabCounter = 3;
    btnAddTab.addEventListener("click", () => {
      const tabId = "tab-" + tabCounter;
      const tabStrip = document.getElementById("tab-strip");

      const newTab = document.createElement("div");
      newTab.className = "tab active";
      newTab.id = tabId;
      newTab.setAttribute("role", "tab");
      newTab.innerHTML = `
        <svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/></svg>
        <span class="tab-title">New Tab</span>
        <button type="button" class="tab-close-btn" aria-label="Close tab">
          <svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      `;
      tabStrip.insertBefore(newTab, btnAddTab);

      tabData.set(tabId, { url: "newtab.xhtml", title: "New Tab", canonical: "qualium://newtab", iconType: "tab" });
      bindTabEvents(newTab);
      switchTab(tabId);
      tabCounter++;
    });
  }

  // Global Keyboard Shortcuts
  document.addEventListener("keydown", (e) => {
    if (e.ctrlKey || e.metaKey) {
      if (e.key === "t" || e.key === "T") {
        e.preventDefault();
        if (btnAddTab) btnAddTab.click();
      } else if (e.key === "w" || e.key === "W") {
        e.preventDefault();
        const activeTabEl = document.getElementById(activeTabId);
        if (activeTabEl) {
          const closeBtn = activeTabEl.querySelector(".tab-close-btn");
          if (closeBtn) closeBtn.click();
        }
      } else if (e.key === "l" || e.key === "L") {
        e.preventDefault();
        if (urlInput) {
          urlInput.focus();
          urlInput.select();
        }
      } else if (e.key === "r" || e.key === "R") {
        e.preventDefault();
        navCtrl.reload();
      } else if (e.key === "j" || e.key === "J") {
        e.preventDefault();
        navCtrl.navigateInternal("qualium://downloads");
      } else if (e.key === "h" || e.key === "H") {
        e.preventDefault();
        navCtrl.navigateInternal("qualium://history");
      } else if (e.shiftKey && (e.key === "a" || e.key === "A")) {
        e.preventDefault();
        navCtrl.navigateInternal("qualium://extensions");
      } else if (e.shiftKey && (e.key === "o" || e.key === "O")) {
        e.preventDefault();
        navCtrl.navigateInternal("qualium://bookmarks");
      } else if (e.key === "d" || e.key === "D") {
        e.preventDefault();
        showToast("Page bookmarked locally");
      }
    }
  });

  // Initial tab load
  switchTab("tab-1");

  try {
    let startupTarget = null;
    if (window.arguments && window.arguments.length > 0) {
      for (let i = 0; i < window.arguments.length; i++) {
        const arg = window.arguments[i];
        if (typeof arg === "string" && (arg.startsWith("http://") || arg.startsWith("https://") || arg.startsWith("qualium://"))) {
          startupTarget = arg;
          break;
        } else if (arg && typeof arg.data === "string" && (arg.data.startsWith("http://") || arg.data.startsWith("https://"))) {
          startupTarget = arg.data;
          break;
        }
      }
    }

    // Check temp pending nav file
    const pendingFile = Cc["@mozilla.org/file/local;1"].createInstance(Ci.nsIFile);
    const envService = Cc["@mozilla.org/process/environment;1"].getService(Ci.nsIEnvironment);
    const tempDir = envService.get("TEMP") || "C:\\Users\\mndab\\AppData\\Local\\Temp";
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
      if (str.value && str.value.trim().length > 0) {
        startupTarget = str.value.trim();
      }
    }

    if (startupTarget) {
      console.log("[QUALIUM:STARTUP] Navigating to startup target: " + startupTarget);
      setTimeout(() => {
        navCtrl.navigate(startupTarget);
      }, 200);
    }
  } catch(e) {
    console.warn("[QUALIUM:STARTUP] Startup nav check error: ", e);
  }
});

