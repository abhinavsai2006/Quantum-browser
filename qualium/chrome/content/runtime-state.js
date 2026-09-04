// Qualium Quantum Browser v5 — Unified Authoritative Runtime State Pipeline

const QualiumRuntimeState = {
  _state: {
    connectionState: "CONNECTED", // CONNECTED | CONNECTING | RECONNECTING | UNAVAILABLE | OFFLINE
    circuitState: "ACTIVE",       // ACTIVE | BUILDING | UNAVAILABLE
    guard: "SOCKS5 Tunnel (127.0.0.1:9050)",
    relay: "Anonymized Multi-Hop",
    exit: "Remote DNS Protected",
    pqSupport: "Available",
    pqNegotiated: "ML-KEM-768 Hybrid",
    dnsState: "Protected (In-Circuit Remote DNS)",
    webrtcState: "Protected (ICE Host Filtering)",
    trackerProtection: "Active (Gecko ETP Strict)",
    adBlocking: "Active",
    fingerprintMode: "Active (RFP Bucket Normalization)",
    privacyLevel: "Level 2: Private",
    adsBlockedCount: 0,
    trackersBlockedCount: 0
  },

  listeners: [],

  getState() {
    return { ...this._state };
  },

  subscribe(listener) {
    this.listeners.push(listener);
    try {
      listener(this._state);
    } catch(e) {
      console.error("[QUALIUM:STATE] Listener error: ", e);
    }
  },

  update(partial) {
    this._state = { ...this._state, ...partial };
    for (const l of this.listeners) {
      try {
        l(this._state);
      } catch(e) {
        console.error("[QUALIUM:STATE] Listener update error: ", e);
      }
    }
  },

  recordAdBlocked() {
    this._state.adsBlockedCount++;
    this.update({});
  },

  recordTrackerBlocked() {
    this._state.trackersBlockedCount++;
    this.update({});
  }
};

if (typeof window !== "undefined") {
  window.QualiumRuntimeState = QualiumRuntimeState;
}
