// Qualium Settings Controller — Full Navigation & Persistence
(function() {
  function switchTab(tabId) {
    if (!tabId) tabId = "general";
    
    // Clean tab name
    tabId = tabId.replace("#", "").trim();

    const targetPane = document.getElementById("pane-" + tabId) || document.getElementById("pane-general");
    const activeTabName = targetPane ? targetPane.id.replace("pane-", "") : "general";

    // Update navigation button active state
    document.querySelectorAll(".nav-btn").forEach(btn => {
      if (btn.dataset.tab === activeTabName) {
        btn.classList.add("active");
      } else {
        btn.classList.remove("active");
      }
    });

    // Hide all panes and display the target
    document.querySelectorAll(".settings-pane").forEach(pane => {
      pane.style.display = "none";
    });

    if (targetPane) {
      targetPane.style.display = "block";
    }
  }

  document.addEventListener("DOMContentLoaded", () => {
    // 1. Attach navigation button listeners
    document.querySelectorAll(".nav-btn").forEach(btn => {
      btn.addEventListener("click", () => {
        const tab = btn.dataset.tab;
        window.location.hash = tab;
        switchTab(tab);
      });
    });

    // 2. Listen for hash changes
    window.addEventListener("hashchange", () => {
      switchTab(window.location.hash);
    });

    // 3. Initial navigation from URL hash or default
    const initialHash = window.location.hash || "general";
    switchTab(initialHash);

    // 4. Search Engine Preference Controller
    const engineSelect = document.getElementById("search-engine-select");
    if (engineSelect) {
      try {
        const saved = localStorage.getItem("qualium_search_engine") || "google";
        engineSelect.value = saved;
      } catch(e) {}

      engineSelect.addEventListener("change", () => {
        try {
          localStorage.setItem("qualium_search_engine", engineSelect.value);
          console.log("[QUALIUM:SETTINGS] Updated search engine to: " + engineSelect.value);
        } catch(e) {}
      });
    }
  });
})();
