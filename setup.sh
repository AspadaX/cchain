#!/bin/bash

set -euo pipefail

# Determine OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture to GitHub release format
case $ARCH in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  armv7l) ARCH="arm" ;;
  powerpc) ARCH="powerpc" ;;
  s390x) ARCH="s390x" ;;
  i686) ARCH="i686" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Construct download filename for macOS
case $OS in
  linux*)
    LIBC="gnu"
    if ldd --version 2>&1 | grep -iq musl || [[ -n $(find /lib -name 'ld-musl-*' -print -quit) ]]; then
      LIBC="musl"
    fi
    FILENAME="cchain-Linux-${LIBC}-${ARCH}.tar.gz"
    ;;
  darwin*)
    if [[ "$ARCH" == "arm64" ]]; then
      FILENAME="cchain-macOS-arm64.tar.gz"
    else
      FILENAME="cchain-macOS-x86_64.tar.gz"
    fi
    OS="macOS"
    ;;
  freebsd*)
    FILENAME="cchain-FreeBSD-x86_64.tar.gz"
    ;;
  *)
    echo "Unsupported operating system: $(uname -s)"
    exit 1
    ;;
esac

# GitHub repository and latest release URL
REPO="AspadaX/cchain"
RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"

# Download URLs
DOWNLOAD_URL=$(curl -s $RELEASE_URL | grep -o "https://.*/$FILENAME" | head -1)
SHA_URL="$DOWNLOAD_URL.sha256"

# Create temporary directory
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Download and verify checksum
echo "Downloading $FILENAME..."
curl -L -o "$TMPDIR/$FILENAME" "$DOWNLOAD_URL"
curl -L -o "$TMPDIR/$FILENAME.sha256" "$SHA_URL"

echo "Verifying checksum..."
(cd "$TMPDIR" && shasum -a 256 -c "$FILENAME.sha256")

# Extract and install
tar xzf "$TMPDIR/$FILENAME" -C "$TMPDIR"
BIN_PATH=$(find "$TMPDIR" -name cchain -type f -print -quit)

if [[ -z $BIN_PATH ]]; then
  echo "Error: Could not find cchain binary in the package"
  exit 1
fi

INSTALL_DIR="/usr/local/bin"
echo "Installing cchain to $INSTALL_DIR..."
sudo mv -f "$BIN_PATH" "$INSTALL_DIR/cchain"
sudo chmod +x "$INSTALL_DIR/cchain"

echo "Successfully installed cchain $(cchain --version)"