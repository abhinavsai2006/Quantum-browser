# Qaulium Quantum Browser v5

<p align="center">
  <img src="qaulium_icon_1024.png" width="128" alt="Qaulium Icon"/>
</p>

<p align="center">
  <strong>The Privacy-First, Post-Quantum-Secure Web Browser</strong><br/>
  Engineered for zero telemetry, NIST FIPS 203 ML-KEM post-quantum cryptography, population-based anti-fingerprinting, native ad/tracker blocking, and anonymous multi-hop onion routing.
</p>

---

## 🏛️ Architecture & Specifications (v5.0)

Qaulium Quantum Browser v5 follows a comprehensive, formal engineering specification suite:

- 📋 **[Complete Software Requirements Specification (SRS v5)](docs/SRS_v5.md)** — The authoritative 54-section specification covering product definition, design philosophy, functional requirements, threat models, and acceptance tests.
- 🏗️ **[System Architecture Specification](docs/SYSTEM_ARCHITECTURE_v5.md)** — Detailed multi-engine model (Browser Engine, Privacy Engine, Security Engine, Network Security Daemon, Shield Q Indicator).
- 🔒 **[Data-Flow & Non-Retention Specification](docs/DATA_FLOW_AND_NON_RETENTION.md)** — Architectural design of *"The Database We Don't Build"*, ephemeral session lifecycles, and private search gateway data flow.
- 🔌 **[Comprehensive API Specification](docs/API_SPECIFICATION_v5.md)** — Browser-Daemon IPC protocol, multi-hop relay onion cell specifications, and `CryptoProvider` trait abstractions.
- 🗺️ **[Module Implementation Plan](docs/MODULE_IMPLEMENTATION_PLAN_v5.md)** — Source code structure, Rust crate boundaries, and continuous verification gates.
- 🔬 **[Cryptographic Proof & Verification Document](docs/CRYPTOGRAPHIC_PROOF_AND_VERIFICATION.md)** — Formal mathematical security reductions (M-LWE, IND-CCA2 hybrid security, transcript binding, onion routing anonymity proofs) and empirical performance benchmarks.

---

## 🚀 Key v5 Features

- 🛡️ **Post-Quantum Cryptography (PQC)** — NIST FIPS 203 ML-KEM-768 primary KEM (plus ML-KEM-512 & 1024) in hybrid key exchange with X25519 and ChaCha20-Poly1305.
- 🚫 **Integrated Ad & Tracker Blocking** — Native, high-throughput filtering engine blocking advertising networks, tracking pixels, and behavioral beacons.
- 🎭 **Population-Based Anti-Fingerprinting** — Replaces unique device signatures with standardized population buckets (screen dimensions, canvas noise, WebAudio, normalized fonts).
- 🌐 **Anonymous Multi-Hop Routing** — Embedded security controller (`qualium-daemon`) managing isolated 3-hop circuits (Guard -> Relay -> Exit) with per-domain stream isolation.
- ⚡ **Zero Telemetry & Non-Retention Policy** — Strict ephemeral memory operation. Zero centralized history or search query retention.
- 🛡️ **Live Quantum Security Indicator** — Toolbar popover (`🛡 Q`) displaying real-time cryptographic handshakes, circuit state, DNS leak status, and blocked tracker counts.
- 🖥️ **Full Multi-Platform Support** — Native distribution packages for **Windows**, **Linux**, and **macOS**.

---

## 📦 Multi-Platform Downloads & Packages

| Platform | Format | Release Artifact / Path | Installation & Usage |
| :--- | :--- | :--- | :--- |
| **Windows x64** | Installer (`.exe`) | [`dist/Qualium-Quantum-Browser-v1.0.0-Setup.exe`](dist/Qualium-Quantum-Browser-v1.0.0-Setup.exe) | Run installer wizard or execute [`QualiumQuantumBrowser.exe`](QualiumQuantumBrowser.exe) directly |
| **Linux x86_64** | Tarball (`.tar.gz`) | [`dist/Qualium-Quantum-Browser-v1.0.0-linux-x86_64.tar.gz`](dist/Qualium-Quantum-Browser-v1.0.0-linux-x86_64.tar.gz) | Extract and run `./AppRun` or run `sudo ./install.sh` |
| **Linux (Debian/Ubuntu)** | Package (`.deb`) | `dist/linux/qualium-deb/` | Run `bash dist/linux/build-deb.sh` to produce `.deb` |
| **macOS Universal** | App Bundle (`.app`) | [`dist/Qualium-Quantum-Browser-v1.0.0-macOS-Universal.tar.gz`](dist/Qualium-Quantum-Browser-v1.0.0-macOS-Universal.tar.gz) | Drag `Qaulium Quantum Browser.app` to `/Applications` |
| **macOS Disk Image** | DMG (`.dmg`) | `dist/macos/create-dmg.sh` | Run `bash dist/macos/create-dmg.sh` on macOS |

---

## 🛠️ Building From Source

### Prerequisites
- **Rust Toolchain**: 1.75+ (`rustup default stable`)
- **Python**: 3.10+
- **C/C++ Build Tools**: `clang`, `lld`, `pkg-config`, `libssl-dev` (Linux) or Xcode Command Line Tools (macOS)

### 1. Build and Test Core Workspace
```bash
# Run 42 security, cryptographic, and privacy unit tests
cargo test --workspace --all-targets --all-features

# Compile release binaries
cargo build --release --workspace
```

### 2. Build for Linux
```bash
# Native Linux build & package script
bash scripts/build-linux.sh

# Or install directly:
tar -xzf dist/Qualium-Quantum-Browser-v1.0.0-linux-x86_64.tar.gz
cd qualium-quantum-browser
sudo ./install.sh
qualium
```

### 3. Build for macOS
```bash
# Native macOS build & package script
bash scripts/build-macos.sh

# Mount DMG or launch app:
open "dist/macos/Qualium Quantum Browser.app"
```

### 4. Build for Windows
```powershell
# Apply branding to omni.ja runtime
py scripts/apply_qualium_branding_omni.py

# Package all cross-platform distribution bundles
py scripts/package-cross-platform.py
```

---

## 🔒 Verification & Supply Chain Security

Every release package is indexed in the signed SHA-256 verification manifest and SPDX Software Bill of Materials (SBOM):
- Checksum Manifest: [`dist/SHA256SUMS.asc`](dist/SHA256SUMS.asc)
- SPDX SBOM: [`dist/sbom-v1.0.0.spdx.json`](dist/sbom-v1.0.0.spdx.json)

---

<p align="center">© 2026 Qaulium AI. Built for the Quantum Era.</p>
