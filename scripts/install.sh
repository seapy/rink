#!/usr/bin/env bash
set -euo pipefail

REPO="seapy/rink"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Error: required command '$1' not found." >&2
    exit 1
  fi
}

print_linux_dependency_help() {
  cat <<'HELP'

Dependencies:
  tmux is required.
  zellij is required for the default split-frame UI. It is not required for: rink --standalone

Next steps:
  rink doctor          # check what is missing
  rink setup           # install missing dependencies
  rink setup --dry-run # show the install plan without running it
HELP
}

need_cmd curl
need_cmd grep
need_cmd sed
need_cmd tar
need_cmd mktemp
need_cmd uname

# Detect platform and architecture.
OS=$(uname -s)
ARCH=$(uname -m)
case "$OS:$ARCH" in
  Darwin:arm64)   TARGET="aarch64-apple-darwin" ;;
  Darwin:x86_64)  TARGET="x86_64-apple-darwin" ;;
  Linux:x86_64)   TARGET="x86_64-unknown-linux-gnu" ;;
  Linux:aarch64)  TARGET="aarch64-unknown-linux-gnu" ;;
  Linux:arm64)    TARGET="aarch64-unknown-linux-gnu" ;;
  *)
    echo "Error: unsupported platform: $OS $ARCH" >&2
    echo "Supported targets: macOS arm64/x86_64, Linux x86_64/aarch64" >&2
    exit 1
    ;;
esac

# Get latest release tag.
echo "Fetching latest release..."
TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed 's/.*: "//;s/".*//')

if [[ -z "$TAG" ]]; then
  echo "Error: could not find latest release." >&2
  exit 1
fi

echo "Installing rink $TAG ($TARGET)..."

# Download and extract.
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

ASSET="rink-$TARGET.tar.gz"
URL="https://github.com/$REPO/releases/download/$TAG/$ASSET"
if ! curl -fsSL "$URL" -o "$TMP_DIR/rink.tar.gz"; then
  echo "Error: release asset not found for $TARGET: $URL" >&2
  echo "If this is an older release, build from source with: cargo install --git https://github.com/$REPO" >&2
  exit 1
fi

tar xzf "$TMP_DIR/rink.tar.gz" -C "$TMP_DIR"

# Install.
mkdir -p "$INSTALL_DIR"
cp "$TMP_DIR/rink" "$INSTALL_DIR/rink"
chmod +x "$INSTALL_DIR/rink"

echo "Installed rink $TAG to $INSTALL_DIR/rink"

# Check PATH.
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo ""
    echo "Add to your shell profile:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
    ;;
esac

if [[ "$OS" == "Linux" ]]; then
  print_linux_dependency_help
else
  echo ""
  echo "Dependencies: tmux and zellij are required, and rink can install them via Homebrew on first run."
fi

echo ""
echo "Run 'rink' to start. Use 'rink --standalone' if you only want the dashboard without zellij."
