#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

check_linux_deps() {
  if ! command -v pkg-config >/dev/null 2>&1; then
    echo "[error] pkg-config is required."
    echo "        Ubuntu/Debian: sudo apt install -y pkg-config"
    echo "        Fedora:        sudo dnf install -y pkgconf-pkg-config"
    echo "        Arch:          sudo pacman -S --needed pkgconf"
    exit 1
  fi

  if ! pkg-config --exists wayland-client; then
    echo "[error] Missing system library: wayland-client"
    echo "        Install Linux GUI build dependencies first."
    echo "        Ubuntu/Debian:"
    echo "          sudo apt update && sudo apt install -y \\"
    echo "            libwayland-dev libxkbcommon-dev libgtk-3-dev \\"
    echo "            libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \\"
    echo "            librsvg2-dev"
    echo "        Fedora:"
    echo "          sudo dnf install -y wayland-devel libxkbcommon-devel \\"
    echo "            gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel \\"
    echo "            librsvg2-devel"
    echo "        Arch:"
    echo "          sudo pacman -S --needed wayland libxkbcommon gtk3 webkit2gtk \\"
    echo "            libappindicator-gtk3 librsvg"
    exit 1
  fi
}

if ! command -v cargo >/dev/null 2>&1; then
  echo "[error] cargo not found. Install Rust: https://rustup.rs"
  exit 1
fi

if [[ "$(uname -s)" == "Linux" ]]; then
  check_linux_deps
fi

if ! cargo tauri --help >/dev/null 2>&1; then
  echo "[info] tauri-cli not found. Installing..."
  cargo install tauri-cli --version '^2.0' --locked
fi

echo "[info] Building desktop installer bundles..."
(
  cd gui/src-tauri
  if [[ "$(uname -s)" == "Linux" ]]; then
    # Skip AppImage on Linux by default because it downloads extra tooling.
    cargo tauri build -b deb,rpm
  else
    cargo tauri build
  fi
)

echo "[done] Bundles generated under: gui/src-tauri/target/release/bundle"
