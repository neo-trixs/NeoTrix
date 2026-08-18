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
  # tauri signer generate 会同时产出 <key>.pub (真实 minisign 公钥文件)
  CI=true npx --yes @tauri-apps/cli signer generate -w "$PRIVATE_KEY" --ci
fi

# Step 2: Extract the real public key from the signer-generated .pub file
if [ -f "${PRIVATE_KEY}.pub" ]; then
  # signer generate 的 .pub 内容即为 tauri.conf.json 需要的 base64 minisign 公钥
  cp "${PRIVATE_KEY}.pub" "$PUBLIC_KEY"
  echo "[2/4] Public key extracted from ${PRIVATE_KEY}.pub → $PUBLIC_KEY"
elif [ -f "$PUBLIC_KEY" ]; then
  echo "[2/4] Using existing public key: $PUBLIC_KEY"
else
  echo "[2/4] ERROR: neither ${PRIVATE_KEY}.pub nor $PUBLIC_KEY exists."
  echo "  Run 'tauri signer generate -w $PRIVATE_KEY' first."
  exit 1
fi

PUBKEY=$(cat "$PUBLIC_KEY")
echo "  Public key: $PUBKEY"

# Step 3: Update tauri.conf.json
echo "[3/4] Updating $TAURI_CONF ..."
if grep -q '"pubkey"' "$TAURI_CONF"; then
  # Replace existing pubkey value (minisign base64 含 '/' '+' '='，用 | 作 sed 定界符)
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
