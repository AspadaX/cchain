#!/bin/bash

set -euo pipefail

# Installation locations
BINARY_PATH="/usr/local/bin/cchain"
CONFIG_DIRS=(
    "/etc/cchain"
    "$HOME/.cchain"
)

# Remove main binary
if [[ -f "$BINARY_PATH" ]]; then
    echo "Removing cchain binary..."
    sudo rm -f "$BINARY_PATH"
fi

# Remove configuration directories
for dir in "${CONFIG_DIRS[@]}"; do
    if [[ -d "$dir" ]]; then
        echo "Removing configuration directory: $dir"
        rm -rf "$dir"
    fi
done

# Verify removal
if ! command -v cchain &> /dev/null; then
    echo "cchain was successfully uninstalled"
else
    echo "Warning: Some cchain components might still remain"
    exit 1
fi