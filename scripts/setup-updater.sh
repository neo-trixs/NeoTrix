#!/usr/bin/env bash
# Tauri Updater signing key setup for NeoTrix
# Usage: bash scripts/setup-updater.sh
set -euo pipefail

NEOTRIX_DIR="${HOME}/.neotrix"
PRIVATE_KEY="${NEOTRIX_DIR}/tauri-updater.key"
PUBLIC_KEY="${NEOTRIX_DIR}/updater.pub"
TAURI_CONF="src-tauri/tauri.conf.json"

echo "=== NeoTrix Tauri Updater Key Setup ==="

# Step 1: Generate keys (if not exist)
if [ -f "$PRIVATE_KEY" ]; then
  echo "[1/4] Private key exists: $PRIVATE_KEY"
  echo "  To regenerate: rm $PRIVATE_KEY && re-run this script"
else
  echo "[1/4] Generating signing key pair..."
  mkdir -p "$NEOTRIX_DIR"
  npx @tauri-apps/cli signer generate -w "$PRIVATE_KEY"
fi

if [ ! -f "$PUBLIC_KEY" ]; then
  echo "[2/4] Public key not found at $PUBLIC_KEY"
  echo "  Copy it from the output above or extract from private key."
  echo "  Temporary: writing placeholder -- replace with actual public key!"
  echo "replace-with-your-ed25519-public-key" > "$PUBLIC_KEY"
else
  echo "[2/4] Public key exists: $PUBLIC_KEY"
fi

PUBKEY=$(cat "$PUBLIC_KEY")
echo "  Public key: $PUBKEY"

# Step 3: Update tauri.conf.json
echo "[3/4] Updating $TAURI_CONF ..."
if grep -q '"pubkey"' "$TAURI_CONF"; then
  # Replace existing pubkey value
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s|\"pubkey\":[[:space:]]*\".*\"|\"pubkey\": \"$PUBKEY\"|" "$TAURI_CONF"
  else
    sed -i "s|\"pubkey\":[[:space:]]*\".*\"|\"pubkey\": \"$PUBKEY\"|" "$TAURI_CONF"
  fi
  echo "  Updated pubkey in $TAURI_CONF"
else
  echo "  WARNING: 'pubkey' field not found in $TAURI_CONF. Add it manually:"
  echo '  "pubkey": "'"$PUBKEY"'"'
fi

# Step 4: GitHub secrets reminder
echo ""
echo "[4/4] === GitHub Secrets Setup ==="
echo "Add these secrets to your GitHub repo (Settings > Secrets and variables > Actions):"
echo ""
echo "  Name: TAURI_PRIVATE_KEY"
echo "  Value: (contents of $PRIVATE_KEY)"
echo ""
echo "  Name: TAURI_KEY_PASSWORD"
echo "  Value: (your password, or leave empty if no password)"
echo ""
echo "=== Done ==="
echo ""
echo "To create a release, push a tag:"
echo "  git tag v0.2.0 && git push origin v0.2.0"
echo ""
echo "Or trigger manually: GitHub -> Actions -> Release -> Run workflow"
