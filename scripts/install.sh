#!/bin/bash
set -e

REPO="seapy/rink"
INSTALL_DIR="$HOME/.local/bin"

# macOS only
if [[ "$(uname)" != "Darwin" ]]; then
  echo "Error: rink only supports macOS."
  exit 1
fi

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
  arm64)  TARGET="aarch64-apple-darwin" ;;
  x86_64) TARGET="x86_64-apple-darwin" ;;
  *)      echo "Error: unsupported architecture: $ARCH"; exit 1 ;;
esac

# Get latest release tag
echo "Fetching latest release..."
TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed 's/.*: "//;s/".*//')

if [[ -z "$TAG" ]]; then
  echo "Error: could not find latest release."
  exit 1
fi

echo "Installing rink $TAG ($TARGET)..."

# Download and extract
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

curl -fsSL "https://github.com/$REPO/releases/download/$TAG/rink-$TARGET.tar.gz" -o "$TMP_DIR/rink.tar.gz"
tar xzf "$TMP_DIR/rink.tar.gz" -C "$TMP_DIR"

# Install
mkdir -p "$INSTALL_DIR"
cp "$TMP_DIR/rink" "$INSTALL_DIR/rink"
chmod +x "$INSTALL_DIR/rink"

echo "Installed rink $TAG to $INSTALL_DIR/rink"

# Check PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  echo ""
  echo "Add to your shell profile:"
  echo '  export PATH="$HOME/.local/bin:$PATH"'
fi

echo ""
echo "Run 'rink' to start."
