#!/bin/bash
set -e

echo "Building rink in release mode..."
cargo build --release

BINARY="target/release/rink"
INSTALL_DIR="$HOME/.local/bin"

mkdir -p "$INSTALL_DIR"
cp "$BINARY" "$INSTALL_DIR/rink"
chmod +x "$INSTALL_DIR/rink"

echo "Installed rink to $INSTALL_DIR/rink"
echo ""
echo "Make sure $INSTALL_DIR is in your PATH:"
echo '  export PATH="$HOME/.local/bin:$PATH"'
