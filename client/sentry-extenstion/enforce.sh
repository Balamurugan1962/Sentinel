#!/usr/bin/env bash
# enforce.sh — Installs Sentinel extensions via enterprise policy
# so the standard user cannot remove, disable, or manage them.
#
# Must be run as root (or via sudo) from the ADMIN account.
# Usage: sudo ./enforce.sh
#
# What this does:
#   Chrome  — writes /etc/opt/chrome/policies/managed/sentinel.json
#             Chrome reads this at startup; managed policies are locked.
#   Firefox — writes /etc/firefox/policies/policies.json
#             Firefox ESR/standard on Ubuntu respects this directory.
#   Both browsers will show the extension as "Installed by your administrator"
#   and the user will have no remove/disable button.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST="$ROOT/dist"

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'
info()  { echo -e "${GREEN}[enforce]${NC} $*"; }
warn()  { echo -e "${YELLOW}[enforce]${NC} $*"; }
die()   { echo -e "${RED}[enforce]${NC} $*" >&2; exit 1; }

# ── Must be root ───────────────────────────────────────────────────────────
[[ "$EUID" -eq 0 ]] || die "Run this script with sudo: sudo ./enforce.sh"

# ── Find built packages ────────────────────────────────────────────────────
CHROME_ZIP=$(ls "$DIST"/sentinel-chrome-*.zip 2>/dev/null | sort -V | tail -1 || true)
FIREFOX_XPI=$(ls "$DIST"/sentinel-firefox-*.xpi 2>/dev/null | sort -V | tail -1 || true)

[[ -n "$CHROME_ZIP"   ]] || die "No Chrome .zip found in dist/. Run bundle.sh first."
[[ -n "$FIREFOX_XPI"  ]] || die "No Firefox .xpi found in dist/. Run bundle.sh first."

# ── Install directory (accessible to browsers, not writable by user) ───────
INSTALL_DIR="/opt/sentinel"
mkdir -p "$INSTALL_DIR"
chmod 755 "$INSTALL_DIR"

cp "$CHROME_ZIP"  "$INSTALL_DIR/sentinel-chrome.zip"
cp "$FIREFOX_XPI" "$INSTALL_DIR/sentinel-firefox.xpi"

# Lock ownership — root owns, others can read but not write or delete
chown -R root:root "$INSTALL_DIR"
chmod 644 "$INSTALL_DIR"/*

info "Extension files installed to $INSTALL_DIR"

# ══════════════════════════════════════════════════════════════════════════
# CHROME / CHROMIUM
# Policy dir: /etc/opt/chrome/policies/managed/   (Chrome)
#             /etc/chromium/policies/managed/      (Chromium)
#
# ExtensionInstallForcelist format: "id;update_url"
# For sideloaded local extensions we use file:// as the update URL.
# Chrome will load the .zip from disk on next start.
#
# ExtensionInstallBlocklist with "*" blocks all other extension installs.
# ══════════════════════════════════════════════════════════════════════════
install_chrome_policy() {
  local policy_dir="$1"
  mkdir -p "$policy_dir"
  chmod 755 "$policy_dir"

  # Read extension ID from the chrome manifest if present
  local manifest="$ROOT/chrome/manifest.json"
  local ext_id=""
  if command -v jq &>/dev/null && [[ -f "$manifest" ]]; then
    ext_id=$(jq -r '.key // ""' "$manifest")
  fi

  # If no key in manifest, we use a placeholder note —
  # for local policy installs Chrome identifies the extension by the
  # directory/zip hash. The admin should fill in the real ID after
  # first load (visible in chrome://extensions).
  cat > "$policy_dir/sentinel.json" <<EOF
{
  "ExtensionInstallForcelist": [
    "SENTINEL_EXTENSION_ID;file://$INSTALL_DIR/sentinel-chrome.zip"
  ],
  "ExtensionInstallBlocklist": ["*"],
  "ExtensionInstallAllowlist": ["SENTINEL_EXTENSION_ID"],
  "ExtensionSettings": {
    "SENTINEL_EXTENSION_ID": {
      "installation_mode": "force_installed",
      "update_url": "file://$INSTALL_DIR/sentinel-chrome.zip",
      "toolbar_pin": "force_pinned"
    }
  }
}
EOF

  chmod 644 "$policy_dir/sentinel.json"
  chown root:root "$policy_dir/sentinel.json"
  warn "Chrome policy written to $policy_dir/sentinel.json"
  warn "ACTION NEEDED: Replace SENTINEL_EXTENSION_ID with the real ID from chrome://extensions after first load."
}

CHROME_POLICY_DIR="/etc/opt/chrome/policies/managed"
CHROMIUM_POLICY_DIR="/etc/chromium/policies/managed"

if [[ -d /etc/opt/chrome || -f /usr/bin/google-chrome || -f /usr/bin/google-chrome-stable ]]; then
  install_chrome_policy "$CHROME_POLICY_DIR"
  info "Chrome policy installed."
fi

if [[ -d /etc/chromium || -f /usr/bin/chromium || -f /usr/bin/chromium-browser ]]; then
  install_chrome_policy "$CHROMIUM_POLICY_DIR"
  info "Chromium policy installed."
fi

# ══════════════════════════════════════════════════════════════════════════
# FIREFOX
# Policy file: /etc/firefox/policies/policies.json
#
# Extensions.Install   — force-installs from local path
# Extensions.Locked    — prevents the user from removing/disabling
# BlockAboutConfig     — hides about:config so user can't override policies
# BlockAboutAddons     — hides about:addons (the extensions page)
# ══════════════════════════════════════════════════════════════════════════
install_firefox_policy() {
  local policy_dir="$1"
  mkdir -p "$policy_dir"
  chmod 755 "$policy_dir"

  # Read extension ID from firefox manifest
  local manifest="$ROOT/firefox/manifest.json"
  local ext_id="sentinel@extension"
  if command -v jq &>/dev/null && [[ -f "$manifest" ]]; then
    ext_id=$(jq -r '.browser_specific_settings.gecko.id // "sentinel@extension"' "$manifest")
  fi

  cat > "$policy_dir/policies.json" <<EOF
{
  "policies": {
    "Extensions": {
      "Install": ["file://$INSTALL_DIR/sentinel-firefox.xpi"],
      "Locked": ["$ext_id"]
    },
    "BlockAboutConfig": true,
    "BlockAboutAddons": true,
    "DisableDeveloperTools": true,
    "DisableSafeMode": true,
    "DisablePrivateBrowsing": true
  }
}
EOF

  chmod 644 "$policy_dir/policies.json"
  chown root:root "$policy_dir/policies.json"
}

FIREFOX_POLICY_DIR="/etc/firefox/policies"
FIREFOX_ESR_POLICY_DIR="/etc/firefox-esr/policies"

if [[ -f /usr/bin/firefox || -f /usr/bin/firefox-bin ]]; then
  install_firefox_policy "$FIREFOX_POLICY_DIR"
  info "Firefox policy installed → $FIREFOX_POLICY_DIR/policies.json"
fi

if [[ -f /usr/bin/firefox-esr ]]; then
  install_firefox_policy "$FIREFOX_ESR_POLICY_DIR"
  info "Firefox ESR policy installed → $FIREFOX_ESR_POLICY_DIR/policies.json"
fi

# ══════════════════════════════════════════════════════════════════════════
# Lock the policy files themselves so the user can't edit or delete them
# (they can't anyway without sudo, but belt-and-suspenders)
# ══════════════════════════════════════════════════════════════════════════
chattr +i "$INSTALL_DIR/sentinel-chrome.zip"  2>/dev/null || true
chattr +i "$INSTALL_DIR/sentinel-firefox.xpi" 2>/dev/null || true

for f in \
  "$CHROME_POLICY_DIR/sentinel.json" \
  "$CHROMIUM_POLICY_DIR/sentinel.json" \
  "$FIREFOX_POLICY_DIR/policies.json" \
  "$FIREFOX_ESR_POLICY_DIR/policies.json"; do
  [[ -f "$f" ]] && chattr +i "$f" 2>/dev/null || true
done

info "Policy files immutably locked (chattr +i)."

# ══════════════════════════════════════════════════════════════════════════
# Summary
# ══════════════════════════════════════════════════════════════════════════
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
info "Enforcement complete. Restart the browser on the user account."
echo ""
echo "  Chrome  — visit chrome://policy  to verify policies loaded"
echo "  Firefox — visit about:policies   to verify policies loaded"
echo ""
warn "Chrome reminder: replace SENTINEL_EXTENSION_ID in:"
echo "  $CHROME_POLICY_DIR/sentinel.json"
echo "  (get the ID from chrome://extensions after first launch)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
