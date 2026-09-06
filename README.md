# Qualium Quantum Browser v1

<p align="center">
  <img src="qaulium_icon_1024.png" width="128" alt="Qualium Icon"/>
</p>

<p align="center">
  <strong>The Post-Quantum Secure, Privacy-First Desktop Web Browser</strong><br/>
  Engineered for zero telemetry, ML-KEM-512/768/1024 post-quantum cryptography, and full cross-platform compatibility across Windows, Linux, and macOS.
</p>

---

## 🚀 Key Features

- 🛡️ **Post-Quantum Cryptography (PQC)** — NIST FIPS 203 ML-KEM-512, ML-KEM-768, and ML-KEM-1024 hybrid key exchange with X25519 and ChaCha20-Poly1305.
- ⚡ **Zero Telemetry & Zero History Retention** — Pure local session memory, automated cookie/state partitioning, and zero-leak DNS-over-HTTPS.
- 🌐 **Anonymous Multi-Hop Routing** — Embedded post-quantum security daemon (`qualium-daemon`) managing isolated local SOCKS5 proxy circuits.
- 🎨 **Modern Browser Chrome** — Obsidian glassmorphism UI, Chromium-style app menu, integrated `qualium://history`, `qualium://extensions`, `qualium://about` internal pages.
- 🖥️ **Full Multi-Platform Support** — Dedicated native distribution packages for **Windows**, **Linux**, and **macOS**.

---

## 📦 Multi-Platform Downloads & Packages

| Platform | Format | Release Artifact / Path | Installation & Usage |
| :--- | :--- | :--- | :--- |
| **Windows x64** | Installer (`.exe`) | [`dist/Qualium-Quantum-Browser-v1.0.0-Setup.exe`](dist/Qualium-Quantum-Browser-v1.0.0-Setup.exe) | Run installer wizard or execute [`QualiumQuantumBrowser.exe`](QualiumQuantumBrowser.exe) directly |
| **Linux x86_64** | Tarball (`.tar.gz`) | [`dist/Qualium-Quantum-Browser-v1.0.0-linux-x86_64.tar.gz`](dist/Qualium-Quantum-Browser-v1.0.0-linux-x86_64.tar.gz) | Extract and run `./AppRun` or run `sudo ./install.sh` |
| **Linux (Debian/Ubuntu)** | Package (`.deb`) | `dist/linux/qualium-deb/` | Run `bash dist/linux/build-deb.sh` to produce `.deb` |
| **macOS Universal** | App Bundle (`.app`) | [`dist/Qualium-Quantum-Browser-v1.0.0-macOS-Universal.tar.gz`](dist/Qualium-Quantum-Browser-v1.0.0-macOS-Universal.tar.gz) | Drag `Qualium Quantum Browser.app` to `/Applications` |
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

<p align="center">© 2026 Qualium AI. Built for the Quantum Era.</p>
