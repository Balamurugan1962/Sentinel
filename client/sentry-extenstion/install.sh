#!/usr/bin/env bash
# =============================================================================
# Sentinel Browser Policy & Extension Installer
# Target: Ubuntu (18.04 / 20.04 / 22.04 / 24.04)
# Scope:  Policy + extension installation only (no binary, no systemd service)
# =============================================================================
set -euo pipefail
IFS=$'\n\t'

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------
readonly SCRIPT_NAME="$(basename "$0")"
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly LOG_FILE="/var/log/sentinel-install.log"
readonly TIMESTAMP="$(date '+%Y-%m-%d %T')"

readonly INSTALL_DIR="/opt/sentinel"
readonly EXT_DIR="$INSTALL_DIR/extensions"

# Firefox
readonly FIREFOX_POLICY_DIR="/etc/firefox/policies"
readonly FIREFOX_SNAP_POLICY_DIR="/etc/firefox/policies"          # snap uses same path via /etc bind
readonly FIREFOX_SNAP_REAL_POLICY_DIR="/snap/firefox/current/usr/lib/firefox/distribution"
readonly FIREFOX_DEB_POLICY_DIR="/usr/lib/firefox/distribution"   # Mozilla PPA / deb
readonly FIREFOX_ESR_POLICY_DIR="/usr/lib/firefox-esr/distribution"
readonly FIREFOX_FLATPAK_POLICY_DIR="$HOME/.var/app/org.mozilla.firefox/data/firefox/policies"

# Chrome / Chromium
readonly CHROME_POLICY_DIR="/etc/opt/chrome/policies/managed"
readonly CHROMIUM_POLICY_DIR="/etc/chromium/policies/managed"
readonly CHROMIUM_SNAP_POLICY_DIR="/etc/chromium-browser/policies/managed"  # older Ubuntu snap
readonly CHROMIUM_FLATPAK_POLICY_DIR="$HOME/.var/app/org.chromium.Chromium/config/chromium/policies/managed"
readonly BRAVE_POLICY_DIR="/etc/opt/brave/policies/managed"

# Extension identifiers
readonly FIREFOX_EXTENSION_ID="monitor@sentinel"
readonly CHROME_EXTENSION_ID="abcdefghijklmnopabcdefghijklmnop"   # replace with real CRX ID

# Source assets (expected alongside this script)
readonly SRC_XPI="$SCRIPT_DIR/sentry-extension/firefox/monitor.xpi"
readonly SRC_CHROME_DIR="$SCRIPT_DIR/sentry-extension/chrome"

# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------
_log()  { local level="$1"; shift; echo "[$TIMESTAMP][$level] $*" | tee -a "$LOG_FILE" >&2; }
log()   { _log "INFO " "$*"; }
warn()  { _log "WARN " "$*"; }
error() { _log "ERROR" "$*"; }
die()   { error "$*"; exit 1; }

# ---------------------------------------------------------------------------
# Preflight checks
# ---------------------------------------------------------------------------
require_root() {
  [[ $EUID -eq 0 ]] || die "This installer must be run as root (sudo $SCRIPT_NAME)"
}

check_ubuntu() {
  if [[ ! -f /etc/os-release ]]; then
    warn "Cannot detect OS; proceeding anyway"
    return
  fi
  # shellcheck source=/dev/null
  source /etc/os-release
  if [[ "${ID:-}" != "ubuntu" ]]; then
    warn "Detected OS: ${PRETTY_NAME:-unknown}. Script is optimised for Ubuntu; results may vary."
  else
    log "Detected OS: ${PRETTY_NAME}"
  fi
}

check_source_assets() {
  local missing=0
  [[ -f "$SRC_XPI" ]]       || { warn "Missing Firefox XPI: $SRC_XPI";         missing=1; }
  [[ -d "$SRC_CHROME_DIR" ]] || { warn "Missing Chrome extension dir: $SRC_CHROME_DIR"; missing=1; }
  if (( missing )); then
    warn "Some source assets are missing — affected browsers will be skipped."
  fi
}

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# Safe directory creation with correct permissions
make_policy_dir() {
  local dir="$1"
  if [[ -L "$dir" ]]; then
    die "Policy path $dir is a symlink — aborting to avoid traversal attack."
  fi
  mkdir -p "$dir"
  chmod 755 "$dir"
}

# Atomically write a policy file (write → verify JSON → move into place)
write_policy() {
  local dest="$1"
  local content="$2"
  local tmp
  tmp="$(mktemp)"
  printf '%s\n' "$content" > "$tmp"

  # Validate JSON if python3 is available
  if command -v python3 &>/dev/null; then
    python3 -c "import sys,json; json.load(open('$tmp'))" \
      || { rm -f "$tmp"; die "Generated invalid JSON for $dest — aborting."; }
  fi

  chmod 644 "$tmp"
  mv "$tmp" "$dest"
  log "  Written: $dest"
}

# ---------------------------------------------------------------------------
# Extension asset installation
# ---------------------------------------------------------------------------
install_extension_assets() {
  log "Installing extension assets to $EXT_DIR"
  if [[ -f "$SRC_XPI" ]]; then
    make_policy_dir "$EXT_DIR/firefox"
    install -m 644 "$SRC_XPI" "$EXT_DIR/firefox/monitor.xpi"
  fi
  if [[ -d "$SRC_CHROME_DIR" ]]; then
    make_policy_dir "$EXT_DIR/chrome"
    cp -r --preserve=mode "$SRC_CHROME_DIR/." "$EXT_DIR/chrome/"
  fi
}

# ---------------------------------------------------------------------------
# Firefox policy
# ---------------------------------------------------------------------------

_firefox_policy_json() {
  cat <<JSON
{
  "policies": {
    "DisableDeveloperTools": true,
    "DNSOverHTTPS": { "Enabled": false },
    "ExtensionSettings": {
      "${FIREFOX_EXTENSION_ID}": {
        "installation_mode": "force_installed",
        "install_url": "file://${EXT_DIR}/firefox/monitor.xpi"
      }
    }
  }
}
JSON
}

_write_firefox_policy() {
  local policy_dir="$1"
  local label="${2:-Firefox}"
  make_policy_dir "$policy_dir"
  write_policy "$policy_dir/policies.json" "$(_firefox_policy_json)"
  log "  $label policy installed → $policy_dir/policies.json"
}

detect_firefox_variants() {
  # Returns a list of "label:policy_dir" pairs for every Firefox variant found
  local variants=()

  # 1. Standard deb (from Mozilla PPA or distro)
  if [[ -f /usr/bin/firefox ]]; then
    # Distinguish snap-wrapped binary vs real deb
    if readlink -f /usr/bin/firefox 2>/dev/null | grep -q snap; then
      # Snap Firefox: policy goes into /etc/firefox/policies (snap bind-mounts it)
      variants+=("Firefox(snap):$FIREFOX_POLICY_DIR")
    else
      variants+=("Firefox(deb):$FIREFOX_DEB_POLICY_DIR")
    fi
  fi

  # 2. Firefox snap (may exist even without /usr/bin/firefox symlink on newer Ubuntu)
  if snap list firefox &>/dev/null 2>&1; then
    # Avoid duplicating if already added above
    local already=0
    for v in "${variants[@]:-}"; do [[ "$v" == *"Firefox(snap)"* ]] && already=1; done
    (( already )) || variants+=("Firefox(snap):$FIREFOX_POLICY_DIR")
  fi

  # 3. Firefox ESR
  if [[ -f /usr/bin/firefox-esr ]]; then
    variants+=("Firefox-ESR:$FIREFOX_ESR_POLICY_DIR")
  fi

  # 4. Firefox Flatpak (user install — not root-owned, best-effort)
  if flatpak list --app 2>/dev/null | grep -q "org.mozilla.firefox"; then
    variants+=("Firefox(flatpak):$FIREFOX_FLATPAK_POLICY_DIR")
  fi

  printf '%s\n' "${variants[@]:-}"
}

install_firefox_policies() {
  log "--- Firefox ---"
  if ! [[ -f "$SRC_XPI" ]]; then
    warn "Skipping Firefox: XPI asset not found."
    return
  fi

  local found=0
  while IFS= read -r entry; do
    [[ -z "$entry" ]] && continue
    local label="${entry%%:*}"
    local policy_dir="${entry#*:}"
    _write_firefox_policy "$policy_dir" "$label"
    found=1
  done < <(detect_firefox_variants)

  if (( found == 0 )); then
    log "  No Firefox installation detected — skipping."
  fi
}

# ---------------------------------------------------------------------------
# Chromium / Chrome policy
# ---------------------------------------------------------------------------

_chrome_policy_json() {
  cat <<JSON
{
  "DeveloperToolsAvailability": 2,
  "ExtensionInstallForcelist": [
    "${CHROME_EXTENSION_ID};https://clients2.google.com/service/update2/crx"
  ]
}
JSON
}

_write_chrome_policy() {
  local policy_dir="$1"
  local label="${2:-Chrome}"
  make_policy_dir "$policy_dir"
  write_policy "$policy_dir/sentinel.json" "$(_chrome_policy_json)"
  log "  $label policy installed → $policy_dir/sentinel.json"
}

detect_chrome_variants() {
  local variants=()

  # Google Chrome (stable / beta / unstable)
  for bin in google-chrome google-chrome-stable google-chrome-beta google-chrome-unstable; do
    if command -v "$bin" &>/dev/null || [[ -f "/usr/bin/$bin" ]]; then
      variants+=("Google-Chrome:$CHROME_POLICY_DIR")
      break
    fi
  done

  # Chromium deb
  if command -v chromium-browser &>/dev/null || [[ -f /usr/bin/chromium-browser ]]; then
    variants+=("Chromium(deb):$CHROMIUM_SNAP_POLICY_DIR")
  fi

  # Chromium snap
  if snap list chromium &>/dev/null 2>&1; then
    local already=0
    for v in "${variants[@]:-}"; do [[ "$v" == *"Chromium"* ]] && already=1; done
    (( already )) || variants+=("Chromium(snap):$CHROMIUM_SNAP_POLICY_DIR")
  fi

  # Chromium flatpak
  if flatpak list --app 2>/dev/null | grep -q "org.chromium.Chromium"; then
    variants+=("Chromium(flatpak):$CHROMIUM_FLATPAK_POLICY_DIR")
  fi

  # Brave
  if command -v brave-browser &>/dev/null || [[ -f /usr/bin/brave-browser ]]; then
    variants+=("Brave:$BRAVE_POLICY_DIR")
  fi

  printf '%s\n' "${variants[@]:-}"
}

install_chrome_policies() {
  log "--- Chromium/Chrome ---"
  if ! [[ -d "$SRC_CHROME_DIR" ]]; then
    warn "Skipping Chromium/Chrome: extension source directory not found."
    return
  fi

  local found=0
  while IFS= read -r entry; do
    [[ -z "$entry" ]] && continue
    local label="${entry%%:*}"
    local policy_dir="${entry#*:}"
    _write_chrome_policy "$policy_dir" "$label"
    found=1
  done < <(detect_chrome_variants)

  if (( found == 0 )); then
    log "  No Chrome/Chromium installation detected — skipping."
  fi
}

# ---------------------------------------------------------------------------
# Verification
# ---------------------------------------------------------------------------
verify_installation() {
  log "--- Verification ---"
  local ok=1

  _check_file() {
    if [[ -f "$1" ]]; then
      log "  ✓ $1"
    else
      warn "  ✗ Missing: $1"
      ok=0
    fi
  }

  # Firefox
  for dir in \
    "$FIREFOX_POLICY_DIR" \
    "$FIREFOX_DEB_POLICY_DIR" \
    "$FIREFOX_ESR_POLICY_DIR"; do
    [[ -f "$dir/policies.json" ]] && _check_file "$dir/policies.json"
  done

  # Chrome/Chromium
  for dir in \
    "$CHROME_POLICY_DIR" \
    "$CHROMIUM_POLICY_DIR" \
    "$CHROMIUM_SNAP_POLICY_DIR"; do
    [[ -f "$dir/sentinel.json" ]] && _check_file "$dir/sentinel.json"
  done

  if (( ok )); then
    log "Verification passed."
  else
    warn "Some expected files are missing — review the log above."
  fi
}

# ---------------------------------------------------------------------------
# Usage / entrypoint
# ---------------------------------------------------------------------------
usage() {
  cat <<USAGE
Usage: sudo $SCRIPT_NAME [OPTIONS]

Options:
  --firefox-only    Install Firefox policy only
  --chrome-only     Install Chrome/Chromium policy only
  --dry-run         Show what would be done without making changes
  -h, --help        Show this help

USAGE
  exit 0
}

main() {
  local do_firefox=1
  local do_chrome=1
  local dry_run=0

  while (( $# > 0 )); do
    case "$1" in
      --firefox-only) do_chrome=0 ;;
      --chrome-only)  do_firefox=0 ;;
      --dry-run)      dry_run=1; warn "DRY-RUN mode — no files will be written." ;;
      -h|--help)      usage ;;
      *) die "Unknown option: $1" ;;
    esac
    shift
  done

  if (( dry_run )); then
    log "Detected Firefox variants:"
    detect_firefox_variants | sed 's/^/  /'
    log "Detected Chrome/Chromium variants:"
    detect_chrome_variants | sed 's/^/  /'
    exit 0
  fi

  # Ensure log file is writable before anything else
  touch "$LOG_FILE" 2>/dev/null || LOG_FILE="/tmp/sentinel-install.log"

  require_root
  check_ubuntu
  check_source_assets

  log "======================================================"
  log " Sentinel Policy & Extension Installer"
  log "======================================================"

  install_extension_assets

  (( do_firefox )) && install_firefox_policies
  (( do_chrome  )) && install_chrome_policies

  verify_installation

  log "======================================================"
  log " Installation complete. Restart all open browsers."
  log " Log: $LOG_FILE"
  log "======================================================"
}

main "$@"
