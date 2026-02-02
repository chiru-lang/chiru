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
mkdir -p "$INSTALL_DIR"

echo "Downloading $URL"
curl -fsSL "$URL" | tar -xz

echo "Installing chiru to $INSTALL_DIR"
mv dist/chiru-${VERSION}-${OS}-${ARCH}/chiru "${INSTALL_DIR}/chiru"
chmod +x "${INSTALL_DIR}/chiru"

echo "Installed: $(chiru --version)"
echo "Ensure ~/.local/bin is in your PATH"
