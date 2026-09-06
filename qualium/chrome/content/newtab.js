// Qaulium Quantum Browser v1 — Dynamic New Tab Controller
// Authoritative Favicon & Bookmark Integration · Zero Mock Data · Zero Hardcoded Brand Artwork

(function() {
  "use strict";

  let currentShortcuts = [];
  let editingShortcutId = null;

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
    return temp.firstElementChild;
  }

  async function renderShortcuts() {
    const container = document.getElementById("shortcuts-grid");
    if (!container) return;

    // Fetch authoritative pinned shortcuts
    if (typeof QualiumBookmarkStore !== "undefined") {
      currentShortcuts = await QualiumBookmarkStore.getPinnedShortcuts();
    } else {
      currentShortcuts = [];
    }

    container.innerHTML = "";

    for (const item of currentShortcuts) {
      const tile = document.createElement("div");
      tile.className = "shortcut-item";
      tile.title = `${item.title}\n${item.url}`;
      tile.dataset.id = item.id;

      const iconBox = document.createElement("div");
      iconBox.className = "shortcut-icon-box";

      // 1. Resolve Favicon Dynamically via QualiumFaviconService
      if (item.isInternal) {
        const iconSvg = QualiumFaviconService.getInternalIconSvg(item.internalIcon || "shield");
        iconBox.appendChild(createSVGElement(iconSvg));
      } else {
        // Asynchronously or synchronously resolve real website favicon
        const resolved = await QualiumFaviconService.getForPage(item.url);
        if (resolved && resolved.dataUrl) {
          const img = document.createElement("img");
          img.src = resolved.dataUrl;
          img.className = "shortcut-favicon-img";
          img.alt = item.title;
          img.style.width = "26px";
          img.style.height = "26px";
          img.style.objectFit = "contain";
          img.style.borderRadius = "4px";
          img.style.display = "block";
          img.onerror = () => {
            // Degrade gracefully to neutral vector fallback if corrupt
            iconBox.innerHTML = "";
            iconBox.appendChild(createSVGElement(QualiumFaviconService.getNeutralFallbackSvg()));
          };
          iconBox.appendChild(img);
        } else {
          // Neutral Qualium shield fallback ONLY while retrieving (NEVER letter avatar!)
          iconBox.innerHTML = "";
          iconBox.appendChild(createSVGElement(QualiumFaviconService.getNeutralFallbackSvg()));

          // Asynchronously resolve official site favicon
          if (typeof QualiumFaviconService.resolveAndFetchOfficialFavicon === "function") {
            QualiumFaviconService.resolveAndFetchOfficialFavicon(item.url).then(rec => {
              if (rec && rec.dataUrl) {
                iconBox.innerHTML = "";
                const img = document.createElement("img");
                img.src = rec.dataUrl;
                img.className = "shortcut-favicon-img";
                img.alt = item.title;
                img.style.width = "26px";
                img.style.height = "26px";
                img.style.objectFit = "contain";
                img.style.borderRadius = "4px";
                img.style.display = "block";
                img.onerror = () => {
                  iconBox.innerHTML = "";
                  iconBox.appendChild(createSVGElement(QualiumFaviconService.getNeutralFallbackSvg()));
                };
                iconBox.appendChild(img);
              }
            }).catch(() => {});
          }
        }
      }

      const titleSpan = document.createElement("span");
      titleSpan.className = "shortcut-title";
      titleSpan.textContent = item.title;

      tile.appendChild(iconBox);
      tile.appendChild(titleSpan);

      // Edit Button
      const editBtn = document.createElement("button");
      editBtn.className = "shortcut-edit-btn";
      editBtn.title = "Edit shortcut";
      editBtn.setAttribute("type", "button");
      editBtn.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>`;
      editBtn.addEventListener("click", (e) => {
        e.preventDefault();
        e.stopPropagation();
        openEditShortcutModal(item);
      });
      tile.appendChild(editBtn);

      // Delete Button
      const delBtn = document.createElement("button");
      delBtn.className = "shortcut-del-btn";
      delBtn.title = "Remove shortcut";
      delBtn.setAttribute("type", "button");
      delBtn.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>`;
      delBtn.addEventListener("click", async (e) => {
        e.preventDefault();
        e.stopPropagation();
        await QualiumBookmarkStore.deleteBookmark(item.id);
      });
      tile.appendChild(delBtn);

      // Click Navigation: Must use Authoritative QualiumNavigationController
      tile.addEventListener("click", () => {
        QualiumNavigationController.navigateTo(item.url, "current");
      });

      container.appendChild(tile);
    }

    // Always append the "+ Add shortcut" tile
    const addTile = document.createElement("div");
    addTile.className = "shortcut-item";
    addTile.title = "Add shortcut";

    const addBox = document.createElement("div");
    addBox.className = "shortcut-icon-box shortcut-add-box";
    addBox.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>`;

    const addTitle = document.createElement("span");
    addTitle.className = "shortcut-title";
    addTitle.textContent = "Add shortcut";

    addTile.appendChild(addBox);
    addTile.appendChild(addTitle);
    addTile.addEventListener("click", () => openAddShortcutModal());

    container.appendChild(addTile);
  }

  function openAddShortcutModal() {
    editingShortcutId = null;
    const modal = document.getElementById("shortcut-modal");
    if (!modal) return;

    const titleEl = modal.querySelector(".modal-title");
    if (titleEl) titleEl.textContent = "Add shortcut";

    const nameInput = document.getElementById("sc-name");
    const urlInput = document.getElementById("sc-url");
    if (nameInput) { nameInput.value = ""; nameInput.focus(); }
    if (urlInput) urlInput.value = "";

    modal.classList.add("open");
  }

  function openEditShortcutModal(item) {
    editingShortcutId = item.id;
    const modal = document.getElementById("shortcut-modal");
    if (!modal) return;

    const titleEl = modal.querySelector(".modal-title");
    if (titleEl) titleEl.textContent = "Edit shortcut";

    const nameInput = document.getElementById("sc-name");
    const urlInput = document.getElementById("sc-url");
    if (nameInput) { nameInput.value = item.title; nameInput.focus(); }
    if (urlInput) urlInput.value = item.url;

    modal.classList.add("open");
  }

  function closeShortcutModal() {
    editingShortcutId = null;
    const modal = document.getElementById("shortcut-modal");
    if (modal) modal.classList.remove("open");
  }

  async function saveShortcut() {
    const nameInput = document.getElementById("sc-name");
    const urlInput = document.getElementById("sc-url");
    if (!nameInput || !urlInput) return;

    const name = nameInput.value.trim();
    let url = urlInput.value.trim();
    if (!name || !url) return;

    if (!/^https?:\/\//i.test(url) && !url.startsWith("qualium://") && !url.startsWith("qaulium://")) {
      url = "https://" + url;
    }

    if (editingShortcutId) {
      await QualiumBookmarkStore.updateBookmark(editingShortcutId, {
        title: name,
        url: url
      });
    } else {
      await QualiumBookmarkStore.addBookmark({
        title: name,
        url: url,
        pinned: true
      });
    }

    closeShortcutModal();
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

    QualiumNavigationController.navigateTo(targetUrl, "current");
  }

  document.addEventListener("DOMContentLoaded", () => {
    window.scrollTo(0, 0);

    // Initial Render
    renderShortcuts();

    // Subscribe to Bookmark updates
    if (typeof QualiumBookmarkStore !== "undefined") {
      QualiumBookmarkStore.subscribe(() => {
        renderShortcuts();
      });
    }

    // Subscribe to Favicon updates (updates icons in place when resolved)
    if (typeof QualiumFaviconService !== "undefined") {
      QualiumFaviconService.subscribe((url, favicon) => {
        renderShortcuts();
      });
    }

    // Search Form Handler
    const searchForm = document.getElementById("search-form");
    if (searchForm) {
      searchForm.addEventListener("submit", handleSearch);
    }

    const searchInput = document.getElementById("search-input");
    if (searchInput) {
      setTimeout(() => {
        try {
          searchInput.focus({ preventScroll: true });
        } catch (e) {
          searchInput.focus();
        }
        window.scrollTo(0, 0);
      }, 50);
    }

    // Subscribe to Authoritative Circuit Status & PQC State
    if (typeof QualiumRuntimeState !== "undefined") {
      QualiumRuntimeState.subscribe(updateCircuitBanner);
    }

    // Modal Events
    const saveBtn = document.getElementById("sc-save-btn");
    if (saveBtn) {
      saveBtn.addEventListener("click", saveShortcut);
    }

    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        closeShortcutModal();
      }
    });
  });

  function updateCircuitBanner(state) {
    const dot = document.getElementById("pill-status-dot");
    const label = document.getElementById("pill-status-label");
    const sep = document.getElementById("pill-status-sep");
    const hops = document.getElementById("pill-status-hops");
    if (!dot || !label) return;

    const isNegotiated = state.pqcState === "Negotiated" || state.pqcState === "negotiated";
    const isCircuitActive = state.circuitState === "ACTIVE" || state.circuitState === "Active";
    const isProxyConnected = state.proxyState === "Connected" || state.proxyState === "connected";

    if (isNegotiated && isCircuitActive && isProxyConnected) {
      dot.className = "status-dot active";
      label.textContent = "● POST-QUANTUM CIRCUIT ACTIVE";
      if (hops && sep) {
        sep.style.display = "inline";
        hops.style.display = "inline";
        const g = state.guard ? state.guard.split(" ")[0] : "Guard";
        const r = state.relay ? state.relay.split(" ")[0] : "Relay";
        const e = state.exit ? state.exit.split(" ")[0] : "Exit";
        hops.textContent = `${g} → ${r} → ${e}`;
      }
    } else if (state.circuitState === "BUILDING CIRCUIT" || state.pqcState === "Negotiating") {
      dot.className = "status-dot negotiating";
      label.textContent = state.pqcState === "Negotiating" ? "● PQC NEGOTIATING" : "● BUILDING CIRCUIT";
      if (hops && sep) {
        sep.style.display = "inline";
        hops.style.display = "inline";
        hops.textContent = "Establishing Multi-Hop Enclave...";
      }
    } else {
      dot.className = "status-dot unavailable";
      label.textContent = "● CIRCUIT UNAVAILABLE";
      if (hops && sep) {
        sep.style.display = "inline";
        hops.style.display = "inline";
        hops.textContent = isProxyConnected ? "Circuit Reconnecting..." : "Proxy Unavailable";
      }
    }
  }

  // Global exports for modal handlers
  window.closeShortcutModal = closeShortcutModal;
  window.saveShortcut = saveShortcut;

})();
