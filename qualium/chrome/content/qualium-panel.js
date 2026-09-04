        const s = QualiumRuntimeState.getState();
const adsEl = document.getElementById("priv-ads");
const trackEl = document.getElementById("priv-trackers");
if (adsEl) adsEl.textContent = s.adsBlockedCount.toString();
if (trackEl) trackEl.textContent = s.trackersBlockedCount.toString();
      }
return;
    }

try {
  const metricsRaw = this.service.getLiveSecurityMetricsJson();
  const metrics = JSON.parse(metricsRaw);

  // Update Network
  document.getElementById("net-anon-routing").textContent =
    metrics.anonymous_routing === "protected" ? "Protected ✓" : "Degraded";
  document.getElementById("net-circuit-status").textContent =
    metrics.circuit_status === "active" ? "Active (3 Hops)" : "Connecting...";

  // Update Counters
  document.getElementById("priv-ads").textContent = metrics.ads_blocked_count.toString();
  document.getElementById("priv-trackers").textContent = metrics.trackers_blocked_count.toString();

  // Update Crypto
  if (metrics.crypto) {
    if (document.getElementById("crypto-pq-cap")) {
      document.getElementById("crypto-pq-cap").textContent = metrics.crypto.pq_capability || "Available";
    }
    if (document.getElementById("crypto-pq-trans")) {
      document.getElementById("crypto-pq-trans").textContent = metrics.crypto.pq_transport || "Negotiated (Relay)";
    }
    if (document.getElementById("crypto-website-tls")) {
      document.getElementById("crypto-website-tls").textContent = metrics.crypto.website_tls || "Host-Dependent";
    }
    if (document.getElementById("crypto-aead")) {
      document.getElementById("crypto-aead").textContent = metrics.crypto.aead || "ChaCha20-Poly1305";
    }
  }
} catch (e) {
  console.error("Failed to refresh Qualium metrics:", e);
}
  },

handleNewIdentity() {
  if (this.service) {
    this.service.rotateIdentity();
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
