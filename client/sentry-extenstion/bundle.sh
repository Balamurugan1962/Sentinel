#!/usr/bin/env bash
# bundle.sh — Packages Chrome and Firefox extensions
# Run as any user (no sudo needed)
# Usage: ./bundle.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHROME_SRC="$ROOT/chrome"
FIREFOX_SRC="$ROOT/firefox"
OUT="$ROOT/dist"

# ── Colours ────────────────────────────────────────────────────────────────
GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'
info()  { echo -e "${GREEN}[bundle]${NC} $*"; }
warn()  { echo -e "${YELLOW}[bundle]${NC} $*"; }
die()   { echo -e "${RED}[bundle]${NC} $*" >&2; exit 1; }

# ── Deps ───────────────────────────────────────────────────────────────────
check_dep() { command -v "$1" &>/dev/null || die "Required tool not found: $1 — install it first."; }
check_dep zip
check_dep jq

mkdir -p "$OUT"

# ══════════════════════════════════════════════════════════════════════════
# CHROME — packed as a plain .zip (load via policy, no store needed)
# Chrome CRX signing requires a private key and Chrome itself;
# for enterprise/policy installs a .zip sideload is the standard approach.
# ══════════════════════════════════════════════════════════════════════════
pack_chrome() {
  info "Packing Chrome extension..."

  [[ -f "$CHROME_SRC/manifest.json" ]] || die "chrome/manifest.json not found"

  local version
  version=$(jq -r '.version' "$CHROME_SRC/manifest.json")
  local out_zip="$OUT/sentinel-chrome-${version}.zip"

  rm -f "$out_zip"
  (cd "$CHROME_SRC" && zip -qr "$out_zip" . --exclude "*.DS_Store" --exclude "__MACOSX/*")

  info "Chrome → $out_zip"
}

# ══════════════════════════════════════════════════════════════════════════
# FIREFOX — packed as a .zip renamed to .xpi (standard self-hosted format)
# For enterprise installs Firefox reads the .xpi directly from disk via policy.
# ══════════════════════════════════════════════════════════════════════════
pack_firefox() {
  info "Packing Firefox extension..."

  [[ -f "$FIREFOX_SRC/manifest.json" ]] || die "firefox/manifest.json not found"

  local version ext_id
  version=$(jq -r '.version' "$FIREFOX_SRC/manifest.json")
  ext_id=$(jq -r '.browser_specific_settings.gecko.id // "sentinel@extension"' "$FIREFOX_SRC/manifest.json")

  local out_xpi="$OUT/sentinel-firefox-${version}.xpi"

  rm -f "$out_xpi"
  (cd "$FIREFOX_SRC" && zip -qr "$out_xpi" . --exclude "*.DS_Store" --exclude "__MACOSX/*")

  info "Firefox → $out_xpi (id: $ext_id)"
}

pack_chrome
pack_firefox

echo ""
info "Done. Output in: $OUT/"
ls -lh "$OUT/"
