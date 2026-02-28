#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

USE_NIX=false
OUT_DIR=""

usage() {
  cat <<'USAGE'
Usage: scripts/smoke.sh [options]

Options:
  --nix       Run command inside nix-shell
  --out <dir> Use a custom output directory
  -h, --help  Show this help message
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --nix)
      USE_NIX=true
      shift
      ;;
    --out)
      if [[ $# -lt 2 ]]; then
        echo "--out requires a directory path" >&2
        exit 1
      fi
      OUT_DIR="$2"
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

run_cmd() {
  if $USE_NIX; then
    nix-shell --run "$*"
  else
    "$@"
  fi
}

INPUT_PDF="tests/fixtures/test_document.pdf"
if [[ ! -f "$INPUT_PDF" ]]; then
  echo "Missing fixture PDF: $INPUT_PDF" >&2
  exit 1
fi

if [[ -z "$OUT_DIR" ]]; then
  OUT_DIR="$(mktemp -d "${TMPDIR:-/tmp}/docstruct-smoke.XXXXXX")"
  CLEANUP_OUT=true
else
  CLEANUP_OUT=false
  rm -rf "$OUT_DIR"
  mkdir -p "$OUT_DIR"
fi

if $CLEANUP_OUT; then
  trap 'rm -rf "$OUT_DIR"' EXIT
fi

echo "[smoke] docstruct convert $INPUT_PDF"
run_cmd ./target/debug/docstruct convert "$INPUT_PDF" -o "$OUT_DIR" --debug --quiet

echo "[smoke] verify output artifacts"
test -f "$OUT_DIR/document.json"
test -f "$OUT_DIR/document.md"
test -f "$OUT_DIR/document.txt"
test -f "$OUT_DIR/debug/page_001.html"

echo "Smoke test passed."
