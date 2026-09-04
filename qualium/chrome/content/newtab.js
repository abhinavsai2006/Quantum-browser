// Qaulium Quantum Browser v5 — Official New Tab Controller

// ── Search Engines ──
const ENGINES = {
  duckduckgo: { name: "DuckDuckGo", icon: "🦆", url: "https://duckduckgo.com/?q=" },
  google: { name: "Google", icon: "🔍", url: "https://www.google.com/search?q=" },
  brave: { name: "Brave Search", icon: "🦁", url: "https://search.brave.com/search?q=" },
  bing: { name: "Bing", icon: "🅱", url: "https://www.bing.com/search?q=" }
};

// ── Default Bookmarks ──
const DEFAULT_BOOKMARKS = [
  { id: "bm-ddg", title: "DuckDuckGo", url: "https://duckduckgo.com" },
  { id: "bm-wiki", title: "Wikipedia", url: "https://wikipedia.org" },
  { id: "bm-gh", title: "GitHub", url: "https://github.com" },
  { id: "bm-mdn", title: "MDN Web Docs", url: "https://developer.mozilla.org" },
  { id: "bm-reddit", title: "Reddit", url: "https://reddit.com" },
  { id: "bm-rust", title: "Rust Language", url: "https://www.rust-lang.org" }
];

// ── Default Shortcuts ──
const DEFAULT_SHORTCUTS = [
  { id: "sc-privacy", title: "Privacy", url: "dashboard.xhtml", icon: "shield", color: "#34d399" },
  { id: "sc-bookmarks", title: "Bookmarks", url: "bookmarks.xhtml", icon: "star", color: "#fbbf24" },
  { id: "sc-downloads", title: "Downloads", url: "downloads.xhtml", icon: "download", color: "#38bdf8" },
  { id: "sc-settings", title: "Settings", url: "settings.xhtml", icon: "settings", color: "#94a3b8" },
  { id: "sc-github", title: "GitHub", url: "https://github.com", icon: "github", color: "#e2e8f0" },
  { id: "sc-youtube", title: "YouTube", url: "https://youtube.com", icon: "play", color: "#ef4444" }
];

// ── Clock & Greeting ──
function updateClockAndGreeting() {
  const now = new Date();
  const hours = now.getHours();
  const mins = String(now.getMinutes()).padStart(2, "0");

  const clockEl = document.getElementById("nt-clock");
  if (clockEl) {
    clockEl.textContent = `${hours}:${mins}`;
  }

  const greetingEl = document.getElementById("nt-greeting");
  if (greetingEl) {
    let greeting = "Good evening";
    if (hours < 12) {
      greeting = "Good morning";
    } else if (hours < 17) {
      greeting = "Good afternoon";
    }
    greetingEl.textContent = greeting;
  }
}

// ── Search Engine Selector ──
function initSearchEngine() {
  const savedKey = localStorage.getItem("qaulium_search_engine") || "duckduckgo";
  setSearchEngine(savedKey);

  const engineBtn = document.getElementById("engine-btn");
  const dropdown = document.getElementById("engine-dropdown");

  if (engineBtn && dropdown) {
    engineBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      dropdown.classList.toggle("open");
    });

    document.addEventListener("click", () => {
      dropdown.classList.remove("open");
    });

    dropdown.querySelectorAll(".engine-opt").forEach(opt => {
      opt.addEventListener("click", () => {
        const engine = opt.getAttribute("data-engine");
        if (engine && ENGINES[engine]) {
          setSearchEngine(engine);
          dropdown.classList.remove("open");
        }
      });
    });
  }

  const searchForm = document.getElementById("nt-search-form");
  if (searchForm) {
    searchForm.addEventListener("submit", (e) => {
      e.preventDefault();
      handleSearch();
    });
  }
}

function setSearchEngine(key) {
  const engine = ENGINES[key] || ENGINES.duckduckgo;
  localStorage.setItem("qaulium_search_engine", key);
  const iconEl = document.getElementById("engine-icon");
  if (iconEl) {
    iconEl.textContent = engine.icon;
  }
  const inputEl = document.getElementById("nt-search-input");
  if (inputEl) {
    inputEl.placeholder = `Search with ${engine.name} or enter address…`;
  }
}

function handleSearch() {
  const input = document.getElementById("nt-search-input");
  if (!input) return;
  const raw = input.value.trim();
  if (!raw) return;

  let targetUrl = "";

  if (/^https?:\/\//i.test(raw)) {
    targetUrl = raw;
  } else if (!raw.includes(" ") && raw.includes(".") && !raw.startsWith(".")) {
    targetUrl = "https://" + raw;
  } else {
    const key = localStorage.getItem("qaulium_search_engine") || "duckduckgo";
    const engine = ENGINES[key] || ENGINES.duckduckgo;
    targetUrl = engine.url + encodeURIComponent(raw);
  }

  if (targetUrl) {
    window.location.href = targetUrl;
  }
}

// ── Bookmarks ──
function getStoredBookmarks() {
  try {
    const raw = localStorage.getItem("qaulium_bookmarks");
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length > 0) return parsed;
    }
  } catch (e) {}
  return DEFAULT_BOOKMARKS;
}

function saveStoredBookmarks(bms) {
  try {
    localStorage.setItem("qaulium_bookmarks", JSON.stringify(bms));
  } catch (e) {}
}

function renderBookmarks() {
  const container = document.getElementById("bookmark-list");
  if (!container) return;

  const bookmarks = getStoredBookmarks();
  container.innerHTML = "";

  bookmarks.forEach(bm => {
    const item = document.createElement("div");
    item.className = "bookmark-item";

    const link = document.createElement("a");
    link.className = "bookmark-link";
    link.href = bm.url;
    link.title = `${bm.title}\n${bm.url}`;

    const icon = document.createElement("span");
    icon.className = "bookmark-icon";
    icon.textContent = "🔖";

    const title = document.createElement("span");
    title.className = "bookmark-title";
    title.textContent = bm.title;

    link.appendChild(icon);
    link.appendChild(title);

    const delBtn = document.createElement("button");
    delBtn.className = "bookmark-del-btn";
    delBtn.title = "Delete bookmark";
    delBtn.textContent = "✕";
    delBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      deleteBookmark(bm.id);
    });

    item.appendChild(link);
    item.appendChild(delBtn);
    container.appendChild(item);
  });
}

function deleteBookmark(id) {
  let bms = getStoredBookmarks().filter(b => b.id !== id);
  saveStoredBookmarks(bms);
  renderBookmarks();
}

function openBmModal() {
  const modal = document.getElementById("bm-modal");
  if (modal) {
    modal.classList.add("open");
    const nameInput = document.getElementById("bm-name-input");
    const urlInput = document.getElementById("bm-url-input");
    if (nameInput) { nameInput.value = ""; nameInput.focus(); }
    if (urlInput) urlInput.value = "";
  }
}

function closeBmModal() {
  const modal = document.getElementById("bm-modal");
  if (modal) modal.classList.remove("open");
}

function saveBm() {
  const nameInput = document.getElementById("bm-name-input");
  const urlInput = document.getElementById("bm-url-input");
  if (!nameInput || !urlInput) return;

  const title = nameInput.value.trim();
  let url = urlInput.value.trim();

  if (!title || !url) {
    alert("Please enter both a title and URL.");
    return;
  }

  if (!/^https?:\/\//i.test(url) && !url.endsWith(".xhtml") && !url.endsWith(".html")) {
    url = "https://" + url;
  }

  const bms = getStoredBookmarks();
  bms.push({
    id: "bm-" + Date.now(),
    title,
    url
  });

  saveStoredBookmarks(bms);
  closeBmModal();
  renderBookmarks();
}

// ── Shortcuts ──
function getStoredShortcuts() {
  try {
    const raw = localStorage.getItem("qaulium_shortcuts");
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length > 0) return parsed;
    }
  } catch (e) {}
  return DEFAULT_SHORTCUTS;
}

function saveStoredShortcuts(scs) {
  try {
    localStorage.setItem("qaulium_shortcuts", JSON.stringify(scs));
  } catch (e) {}
}

function renderShortcuts() {
  const container = document.getElementById("shortcuts-grid");
  if (!container) return;

  const shortcuts = getStoredShortcuts();
  container.innerHTML = "";

  shortcuts.forEach(item => {
    const card = document.createElement("a");
    card.className = "nt-shortcut-card";
    card.href = item.url;
    card.title = item.title;

    const iconWrap = document.createElement("div");
    iconWrap.className = "nt-shortcut-icon";

    const initial = (item.title || "W").charAt(0).toUpperCase();
    const hue = Math.abs(item.title.split("").reduce((acc, c) => acc + c.charCodeAt(0), 0) * 53) % 360;
    iconWrap.style.background = item.color 
      ? `linear-gradient(135deg, ${item.color}33, ${item.color}66)`
      : `linear-gradient(135deg, hsl(${hue}, 65%, 45%), hsl(${(hue + 40) % 360}, 70%, 35%))`;
    iconWrap.style.color = item.color || "#fff";
    iconWrap.textContent = initial;

    const titleSpan = document.createElement("span");
    titleSpan.className = "nt-shortcut-title";
    titleSpan.textContent = item.title;

    card.appendChild(iconWrap);
    card.appendChild(titleSpan);

    if (!item.id.startsWith("sc-privacy") && !item.id.startsWith("sc-bookmarks") && !item.id.startsWith("sc-settings")) {
      const delBtn = document.createElement("button");
      delBtn.className = "nt-shortcut-del";
      delBtn.title = "Remove";
      delBtn.textContent = "✕";
      delBtn.addEventListener("click", (e) => {
        e.preventDefault();
        e.stopPropagation();
        deleteShortcut(item.id);
      });
      card.appendChild(delBtn);
    }

    container.appendChild(card);
  });

  // Append Add Shortcut button
  const addBtn = document.createElement("div");
  addBtn.className = "nt-shortcut-card nt-shortcut-add";
  addBtn.title = "Add Shortcut";

  const addIcon = document.createElement("div");
  addIcon.className = "nt-shortcut-icon";
  addIcon.textContent = "+";

  const addTitle = document.createElement("span");
  addTitle.className = "nt-shortcut-title";
  addTitle.textContent = "Add";

  addBtn.appendChild(addIcon);
  addBtn.appendChild(addTitle);
  addBtn.addEventListener("click", () => openScModal());

  container.appendChild(addBtn);
}

function deleteShortcut(id) {
  let scs = getStoredShortcuts().filter(s => s.id !== id);
  saveStoredShortcuts(scs);
  renderShortcuts();
}

function openScModal() {
  const modal = document.getElementById("sc-modal");
  if (modal) {
    modal.classList.add("open");
    const nameInput = document.getElementById("sc-name-input");
    const urlInput = document.getElementById("sc-url-input");
    if (nameInput) { nameInput.value = ""; nameInput.focus(); }
    if (urlInput) urlInput.value = "";
  }
}

function closeScModal() {
  const modal = document.getElementById("sc-modal");
  if (modal) modal.classList.remove("open");
}

function saveSc() {
  const nameInput = document.getElementById("sc-name-input");
  const urlInput = document.getElementById("sc-url-input");
  if (!nameInput || !urlInput) return;

  const title = nameInput.value.trim();
  let url = urlInput.value.trim();

  if (!title || !url) {
    alert("Please enter both a name and URL.");
    return;
  }

  if (!/^https?:\/\//i.test(url) && !url.endsWith(".xhtml") && !url.endsWith(".html")) {
    url = "https://" + url;
  }

  const scs = getStoredShortcuts();
  scs.push({
    id: "custom-" + Date.now(),
    title,
    url
  });

  saveStoredShortcuts(scs);
  closeScModal();
  renderShortcuts();
}

// ── Post-Quantum Status & Metrics ──
function initSecurityMetrics() {
  const guardEl = document.getElementById("c-guard");
  const relayEl = document.getElementById("c-relay");
  const exitEl = document.getElementById("c-exit");
  const adsEl = document.getElementById("cnt-ads");
  const trackersEl = document.getElementById("cnt-trackers");

  if (guardEl) guardEl.textContent = "Reykjavik · IS";
  if (relayEl) relayEl.textContent = "Zurich · CH";
  if (exitEl) exitEl.textContent = "Stockholm · SE";

  // Check Qualium Runtime IPC state if present
  if (typeof QualiumRuntimeState !== "undefined") {
    QualiumRuntimeState.subscribe(state => {
      if (guardEl && state.guard) guardEl.textContent = state.guard;
      if (relayEl && state.relay) relayEl.textContent = state.relay;
      if (exitEl && state.exit) exitEl.textContent = state.exit;
      if (adsEl && state.adsBlocked != null) adsEl.textContent = state.adsBlocked;
      if (trackersEl && state.trackersBlocked != null) trackersEl.textContent = state.trackersBlocked;
    });
  }
}

// ── Initialization ──
document.addEventListener("DOMContentLoaded", () => {
  updateClockAndGreeting();
  setInterval(updateClockAndGreeting, 10000);

  initSearchEngine();
  renderBookmarks();
  renderShortcuts();
  initSecurityMetrics();

  const bmAddBtn = document.getElementById("bm-add-btn");
  if (bmAddBtn) {
    bmAddBtn.addEventListener("click", () => openBmModal());
  }

  // Keyboard Escape listener for modals
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      closeBmModal();
      closeScModal();
    }
  });
});
