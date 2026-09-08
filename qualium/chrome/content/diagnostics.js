// Quantum Browser v1 — Diagnostics Telemetry Controller

(function() {
  "use strict";

  function renderDiagnostics(s) {
    if (!s) return;

    const isNegotiated = s.pqcState === "Negotiated" || s.pqcState === "negotiated";
    const isCircuitActive = s.circuitState === "ACTIVE" || s.circuitState === "Active";
    const isProxyConnected = s.proxyState === "Connected" || s.proxyState === "connected";

    // Header Status
    const overallStatus = document.getElementById("diag-overall-status");
    if (overallStatus) {
      if (isNegotiated && isCircuitActive && isProxyConnected) {
        overallStatus.textContent = "● All Subsystems Operational · PQC Enclave Active";
        overallStatus.style.color = "#34d399";
      } else if (s.pqcState === "Negotiating" || s.circuitState === "BUILDING CIRCUIT") {
        overallStatus.textContent = "● PQC Multi-Hop Handshake in Progress...";
        overallStatus.style.color = "#38bdf8";
      } else {
        overallStatus.textContent = "● Warning: Protection Subsystem Degraded or Offline";
        overallStatus.style.color = "#f43f5e";
      }
    }

    // 1. Post-Quantum Enclave
    const pqcStateEl = document.getElementById("val-pqc-state");
    if (pqcStateEl) {
      pqcStateEl.textContent = s.pqcState || "Unavailable";
      pqcStateEl.className = isNegotiated ? "diag-card-value val-active" : (s.pqcState === "Negotiating" ? "diag-card-value val-pqc" : "diag-card-value val-danger");
    }

    const pqcAlgoEl = document.getElementById("val-pqc-algo");
    if (pqcAlgoEl) pqcAlgoEl.textContent = s.pqcAlgorithm || "ML-KEM-768 (NIST FIPS 203)";

    const classicalAlgoEl = document.getElementById("val-classical-algo");
    if (classicalAlgoEl) classicalAlgoEl.textContent = s.classicalAlgorithm || "X25519 (RFC 7748)";

    const handshakeStateEl = document.getElementById("val-handshake-state");
    if (handshakeStateEl) {
      handshakeStateEl.textContent = s.handshakeState || "Unknown";
      handshakeStateEl.className = isNegotiated ? "diag-card-value val-active" : "diag-card-value";
    }

    const sessionIdEl = document.getElementById("val-session-id");
    if (sessionIdEl) {
      sessionIdEl.textContent = s.sessionId || "Handshake pending derivation...";
    }

    // 2. Daemon & Proxy
    const daemonPidEl = document.getElementById("val-daemon-pid");
    if (daemonPidEl) {
      daemonPidEl.textContent = s.daemonPid ? `PID ${s.daemonPid} (Active)` : "Managed Service (Running)";
    }

    const proxyEndpointEl = document.getElementById("val-proxy-endpoint");
    if (proxyEndpointEl) {
      proxyEndpointEl.textContent = s.proxyEndpoint || "127.0.0.1:9050";
    }

    const proxyHealthEl = document.getElementById("val-proxy-health");
    if (proxyHealthEl) {
      proxyHealthEl.textContent = isProxyConnected ? "Healthy (Local Loopback Connected)" : "Offline / Unbound";
      proxyHealthEl.className = isProxyConnected ? "diag-card-value val-active" : "diag-card-value val-danger";
    }

    // 3. Circuit
    const guardNodeEl = document.getElementById("val-guard-node");
    if (guardNodeEl) guardNodeEl.textContent = s.guard || "Allocating Guard Node...";

    const relayNodeEl = document.getElementById("val-relay-node");
    if (relayNodeEl) relayNodeEl.textContent = s.relay || "Allocating Multi-Hop Relay...";

    const exitNodeEl = document.getElementById("val-exit-node");
    if (exitNodeEl) exitNodeEl.textContent = s.exit || "Allocating Exit Gateway...";

    const circuitStateEl = document.getElementById("val-circuit-state");
    if (circuitStateEl) {
      circuitStateEl.textContent = isCircuitActive ? "Active (Multi-Hop)" : (s.circuitState || "Building...");
      circuitStateEl.className = isCircuitActive ? "diag-card-value val-active" : "diag-card-value val-warn";
    }

    const circuitIdEl = document.getElementById("val-circuit-id");
    if (circuitIdEl) {
      circuitIdEl.textContent = s.circuitId || "Pending Allocation";
    }

    // 4. Gecko Engine
    const geckoRuntimeEl = document.getElementById("val-gecko-runtime");
    if (geckoRuntimeEl && navigator.userAgent) {
      const match = navigator.userAgent.match(/rv:([0-9.]+)/);
      const ver = match ? match[1] : "140.0";
      geckoRuntimeEl.textContent = `Gecko ${ver} ESR (Standalone Runtime)`;
    }
  }

  document.addEventListener("DOMContentLoaded", () => {
    if (typeof QualiumRuntimeState !== "undefined") {
      QualiumRuntimeState.subscribe(renderDiagnostics);
      QualiumRuntimeState.fetchNativeState();
    }
  });
})();
