#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VENV_DIR="$ROOT_DIR/.venv"
VENV_PYTHON="$VENV_DIR/bin/python"

if ! command -v cargo >/dev/null 2>&1; then
  echo "[error] Rust/Cargo is not installed."
  echo "        Install it and try again: https://rustup.rs"
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "[error] python3 is not installed."
  exit 1
fi

if [[ ! -x "$VENV_PYTHON" ]]; then
  echo "[info] Creating Python virtual environment at .venv..."
  python3 -m venv "$VENV_DIR"
fi

echo "[info] Ensuring Python dependencies in .venv..."
"$VENV_PYTHON" -m pip install --upgrade pip >/dev/null
"$VENV_PYTHON" -m pip install -r "$ROOT_DIR/requirements.txt"

echo "[info] Launching DocStruct GUI..."
export DOCSTRUCT_PYTHON="$VENV_PYTHON"
exec cargo run --manifest-path gui/src-tauri/Cargo.toml --release
