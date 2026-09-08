# Qualium Quantum Browser v5 — Detailed System Architecture

**Document Version:** 5.0.0  
**Classification:** Engineering Architecture & Design  
**Author:** Qualium AI Engineering Team  

---

## 1. Architectural Overview & System Model

Qualium Quantum Browser v5 is structured as a decoupled, multi-process architecture combining a hardened web browser engine frontend, native post-quantum cryptographic primitives, an ad/tracker filtering pipeline, and an external network security daemon managing isolated anonymous circuits.

```text
                           USER
                            │
                            ▼
              ┌──────────────────────────┐
              │   QUALIUM QUANTUM        │
              │        BROWSER           │
              └────────────┬─────────────┘
                           │
        ┌──────────────────┼───────────────────┐
        │                  │                   │
        ▼                  ▼                   ▼
   Browser Engine     Privacy Engine      Security Engine
        │                  │                   │
        │            ┌─────┼─────┐        ┌────┴────┐
        │            │     │     │        │         │
        │          Ads  Track  Finger   PQC      Threat
        │          Block Block  print  ML-KEM    Engine
        │            │     │     │        │
        └────────────┴─────┴─────┴────────┘
                           │
                           ▼
                  Network Controller
                           │
                           ▼
                 ┌──────────────────┐
                 │ Privacy Network   │
                 └────────┬─────────┘
                          │
                   ┌──────┼──────┐
                   ▼      ▼      ▼
                 GUARD  RELAY   EXIT
                   │      │      │
                   └──────┼──────┘
                          │
                          ▼
                 QUALIUM SEARCH
                          │
                          ▼
                       INTERNET
```

---

## 2. Engine Subsystems

### 2.1 Browser Engine (Hardened Firefox ESR Base)
- **Engine Core:** Mature Gecko runtime (`runtime/qualium-core.exe`).
- **Profile Hardening:** Custom launcher (`crates/qualium-browser`) initializes an ephemeral, isolated profile on each launch:
  - `privacy.firstparty.isolate = true` — Dynamic first-party cookie/storage jar partitioning.
  - `privacy.resistFingerprinting = true` — Baseline canvas/timing/locale shielding.
  - `dom.webrtc.ip_handling_policy = disable_non_proxied_udp` — Disables host ICE candidate leakage.
  - `network.proxy.type = 1` (Manual SOCKS5) bound strictly to `127.0.0.1:<PORT>` (Qualium Daemon).
  - `network.proxy.socks_remote_dns = true` — All DNS resolution forced over the remote multi-hop circuit.
  - `network.http.speculative-parallel-limit = 0` — Prevents speculative network pre-connects that bypass proxy.
  - `datareporting.policy.dataSubmissionEnabled = false` — Telemetry completely stripped.

### 2.2 Privacy Engine
1. **Ad & Tracker Blocking Policy Engine (`crates/qualium-filter`):**
   - High-performance pattern matching engine compiling EasyList, EasyPrivacy, and Peter Lowe's Blocklist into memory-mapped bloom filters and trie structures.
   - Evaluates network request type, domain hierarchy, third-party status, and script context before socket assignment.
2. **Population-Based Anti-Fingerprinting (`privacy/fingerprint`):**
   - Replaces device-unique attributes with standardized population buckets:
     - **Screen/Viewport:** Normalizes inner/outer window to uniform 200x100px steps (e.g., 1280x800, 1440x900, 1920x1080) with letterboxing.
     - **Canvas / WebGL:** Introduces imperceptible, deterministic cryptographic noise per-session to prevent hash extraction while maintaining rendering fidelity.
     - **WebAudio API:** Latency and frequency response normalization to neutralize audio fingerprinting.
     - **Hardware Concurrency:** Pinned to 4 or 8 cores regardless of actual CPU topology.
     - **Device Memory:** Pinned to 8GB.
     - **Font Enumeration:** Restricts font access to a standardized bundle of platform-generic fonts.

### 2.3 Security Engine
1. **Post-Quantum Cryptography Engine (`crates/qualium-crypto`):**
   - **KEM Standard:** NIST FIPS 203 ML-KEM (Module-Lattice-Based Key-Encapsulation Mechanism).
   - **Supported Parameter Sets:**
     - `ML-KEM-512` (NIST Security Category 1)
     - `ML-KEM-768` (NIST Security Category 3 — **Primary Default**)
     - `ML-KEM-1024` (NIST Security Category 5)
   - **Classical Hybrid Construction:**
     - Ephemeral Diffie-Hellman: X25519 (RFC 7748).
     - Key Derivation: HKDF-SHA256:
       $$K_{\text{hybrid}} = \text{HKDF-Extract}(\text{salt}, K_{\text{X25519}} \parallel K_{\text{ML-KEM}})$$
   - **AEAD Encryption:** ChaCha20-Poly1305 (IETF RFC 8439) and AES-256-GCM.
   - **Digital Signatures:** ML-DSA (NIST FIPS 204) and Ed25519.
2. **Threat Engine:**
   - Real-time heuristic evaluation of downloaded executables, phishing domains, and known malicious IP addresses without centralized URL logging.

---

## 3. Network Security Layer & Daemon (`crates/qualium-daemon`)

The network daemon operates as an autonomous, unprivileged background process managing network circuits, DNS encapsulation, and local proxy endpoints:

```text
Browser Window
     │
     │ SOCKS5 / HTTP Proxy (127.0.0.1:PORT)
     ▼
Qualium Daemon (qualium-daemon.exe)
     │
     ├── Inbound Protocol Demuxer
     ├── Circuit Manager & State Controller
     │     │
     │     ├── Circuit Pool (Guard -> Relay -> Exit)
     │     └── Stream Isolator (Host Domain -> Circuit ID)
     │
     ├── Post-Quantum Hybrid Transport (ML-KEM-768 + X25519)
     │
     └── Secure DNS Tunnel (Remote Encapsulated DNS)
```

### 3.1 Anonymous Multi-Hop Routing
- **Guard Node:** Selected from verified relays, pinned per-identity to prevent churn correlation attacks.
- **Relay Node:** Intermediate hop providing multi-layer onion routing.
- **Exit Node:** Terminates the circuit and relays TLS traffic to destination servers.
- **Circuit Isolation:** Requests to distinct origin domains (e.g., `bank.com` vs `news.com`) are mapped to isolated circuits with independent hop keys.

### 3.2 Direct Mode
- An optional bypass mode allowing users to connect directly to local LAN or trusted intranet resources without anonymous multi-hop latency, while preserving ad-blocking, anti-fingerprinting, and local zero-history invariants.

---

## 4. User Interface & Security Indicators

### 4.1 Shield Q Security Popover (12 Live Metrics)
Clicking the `🛡 Q` button in the browser chrome toolbar opens the live security inspector querying `qualium-daemon` via IPC:

```text
┌──────────────────────────────────────┐
│       QUALIUM SECURITY               │
├──────────────────────────────────────┤
│ Anonymous routing       ✓            │
│ PQ handshake            ✓            │
│ ML-KEM                  ✓            │
│ Classical hybrid        ✓            │
│ HTTPS                   ✓            │
│ DNS leak                ✓            │
│ WebRTC leak             ✓            │
│ Fingerprint defense     ✓            │
│ Ads blocked             24           │
│ Trackers blocked        13           │
│ Local history           OFF          │
│ Qualium telemetry       OFF          │
└──────────────────────────────────────┘
```

### 4.2 Privacy Operational Levels
- **Level 1 — Balanced:**
  - Ad & tracker blocking active.
  - HTTPS enforced.
  - Standard anti-fingerprinting.
  - Direct or single-hop routing allowed for maximum performance.
- **Level 2 — Private (Default):**
  - Full 3-hop post-quantum hybrid onion circuit.
  - Zero browsing and search history retention.
  - Dynamic first-party cookie/storage isolation.
  - Remote DNS through circuit.
- **Level 3 — Maximum:**
  - Everything in Level 2.
  - JavaScript restricted or disabled for non-whitelisted sites.
  - Aggressive canvas/audio/font neutralization.
  - Per-tab unique circuit rotation.

### 4.3 New Identity Workflow
When the user clicks "New Identity" (`Ctrl+Shift+U`):
1. All open non-pinned tabs and session state are terminated.
2. HTTP cache, cookies, IndexedDB, and localStorage are completely purged.
3. IPC command `NEW_IDENTITY` is dispatched to `qualium-daemon`.
4. The daemon tears down all active circuits, purges internal DNS caches, and re-negotiates a fresh Guard/Relay/Exit path via ML-KEM-768 hybrid handshakes.
5. A fresh New Tab is opened at `qualium://newtab`.

---

## 5. Security & Threat Mitigation Summary

| Threat | Description | Qualium v5 Defense |
| :--- | :--- | :--- |
| **T1: ISP Surveillance** | Local ISP logs visited hosts and DNS | 3-hop onion circuit + Remote DNS encapsulation |
| **T2: Wi-Fi Eavesdropper** | Local sniffer intercepts unencrypted traffic | Universal HTTPS enforcement + PQC hybrid transport tunnel |
| **T4/T5: Ad/Tracker Networks** | Cross-site profiling across domains | Dynamic first-party storage isolation + Native blocking engine |
| **T7: Quantum Attacker** | "Harvest Now, Decrypt Later" on TLS | ML-KEM-768 hybrid key exchange on all network circuits |
| **T11: Browser Fingerprinting** | Identifying users via canvas/fonts/screen | Population-based bucketing + Canvas/Audio noise injection |
| **DNS / WebRTC Leaks** | Browser bypasses proxy for DNS or IP STUN | `socks_remote_dns=true` + WebRTC non-proxied UDP disabled |
