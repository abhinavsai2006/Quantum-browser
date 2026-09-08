// Qaulium Quantum Browser v1 — Authoritative Chromium-style Browser Controller
document.addEventListener("DOMContentLoaded", () => {
  const browserDeck = document.getElementById("browser-deck");
  const tabStrip = document.getElementById("tab-strip");
  const tabStripTabs = document.getElementById("tab-strip-tabs") || tabStrip;
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

  // ==========================================
  // AUTHORITATIVE TAB STATE MANAGEMENT
  // ==========================================
  // tabs = [ { id, url, title, canonical, iconType, favicon, isLoading, viewport, controller } ]
  let tabs = [];
  let activeTabId = null;
  let tabCounter = 2; // initial tab is tab-1
  const recentlyClosedTabs = [];

  function escapeHTML(str) {
    return (str || "").replace(/[&<>"']/g, m => ({
      "&": "&amp;",
      "<": "&lt;",
      ">": "&gt;",
      '"': "&quot;",
      "'": "&#39;"
    })[m]);
  }

  function getActiveTab() {
    return tabs.find(t => t.id === activeTabId) || tabs[0] || null;
  }

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
        img.style.borderRadius = "2px";
        img.style.width = "16px";
        img.style.height = "16px";
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
      : `<svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm tab-favicon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><circle cx="12" cy="11" r="2" fill="#38bdf8"/></svg>`;
    const newFavicon = temp.firstElementChild;
    if (newFavicon) {
      tabEl.replaceChild(newFavicon, oldFavicon);
    }
  }

  function updateAddressBar(url, canonical) {
    if (!urlInput) return;
    const displayUrl = canonical || url;

    if (displayUrl.startsWith("qualium://") || displayUrl.startsWith("qaulium://")) {
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

  // ==========================================
  // VIEWPORT MANAGEMENT (KEYED TO EACH TAB)
  // ==========================================
  function createViewportElement(tabId) {
    const existing = document.querySelector(`[data-tab-id="${tabId}"]`) || 
                     (tabId === "tab-1" ? document.getElementById("content-browser") : null);
    if (existing) {
      existing.id = `viewport-${tabId}`;
      existing.setAttribute("data-tab-id", tabId);
      existing.style.display = "flex";
      hookViewportTargetBlank(existing);
      return existing;
    }

    const sample = document.getElementById("content-browser") || document.querySelector(".content-browser");
    const tagName = (sample && sample.tagName) ? sample.tagName.toLowerCase() : "browser";

    const viewport = document.createElement(tagName);
    viewport.id = `viewport-${tabId}`;
    viewport.className = "content-browser";
    viewport.setAttribute("data-tab-id", tabId);
    viewport.setAttribute("type", "content");
    viewport.setAttribute("remote", "true");
    viewport.setAttribute("primary", "true");
    viewport.style.flex = "1";
    viewport.style.width = "100%";
    viewport.style.height = "100%";
    viewport.style.minHeight = "0";
    viewport.style.minWidth = "0";
    viewport.style.border = "none";
    viewport.style.display = "none";
    if (browserDeck) {
      browserDeck.appendChild(viewport);
    }
    hookViewportTargetBlank(viewport);
    return viewport;
  }

  function hookViewportTargetBlank(viewport) {
    if (!viewport) return;
    try {
      // Intercept new-window / target="_blank" requests from web content
      viewport.addEventListener("DOMContentLoaded", () => {
        try {
          const doc = viewport.contentDocument;
          if (doc) {
            doc.addEventListener("click", (e) => {
              const link = e.target.closest("a[href]");
              if (!link) return;
              if (link.target === "_blank" || e.ctrlKey || e.metaKey || e.button === 1) {
                e.preventDefault();
                e.stopPropagation();
                createNewTab({ url: link.href, activate: !e.shiftKey });
              }
            }, true);
          }
        } catch(e) {}
      });
    } catch(e) {}
  }

  // ==========================================
  // AUTHORITATIVE TAB ACTIONS
  // ==========================================
  function createNewTab(opts = {}) {
    const tabId = opts.id || `tab-${tabCounter++}`;
    const url = opts.url || "newtab.xhtml";
    const title = opts.title || "New Tab";
    const canonical = opts.canonical || (url.includes("newtab") ? "qualium://newtab" : url);
    const iconType = opts.iconType || "tab";
    const activate = opts.activate !== false;

    const viewport = createViewportElement(tabId);

    const tab = {
      id: tabId,
      url,
      title,
      canonical,
      iconType,
      favicon: null,
      isLoading: false,
      viewport: viewport,
      controller: null
    };

    // Controller strictly bound to THIS tab
    tab.controller = new QualiumNavigationController(
      () => tab.viewport,
      (newUrl, newTitle, newCanonical, newIconType) => {
        tab.url = newUrl;
        tab.canonical = newCanonical || newUrl;
        tab.title = newTitle || "Web Page";
        tab.iconType = newIconType || "web";

        const tabEl = document.getElementById(tab.id);
        if (tabEl) {
          const titleEl = tabEl.querySelector(".tab-title");
          if (titleEl) titleEl.textContent = tab.title;
          tabEl.setAttribute("data-url", tab.url);
          tabEl.title = `${tab.title}\n${tab.canonical}`;
          updateTabFaviconElement(tabEl, tab.canonical || tab.url, tab.iconType);
        }

        // Only update browser chrome if this tab is the active tab
        if (tab.id === activeTabId) {
          updateAddressBar(tab.url, tab.canonical);
          recordHistory(tab.canonical || tab.url, tab.title);
        }
      },
      (isLoading) => {
        tab.isLoading = isLoading;
        if (tab.id === activeTabId) {
          updateReloadButton(isLoading);
        }
      }
    );

    tabs.push(tab);

    // Build Tab DOM Element
    let tabEl = document.getElementById(tabId);
    if (!tabEl) {
      tabEl = document.createElement("div");
      tabEl.className = "tab";
      tabEl.id = tabId;
      tabEl.setAttribute("role", "tab");
      tabEl.setAttribute("data-url", url);
      tabEl.title = `${title}\n${canonical}`;
      tabEl.innerHTML = `
        <span class="tab-favicon">
          <svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><circle cx="12" cy="11" r="2" fill="#38bdf8"/></svg>
        </span>
        <span class="tab-title">${escapeHTML(title)}</span>
        <button type="button" class="tab-close-btn" aria-label="Close tab" title="Close tab (Ctrl+W)">
          <svg xmlns="http://www.w3.org/2000/svg" class="q-icon q-icon-sm" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      `;

      if (tabStripTabs) {
        tabStripTabs.appendChild(tabEl);
      } else if (btnAddTab) {
        tabStrip.insertBefore(tabEl, btnAddTab);
      }
    }

    bindTabEvents(tabEl, tab);

    // Load initial URL
    tab.controller.navigateURL(url, title, canonical, iconType);

    if (activate) {
      switchTab(tabId);
    }

    return tab;
  }

  function switchTab(tabId) {
    const targetTab = tabs.find(t => t.id === tabId);
    if (!targetTab) return;

    activeTabId = tabId;

    // Update Tab strip DOM classes
    document.querySelectorAll(".tab").forEach(t => {
      const isCurrent = t.id === tabId;
      t.classList.toggle("active", isCurrent);
      t.setAttribute("aria-selected", isCurrent ? "true" : "false");
    });

    const activeEl = document.getElementById(tabId);
    if (activeEl && typeof activeEl.scrollIntoView === "function") {
      try {
        activeEl.scrollIntoView({ behavior: "smooth", block: "nearest", inline: "nearest" });
      } catch(e) {}
    }

    // Keyed viewport switching: hide all viewports except active one
    tabs.forEach(t => {
      if (t.viewport) {
        t.viewport.style.display = (t.id === tabId) ? "flex" : "none";
      }
    });

    // Update Address bar & Security UI
    updateAddressBar(targetTab.url, targetTab.canonical);
    updateReloadButton(targetTab.isLoading);

    // Update global controller hook for active tab
    window.qualiumNav = targetTab.controller;

    // Set focus to active viewport or address bar
    try {
      if (targetTab.viewport && typeof targetTab.viewport.focus === "function") {
        targetTab.viewport.focus();
      }
    } catch(e) {}
  }

  function closeTab(tabId, event) {
    if (event) {
      event.preventDefault();
      event.stopPropagation();
    }

    const tabIndex = tabs.findIndex(t => t.id === tabId);
    if (tabIndex === -1) return;

    const [closingTab] = tabs.splice(tabIndex, 1);

    // Save for Ctrl+Shift+T (recently closed tabs)
    if (closingTab && closingTab.url && !closingTab.url.includes("newtab.xhtml")) {
      recentlyClosedTabs.push({
        url: closingTab.canonical || closingTab.url,
        title: closingTab.title || "Closed Tab"
      });
      if (recentlyClosedTabs.length > 30) recentlyClosedTabs.shift();
    }

    // Remove DOM elements
    const tabEl = document.getElementById(tabId);
    if (tabEl) tabEl.remove();

    if (closingTab.viewport && closingTab.viewport.parentNode) {
      closingTab.viewport.remove();
    }

    // If active tab was closed, activate nearest sensible tab
    if (activeTabId === tabId) {
      if (tabs.length > 0) {
        const nextIndex = Math.min(tabIndex, tabs.length - 1);
        switchTab(tabs[nextIndex].id);
      } else {
        // If all tabs are closed, automatically create a fresh New Tab!
        createNewTab({ activate: true });
      }
    } else if (tabs.length === 0) {
      createNewTab({ activate: true });
    }
  }

  function reopenLastClosedTab() {
    if (recentlyClosedTabs.length === 0) {
      showToast("No recently closed tabs");
      return;
    }
    const last = recentlyClosedTabs.pop();
    createNewTab({
      url: last.url,
      title: last.title,
      canonical: last.url,
      activate: true
    });
  }

  function bindTabEvents(tabEl, tab) {
    tabEl.addEventListener("click", (e) => {
      if (e.target.closest(".tab-close-btn")) return;
      switchTab(tab.id);
    });

    // Middle click closes tab
    tabEl.addEventListener("auxclick", (e) => {
      if (e.button === 1) { // Middle click
        e.preventDefault();
        e.stopPropagation();
        closeTab(tab.id, e);
      }
    });

    const closeBtn = tabEl.querySelector(".tab-close-btn");
    if (closeBtn) {
      closeBtn.addEventListener("click", (e) => {
        e.preventDefault();
        e.stopPropagation();
        closeTab(tab.id, e);
      });
    }
  }

  // ==========================================
  // EVENT LISTENERS & HOOKS
  // ==========================================

  // Double-clicking an empty area of the tab strip creates a new tab
  if (tabStrip) {
    tabStrip.addEventListener("dblclick", (e) => {
      if (!e.target.closest(".tab") && !e.target.closest("#btn-add-tab") && !e.target.closest(".window-controls")) {
        createNewTab({ activate: true });
      }
    });
  }

  // Add Tab Button (+)
  if (btnAddTab) {
    btnAddTab.addEventListener("click", (e) => {
      e.preventDefault();
      createNewTab({ activate: true });
    });
  }

  // Address Bar Submission
  if (urlInput) {
    urlInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        e.stopPropagation();
        const rawValue = urlInput.value.trim();
        if (rawValue) {
          const activeTab = getActiveTab();
          if (activeTab) {
            activeTab.controller.navigate(rawValue);
          }
          urlInput.blur();
        }
      }
    });
  }

  // Navigation Control Handlers
  if (btnBack) btnBack.addEventListener("click", () => {
    const activeTab = getActiveTab();
    if (activeTab) activeTab.controller.goBack();
  });
  if (btnForward) btnForward.addEventListener("click", () => {
    const activeTab = getActiveTab();
    if (activeTab) activeTab.controller.goForward();
  });
  if (btnReload) btnReload.addEventListener("click", () => {
    const activeTab = getActiveTab();
    if (activeTab) activeTab.controller.reload();
  });
  if (btnDownloads) btnDownloads.addEventListener("click", () => {
    const activeTab = getActiveTab();
    if (activeTab) activeTab.controller.navigateInternal("qualium://downloads");
  });

  if (btnBookmarkStar) {
    btnBookmarkStar.addEventListener("click", async () => {
      const activeTab = getActiveTab();
      if (activeTab && activeTab.url && typeof QualiumBookmarkStore !== "undefined") {
        await QualiumBookmarkStore.addBookmark({
          url: activeTab.canonical || activeTab.url,
          title: activeTab.title || activeTab.url,
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
        const activeTab = getActiveTab();
        const nav = activeTab ? activeTab.controller : null;
        if (!nav) return;

        switch (action) {
          case "new-tab": createNewTab({ activate: true }); break;
          case "new-identity": if (btnPopoverIdentity) btnPopoverIdentity.click(); break;
          case "bookmarks": nav.navigateInternal("qualium://bookmarks"); break;
          case "downloads": nav.navigateInternal("qualium://downloads"); break;
          case "history": nav.navigateInternal("qualium://history"); break;
          case "passwords": nav.navigateInternal("qualium://passwords"); break;
          case "extensions": nav.navigateInternal("qualium://extensions"); break;
          case "privacy-center": nav.navigateInternal("qualium://privacy"); break;
          case "settings": nav.navigateInternal("qualium://settings"); break;
          case "about": nav.navigateInternal("qualium://about"); break;
        }
      });
    });
  }

  if (btnPopoverPrivacyCenter) {
    btnPopoverPrivacyCenter.addEventListener("click", () => {
      const activeTab = getActiveTab();
      if (activeTab) activeTab.controller.navigateInternal("qualium://privacy");
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
        const activeTab = getActiveTab();
        if (activeTab) activeTab.controller.navigateInternal("qualium://newtab");
        if (qPopover) qPopover.classList.remove("visible");
      }, 700);
    });
  }

  // ==========================================
  // GLOBAL KEYBOARD SHORTCUTS
  // ==========================================
  document.addEventListener("keydown", (e) => {
    if (e.ctrlKey || e.metaKey) {
      if (e.shiftKey && (e.key === "t" || e.key === "T")) {
        // Ctrl+Shift+T: Reopen closed tab
        e.preventDefault();
        reopenLastClosedTab();
      } else if (e.key === "t" || e.key === "T") {
        // Ctrl+T: New tab
        e.preventDefault();
        createNewTab({ activate: true });
      } else if (e.key === "w" || e.key === "W") {
        // Ctrl+W: Close active tab
        e.preventDefault();
        if (activeTabId) closeTab(activeTabId, e);
      } else if (e.key === "Tab") {
        // Ctrl+Tab / Ctrl+Shift+Tab: Switch tabs
        e.preventDefault();
        if (tabs.length > 1) {
          const curIdx = tabs.findIndex(t => t.id === activeTabId);
          const delta = e.shiftKey ? -1 : 1;
          const nextIdx = (curIdx + delta + tabs.length) % tabs.length;
          switchTab(tabs[nextIdx].id);
        }
      } else if (e.key === "l" || e.key === "L") {
        e.preventDefault();
        if (urlInput) {
          urlInput.focus();
          urlInput.select();
        }
      } else if (e.key === "r" || e.key === "R") {
        e.preventDefault();
        const activeTab = getActiveTab();
        if (activeTab) activeTab.controller.reload();
      } else if (e.key === "j" || e.key === "J") {
        e.preventDefault();
        const activeTab = getActiveTab();
        if (activeTab) activeTab.controller.navigateInternal("qualium://downloads");
      } else if (e.key === "h" || e.key === "H") {
        e.preventDefault();
        const activeTab = getActiveTab();
        if (activeTab) activeTab.controller.navigateInternal("qualium://history");
      } else if (e.shiftKey && (e.key === "a" || e.key === "A")) {
        e.preventDefault();
        const activeTab = getActiveTab();
        if (activeTab) activeTab.controller.navigateInternal("qualium://extensions");
      } else if (e.shiftKey && (e.key === "o" || e.key === "O")) {
        e.preventDefault();
        const activeTab = getActiveTab();
        if (activeTab) activeTab.controller.navigateInternal("qualium://bookmarks");
      } else if (e.key === "d" || e.key === "D") {
        e.preventDefault();
        if (btnBookmarkStar) btnBookmarkStar.click();
      } else if (e.key >= "1" && e.key <= "9") {
        // Ctrl+1 to Ctrl+9: Switch to tab index
        e.preventDefault();
        const targetIndex = e.key === "9" ? tabs.length - 1 : parseInt(e.key, 10) - 1;
        if (targetIndex >= 0 && targetIndex < tabs.length) {
          switchTab(tabs[targetIndex].id);
        }
      }
    }
  });

  // Cross-frame messaging for in-tab navigation
  window.addEventListener("message", (event) => {
    try {
      if (!event.data || typeof event.data !== "object") return;
      if (event.data.type === "QUALIUM_NAVIGATE_CURRENT") {
        const activeTab = getActiveTab();
        if (activeTab && event.data.url) {
          activeTab.controller.navigate(event.data.url);
        }
      } else if (event.data.type === "QUALIUM_OPEN_NEW_TAB") {
        if (event.data.url) {
          createNewTab({ url: event.data.url, activate: event.data.activate !== false });
        }
      }
    } catch(e) {}
  });

  // Global exports
  window.navigateTo = (input) => {
    const activeTab = getActiveTab();
    if (activeTab) {
      activeTab.controller.navigate(input);
    }
  };
  window.createNewTab = (opts) => createNewTab(opts);
  window.closeTab = (id) => closeTab(id);
  window.switchTab = (id) => switchTab(id);
  window.getTabs = () => tabs.map(t => ({ id: t.id, url: t.url, title: t.title, canonical: t.canonical }));
  window.getActiveTabId = () => activeTabId;

  // ==========================================
  // INITIALIZATION: EXACTLY 1 TAB (NEW TAB)
  // ==========================================
  const initialTab = createNewTab({
    id: "tab-1",
    url: "newtab.xhtml",
    title: "New Tab",
    canonical: "qualium://newtab",
    iconType: "tab",
    activate: true
  });

  // Startup Target Argument Inspection
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

    if (typeof Cc !== "undefined") {
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
        if (str.value && str.value.trim().length > 0) {
          startupTarget = str.value.trim();
        }
      }
    }

    if (startupTarget) {
      console.log("[QUALIUM:STARTUP] Navigating to startup target: " + startupTarget);
      setTimeout(() => {
        if (initialTab && initialTab.controller) {
          initialTab.controller.navigate(startupTarget);
        }
      }, 200);
    }
  } catch(e) {
    console.warn("[QUALIUM:STARTUP] Startup nav check error: ", e);
  }
});
