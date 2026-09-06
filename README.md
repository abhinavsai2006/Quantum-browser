# Qaulium Quantum Browser

<p align="center">
  <img src="qaulium_icon_1024.png" width="128" alt="Qaulium Icon"/>
</p>

<p align="center">
  <strong>A true independent desktop web browser powered by Gecko</strong><br/>
  Engineered for privacy, speed, and a premium user experience.
</p>

---

## Features

- 🛡️ **Privacy-First** — Built-in tracker blocking, DNS-over-HTTPS, quantum-resistant cryptography
- ⚡ **Gecko Engine** — Full web standards support via embedded Mozilla Gecko (no Firefox required)
- 🎨 **Premium UI** — Dark obsidian theme, Chromium-style menus, custom New Tab page
- 🔒 **Encrypted Sync** — Zero-knowledge encrypted bookmark and history sync
- 🌐 **Cross-Platform** — Windows, Linux, macOS installers

## Architecture

```
Qaulium Quantum Browser
├── crates/              Rust core (network daemon, crypto, UI shell)
├── qualium/             Browser chrome (XUL/XHTML/CSS/JS UI layer)
│   └── chrome/
│       ├── content/     New Tab page, panels, modals
│       └── userChrome.css
├── scripts/             Build, branding, and packaging scripts
├── patches/             Gecko customisation patches
└── dist/                Release binaries and installers
```

## Building

### Prerequisites
- Rust 1.75+ (`rustup`)
- Python 3.10+
- Gecko runtime (placed in `runtime/browser/`)

### Build
```powershell
# Build the Rust shell
cargo build --release

# Apply Qaulium branding to omni.ja
py -3 scripts/apply_qualium_branding_omni.py

# Package installer
py -3 scripts/build_installer.py
```

## Download

Download the latest pre-built Windows standalone installer:

[![Download Windows Installer](https://img.shields.io/badge/Download-Windows_Installer_(.exe)-00c853?style=for-the-badge&logo=windows)](https://github.com/Qaulium-AI-Browser/Abhinav-s-repo-for-qualium-browser-backend/releases/download/v5.0.0/Qaulium-Quantum-Browser-v5.0.0-Setup.exe)

**Direct Download Link:**  
👉 **[Qaulium-Quantum-Browser-v5.0.0-Setup.exe](https://github.com/Qaulium-AI-Browser/Abhinav-s-repo-for-qualium-browser-backend/releases/download/v5.0.0/Qaulium-Quantum-Browser-v5.0.0-Setup.exe)**

| Platform | File / Download Link | Description |
|----------|----------------------|-------------|
| **Windows (Installer)** | [Qaulium-Quantum-Browser-v5.0.0-Setup.exe](https://github.com/Qaulium-AI-Browser/Abhinav-s-repo-for-qualium-browser-backend/releases/download/v5.0.0/Qaulium-Quantum-Browser-v5.0.0-Setup.exe) | Complete standalone installer with embedded Gecko runtime & daemon |
| **Windows (Runner)** | [`QauliumQuantumBrowser.exe`](QauliumQuantumBrowser.exe) | Standalone host shell executable in repository root |
| **Network Daemon** | [`qualium-daemon.exe`](qualium-daemon.exe) | Post-quantum ML-KEM-768 privacy proxy executable |

## License

Qaulium is proprietary software. The Gecko rendering engine is used under the Mozilla Public License 2.0.

---

<p align="center">© 2025 Qaulium. All rights reserved.</p>

# Abhinav-s-repo-for-qualium-browser-backend
