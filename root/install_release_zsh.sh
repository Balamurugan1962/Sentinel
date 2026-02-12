#!/bin/zsh
set -e

BINARY_NAME="sentinel"
INSTALL_DIR="$HOME/.local/bin"

URL="https://github.com/Balamurugan1962/Sentinel/releases/download/alpha/sentinel"
SHELL_RC="$HOME/.zshrc"

echo "### Sentinel Downloader (Zsh)"

echo "Downloading $BINARY_NAME..."
mkdir -p "$INSTALL_DIR"

if command -v curl >/dev/null; then
    curl -fsL "$URL" -o "$INSTALL_DIR/$BINARY_NAME"
elif command -v wget >/dev/null; then
    wget -qO "$INSTALL_DIR/$BINARY_NAME" "$URL"
else
    echo "Error: Need curl or wget to download."
    exit 1
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"
echo "Installed to $INSTALL_DIR/$BINARY_NAME"

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
