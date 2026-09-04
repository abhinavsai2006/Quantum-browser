// Qaulium Quantum Browser v5 — Official Clean New Tab Controller
// Pure Vector SVG · Zero Emojis · Chrome & Firefox Architecture

const OFFICIAL_SVGS = {
  google: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="#4285F4" d="M23.745 12.27c0-.7-.06-1.4-.19-2.07H12v4.51h6.6c-.29 1.52-1.14 2.82-2.4 3.68v3.05h3.88c2.27-2.09 3.665-5.17 3.665-9.17z"/><path fill="#34A853" d="M12 24c3.24 0 5.95-1.08 7.93-2.91l-3.88-3.05c-1.08.72-2.45 1.16-4.05 1.16-3.12 0-5.77-2.1-6.72-4.93H1.25v3.15C3.26 21.36 7.34 24 12 24z"/><path fill="#FBBC05" d="M5.28 14.27c-.25-.72-.38-1.49-.38-2.27s.13-1.55.38-2.27V6.58H1.25C.45 8.18 0 10.03 0 12s.45 3.82 1.25 5.42l4.03-3.15z"/><path fill="#EA4335" d="M12 4.75c1.77 0 3.35.61 4.6 1.8l3.42-3.42C17.95 1.19 15.24 0 12 0 7.34 0 3.26 2.64 1.25 6.58l4.03 3.15c.95-2.83 3.6-4.98 6.72-4.98z"/></svg>`,
  youtube: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="#FF0000" d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814z"/><polygon points="9.545 15.568 15.818 12 9.545 8.432" fill="#FFFFFF"/></svg>`,
  github: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="#f1f5f9"><path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/></svg>`,
  wikipedia: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="#e2e8f0"><path d="M12.09 13.124l2.678-6.988h2.094l-4.148 10.728h-1.63L8.38 8.878l-2.705 7.986H4.05L0 6.136h2.174l2.84 8.012 2.625-7.986h1.764l2.687 6.962zm8.397-6.988h3.513v1.63h-1.004l-1.984 5.344 1.825 3.754h1.163v1.63h-3.328v-1.63h.979l-1.428-2.937-2.222 5.567h-1.852l4.026-10.728h1.416l-2.104 5.594 1.004-2.724z"/></svg>`,
  reddit: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#FF4500"/><path fill="#ffffff" d="M16.67 13.12a1.36 1.36 0 0 0-.87-.4 5.86 5.86 0 0 0-3.8-1.2l.65-3.05 2.12.45a1 1 0 1 0 .22-.68l-2.43-.52a.23.23 0 0 0-.27.18l-.75 3.53a5.83 5.83 0 0 0-3.87 1.29 1.36 1.36 0 1 0-1.2 2.25 2.87 2.87 0 0 0 0 .54c0 2.37 2.5 4.3 5.58 4.3s5.58-1.93 5.58-4.3a2.9 2.9 0 0 0 0-.54 1.36 1.36 0 0 0-.96-1.85zM9.05 14.19a.9.9 0 1 1 .9.9.9.9 0 0 1-.9-.9zm5.9 3a3.48 3.48 0 0 1-2.95.77 3.48 3.48 0 0 1-2.95-.77.19.19 0 0 1 .28-.26 3.1 3.1 0 0 0 2.67.69 3.1 3.1 0 0 0 2.67-.69.19.19 0 1 1 .28.26zm-.9-2.1a.9.9 0 1 1 .9-.9.9.9 0 0 1-.9.9z"/></svg>`,
  twitter: `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="#f1f5f9"><path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/></svg>`,
  amazon: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="#f1f5f9"><path d="M13.9 12.8c-.8 0-1.4.3-1.6.8-.2.5-.1 1.2.3 1.5.4.3 1 .4 1.5.2.6-.2 1-.6 1-1.3v-.6c-.4-.4-.8-.6-1.2-.6zm2.8 3.9c-.3.4-.7.6-1.1.8-.5.2-1 .3-1.6.3-1.1 0-2.1-.4-2.8-1.1-.7-.8-.9-1.9-.6-2.9.3-1.1 1.1-1.8 2.3-2.1.8-.2 1.8-.2 2.6-.3v-.4c0-.7-.2-1.3-.7-1.6-.5-.4-1.3-.4-2.1-.2-.6.2-1.1.5-1.5 1l-.9-1.2c.6-.7 1.3-1.1 2.2-1.3 1-.3 2.1-.3 3 .1 1 .4 1.5 1.1 1.7 2.1.1.5.1 1.1.1 1.6v3.7c0 .5.1.9.2 1.3h-1.5c-.1-.3-.2-.6-.2-.9z"/><path fill="#FF9900" d="M18.8 19.3c-3.1 2.3-7.5 3-11.2 1.8-2.6-.8-4.8-2.5-6.3-4.7-.2-.3 0-.7.3-.8.3-.1.6 0 .8.3 1.3 1.9 3.3 3.4 5.6 4.1 3.2 1 7.1.4 9.8-1.5.3-.2.7-.1.9.2.2.3.1.7-.1.9z"/><path fill="#FF9900" d="M19.7 18.2c-.3-.4-1.9-.8-2.7-.9-.3 0-.4-.2-.2-.4.7-.7 2.4-.5 2.7-.2.3.3.3 1.9-.1 2.6-.1.2-.3.2-.4.1-.1-.2.4-.8.7-1.2z"/></svg>`,
  security: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#34d399" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><circle cx="12" cy="11" r="2" fill="#34d399"/></svg>`,
  add: `<svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>`
};

function createSVGElement(svgStr) {
  try {
    const parser = new DOMParser();
    const doc = parser.parseFromString(svgStr, "image/svg+xml");
    if (doc.documentElement && doc.documentElement.nodeName === "svg") {
      return document.importNode(doc.documentElement, true);
    }
  } catch (e) {}
  const temp = document.createElement("div");
  temp.innerHTML = svgStr;
  return temp.firstChild;
}

const DEFAULT_SHORTCUTS = [
  { id: "def-google", title: "Google", url: "https://www.google.com", iconKey: "google" },
  { id: "def-youtube", title: "YouTube", url: "https://www.youtube.com", iconKey: "youtube" },
  { id: "def-github", title: "GitHub", url: "https://github.com", iconKey: "github" },
  { id: "def-wikipedia", title: "Wikipedia", url: "https://wikipedia.org", iconKey: "wikipedia" },
  { id: "def-reddit", title: "Reddit", url: "https://reddit.com", iconKey: "reddit" },
  { id: "def-twitter", title: "X", url: "https://x.com", iconKey: "twitter" },
  { id: "def-amazon", title: "Amazon", url: "https://amazon.com", iconKey: "amazon" },
  { id: "def-security", title: "Security", url: "dashboard.xhtml", iconKey: "security" }
];

function getStoredShortcuts() {
  try {
    const raw = localStorage.getItem("qaulium_clean_shortcuts");
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length > 0) return parsed;
    }
  } catch (e) {}
  return DEFAULT_SHORTCUTS;
}

function saveStoredShortcuts(shortcuts) {
  try {
    localStorage.setItem("qaulium_clean_shortcuts", JSON.stringify(shortcuts));
  } catch (e) {}
}

function renderShortcuts() {
  const container = document.getElementById("shortcuts-grid");
  if (!container) return;

  const shortcuts = getStoredShortcuts();
  container.innerHTML = "";

  shortcuts.forEach(item => {
    const tile = document.createElement("a");
    tile.className = "shortcut-item";
    tile.href = item.url;
    tile.title = `${item.title}\n${item.url}`;

    const iconBox = document.createElement("div");
    iconBox.className = "shortcut-icon-box";

    if (item.iconKey && OFFICIAL_SVGS[item.iconKey]) {
      iconBox.appendChild(createSVGElement(OFFICIAL_SVGS[item.iconKey]));
    } else {
      // Clean letter badge for custom bookmarks
      const letter = (item.title || "W").charAt(0).toUpperCase();
      const hue = Math.abs(item.title.split("").reduce((acc, c) => acc + c.charCodeAt(0), 0) * 53) % 360;
      iconBox.style.background = `linear-gradient(135deg, hsl(${hue}, 60%, 28%), hsl(${(hue + 40) % 360}, 65%, 20%))`;
      iconBox.style.color = "#ffffff";
      iconBox.style.fontSize = "18px";
      iconBox.style.fontWeight = "600";
      iconBox.textContent = letter;
    }

    const titleSpan = document.createElement("span");
    titleSpan.className = "shortcut-title";
    titleSpan.textContent = item.title;

    tile.appendChild(iconBox);
    tile.appendChild(titleSpan);

    // If custom shortcut, add subtle delete button
    if (!item.id.startsWith("def-")) {
      const delBtn = document.createElement("button");
      delBtn.className = "shortcut-del-btn";
      delBtn.title = "Remove shortcut";
      delBtn.appendChild(createSVGElement('<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>'));
      delBtn.addEventListener("click", (e) => {
        e.preventDefault();
        e.stopPropagation();
        deleteShortcut(item.id);
      });
      tile.appendChild(delBtn);
    }

    tile.addEventListener("click", (e) => {
      const href = tile.getAttribute("href");
      if (!href) return;
      if (href.startsWith("http://") || href.startsWith("https://")) {
        return; // Standard navigation
      }
      e.preventDefault();
      window.location.href = href;
    });

    container.appendChild(tile);
  });

  // Always append the clean "+ Add shortcut" tile
  const addTile = document.createElement("div");
  addTile.className = "shortcut-item";
  addTile.title = "Add shortcut";

  const addBox = document.createElement("div");
  addBox.className = "shortcut-icon-box shortcut-add-box";
  addBox.appendChild(createSVGElement(OFFICIAL_SVGS.add));

  const addTitle = document.createElement("span");
  addTitle.className = "shortcut-title";
  addTitle.textContent = "Add shortcut";

  addTile.appendChild(addBox);
  addTile.appendChild(addTitle);
  addTile.addEventListener("click", () => openShortcutModal());

  container.appendChild(addTile);
}

function deleteShortcut(id) {
  let shortcuts = getStoredShortcuts().filter(s => s.id !== id);
  saveStoredShortcuts(shortcuts);
  renderShortcuts();
}

function openShortcutModal() {
  const modal = document.getElementById("shortcut-modal");
  if (modal) {
    modal.classList.add("open");
    const nameInput = document.getElementById("sc-name");
    const urlInput = document.getElementById("sc-url");
    if (nameInput) { nameInput.value = ""; nameInput.focus(); }
    if (urlInput) urlInput.value = "";
  }
}

function closeShortcutModal() {
  const modal = document.getElementById("shortcut-modal");
  if (modal) modal.classList.remove("open");
}

function saveShortcut() {
  const nameInput = document.getElementById("sc-name");
  const urlInput = document.getElementById("sc-url");
  if (!nameInput || !urlInput) return;

  const name = nameInput.value.trim();
  let url = urlInput.value.trim();

  if (!name || !url) return;

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
  closeShortcutModal();
  renderShortcuts();
}

function handleSearch(e) {
  if (e) e.preventDefault();
  const input = document.getElementById("search-input");
  if (!input) return;

  const raw = input.value.trim();
  if (!raw) return;

  let targetUrl = "";

  if (/^https?:\/\//i.test(raw)) {
    targetUrl = raw;
  } else if (!raw.includes(" ") && raw.includes(".") && !raw.startsWith(".")) {
    targetUrl = "https://" + raw;
  } else {
    targetUrl = "https://www.google.com/search?q=" + encodeURIComponent(raw);
  }

  window.location.href = targetUrl;
}

document.addEventListener("DOMContentLoaded", () => {
  renderShortcuts();

  const searchForm = document.getElementById("search-form");
  if (searchForm) {
    searchForm.addEventListener("submit", handleSearch);
  }

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      closeShortcutModal();
    }
  });
});
