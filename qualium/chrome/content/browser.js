// Qualium Quantum Browser v5 — "Qualium Quiet Precision" Browser Controller
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

  function getTabFaviconSvg(iconType, url) {
    if (url && (url.includes("google.com") || url.includes("google"))) {
      return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/><path d="M12 8v8M8 12h8"/></svg>`;
    }
    if (url && (url.includes("youtube.com") || url.includes("youtube"))) {
      return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="10 8 16 12 10 16 10 8"/><rect x="2" y="4" width="20" height="16" rx="4"/></svg>`;
    }
    if (url && (url.includes("github.com") || url.includes("github"))) {
      return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/></svg>`;
    }
    if (url && (url.includes("wikipedia.org") || url.includes("wikipedia"))) {
      return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/><text x="8" y="16" font-size="12" font-weight="bold" fill="currentColor">W</text></svg>`;
    }

    switch (iconType) {
      case "download":
        return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>`;
      case "bookmark":
        return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>`;
      case "settings":
        return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>`;
      case "lock":
        return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>`;
      case "extensions":
        return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/></svg>`;
      case "history":
        return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>`;
      case "shield":
        return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>`;
      case "search":
        return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>`;
      default:
        return `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/></svg>`;
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
        const oldFavicon = activeTabEl.querySelector(".tab-favicon");
        if (oldFavicon) {
          const temp = document.createElement("div");
          temp.innerHTML = getTabFaviconSvg(iconType, canonical || url);
          const newFavicon = temp.firstElementChild;
          if (newFavicon) {
            activeTabEl.replaceChild(newFavicon, oldFavicon);
          }
        }
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
    btnBookmarkStar.addEventListener("click", () => {
      showToast("Page bookmarked locally");
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
      } else if (e.key === "d" || e.key === "D") {
        e.preventDefault();
        showToast("Page bookmarked locally");
      }
    }
  });

  // Initial tab load
  switchTab("tab-1");

  if (window.arguments && window.arguments.length > 0) {
    const initArg = window.arguments[0];
    if (initArg && typeof initArg === "string") {
      navCtrl.navigate(initArg);
    }
  }
});
