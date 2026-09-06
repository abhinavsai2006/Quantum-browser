// Qualium Quantum Browser v5 — Central Authoritative Internal Route Registry
// Single source of truth for all qualium:// public protocols and internal chrome resources

(function(global) {
  "use strict";

  // Authoritative Route Table: Public Protocol -> Internal Gecko Resource
  const ROUTES = {
    "qualium://newtab": "chrome://qualium/content/newtab.xhtml",
    "qualium://privacy": "chrome://qualium/content/dashboard.xhtml",
    "qualium://security": "chrome://qualium/content/dashboard.xhtml",
    "qualium://settings": "chrome://qualium/content/settings.xhtml",
    "qualium://downloads": "chrome://qualium/content/downloads.xhtml",
    "qualium://bookmarks": "chrome://qualium/content/bookmarks.xhtml",
    "qualium://history": "chrome://qualium/content/history.xhtml",
    "qualium://passwords": "about:logins",
    "qualium://extensions": "about:addons",
    "qualium://welcome": "chrome://qualium/content/onboarding.xhtml",
    "qualium://about": "chrome://qualium/content/settings.xhtml#about",
    "qualium://error": "chrome://qualium/content/error.xhtml"
  };

  // Reverse Mapping: Internal Implementation Resource -> Public qualium:// Protocol
  const REVERSE_ROUTES = [
    { internal: "chrome://qualium/content/newtab.xhtml", public: "qualium://newtab" },
    { internal: "about:newtab", public: "qualium://newtab" },
    { internal: "about:home", public: "qualium://newtab" },
    { internal: "about:privatebrowsing", public: "qualium://newtab" },
    { internal: "chrome://browser/content/blanktab.html", public: "qualium://newtab" },
    { internal: "chrome://qualium/content/settings.xhtml#about", public: "qualium://about" },
    { internal: "chrome://qualium/content/settings.xhtml", public: "qualium://settings" },
    { internal: "about:preferences", public: "qualium://settings" },
    { internal: "chrome://qualium/content/dashboard.xhtml", public: "qualium://privacy" },
    { internal: "about:protections", public: "qualium://privacy" },
    { internal: "chrome://qualium/content/downloads.xhtml", public: "qualium://downloads" },
    { internal: "about:downloads", public: "qualium://downloads" },
    { internal: "chrome://qualium/content/bookmarks.xhtml", public: "qualium://bookmarks" },
    { internal: "chrome://browser/content/places/places.xhtml", public: "qualium://bookmarks" },
    { internal: "chrome://qualium/content/history.xhtml", public: "qualium://history" },
    { internal: "chrome://browser/content/places/history.xhtml", public: "qualium://history" },
    { internal: "about:logins", public: "qualium://passwords" },
    { internal: "about:addons", public: "qualium://extensions" },
    { internal: "chrome://qualium/content/onboarding.xhtml", public: "qualium://welcome" },
    { internal: "about:welcome", public: "qualium://welcome" },
    { internal: "chrome://qualium/content/error.xhtml", public: "qualium://error" }
  ];

  // User-Facing Tab / Page Titles
  const TITLES = {
    "qualium://newtab": "New Tab",
    "qualium://privacy": "Qualium Privacy",
    "qualium://security": "Qualium Security",
    "qualium://settings": "Qualium Settings",
    "qualium://downloads": "Qualium Downloads",
    "qualium://bookmarks": "Qualium Bookmarks",
    "qualium://history": "Qualium History",
    "qualium://passwords": "Qualium Passwords",
    "qualium://extensions": "Qualium Extensions",
    "qualium://welcome": "Welcome to Qaulium",
    "qualium://about": "Qualium About",
    "qualium://error": "Qualium Error"
  };

  const QualiumRouteRegistry = {
    // Normalizes input string (lowercases scheme, maps qaulium:// to qualium://, strips trailing slash)
    normalize(url) {
      if (!url) return "";
      let s = url.trim();
      if (s.toLowerCase().startsWith("qaulium://")) {
        s = "qualium://" + s.slice(10);
      }
      if (s.toLowerCase().startsWith("qualium://")) {
        // Lowercase the path portion for canonical routing: qualium://Settings -> qualium://settings
        const afterScheme = s.slice(10);
        let path = afterScheme;
        let hashOrQuery = "";
        const hashIdx = path.indexOf("#");
        const queryIdx = path.indexOf("?");
        let cut = -1;
        if (hashIdx !== -1 && queryIdx !== -1) cut = Math.min(hashIdx, queryIdx);
        else if (hashIdx !== -1) cut = hashIdx;
        else if (queryIdx !== -1) cut = queryIdx;

        if (cut !== -1) {
          hashOrQuery = path.slice(cut);
          path = path.slice(0, cut);
        }
        path = path.toLowerCase();
        if (path.endsWith("/") && path.length > 1) {
          path = path.slice(0, -1);
        }
        s = "qualium://" + path + hashOrQuery;
      }
      return s;
    },

    isQualiumRoute(url) {
      if (!url) return false;
      const n = this.normalize(url);
      return n.startsWith("qualium://");
    },

    isInternalResource(url) {
      if (!url) return false;
      const s = url.trim();
      return s.startsWith("chrome://qualium/") || s.startsWith("about:") || s.startsWith("qualium://") || s.startsWith("qaulium://");
    },

    // Maps public qualium:// URL -> internal Gecko chrome:// resource
    publicToInternal(url) {
      const n = this.normalize(url);
      if (!n.startsWith("qualium://")) return url;

      // Direct match
      if (ROUTES[n]) {
        return ROUTES[n];
      }

      // Base route match (ignoring query/hash if any)
      const base = n.split("?")[0].split("#")[0];
      if (ROUTES[base]) {
        const extra = n.slice(base.length);
        return ROUTES[base] + extra;
      }

      // Unknown internal route -> return clean Qualium error page with target route
      return "chrome://qualium/content/error.xhtml?route=" + encodeURIComponent(n);
    },

    // Maps internal Gecko chrome:// resource -> public clean qualium:// URL for Omnibox display
    internalToPublic(internalUrl) {
      if (!internalUrl) return "";
      const trimmed = internalUrl.trim();

      // Check if it's an error page with route param
      if (trimmed.includes("error.xhtml") && trimmed.includes("route=")) {
        try {
          const match = trimmed.match(/route=([^&]+)/);
          if (match) {
            return decodeURIComponent(match[1]);
          }
        } catch(e) {}
        return "qualium://error";
      }

      // Check reverse mappings
      for (const r of REVERSE_ROUTES) {
        if (trimmed === r.internal || trimmed.startsWith(r.internal + "?") || (r.internal.includes("#") && trimmed === r.internal)) {
          return r.public;
        }
      }

      // If it's already qualium:// or qaulium://
      if (trimmed.toLowerCase().startsWith("qualium://") || trimmed.toLowerCase().startsWith("qaulium://")) {
        return this.normalize(trimmed);
      }

      // Default: do not modify real external URLs (http, https, file, etc.)
      return trimmed;
    },

    // Returns user-facing title for any internal route or resource
    getTitleForRoute(routeOrInternal) {
      const pub = this.internalToPublic(routeOrInternal);
      return TITLES[pub] || "Qaulium Quantum Browser";
    },

    // Classifies user input from omnibox
    classifyNavigation(input) {
      if (!input) return { type: "empty", target: "chrome://qualium/content/newtab.xhtml", display: "qualium://newtab" };
      const trimmed = input.trim();
      const normalized = this.normalize(trimmed);

      // A. qualium:// internal route
      if (normalized.startsWith("qualium://")) {
        const internal = this.publicToInternal(normalized);
        return { type: "qualium", target: internal, display: normalized };
      }

      // B. Explicit http:// or https://
      if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
        return { type: "url", target: trimmed, display: trimmed };
      }

      // C. Internal chrome or about protocol
      if (trimmed.startsWith("chrome://") || trimmed.startsWith("about:")) {
        const pub = this.internalToPublic(trimmed);
        return { type: "qualium", target: trimmed, display: pub };
      }

      // D. Domain name pattern (e.g. google.com, news.ycombinator.com)
      if (!trimmed.includes(" ") && trimmed.includes(".") && !trimmed.startsWith(".")) {
        const fullUrl = "https://" + trimmed;
        return { type: "url", target: fullUrl, display: fullUrl };
      }

      // E. Default search query
      const searchUrl = "https://www.google.com/search?q=" + encodeURIComponent(trimmed);
      return { type: "search", target: searchUrl, display: trimmed };
    },

    getAllPublicRoutes() {
      return Object.keys(ROUTES);
    }
  };

  global.QualiumRouteRegistry = QualiumRouteRegistry;
  if (typeof module !== "undefined" && module.exports) {
    module.exports = QualiumRouteRegistry;
  }
})(typeof globalThis !== "undefined" ? globalThis : this);
