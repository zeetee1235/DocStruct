#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VERIFY_ARGS=()
SMOKE_ARGS=()

usage() {
  cat <<'USAGE'
Usage: scripts/test.sh [options]

This is a wrapper that runs:
  1) scripts/verify.sh
  2) scripts/smoke.sh

Options:
  --ignored   Forwarded to verify.sh
  --nix       Forwarded to verify.sh and smoke.sh
  --out <dir> Forwarded to smoke.sh
  -h, --help  Show this help message
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ignored)
      VERIFY_ARGS+=("--ignored")
      shift
      ;;
    --nix)
      VERIFY_ARGS+=("--nix")
      SMOKE_ARGS+=("--nix")
      shift
      ;;
    --out)
      if [[ $# -lt 2 ]]; then
        echo "--out requires a directory path" >&2
        exit 1
      fi
      SMOKE_ARGS+=("--out" "$2")
      shift 2
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

scripts/verify.sh "${VERIFY_ARGS[@]}"
scripts/smoke.sh "${SMOKE_ARGS[@]}"

echo "All tests passed."
