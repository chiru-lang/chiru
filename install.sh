#!/usr/bin/env sh
set -e

VERSION="v0.1.0"
REPO="chiru-lang/chiru"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux) OS="linux" ;;
  Darwin) OS="macos" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARBALL="chiru-${VERSION}-${OS}-${ARCH}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${TARBALL}"

INSTALL_DIR="${HOME}/.local/bin"
TMP_DIR="$(mktemp -d)"

echo "Downloading $URL"
curl -fsSL "$URL" -o "$TMP_DIR/chiru.tar.gz"

echo "Extracting..."
tar -xzf "$TMP_DIR/chiru.tar.gz" -C "$TMP_DIR"

echo "Installing chiru to $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
mv "$TMP_DIR"/dist/chiru-${VERSION}-${OS}-${ARCH}/chiru "$INSTALL_DIR/chiru"
chmod +x "$INSTALL_DIR/chiru"

echo ""
echo "Chiru installed successfully."
echo "Binary location: $INSTALL_DIR/chiru"
echo ""
echo "If this is your first install, ensure ~/.local/bin is in your PATH:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
