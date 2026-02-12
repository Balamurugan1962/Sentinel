#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="sentinel"

echo "══════════════════════════════════════════"
echo "  Sentinel — Build & Install"
echo "══════════════════════════════════════════"
echo ""

# Build release binary
echo "→ Building release binary..."
cd "$SCRIPT_DIR"
cargo build --release

echo "→ Build complete."
echo ""

# Create install directory if needed
mkdir -p "$INSTALL_DIR"

# Copy binary
cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"
echo "→ Installed to $INSTALL_DIR/$BINARY_NAME"

# Add to PATH if not already there
SHELL_RC="$HOME/.zshrc"
if ! grep -q "$INSTALL_DIR" "$SHELL_RC" 2>/dev/null; then
    echo "" >> "$SHELL_RC"
    echo "# Sentinel CLI" >> "$SHELL_RC"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_RC"
    echo "→ Added $INSTALL_DIR to PATH in $SHELL_RC"
    echo ""
    echo "  Run 'source ~/.zshrc' or open a new terminal to use it."
else
    echo "→ PATH already configured."
fi

echo ""
echo "══════════════════════════════════════════"
echo "  ✅ Done! You can now run:"
echo ""
echo "    sentinel            # start server"
echo "    sentinel -ls        # list nodes"
echo "    sentinel --shutdown # stop server"
echo "    sentinel -h         # help"
echo "══════════════════════════════════════════"
