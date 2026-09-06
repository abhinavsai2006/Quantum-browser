#!/usr/bin/env bash
set -e

# ==============================================================================
# Qualium Quantum Browser v1.0.0 — Native macOS Build & Package Script
# Target: macOS Universal (Apple Silicon arm64 + Intel x86_64)
# Output: .app bundle, .tar.gz archive, .dmg Disk Image
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

echo "======================================================================"
echo "    QUALIUM QUANTUM BROWSER v1 — MACOS BUILD & PACKAGING PIPELINE     "
echo "======================================================================"

# 1. Check toolchain dependencies
echo "[1/4] Verifying macOS toolchain..."
command -v cargo >/dev/null 2>&1 || { echo "[!] Cargo/Rust is required. Install via https://rustup.rs"; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "[!] Python 3 is required."; exit 1; }

# 2. Run test suite
echo "[2/4] Executing security & cryptographic test suite..."
cargo test --workspace --all-targets --all-features -- --nocapture

# 3. Build release binaries
echo "[3/4] Compiling release binaries (qualium-daemon, qualium-browser)..."
cargo build --release --workspace

# 4. Assemble packages
echo "[4/4] Packaging native macOS .app bundle & archive..."
python3 scripts/package-cross-platform.py

# Optional: Build .dmg if hdiutil is available
if command -v hdiutil >/dev/null 2>&1; then
    echo "[*] Creating macOS Disk Image (.dmg)..."
    bash dist/macos/create-dmg.sh
fi

echo ""
echo "======================================================================"
echo "[+] macOS build & packaging completed successfully!"
echo "Distribution artifacts generated in: $REPO_ROOT/dist"
echo "  - Application Bundle: dist/macos/Qualium Quantum Browser.app"
echo "  - Tarball: dist/Qualium-Quantum-Browser-v1.0.0-macOS-Universal.tar.gz"
echo "======================================================================"
