#!/bin/bash
set -e

# Configuration
BINARY_NAME="sentinel"
INSTALL_DIR="$HOME/.local/bin"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "──────── Sentinel Installer (Ubuntu/Linux) ────────"

# 1. Prerequisite Check
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust/Cargo is not installed."
    echo "   Please install it first: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 2. Build Release Binary
echo "Building release binary..."
cd "$SCRIPT_DIR"
cargo build --release --quiet
echo "Build complete."

# 3. Install Binary
mkdir -p "$INSTALL_DIR"
cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"
echo "Installed to $INSTALL_DIR/$BINARY_NAME"

# 4. Configure PATH (Robust Shell Detection)
SHELL_NAME=$(basename "$SHELL")
PROFILE=""

if [ "$SHELL_NAME" = "zsh" ]; then
    PROFILE="$HOME/.zshrc"
elif ["$SHELL_NAME" = "bash" ]; then
    # Prefer .bashrc for interactive shells on Ubuntu
    if [ -f "$HOME/.bashrc" ]; then
        PROFILE="$HOME/.bashrc"
    else
        PROFILE="$HOME/.profile"
    fi
else
    # Fallback to .profile
    PROFILE="$HOME/.profile"
fi

if [ -f "$PROFILE" ] && grep -q "$INSTALL_DIR" "$PROFILE"; then
    echo "PATH already configured in $PROFILE"
else
    echo "" >> "$PROFILE"
    echo "# Sentinel CLI" >> "$PROFILE"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$PROFILE"
    echo "Added $INSTALL_DIR to PATH in $PROFILE"
fi

echo ""
echo "Installation successful!"
echo "Restart your terminal or run: source $PROFILE"
echo "Then verify with: sentinel --help"
echo "───────────────────────────────────────────────────"
