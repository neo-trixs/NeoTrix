#!/bin/bash
set -euo pipefail

# NeoTrix Uninstaller (thin wrapper → sysops uninstall)

if ! command -v neotrix >/dev/null 2>&1; then
    echo "neotrix not found in PATH; cannot run safe uninstall (sysops uninstall)"
    echo "Falling back to legacy removal..."
    BIN_DIR="${NEOTRIX_HOME:-$HOME/.neotrix}"
    echo "Removing $BIN_DIR"
    rm -rf "$BIN_DIR"
    SHELL_RC="${HOME}/.$(basename "$SHELL" 2>/dev/null || echo "bash")rc"
    sed -i '' '/NEOTRIX_HOME/d' "$SHELL_RC" 2>/dev/null || true
    echo "✅ NeoTrix uninstalled (legacy)"
    exit 0
fi

exec neotrix sysops uninstall