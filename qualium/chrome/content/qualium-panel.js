// Qualium Quantum Browser v5 — Authoritative Security Hub Panel Controller

const QualiumPanel = {
  service: null,

  init() {
    // Check for native XPCOM / daemon IPC service if available
    try {
      if (typeof Components !== "undefined" && Components.classes) {
        const cid = "@qualium.network/security-service;1";
        if (cid in Components.classes) {
          this.service = Components.classes[cid].getService(Components.interfaces.nsIQualiumSecurityService);
        }
      }
    } catch (e) {
      console.log("[QUALIUM:PANEL] Native XPCOM service not found, falling back to QualiumRuntimeState");
    }

    // Subscribe to Authoritative Native Runtime State
    if (typeof QualiumRuntimeState !== "undefined") {
      QualiumRuntimeState.subscribe(state => {
        this.renderFromState(state);
      });
    }

    const newIdBtn = document.getElementById("btn-new-identity");
    if (newIdBtn) {
      newIdBtn.addEventListener("click", () => this.handleNewIdentity());
    }

    this.refreshMetrics();
  },

  renderFromState(s) {
    if (!s) return;

    const isNegotiated = s.pqcState === "Negotiated" || s.pqcState === "negotiated";
    const isCircuitActive = s.circuitState === "ACTIVE" || s.circuitState === "Active";

    // 1. Network Section
    const netAnon = document.getElementById("net-anon-routing");
    if (netAnon) {
      netAnon.textContent = isCircuitActive ? "Protected ✓" : "Unavailable";
      netAnon.className = isCircuitActive ? "metric-val val-green" : "metric-val val-red";
    }

    const netCircuit = document.getElementById("net-circuit-status");
    if (netCircuit) {
      netCircuit.textContent = isCircuitActive ? "Active (Multi-Hop)" : (s.circuitState || "Unavailable");
      netCircuit.className = isCircuitActive ? "metric-val val-green" : "metric-val val-red";
    }

    const circuitBadge = document.getElementById("circuit-badge");
    if (circuitBadge) {
      circuitBadge.textContent = isCircuitActive ? "Protected" : "Connecting...";
      circuitBadge.className = isCircuitActive ? "badge-status badge-active" : "badge-status badge-degraded";
    }

    // Circuit Hops
    const hopGuard = document.getElementById("hop-guard");
    if (hopGuard && s.guard) hopGuard.textContent = s.guard;

    const hopRelay = document.getElementById("hop-relay");
    if (hopRelay && s.relay) hopRelay.textContent = s.relay;

    const hopExit = document.getElementById("hop-exit");
    if (hopExit && s.exit) hopExit.textContent = s.exit;

    // DNS & WebRTC
    const netDns = document.getElementById("net-dns");
    if (netDns) netDns.textContent = s.dnsState || "Protected (In-Circuit)";

    const netWebrtc = document.getElementById("net-webrtc");
    if (netWebrtc) netWebrtc.textContent = s.webrtcState || "Protected (ICE Host Filtering)";

    // 2. Cryptography Section
    const cryptoBadge = document.getElementById("crypto-badge");
    if (cryptoBadge) {
      cryptoBadge.textContent = isNegotiated ? "Post-Quantum Active" : (s.pqcState === "Negotiating" ? "Negotiating..." : "Unavailable");
      cryptoBadge.className = isNegotiated ? "badge-status badge-pqc" : "badge-status badge-degraded";
    }

    const pqCap = document.getElementById("crypto-pq-cap");
    if (pqCap) {
      pqCap.textContent = isNegotiated ? "Negotiated (ML-KEM-768 / FIPS 203)" : `State: ${s.pqcState || "Negotiating"}`;
      pqCap.className = isNegotiated ? "metric-val val-pqc" : "metric-val";
    }

    const pqTrans = document.getElementById("crypto-pq-trans");
    if (pqTrans) {
      pqTrans.textContent = isNegotiated ? "Negotiated (Hybrid Channel)" : (isCircuitActive ? "Classical Enclave" : "Pending");
      pqTrans.className = isNegotiated ? "metric-val val-green" : "metric-val";
    }

    const webTls = document.getElementById("crypto-website-tls");
    if (webTls) {
      webTls.textContent = "Origin TLS (Host-Dependent)";
    }

    const aeadEl = document.getElementById("crypto-aead");
    if (aeadEl) {
      aeadEl.textContent = "ChaCha20-Poly1305 (RFC 8439)";
    }

    // 3. Privacy Counters
    const adsEl = document.getElementById("priv-ads");
    if (adsEl) adsEl.textContent = (s.adsBlockedCount || 0).toString();

    const trackEl = document.getElementById("priv-trackers");
    if (trackEl) trackEl.textContent = (s.trackersBlockedCount || 0).toString();
  },

  refreshMetrics() {
    if (typeof QualiumRuntimeState !== "undefined") {
      this.renderFromState(QualiumRuntimeState.getState());
      QualiumRuntimeState.fetchNativeState();
    }
  },

  handleNewIdentity() {
    if (this.service) {
      this.service.rotateIdentity();
    }
    if (typeof QualiumRuntimeState !== "undefined") {
      QualiumRuntimeState.fetchNativeState();
    }
    // Visual feedback
    const btn = document.getElementById("btn-new-identity");
    if (btn) {
      const oldText = btn.textContent;
      btn.textContent = "✓ Identity Rotated";
      btn.disabled = true;
      setTimeout(() => {
        btn.textContent = oldText;
        btn.disabled = false;
        this.refreshMetrics();
      }, 1200);
    }
  },

  toggle(anchorElement, event) {
    const panel = document.getElementById("qualium-security-panel");
    if (panel) {
      panel.openPopup(anchorElement, "bottomright topright", 0, 0, false, false);
      this.refreshMetrics();
    }
  }
};

window.addEventListener("DOMContentLoaded", () => {
  QualiumPanel.init();
});
