#!/bin/bash
set -e

# Configuration
BINARY_NAME="sentinel"
INSTALL_DIR="$HOME/.local/bin"
URL="https://github.com/Balamurugan1962/Sentinel/releases/download/alpha/sentinel"

echo "### Sentinel Downloader (Ubuntu/Bash)"

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

# Path Configuration (Bash/Profile)
PROFILE="$HOME/.bashrc"
if [ ! -f "$PROFILE" ]; then
    PROFILE="$HOME/.profile"
fi

if grep -q "$INSTALL_DIR" "$PROFILE" 2>/dev/null; then
    echo "Path already configured in $PROFILE"
else
    echo "" >> "$PROFILE"
    echo "# Sentinel CLI" >> "$PROFILE"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$PROFILE"
    echo "Added $INSTALL_DIR to PATH in $PROFILE"
fi

echo ""
echo "Done!"
echo "Run 'source $PROFILE' to use sentinel."
