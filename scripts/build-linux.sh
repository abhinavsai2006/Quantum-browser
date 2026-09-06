#!/usr/bin/env bash
set -e

# ==============================================================================
# Qualium Quantum Browser v1.0.0 — Native Linux Build & Package Script
# Target: x86_64 / aarch64 Linux
# Output: .tar.gz bundle, .deb package (Debian/Ubuntu), AppDir / AppImage
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

echo "======================================================================"
echo "    QUALIUM QUANTUM BROWSER v1 — LINUX BUILD & PACKAGING PIPELINE     "
echo "======================================================================"

# 1. Check toolchain dependencies
echo "[1/4] Verifying Linux toolchain..."
command -v cargo >/dev/null 2>&1 || { echo "[!] Cargo/Rust is required. Install via https://rustup.rs"; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "[!] Python 3 is required."; exit 1; }

# 2. Run test suite
echo "[2/4] Executing security & cryptographic test suite..."
cargo test --workspace --all-targets --all-features -- --nocapture

# 3. Build release binaries
echo "[3/4] Compiling release binaries (qualium-daemon, qualium-browser)..."
cargo build --release --workspace

# 4. Assemble packages
echo "[4/4] Generating cross-platform Linux distribution packages..."
python3 scripts/package-cross-platform.py

# Optional: Build .deb if dpkg-deb is available
if command -v dpkg-deb >/dev/null 2>&1; then
    echo "[*] Building native .deb package..."
    bash dist/linux/build-deb.sh
fi

echo ""
echo "======================================================================"
echo "[+] Linux build & packaging completed successfully!"
echo "Distribution artifacts generated in: $REPO_ROOT/dist"
echo "  - Tarball: dist/Qualium-Quantum-Browser-v1.0.0-linux-x86_64.tar.gz"
echo "  - Debian Staging: dist/linux/qualium-deb/"
echo "======================================================================"
