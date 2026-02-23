#!/bin/bash
set -e

REPO="seapy/rink"
INSTALL_DIR="$HOME/.local/bin"
TMP_DIR=$(mktemp -d)

cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

echo "Installing rink..."

# Check for cargo
if ! command -v cargo &>/dev/null; then
  echo "Rust not found. Installing via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
fi

# Clone and build
echo "Cloning $REPO..."
git clone --depth 1 "https://github.com/$REPO.git" "$TMP_DIR/rink"

echo "Building..."
cd "$TMP_DIR/rink"
cargo build --release

# Install
mkdir -p "$INSTALL_DIR"
cp target/release/rink "$INSTALL_DIR/rink"
chmod +x "$INSTALL_DIR/rink"

echo ""
echo "Installed rink to $INSTALL_DIR/rink"

# Check PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  echo ""
  echo "Add to your shell profile:"
  echo '  export PATH="$HOME/.local/bin:$PATH"'
fi

echo ""
echo "Run 'rink' to start."
