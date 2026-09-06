// Qualium Privacy Center Navigation & Live Telemetry Binding

document.addEventListener("DOMContentLoaded", () => {
  // 1. Sidebar Tab Navigation
  const navItems = document.querySelectorAll(".nav-item");
  const panes = document.querySelectorAll(".content-pane");

  navItems.forEach(item => {
    item.addEventListener("click", () => {
      navItems.forEach(i => i.classList.remove("active"));
      panes.forEach(p => p.classList.remove("active"));

      item.classList.add("active");
      const targetId = "pane-" + item.getAttribute("data-target");
      const targetPane = document.getElementById(targetId);
      if (targetPane) {
        targetPane.classList.add("active");
      }
    });
  });

  // Check URL hash for direct tab navigation (e.g. #connection)
  const hash = window.location.hash ? window.location.hash.slice(1).toLowerCase() : "";
  if (hash) {
    const matchingBtn = document.querySelector(`.nav-item[data-target="${hash}"]`);
    if (matchingBtn) {
      matchingBtn.click();
    }
  }

  // 2. Authoritative Security State Binding
  function renderLiveMetrics(s) {
    if (!s) return;

    const isNegotiated = s.pqcState === "Negotiated" || s.pqcState === "negotiated";
    const isCircuitActive = s.circuitState === "ACTIVE" || s.circuitState === "Active";
    const isProxyConnected = s.proxyState === "Connected" || s.proxyState === "connected";

    // Proxy endpoint
    const proxyDesc = document.getElementById("dash-proxy-desc");
    if (proxyDesc) {
      proxyDesc.textContent = `${s.proxyEndpoint || "127.0.0.1:9050"} (${isProxyConnected ? "Active & bound to Necko network layer" : "Unavailable"})`;
    }
    const proxyChip = document.getElementById("dash-proxy-chip");
    if (proxyChip) {
      proxyChip.textContent = isProxyConnected ? "Bound" : "Unavailable";
      proxyChip.className = isProxyConnected ? "q-chip q-chip-success" : "q-chip q-chip-danger";
    }

    // Circuit isolation
    const circuitDesc = document.getElementById("dash-circuit-desc");
    if (circuitDesc && s.guard && s.exit) {
      circuitDesc.textContent = isCircuitActive ? `Active 3-Hop Circuit: ${s.guard} → ${s.relay} → ${s.exit}` : "Circuit building...";
    }
    const circuitChip = document.getElementById("dash-circuit-chip");
    if (circuitChip) {
      circuitChip.textContent = isCircuitActive ? "Enforced" : (s.circuitState || "Building");
      circuitChip.className = isCircuitActive ? "q-chip q-chip-success" : "q-chip";
    }

    // PQC KEM chip
    const kemChip = document.getElementById("kem-chip");
    if (kemChip) {
      if (isNegotiated) {
        kemChip.textContent = "Negotiated";
        kemChip.className = "q-chip q-chip-success";
      } else if (s.pqcState === "Negotiating") {
        kemChip.textContent = "Negotiating";
        kemChip.className = "q-chip";
      } else {
        kemChip.textContent = s.pqcState || "Unavailable";
        kemChip.className = "q-chip q-chip-danger";
      }
    }

    const kemDesc = document.getElementById("kem-desc");
    if (kemDesc) {
      if (isNegotiated && s.sessionId) {
        kemDesc.textContent = `NIST FIPS 203 ML-KEM-768 (Session ID: ${s.sessionId.slice(0, 16)}...)`;
      } else {
        kemDesc.textContent = "NIST FIPS 203 ML-KEM-768 (Kyber768 hybrid key encapsulation).";
      }
    }
  }

  if (typeof QualiumRuntimeState !== "undefined") {
    QualiumRuntimeState.subscribe(renderLiveMetrics);
    QualiumRuntimeState.fetchNativeState();
  }
});
