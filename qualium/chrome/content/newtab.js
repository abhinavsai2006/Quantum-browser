// Qaulium Quantum Browser v5 — Official New Tab Controller

const DEFAULT_SHORTCUTS = [
  { id: "def-privacy", title: "Privacy", url: "dashboard.xhtml", type: "privacy" },
  { id: "def-bookmarks", title: "Bookmarks", url: "bookmarks.xhtml", type: "bookmarks" },
  { id: "def-downloads", title: "Downloads", url: "downloads.xhtml", type: "downloads" },
  { id: "def-github", title: "GitHub", url: "https://github.com", type: "github" },
  { id: "def-wikipedia", title: "Wikipedia", url: "https://wikipedia.org", type: "wikipedia" },
  { id: "def-settings", title: "Settings", url: "settings.xhtml", type: "settings" }
];

const SVG_ICONS = {
  privacy: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#34d399" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><path d="M9 12l2 2 4-4"/></svg>`,
  bookmarks: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#fbbf24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>`,
  downloads: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#38bdf8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>`,
  github: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#f1f5f9" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/></svg>`,
  wikipedia: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#60a5fa" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>`,
  settings: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>`,
  add: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>`
};

function createSVGElement(svgString) {
  const parser = new DOMParser();
  const doc = parser.parseFromString(svgString, "image/svg+xml");
  return document.importNode(doc.documentElement, true);
}

function getStoredShortcuts() {
  try {
    const raw = localStorage.getItem("qaulium_shortcuts");
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length > 0) return parsed;
    }
  } catch (e) {
    console.warn("Could not read qaulium_shortcuts from localStorage", e);
  }
  return DEFAULT_SHORTCUTS;
}

function saveStoredShortcuts(shortcuts) {
  try {
    localStorage.setItem("qaulium_shortcuts", JSON.stringify(shortcuts));
  } catch (e) {
    console.warn("Could not save qaulium_shortcuts", e);
  }
}

function renderShortcuts() {
  const container = document.getElementById("shortcuts-grid");
  if (!container) return;

  const shortcuts = getStoredShortcuts();
  container.innerHTML = "";

  shortcuts.forEach(item => {
    const card = document.createElement("a");
    card.className = "shortcut-card";
    card.href = item.url;
    card.title = item.title;

    const iconWrap = document.createElement("div");
    iconWrap.className = "shortcut-icon-wrap" + (item.type ? ` icon-${item.type}` : "");

    if (item.type && SVG_ICONS[item.type]) {
      iconWrap.appendChild(createSVGElement(SVG_ICONS[item.type]));
    } else {
      // Dynamic Letter Badge for custom shortcut
      const initial = (item.title || "W").charAt(0).toUpperCase();
      const badge = document.createElement("div");
      badge.className = "shortcut-avatar-badge";
      const hue = Math.abs(item.title.split("").reduce((acc, c) => acc + c.charCodeAt(0), 0) * 47) % 360;
      badge.style.background = `linear-gradient(135deg, hsl(${hue}, 70%, 55%), hsl(${(hue + 40) % 360}, 75%, 45%))`;
      badge.textContent = initial;
      iconWrap.appendChild(badge);
    }

    const titleSpan = document.createElement("span");
    titleSpan.className = "shortcut-title";
    titleSpan.textContent = item.title;

    card.appendChild(iconWrap);
    card.appendChild(titleSpan);

    // If it is a custom item (not default), add delete button
    if (!item.id.startsWith("def-")) {
      const delBtn = document.createElement("button");
      delBtn.className = "shortcut-del-btn";
      delBtn.title = "Remove shortcut";
      delBtn.innerHTML = "&#10005;";
      delBtn.addEventListener("click", (e) => {
        e.preventDefault();
        e.stopPropagation();
        deleteShortcut(item.id);
      });
      card.appendChild(delBtn);
    }

    card.addEventListener("click", (e) => {
      const href = card.getAttribute("href");
      if (!href) return;
      if (href.startsWith("http://") || href.startsWith("https://")) {
        return; // standard navigation
      }
      e.preventDefault();
      window.location.href = href;
    });

    container.appendChild(card);
  });

  // Always append the [+ Add Shortcut] tile
  const addCard = document.createElement("div");
  addCard.className = "shortcut-card shortcut-add-card";
  addCard.title = "Add custom shortcut";

  const addIconWrap = document.createElement("div");
  addIconWrap.className = "shortcut-icon-wrap";
  addIconWrap.appendChild(createSVGElement(SVG_ICONS.add));

  const addTitle = document.createElement("span");
  addTitle.className = "shortcut-title";
  addTitle.textContent = "Add";

  addCard.appendChild(addIconWrap);
  addCard.appendChild(addTitle);

  addCard.addEventListener("click", () => {
    openAddShortcutModal();
  });

  container.appendChild(addCard);
}

function openAddShortcutModal() {
  const modal = document.getElementById("add-shortcut-modal");
  if (!modal) return;
  modal.classList.add("open");
  const nameInput = document.getElementById("shortcut-name-input");
  const urlInput = document.getElementById("shortcut-url-input");
  if (nameInput) {
    nameInput.value = "";
    nameInput.focus();
  }
  if (urlInput) urlInput.value = "";
}

function closeAddShortcutModal() {
  const modal = document.getElementById("add-shortcut-modal");
  if (modal) modal.classList.remove("open");
}

function saveCustomShortcut() {
  const nameInput = document.getElementById("shortcut-name-input");
  const urlInput = document.getElementById("shortcut-url-input");
  if (!nameInput || !urlInput) return;

  const name = nameInput.value.trim();
  let url = urlInput.value.trim();

  if (!name || !url) {
    alert("Please enter both a shortcut name and URL.");
    return;
  }

  if (!/^https?:\/\//i.test(url) && !url.endsWith(".xhtml") && !url.endsWith(".html")) {
    url = "https://" + url;
  }

  const shortcuts = getStoredShortcuts();
  shortcuts.push({
    id: "custom-" + Date.now(),
    title: name,
    url: url
  });

  saveStoredShortcuts(shortcuts);
  closeAddShortcutModal();
  renderShortcuts();
}

function deleteShortcut(id) {
  let shortcuts = getStoredShortcuts();
  shortcuts = shortcuts.filter(s => s.id !== id);
  saveStoredShortcuts(shortcuts);
  renderShortcuts();
}

function handleNewTabSearch() {
  const searchInput = document.getElementById("search-input");
  if (!searchInput) return;
  const rawInput = searchInput.value.trim();
  if (!rawInput) return;

  let targetUrl = "";
  if (typeof classifyInput === "function") {
    const classified = classifyInput(rawInput);
    targetUrl = classified.url;
  } else {
    if (/^https?:\/\//i.test(rawInput)) {
      targetUrl = rawInput;
    } else if (/^[a-zA-Z0-9-]+\.[a-zA-Z]{2,}(\/.*)?$/.test(rawInput)) {
      targetUrl = "https://" + rawInput;
    } else {
      const savedEngine = localStorage.getItem("qualium_search_engine") || "duckduckgo";
      const engines = {
        google: "https://www.google.com/search?q=",
        duckduckgo: "https://duckduckgo.com/?q=",
        brave: "https://search.brave.com/search?q=",
        bing: "https://www.bing.com/search?q="
      };
      const base = engines[savedEngine] || engines.duckduckgo;
      targetUrl = base + encodeURIComponent(rawInput);
    }
  }

  if (targetUrl) {
    window.location.href = targetUrl;
  }
}

document.addEventListener("DOMContentLoaded", () => {
  // 1. Subscribe to Live Runtime State
  if (typeof QualiumRuntimeState !== "undefined") {
    QualiumRuntimeState.subscribe(state => {
      const headline = document.getElementById("circuit-headline");
      const dot = document.getElementById("circuit-dot");
      const guard = document.getElementById("circuit-guard");
      const relay = document.getElementById("circuit-relay");
      const exit = document.getElementById("circuit-exit");

      if (headline && dot) {
        if (state.connectionState === "CONNECTED" && state.circuitState === "ACTIVE") {
          headline.textContent = "POST-QUANTUM CIRCUIT ACTIVE";
          headline.style.color = "#f1f5f9";
          dot.style.background = "#34d399";
          dot.style.boxShadow = "0 0 10px #34d399";
        } else if (state.connectionState === "CONNECTING") {
          headline.textContent = "CIRCUIT CONNECTING";
          headline.style.color = "#f59e0b";
          dot.style.background = "#f59e0b";
          dot.style.boxShadow = "0 0 8px #f59e0b";
        } else {
          headline.textContent = "CIRCUIT UNAVAILABLE";
          headline.style.color = "#ef4444";
          dot.style.background = "#ef4444";
          dot.style.boxShadow = "0 0 8px #ef4444";
        }
      }

      if (guard) guard.textContent = state.guard;
      if (relay) relay.textContent = state.relay;
      if (exit) exit.textContent = state.exit;
    });
  }

  // 2. Render all Shortcuts (Defaults + Custom + Add Tile)
  renderShortcuts();

  // 3. Modal Keydown Listener (Escape / Enter)
  document.addEventListener("keydown", (e) => {
    const modal = document.getElementById("add-shortcut-modal");
    if (modal && modal.classList.contains("open")) {
      if (e.key === "Escape") {
        closeAddShortcutModal();
      } else if (e.key === "Enter") {
        e.preventDefault();
        saveCustomShortcut();
      }
    }
  });

  // 4. Search Form listener
  const searchForm = document.getElementById("search-form");
  if (searchForm) {
    searchForm.addEventListener("submit", (e) => {
      e.preventDefault();
      handleNewTabSearch();
    });
  }
});
