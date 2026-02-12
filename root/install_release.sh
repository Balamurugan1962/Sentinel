#!/bin/bash
set -e

# Configuration
REPO="balamurugan1962/sentinel" # ⚠️ REPLACE THIS with your specific repository
BINARY_NAME="sentinel"
INSTALL_DIR="$HOME/.local/bin"

echo "──────── Sentinel Downloader ────────"

# 1. Detect Architecture & OS
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

if [ "$ARCH" = "x86_64" ]; then
    ARCH="amd64"
elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
    ARCH="arm64"
else
    echo "❌ Unsupported architecture: $ARCH"
    exit 1
fi

TARGET="${BINARY_NAME}-${OS}-${ARCH}"
URL="https://github.com/${REPO}/releases/latest/download/${TARGET}"

# 2. Download
echo "→ Downloading $TARGET from $REPO..."
mkdir -p "$INSTALL_DIR"

if command -v curl >/dev/null; then
    curl -sL "$URL" -o "$INSTALL_DIR/$BINARY_NAME"
elif command -v wget >/dev/null; then
    wget -qO "$INSTALL_DIR/$BINARY_NAME" "$URL"
else
    echo "❌ Error: Need curl or wget to download."
    exit 1
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"
echo "✓ Installed to $INSTALL_DIR/$BINARY_NAME"

# 3. Path Configuration (Minimal)
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo "⚠️  Make sure $INSTALL_DIR is in your PATH." ;;
esac

echo "✅ Done!"
