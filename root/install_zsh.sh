#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="sentinel"
SHELL_RC="$HOME/.zshrc"

echo "### Sentinel Installer (macOS/Zsh)"

# 1. Build
echo "Building release binary..."
cd "$SCRIPT_DIR"
cargo build --release --quiet
echo "Build complete."

# 2. Install
mkdir -p "$INSTALL_DIR"
rm -f "$INSTALL_DIR/$BINARY_NAME"
cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"
# Remove quarantine attributes if present (fixes 'killed' on macOS)
xattr -c "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null || true
echo "Installed to $INSTALL_DIR/$BINARY_NAME"

# 3. Path Configuration
if grep -q "$INSTALL_DIR" "$SHELL_RC" 2>/dev/null; then
    echo "Path already configured in $SHELL_RC"
else
    echo "" >> "$SHELL_RC"
    echo "# Sentinel CLI" >> "$SHELL_RC"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_RC"
    echo "Added $INSTALL_DIR to PATH in $SHELL_RC"
fi

echo ""
echo "Done!"
echo "Run 'source ~/.zshrc' to start using sentinel."
