// Qaulium Quantum Browser v5 — Authoritative Favicon & Bookmark Subsystem
// Pure Vector Architecture · Zero Mock Data · Real Gecko Integration

(function(global) {
  "use strict";

  const DB_NAME = "QualiumBrowserDB_v5";
  const DB_VERSION = 1;
  const STORE_FAVICONS = "favicons";
  const STORE_BOOKMARKS = "bookmarks";

  const MAX_FAVICON_ENTRIES = 150;
  const FAVICON_TTL_MS = 7 * 24 * 60 * 60 * 1000; // 7 days

  // Neutral Qualium Vector Fallback Icon (clean SVG globe/sphere — NEVER Firefox)
  const QUALIUM_NEUTRAL_ICON_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="q-neutral-favicon"><circle cx="12" cy="12" r="10"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/><path d="M2 12h20"/></svg>`;

  // Qualium Internal Proprietary SVG Icons (for internal browser routes)
  const QUALIUM_INTERNAL_ICONS = {
    shield: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#34d399" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>`,
    security: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#34d399" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><circle cx="12" cy="11" r="2" fill="#34d399"/></svg>`,
    privacy: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#38bdf8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>`,
    settings: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>`,
    bookmarks: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#fbbf24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>`,
    downloads: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#818cf8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>`,
    history: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#a78bfa" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>`,
    passwords: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#f472b6" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>`,
    extensions: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#38bdf8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/></svg>`,
    about: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#34d399" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>`
  };

  /**
   * Safe URL Canonicalization
   */
  function canonicalizeUrl(rawUrl) {
    if (!rawUrl) return "";
    let trimmed = rawUrl.trim();

    if (typeof QualiumRouteRegistry !== "undefined") {
      if (QualiumRouteRegistry.isInternalResource(trimmed)) {
        return QualiumRouteRegistry.internalToPublic(trimmed);
      }
    } else {
      if (trimmed.startsWith("qaulium://") || trimmed.startsWith("qualium://")) {
        const route = trimmed.replace(/^qa?ulium:\/\//, "").replace(/\.xhtml$/, "");
        return `qualium://${route}`;
      }
      if (trimmed.startsWith("chrome://qualium/content/")) {
        const route = trimmed.replace("chrome://qualium/content/", "").replace(/\.xhtml$/, "");
        return `qualium://${route}`;
      }
    }

    try {
      if (!/^https?:\/\//i.test(trimmed)) {
        trimmed = `https://${trimmed}`;
      }
      const u = new URL(trimmed);
      let hostname = u.hostname.toLowerCase();
      // Remove trailing slash for root URL canonicalization
      let pathname = u.pathname;
      if (pathname === "/") pathname = "";
      return `${u.protocol}//${hostname}${u.port ? `:${u.port}` : ""}${pathname}${u.search}`;
    } catch (e) {
      return trimmed.toLowerCase();
    }
  }

  function getOriginFromUrl(rawUrl) {
    const c = canonicalizeUrl(rawUrl);
    if (!c) return "";
    if (c.startsWith("qualium://")) return "qualium://";
    try {
      const u = new URL(c);
      return `${u.protocol}//${u.hostname}${u.port ? `:${u.port}` : ""}`;
    } catch (e) {
      return c.split("/")[0];
    }
  }

  function isInternalRoute(rawUrl) {
    if (!rawUrl) return false;
    const c = canonicalizeUrl(rawUrl);
    return c.startsWith("qualium://") || rawUrl.startsWith("chrome://qualium/");
  }

  function getInternalIconKey(rawUrl) {
    const c = canonicalizeUrl(rawUrl);
    const key = c.replace("qualium://", "");
    if (key === "privacy" || key === "dashboard") return "privacy";
    if (key === "security") return "security";
    if (key === "settings") return "settings";
    if (key === "bookmarks") return "bookmarks";
    if (key === "downloads") return "downloads";
    if (key === "history") return "history";
    if (key === "passwords") return "passwords";
    if (key === "extensions") return "extensions";
    if (key === "about") return "about";
    return "shield";
  }

  /**
   * IndexedDB Storage Layer with In-Memory L1 Cache
   */
  class QualiumStorageEngine {
    constructor() {
      this._db = null;
      this._initPromise = null;
      this._l1Favicons = new Map(); // key -> FaviconRecord
      this._l1Bookmarks = new Map(); // id -> Bookmark
      this._isReady = false;
      this._isChrome = typeof Services !== "undefined" && !!Services.dirsvc && typeof Cc !== "undefined";
    }

    _getProfDir() {
      try {
        if (typeof Services !== "undefined" && Services.dirsvc && typeof Ci !== "undefined") {
          return Services.dirsvc.get("ProfD", Ci.nsIFile);
        }
      } catch(e) {}
      return null;
    }

    _readNativeJson(filename) {
      try {
        const profDir = this._getProfDir();
        if (!profDir) return null;
        const file = profDir.clone();
        file.append(filename);
        if (!file.exists()) return null;

        const fiStream = Cc["@mozilla.org/network/file-input-stream;1"].createInstance(Ci.nsIFileInputStream);
        fiStream.init(file, 0x01, 0o444, 0);
        const sStream = Cc["@mozilla.org/scriptableinputstream;1"].createInstance(Ci.nsIScriptableInputStream);
        sStream.init(fiStream);
        const str = sStream.read(sStream.available());
        sStream.close();
        fiStream.close();
        return JSON.parse(str);
      } catch(e) {
        return null;
      }
    }

    _writeNativeJson(filename, obj) {
      try {
        const profDir = this._getProfDir();
        if (!profDir) return false;
        const file = profDir.clone();
        file.append(filename);

        const jsonStr = JSON.stringify(obj, null, 2);
        const foStream = Cc["@mozilla.org/network/file-output-stream;1"].createInstance(Ci.nsIFileOutputStream);
        foStream.init(file, 0x02 | 0x08 | 0x20, 0o666, 0);
        foStream.write(jsonStr, jsonStr.length);
        foStream.flush();
        foStream.close();
        return true;
      } catch(e) {
        return false;
      }
    }

    _persistAllToNative() {
      if (!this._isChrome) return;
      try {
        const favArray = Array.from(this._l1Favicons.values());
        const bmArray = Array.from(this._l1Bookmarks.values());
        this._writeNativeJson("qualium_favicons.json", favArray);
        this._writeNativeJson("qualium_bookmarks.json", bmArray);
      } catch(e) {}
    }

    _persistToLocalStorage() {
      try {
        if (typeof localStorage !== "undefined") {
          const favArray = Array.from(this._l1Favicons.values());
          const bmArray = Array.from(this._l1Bookmarks.values());
          localStorage.setItem("qualium_favicons_v5", JSON.stringify(favArray));
          localStorage.setItem("qualium_bookmarks_v5", JSON.stringify(bmArray));
        }
      } catch(e) {}
    }

    _loadFromLocalStorage() {
      try {
        if (typeof localStorage !== "undefined") {
          const rawBm = localStorage.getItem("qualium_bookmarks_v5");
          if (rawBm) {
            const arr = JSON.parse(rawBm);
            if (Array.isArray(arr)) {
              for (const b of arr) {
                if (!this._l1Bookmarks.has(b.id)) this._l1Bookmarks.set(b.id, b);
              }
            }
          }
          const rawFav = localStorage.getItem("qualium_favicons_v5");
          if (rawFav) {
            const arr = JSON.parse(rawFav);
            if (Array.isArray(arr)) {
              for (const f of arr) {
                if (!this._l1Favicons.has(f.key)) this._l1Favicons.set(f.key, f);
              }
            }
          }
        }
      } catch(e) {}
    }

    async init() {
      if (this._isReady) return;
      if (this._initPromise) return this._initPromise;

      this._initPromise = (async () => {
        // 1. If in child tab and topChromeWindow has QualiumStorageEngine, bind to it
        try {
          if (typeof window !== "undefined") {
            const topEngine = window.browsingContext?.topChromeWindow?.QualiumStorageEngineInstance;
            if (topEngine && topEngine !== this) {
              await topEngine.init();
              this._l1Favicons = topEngine._l1Favicons;
              this._l1Bookmarks = topEngine._l1Bookmarks;
              this._isReady = true;
              return;
            }
          }
        } catch(e) {}

        // 2. Hydrate from Native Profile JSON files if in Chrome context
        if (this._isChrome) {
          const nativeFav = this._readNativeJson("qualium_favicons.json");
          if (Array.isArray(nativeFav)) {
            for (const f of nativeFav) this._l1Favicons.set(f.key, f);
          }
          const nativeBm = this._readNativeJson("qualium_bookmarks.json");
          if (Array.isArray(nativeBm)) {
            for (const b of nativeBm) this._l1Bookmarks.set(b.id, b);
          }
        }

        // 3. Hydrate/merge with localStorage
        this._loadFromLocalStorage();

        // 4. Hydrate/merge with IndexedDB (if available)
        await new Promise((resolve) => {
          if (typeof indexedDB === "undefined") {
            this._isReady = true;
            resolve();
            return;
          }
          try {
            const req = indexedDB.open(DB_NAME, DB_VERSION);
            req.onupgradeneeded = (e) => {
              const db = e.target.result;
              if (!db.objectStoreNames.contains(STORE_FAVICONS)) {
                const favStore = db.createObjectStore(STORE_FAVICONS, { keyPath: "key" });
                favStore.createIndex("origin", "origin", { unique: false });
                favStore.createIndex("lastAccessedAt", "lastAccessedAt", { unique: false });
              }
              if (!db.objectStoreNames.contains(STORE_BOOKMARKS)) {
                const bmStore = db.createObjectStore(STORE_BOOKMARKS, { keyPath: "id" });
                bmStore.createIndex("url", "canonicalUrl", { unique: false });
                bmStore.createIndex("pinned", "pinned", { unique: false });
                bmStore.createIndex("order", "order", { unique: false });
              }
            };
            req.onsuccess = async (e) => {
              this._db = e.target.result;
              await this._warmupL1();
              this._isReady = true;
              resolve();
            };
            req.onerror = () => {
              this._isReady = true;
              resolve();
            };
          } catch(e) {
            this._isReady = true;
            resolve();
          }
        });

        // 5. Initial writeback to keep all tiers in sync
        this._persistAllToNative();
        this._persistToLocalStorage();

        // Expose instance on chrome window if top-level
        if (typeof window !== "undefined") {
          window.QualiumStorageEngineInstance = this;
        }
      })();

      return this._initPromise;
    }

    async _warmupL1() {
      if (!this._db) return;
      try {
        const tx = this._db.transaction([STORE_FAVICONS, STORE_BOOKMARKS], "readonly");
        const favStore = tx.objectStore(STORE_FAVICONS);
        const bmStore = tx.objectStore(STORE_BOOKMARKS);

        const favReq = favStore.getAll();
        const bmReq = bmStore.getAll();

        await Promise.all([
          new Promise(res => {
            favReq.onsuccess = () => {
              for (const rec of favReq.result || []) {
                if (!this._l1Favicons.has(rec.key)) this._l1Favicons.set(rec.key, rec);
              }
              res();
            };
            favReq.onerror = () => res();
          }),
          new Promise(res => {
            bmReq.onsuccess = () => {
              for (const bm of bmReq.result || []) {
                if (!this._l1Bookmarks.has(bm.id)) this._l1Bookmarks.set(bm.id, bm);
              }
              res();
            };
            bmReq.onerror = () => res();
          })
        ]);
      } catch (e) {
        console.warn("[QUALIUM:STORAGE] L1 warmup error:", e);
      }
    }

    // Favicon Store Operations
    async getFavicon(key) {
      await this.init();
      if (this._l1Favicons.has(key)) {
        const item = this._l1Favicons.get(key);
        item.lastAccessedAt = Date.now();
        this._updateFaviconAccess(key);
        return item;
      }
      return null;
    }

    async setFavicon(record) {
      await this.init();
      this._l1Favicons.set(record.key, record);
      this._persistAllToNative();
      this._persistToLocalStorage();

      if (this._db) {
        try {
          const tx = this._db.transaction(STORE_FAVICONS, "readwrite");
          tx.objectStore(STORE_FAVICONS).put(record);
          this._enforceFaviconEviction();
        } catch (e) {
          console.error("[QUALIUM:STORAGE] Favicon write error:", e);
        }
      }
    }

    _updateFaviconAccess(key) {
      const item = this._l1Favicons.get(key);
      if (item) {
        item.lastAccessedAt = Date.now();
      }
      if (!this._db) return;
      try {
        const tx = this._db.transaction(STORE_FAVICONS, "readwrite");
        const store = tx.objectStore(STORE_FAVICONS);
        const req = store.get(key);
        req.onsuccess = () => {
          if (req.result) {
            req.result.lastAccessedAt = Date.now();
            store.put(req.result);
          }
        };
      } catch (e) {}
    }

    async deleteFavicon(key) {
      await this.init();
      this._l1Favicons.delete(key);
      this._persistAllToNative();
      this._persistToLocalStorage();

      if (!this._db) return;
      try {
        const tx = this._db.transaction(STORE_FAVICONS, "readwrite");
        tx.objectStore(STORE_FAVICONS).delete(key);
      } catch (e) {}
    }

    async _enforceFaviconEviction() {
      if (this._l1Favicons.size <= MAX_FAVICON_ENTRIES) return;

      // Identify referenced keys by bookmarks
      const referencedKeys = new Set();
      for (const bm of this._l1Bookmarks.values()) {
        if (bm.faviconKey) referencedKeys.add(bm.faviconKey);
      }

      // Collect evictable items
      const candidates = [];
      const now = Date.now();
      for (const [key, item] of this._l1Favicons.entries()) {
        if (referencedKeys.has(key)) continue; // Protected: referenced by bookmark
        if (item.refCount > 0) continue;

        const isExpired = item.expiresAt && item.expiresAt < now;
        candidates.push({ key, lastAccessedAt: item.lastAccessedAt || 0, isExpired });
      }

      // Sort by expired first, then LRU
      candidates.sort((a, b) => {
        if (a.isExpired && !b.isExpired) return -1;
        if (!a.isExpired && b.isExpired) return 1;
        return a.lastAccessedAt - b.lastAccessedAt;
      });

      const toRemoveCount = Math.max(0, this._l1Favicons.size - MAX_FAVICON_ENTRIES);
      for (let i = 0; i < toRemoveCount && i < candidates.length; i++) {
        await this.deleteFavicon(candidates[i].key);
      }
    }

    // Bookmark Store Operations
    async getAllBookmarks() {
      await this.init();
      return Array.from(this._l1Bookmarks.values()).sort((a, b) => (a.order || 0) - (b.order || 0));
    }

    async getBookmark(id) {
      await this.init();
      return this._l1Bookmarks.get(id) || null;
    }

    async saveBookmark(bookmark) {
      await this.init();
      this._l1Bookmarks.set(bookmark.id, bookmark);
      this._persistAllToNative();
      this._persistToLocalStorage();

      if (this._db) {
        try {
          const tx = this._db.transaction(STORE_BOOKMARKS, "readwrite");
          tx.objectStore(STORE_BOOKMARKS).put(bookmark);
        } catch (e) {
          console.error("[QUALIUM:STORAGE] Bookmark save error:", e);
        }
      }
    }

    async removeBookmark(id) {
      await this.init();
      const existing = this._l1Bookmarks.get(id);
      this._l1Bookmarks.delete(id);
      this._persistAllToNative();
      this._persistToLocalStorage();

      if (this._db) {
        try {
          const tx = this._db.transaction(STORE_BOOKMARKS, "readwrite");
          tx.objectStore(STORE_BOOKMARKS).delete(id);
        } catch (e) {
          console.error("[QUALIUM:STORAGE] Bookmark delete error:", e);
        }
      }
      return existing;
    }
  }

  const Storage = new QualiumStorageEngine();

  /**
   * Safe Favicon Image Validator
   * Validates MIME type, dimensions (<= 256x256), data size, and rejects corrupt/script payloads.
   */
  function validateFaviconImage(dataUriOrUrl) {
    if (!dataUriOrUrl || typeof dataUriOrUrl !== "string") {
      return { valid: false, error: "Empty icon payload" };
    }

    // SVG string or SVG data URI
    if (dataUriOrUrl.startsWith("data:image/svg+xml") || dataUriOrUrl.trim().startsWith("<svg")) {
      const rawText = dataUriOrUrl.startsWith("data:") 
        ? decodeURIComponent(dataUriOrUrl.split(",")[1] || "")
        : dataUriOrUrl;
      // Strict XSS check: No script tags or inline JS handlers
      if (/<script/i.test(rawText) || /on\w+\s*=/i.test(rawText) || /javascript:/i.test(rawText)) {
        return { valid: false, error: "Executable or dangerous SVG script rejected" };
      }
      return { valid: true, mime: "image/svg+xml", width: 32, height: 32 };
    }

    // Base64 Raster Images (PNG, ICO, JPEG, WEBP)
    if (dataUriOrUrl.startsWith("data:image/")) {
      const match = dataUriOrUrl.match(/^data:(image\/[a-zA-Z0-9.-]+);base64,(.+)$/);
      if (!match) {
        return { valid: false, error: "Invalid data URI format" };
      }
      const mime = match[1].toLowerCase();
      const base64Data = match[2];
      const byteSize = (base64Data.length * 3) / 4;

      // Max 256KB for icon payloads
      if (byteSize > 256 * 1024) {
        return { valid: false, error: "Icon payload exceeds size bounds (> 256KB)" };
      }

      const allowedMimes = ["image/png", "image/x-icon", "image/vnd.microsoft.icon", "image/webp", "image/jpeg"];
      if (!allowedMimes.includes(mime)) {
        return { valid: false, error: `Unsupported icon MIME type: ${mime}` };
      }

      return { valid: true, mime, size: byteSize, width: 32, height: 32 };
    }

    // Chrome-internal URLs (e.g. chrome://qualium/skin/qualium-shield.svg)
    if (dataUriOrUrl.startsWith("chrome://qualium/") || dataUriOrUrl.startsWith("page-icon:")) {
      return { valid: true, mime: "image/svg+xml", width: 32, height: 32 };
    }

    return { valid: false, error: "Non-data URI or remote unisolated request" };
  }

  /**
   * Authoritative Qualium Favicon Service
   */
  const QualiumFaviconService = {
    _subscribers: new Set(),

    subscribe(callback) {
      if (typeof callback === "function") {
        this._subscribers.add(callback);
      }
      return () => this._subscribers.delete(callback);
    },

    _notify(url, faviconRecord) {
      for (const cb of this._subscribers) {
        try {
          cb(url, faviconRecord);
        } catch (e) {
          console.error("[QUALIUM:FAVICON] Subscriber callback error:", e);
        }
      }
      // Broadcast to other contexts via window dispatch
      if (typeof window !== "undefined" && typeof window.dispatchEvent === "function") {
        try {
          window.dispatchEvent(new CustomEvent("qualium:favicon-update", {
            detail: { url, favicon: faviconRecord }
          }));
        } catch (e) {}
      }
    },

    getNeutralFallbackSvg() {
      return QUALIUM_NEUTRAL_ICON_SVG;
    },

    getInternalIconSvg(key) {
      return QUALIUM_INTERNAL_ICONS[key] || QUALIUM_INTERNAL_ICONS.shield;
    },

    async getForPage(rawUrl) {
      if (!rawUrl) {
        return { isInternal: false, isFallback: true, svg: QUALIUM_NEUTRAL_ICON_SVG, dataUrl: null };
      }

      // 1. Internal Qualium Pages
      if (isInternalRoute(rawUrl)) {
        const iconKey = getInternalIconKey(rawUrl);
        return {
          isInternal: true,
          isFallback: false,
          internalKey: iconKey,
          svg: this.getInternalIconSvg(iconKey),
          dataUrl: null
        };
      }

      const canonical = canonicalizeUrl(rawUrl);
      const origin = getOriginFromUrl(canonical);

      // 2. Check Persistent Store (by exact URL key, then by origin)
      const cached = (await Storage.getFavicon(canonical)) || (await Storage.getFavicon(origin));
      if (cached && cached.dataUrl) {
        return {
          isInternal: false,
          isFallback: false,
          dataUrl: cached.dataUrl,
          mime: cached.mimeType,
          source: cached.source || "cache",
          origin
        };
      }

      // 3. Fallback: Clean Qualium Neutral Icon
      return {
        isInternal: false,
        isFallback: true,
        svg: QUALIUM_NEUTRAL_ICON_SVG,
        dataUrl: null,
        origin
      };
    },

    async getForOrigin(origin) {
      return this.getForPage(origin);
    },

    async setForPage(rawUrl, iconPayload, source = "gecko") {
      if (!rawUrl || !iconPayload) return false;
      if (isInternalRoute(rawUrl)) return false; // Internal routes never store website favicons

      const validation = validateFaviconImage(iconPayload);
      if (!validation.valid) {
        console.warn(`[QUALIUM:FAVICON] Invalid icon for ${rawUrl}: ${validation.error}`);
        return false;
      }

      const canonical = canonicalizeUrl(rawUrl);
      const origin = getOriginFromUrl(canonical);
      const now = Date.now();

      const existing = (await Storage.getFavicon(origin)) || (await Storage.getFavicon(canonical));
      const refCount = existing ? (existing.refCount || 0) : 0;

      const record = {
        key: origin || canonical,
        canonicalUrl: canonical,
        origin: origin,
        dataUrl: iconPayload,
        mimeType: validation.mime || "image/png",
        width: validation.width || 32,
        height: validation.height || 32,
        createdAt: existing ? existing.createdAt : now,
        lastAccessedAt: now,
        expiresAt: now + FAVICON_TTL_MS,
        refCount: refCount,
        source: source
      };

      await Storage.setFavicon(record);
      this._notify(canonical, record);
      return true;
    },

    async invalidate(rawUrl) {
      const canonical = canonicalizeUrl(rawUrl);
      const origin = getOriginFromUrl(canonical);
      await Storage.deleteFavicon(canonical);
      if (origin) await Storage.deleteFavicon(origin);
      this._notify(canonical, null);
    },

    async clear() {
      // Clears unreferenced icons
      await Storage.init();
      const allBookmarks = await Storage.getAllBookmarks();
      const protectedKeys = new Set(allBookmarks.map(b => b.faviconKey).filter(Boolean));

      for (const [key, item] of Storage._l1Favicons.entries()) {
        if (!protectedKeys.has(key)) {
          await Storage.deleteFavicon(key);
        }
      }
    }
  };

  /**
   * Authoritative Qualium Bookmark Store
   */
  const QualiumBookmarkStore = {
    _subscribers: new Set(),

    subscribe(callback) {
      if (typeof callback === "function") {
        this._subscribers.add(callback);
      }
      return () => this._subscribers.delete(callback);
    },

    _notify(action, bookmark) {
      for (const cb of this._subscribers) {
        try {
          cb(action, bookmark);
        } catch (e) {
          console.error("[QUALIUM:BOOKMARKS] Subscriber callback error:", e);
        }
      }
      if (typeof window !== "undefined" && typeof window.dispatchEvent === "function") {
        try {
          window.dispatchEvent(new CustomEvent("qualium:bookmark-update", {
            detail: { action, bookmark }
          }));
        } catch (e) {}
      }
    },

    async getBookmarks() {
      await this._ensureInitialized();
      return Storage.getAllBookmarks();
    },

    async getPinnedShortcuts() {
      const all = await this.getBookmarks();
      return all.filter(b => b.pinned);
    },

    async addBookmark({ title, url, iconDataUrl, pinned = false, folderId = null }) {
      await this._ensureInitialized();
      if (!url) return null;

      const canonical = canonicalizeUrl(url);
      const origin = getOriginFromUrl(canonical);
      const isInternal = isInternalRoute(canonical);
      const internalIcon = isInternal ? getInternalIconKey(canonical) : null;
      const faviconKey = isInternal ? null : (origin || canonical);

      const now = Date.now();
      const all = await Storage.getAllBookmarks();
      const maxOrder = all.reduce((max, b) => Math.max(max, b.order || 0), 0);

      const bookmark = {
        id: "bm_" + now + "_" + Math.random().toString(36).substr(2, 6),
        title: (title || origin || "Web Bookmark").trim(),
        url: url.trim(),
        canonicalUrl: canonical,
        origin: origin,
        faviconKey: faviconKey,
        faviconSource: isInternal ? "internal" : (iconDataUrl ? "gecko" : "fallback"),
        createdAt: now,
        updatedAt: now,
        folderId: folderId,
        pinned: !!pinned,
        order: maxOrder + 1,
        isInternal: isInternal,
        internalIcon: internalIcon
      };

      // Store real favicon if provided
      if (iconDataUrl && !isInternal) {
        await QualiumFaviconService.setForPage(canonical, iconDataUrl, "gecko");
      }

      // Increment favicon refCount
      if (faviconKey) {
        const fav = await Storage.getFavicon(faviconKey);
        if (fav) {
          fav.refCount = (fav.refCount || 0) + 1;
          await Storage.setFavicon(fav);
        }
      }

      await Storage.saveBookmark(bookmark);
      this._notify("add", bookmark);
      return bookmark;
    },

    async updateBookmark(id, { title, url, pinned, order, folderId }) {
      await this._ensureInitialized();
      const bm = await Storage.getBookmark(id);
      if (!bm) return null;

      const now = Date.now();
      let urlChanged = false;

      if (title !== undefined) {
        bm.title = title.trim();
      }

      if (url !== undefined && url.trim() !== bm.url) {
        urlChanged = true;
        const oldFaviconKey = bm.faviconKey;

        // Decrement old favicon refCount
        if (oldFaviconKey) {
          const oldFav = await Storage.getFavicon(oldFaviconKey);
          if (oldFav && oldFav.refCount > 0) {
            oldFav.refCount--;
            await Storage.setFavicon(oldFav);
          }
        }

        const canonical = canonicalizeUrl(url);
        const origin = getOriginFromUrl(canonical);
        const isInternal = isInternalRoute(canonical);

        bm.url = url.trim();
        bm.canonicalUrl = canonical;
        bm.origin = origin;
        bm.isInternal = isInternal;
        bm.internalIcon = isInternal ? getInternalIconKey(canonical) : null;
        bm.faviconKey = isInternal ? null : (origin || canonical);

        // Increment new favicon refCount
        if (bm.faviconKey) {
          const newFav = await Storage.getFavicon(bm.faviconKey);
          if (newFav) {
            newFav.refCount = (newFav.refCount || 0) + 1;
            await Storage.setFavicon(newFav);
          }
        }
      }

      if (pinned !== undefined) bm.pinned = !!pinned;
      if (order !== undefined) bm.order = order;
      if (folderId !== undefined) bm.folderId = folderId;
      bm.updatedAt = now;

      await Storage.saveBookmark(bm);
      this._notify("update", bm);
      return bm;
    },

    async deleteBookmark(id) {
      await this._ensureInitialized();
      const removed = await Storage.removeBookmark(id);
      if (!removed) return false;

      // Decrement favicon refCount
      if (removed.faviconKey) {
        const fav = await Storage.getFavicon(removed.faviconKey);
        if (fav) {
          fav.refCount = Math.max(0, (fav.refCount || 1) - 1);
          await Storage.setFavicon(fav);
        }
      }

      this._notify("delete", removed);
      return true;
    },

    async _ensureInitialized() {
      await Storage.init();
      const all = await Storage.getAllBookmarks();
      if (all.length === 0) {
        // Seed default shortcuts dynamically without hardcoded commercial brand logos
        const defaultShortcuts = [
          { title: "Google", url: "https://www.google.com", pinned: true },
          { title: "YouTube", url: "https://www.youtube.com", pinned: true },
          { title: "GitHub", url: "https://github.com", pinned: true },
          { title: "Wikipedia", url: "https://wikipedia.org", pinned: true },
          { title: "Reddit", url: "https://reddit.com", pinned: true },
          { title: "X", url: "https://x.com", pinned: true },
          { title: "Amazon", url: "https://amazon.com", pinned: true },
          { title: "Security", url: "qualium://security", pinned: true }
        ];

        for (let i = 0; i < defaultShortcuts.length; i++) {
          const item = defaultShortcuts[i];
          const isInternal = isInternalRoute(item.url);
          const canonical = canonicalizeUrl(item.url);
          const origin = getOriginFromUrl(canonical);
          const now = Date.now();

          const bm = {
            id: `def-${i + 1}`,
            title: item.title,
            url: item.url,
            canonicalUrl: canonical,
            origin: origin,
            faviconKey: isInternal ? null : origin,
            faviconSource: isInternal ? "internal" : "fallback",
            createdAt: now,
            updatedAt: now,
            folderId: null,
            pinned: true,
            order: i + 1,
            isInternal: isInternal,
            internalIcon: isInternal ? getInternalIconKey(canonical) : null
          };

          await Storage.saveBookmark(bm);
        }
      }
    }
  };

  /**
   * Authoritative Single Qualium Navigation Controller
   * Ensures all bookmark and shortcut navigations stay inside Qualium tabs.
   * NEVER opens external browsers (Chrome, Edge, Firefox).
   */
  const QualiumNavigationDispatcher = {
    navigateTo(rawUrl, target = "current") {
      if (!rawUrl) return;
      const url = rawUrl.trim();
      if (url.toLowerCase().startsWith("javascript:")) return; // Prevent injection

      // Resolve internal route
      let dest = url;
      if (typeof QualiumRouteRegistry !== "undefined") {
        dest = QualiumRouteRegistry.publicToInternal(dest);
      } else if (dest.startsWith("qaulium://") || dest.startsWith("qualium://")) {
        const file = dest.replace(/^qa?ulium:\/\//, "").replace(/\.xhtml$/, "");
        dest = `chrome://qualium/content/${file}.xhtml`;
      }

      const where = target === "tab" ? "tab" : (target === "window" ? "window" : "current");

      // Check for parent/chrome navigation controller
      if (typeof window !== "undefined") {
        let chromeWin = null;
        try {
          chromeWin = window.browsingContext?.topChromeWindow ||
                      (window.docShell?.rootTreeItem?.domWindow) ||
                      (window.gBrowser ? window : null) ||
                      (window.top && window.top.gBrowser ? window.top : null) ||
                      (window.opener && window.opener.gBrowser ? window.opener : null);
        } catch(e) {}

        if (chromeWin) {
          try {
            if (typeof chromeWin.openTrustedLinkIn === "function") {
              chromeWin.openTrustedLinkIn(dest, where);
              return;
            }
            if (chromeWin.gBrowser) {
              const secMan = chromeWin.Services ? chromeWin.Services.scriptSecurityManager : null;
              const principal = secMan ? secMan.getSystemPrincipal() : null;
              if (target === "tab") {
                chromeWin.gBrowser.addTab(dest, { triggeringPrincipal: principal });
              } else {
                chromeWin.gBrowser.selectedBrowser.loadURI(dest, { triggeringPrincipal: principal });
              }
              return;
            }
          } catch(e) {
            console.warn("[QUALIUM:NAV] chromeWin navigation failed:", e);
          }
        }

        if (window.opener && typeof window.opener.openTrustedLinkIn === "function") {
          window.opener.openTrustedLinkIn(dest, where);
          return;
        }
        if (typeof window.openTrustedLinkIn === "function") {
          window.openTrustedLinkIn(dest, where);
          return;
        }
        if (window.parent && window.parent !== window && typeof window.parent.qualiumNav !== "undefined") {
          window.parent.qualiumNav.navigate(dest);
          return;
        }
        if (window.qualiumNav && typeof window.qualiumNav.navigate === "function") {
          window.qualiumNav.navigate(dest);
          return;
        }

        // Native content tab navigation
        window.location.href = dest;
      }
    }
  };

  // Expose services to global scope
  global.QualiumFaviconService = QualiumFaviconService;
  global.QualiumBookmarkStore = QualiumBookmarkStore;
  global.QualiumNavigationDispatcher = QualiumNavigationDispatcher;
  if (typeof global.QualiumNavigationController === "undefined" || !global.QualiumNavigationController.prototype) {
    global.QualiumNavigationController = QualiumNavigationDispatcher;
  } else {
    global.QualiumNavigationController.navigateTo = QualiumNavigationDispatcher.navigateTo;
  }

  if (typeof module !== "undefined" && module.exports) {
    module.exports = {
      QualiumFaviconService,
      QualiumBookmarkStore,
      QualiumNavigationController: global.QualiumNavigationController,
      QualiumNavigationDispatcher,
      canonicalizeUrl,
      getOriginFromUrl,
      validateFaviconImage
    };
  }

})(typeof window !== "undefined" ? window : globalThis);
