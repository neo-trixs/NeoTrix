#!/bin/bash
set -euo pipefail

# Build the React frontend for neotrix-web.
# Output goes to neotrix-web-frontend/dist/
#
# Prerequisites: Node.js >= 18, npm

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FRONTEND_DIR="$SCRIPT_DIR/../neotrix-web-frontend"

echo "==> Installing dependencies..."
cd "$FRONTEND_DIR"
npm install

echo "==> Building frontend..."
npm run build

echo "==> Done! Output: $FRONTEND_DIR/dist"
echo "==> neotrix-web will serve this directory when NEOTRIX_WEB_DIST is unset."
