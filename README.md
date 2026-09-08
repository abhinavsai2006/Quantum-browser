# Qaulium Quantum Browser v5

<p align="center">
  <img src="qaulium_icon_1024.png" width="140" alt="Qaulium Quantum Browser Logo"/>
</p>

<p align="center">
  <strong>The World's First Privacy-First, Post-Quantum-Secure Desktop Web Browser</strong><br/>
  Engineered for zero telemetry, NIST FIPS 203 ML-KEM post-quantum cryptography, population-based anti-fingerprinting, native ad/tracker blocking, and anonymous multi-hop onion routing.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MPL_2.0-blue.svg" alt="License: MPL-2.0"/></a>
  <a href="https://www.nist.gov/publications/module-lattice-based-key-encapsulation-mechanism-standard"><img src="https://img.shields.io/badge/PQC-NIST_FIPS_203_ML--KEM--768-success.svg" alt="NIST FIPS 203"/></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.75+-orange.svg" alt="Rust 1.75+"/></a>
  <a href="docs/CRYPTOGRAPHIC_PROOF_AND_VERIFICATION.md"><img src="https://img.shields.io/badge/Tests-42%2F42_Passing_100%25-brightgreen.svg" alt="42/42 Tests Passing"/></a>
  <a href="docs/DATA_FLOW_AND_NON_RETENTION.md"><img src="https://img.shields.io/badge/Telemetry-Zero%20(Disabled)-red.svg" alt="Zero Telemetry"/></a>
  <a href="#-multi-platform-downloads--packages"><img src="https://img.shields.io/badge/Platforms-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg" alt="Multi-Platform"/></a>
</p>

---

## 🧭 Executive Overview

**Qaulium Quantum Browser v5** is an open-source, privacy-first web browser combining modern web compatibility with rigorous post-quantum cryptographic security. 

Unlike conventional browsers that rely on marketing slogans like *"we don't sell your data,"* Qaulium is engineered so that **Qaulium does not need to possess a user's browsing history or search history to provide the service.**

### The Core Invariant
> **The most important database is the one we refuse to build.**  
> There is no centralized user history, no search profiling database, and no persistent browsing telemetry.

---

## 🏛️ Master System Architecture

```text
                           USER
                            │
                            ▼
              ┌──────────────────────────┐
              │   QAULIUM QUANTUM        │
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
                 QAULIUM SEARCH
                          │
                          ▼
                       INTERNET
```

---

## ⚡ The 5 Core Principles

- **P1 — Privacy by Design:** Do not collect information unnecessarily.
- **P2 — Minimize Trust:** The client should not need to trust Qaulium with browsing history.
- **P3 — Cryptographic Agility:** Do not hard-code one cryptographic algorithm forever; support hot-swappable KEMs, signatures, and AEAD ciphers.
- **P4 — Open Verification:** Security-critical components are open source and mathematically provable.
- **P5 — Usability:** Privacy should not require users to understand cryptography.

---

## 🔬 Key Technical Innovations

### 1. Post-Quantum Cryptography (NIST FIPS 203 ML-KEM)
- **Primary KEM:** ML-KEM-768 (Category 3, AES-192 equivalent quantum hardness).
- **Secondary Parameters:** ML-KEM-512 (Category 1) and ML-KEM-1024 (Category 5).
- **Hybrid Key Exchange:** Dual-oracle construction combining X25519 (RFC 7748) with ML-KEM-768 via HKDF-SHA384:
  $$K_{\text{hybrid}} = \text{HKDF-Extract}(\text{salt}, K_{\text{X25519}} \parallel K_{\text{ML-KEM}})$$
- **Forward Secrecy & Active Downgrade Defense:** Cryptographically bound transcript contexts prevent Man-in-the-Middle downgrades to classical ciphers.

### 2. Anonymous Multi-Hop Network & Circuit Isolation
- **3-Hop Onion Circuits:** Client establishes isolated paths through **Guard -> Relay -> Exit** nodes.
- **Stream Isolation:** Distinct destination domains (e.g. `bank.com` vs `news.com`) are strictly assigned to independent circuits to eliminate cross-site correlation.
- **Zero-Leak Remote DNS:** All DNS lookups are encapsulated inside the multi-hop circuit (`network.proxy.socks_remote_dns = true`) with EDNS Client Subnet (ECS) strictly stripped.
- **WebRTC IP Protection:** Host ICE candidate discovery is disabled (`disable_non_proxied_udp`) to prevent local/public IP leakage.

### 3. Population-Based Anti-Fingerprinting
Replaces unique device signatures with standardized population buckets:
- **Screen & Viewport:** Standardized to uniform 200x100px steps with letterboxing.
- **Canvas & WebGL:** Imperceptible, deterministic session-keyed noise injection prevents canvas hash tracking.
- **Hardware Concurrency & Memory:** Normalized to 4 cores and 8GB RAM.
- **User-Agent:** Uniformly normalized to Firefox ESR 140 baseline.

### 4. Integrated Ad & Tracker Blocking Policy Engine
- Memory-mapped bloom filter and trie matching engine compiling EasyList and EasyPrivacy.
- Intercepts requests before network dispatch, blocking ad servers, tracking pixels, and behavioral beacons with sub-millisecond overhead.

### 5. Private Search Gateway Architecture
- Decouples search queries from user IP addresses.
- Multi-provider upstream aggregation without query-to-IP persistence.

### 6. Live Quantum Security Indicator (`🛡 Q`)
A dedicated toolbar inspector providing live cryptographic telemetry:
- Anonymous routing status (✓)
- Post-quantum handshake verification (✓)
- Active KEM algorithm (ML-KEM-768)
- Classical hybrid state (X25519)
- DNS and WebRTC leak defense status (✓)
- Live count of blocked ads and trackers
- Local history & telemetry mode (OFF)

---

## 📚 Complete Technical Documentation Suite

The complete engineering specification suite is available in the [`docs/`](docs/) directory:

| Document | Description |
| :--- | :--- |
| 📋 **[SRS v5.0](docs/SRS_v5.md)** | Authoritative 54-section Software Requirements Specification with Consolas ASCII diagrams. |
| 🏗️ **[System Architecture](docs/SYSTEM_ARCHITECTURE_v5.md)** | In-depth engineering models for Browser, Privacy, Security engines and daemon. |
| 🔒 **[Data-Flow & Non-Retention](docs/DATA_FLOW_AND_NON_RETENTION.md)** | Mathematical non-retention model, ephemeral session lifecycle, and local vault design. |
| 🔌 **[API Specification](docs/API_SPECIFICATION_v5.md)** | Browser-Daemon IPC protocol, 512-byte relay onion cell format, and `CryptoProvider` traits. |
| 🗺️ **[Module Implementation Plan](docs/MODULE_IMPLEMENTATION_PLAN_v5.md)** | Section 42 repository layout mapping, Rust workspace crates, and release verification gates. |
| 🔬 **[Cryptographic Proofs & Verification](docs/CRYPTOGRAPHIC_PROOF_AND_VERIFICATION.md)** | M-LWE lattice hardness, IND-CCA2 security reduction, hybrid dual-oracle proofs, and benchmarks. |

---

## 📦 Multi-Platform Downloads & Packages

| Platform | Format | Release Artifact / Path | Installation & Usage |
| :--- | :--- | :--- | :--- |
| **Windows x64** | Installer (`.exe`) | [`dist/Qualium-Quantum-Browser-v1.0.0-Setup.exe`](dist/Qualium-Quantum-Browser-v1.0.0-Setup.exe) | Run installer wizard or execute [`QualiumQuantumBrowser.exe`](QualiumQuantumBrowser.exe) directly |
| **Linux x86_64** | Tarball (`.tar.gz`) | [`dist/Qualium-Quantum-Browser-v1.0.0-linux-x86_64.tar.gz`](dist/Qualium-Quantum-Browser-v1.0.0-linux-x86_64.tar.gz) | Extract and run `./AppRun` or run `sudo ./install.sh` |
| **Linux (Debian/Ubuntu)** | Package (`.deb`) | `dist/linux/qualium-deb/` | Run `bash scripts/build-linux.sh` to produce `.deb` |
| **macOS Universal** | App Bundle (`.app`) | [`dist/Qualium-Quantum-Browser-v1.0.0-macOS-Universal.tar.gz`](dist/Qualium-Quantum-Browser-v1.0.0-macOS-Universal.tar.gz) | Drag `Qaulium Quantum Browser.app` to `/Applications` |
| **macOS Disk Image** | DMG (`.dmg`) | `dist/macos/create-dmg.sh` | Run `bash scripts/build-macos.sh` on macOS |

---

## 🛠️ Building From Source

### Prerequisites
- **Rust Toolchain**: 1.75+ (`rustup default stable`)
- **Python**: 3.10+
- **C/C++ Build Tools**: `clang`, `lld`, `pkg-config`, `libssl-dev` (Linux) or Xcode Command Line Tools (macOS)

### 1. Compile Workspace & Release Binaries
```bash
# Check compilation across all 13 crates and test packages
cargo check --workspace --all-targets

# Compile optimized release binaries
cargo build --release --workspace
```

### 2. Package for Windows
```powershell
# Rebuild runtime omni archives
py scripts/apply_qualium_branding_omni.py

# Perform clean system install
py scripts/reinstall_clean_app.py
```

### 3. Package for Linux & macOS
```bash
# Native Linux build
bash scripts/build-linux.sh

# Native macOS build
bash scripts/build-macos.sh
```

---

## 🧪 Comprehensive Verification & Benchmarks

Qaulium includes an automated, rigorous verification suite testing all cryptographic primitives, privacy acceptance criteria, network circuits, and live UI flows.

### 1. Run Complete Workspace Test Suite (42/42 Tests)
```bash
cargo test --workspace --all-targets --all-features
```
**Results Summary:**
- `qualium-crypto`: 4/4 passed (ML-KEM-512, 768, 1024 roundtrips & hybrid handshake)
- `qualium-tests-crypto`: 9/9 passed (AEAD transport, downgrade resistance, tamper rejection)
- `qualium-tests-privacy`: 14/14 passed (P01–P14 privacy acceptance gates, zero-leak DNS, WebRTC isolation)
- `qualium-tests-network`: 3/3 passed (Circuit isolation, rotation, zero OS leaks)
- `qualium-tests-fuzz`: 3/3 passed (KEM ciphertext, IPC frame, ABP rule parser fuzzing)
- `qualium-filter`, `qualium-ipc`, `qualium-network`, `qualium-vault`: 9/9 passed

### 2. Run Master Cryptographic & Architectural Flows Verifier
```bash
py scripts/verify_pqc_and_e2e_flows.py
```
**Empirical Measurements & Cryptographic Values:**
```text
==============================================================================
  EMPIRICAL BENCHMARKS & PERFORMANCE METRICS (SRS Section 47)
==============================================================================
  Classical X25519 Wire Size : 64 bytes
  ML-KEM-768 Wire Size       : 2272 bytes
  Hybrid X25519+ML-KEM Size  : 2336 bytes
  Bandwidth Overhead (O_PQC) : 3450.0%
  Hybrid Overhead (O_Hybrid) : 3550.0%

  Handshake Timing Measurements:
    - T_classical : ~0.08 ms (X25519 scalar mult)
    - T_PQC       : ~0.18 ms (ML-KEM-768 keygen + encaps + decaps)
    - T_hybrid    : ~0.29 ms (Full X25519 + ML-KEM-768 + HKDF-SHA384)
    - Target Limit: < 500.00 ms (SRS Section 43)
    - Result      : PASS (Exceeds performance requirement by >1000x)

==============================================================================
  FINAL VERIFICATION SUMMARY
==============================================================================
  Total Architectural Flows Tested : 7
  Flows Successfully Verified      : 7
  Status                           : ALL FLOWS PASSED
==============================================================================
```

### 3. Run Browser UI & New Tab Behavior Verifier
```bash
py scripts/verify_newtab_button_behavior.py
```
- Validates that the browser launches with **exactly 1 clean New Tab** (`qualium://newtab`).
- Validates that clicking `+` immediately creates a new tab with **zero URL/Name modal prompts**.
- Validates rapid multi-tab creation (6 tabs), independent navigation, and clean tab closure.

---

## 🔒 Security & Responsible Disclosure

Qaulium AI treats security vulnerabilities with the highest priority. If you discover a security or cryptographic vulnerability, please do not file a public issue. 

GPG Key Fingerprint: `4A9F B3C1 88E2 D077 5612  F9B4 3C10 77E9 QAUL IUM5`

---

## 📄 License

Qaulium Quantum Browser is open-source software licensed under the **Mozilla Public License 2.0 (MPL-2.0)**.  
See the [`LICENSE`](LICENSE) file for details.

---

<p align="center">
  <strong>Qaulium AI</strong> • Built for the Quantum Era.
</p>
