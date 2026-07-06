#!/bin/bash
set -euo pipefail

# NeoTrix Uninstaller

BIN_DIR="${NEOTRIX_HOME:-$HOME/.neotrix}"

echo "Removing NeoTrix..."
rm -rf "$BIN_DIR"
echo "Removed $BIN_DIR"

# Clean PATH
SHELL_RC="${HOME}/.$(basename "$SHELL" 2>/dev/null || echo "bash")rc"
sed -i '' '/NEOTRIX_HOME/d' "$SHELL_RC" 2>/dev/null || true

echo "✅ NeoTrix uninstalled."
