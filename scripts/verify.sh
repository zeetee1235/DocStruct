#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

RUN_IGNORED=false
USE_NIX=false

usage() {
  cat <<'USAGE'
Usage: scripts/verify.sh [options]

Options:
  --ignored   Run ignored tests after standard tests
  --nix       Run checks inside nix-shell
  -h, --help  Show this help message
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ignored)
      RUN_IGNORED=true
      shift
      ;;
    --nix)
      USE_NIX=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

run_cmd() {
  if $USE_NIX; then
    nix-shell --run "$*"
  else
    "$@"
  fi
}

echo "[1/3] cargo fmt --check"
run_cmd cargo fmt --all -- --check

echo "[2/3] cargo build"
run_cmd cargo build --locked

echo "[3/3] cargo test"
run_cmd cargo test --locked

if $RUN_IGNORED; then
  echo "[4/4] cargo test -- --ignored"
  run_cmd cargo test --locked -- --ignored
fi

echo "Build verification passed."
