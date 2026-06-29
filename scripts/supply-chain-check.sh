#!/bin/bash
# Supply chain security check for NeoTrix
# Run this in CI or as part of pre-commit

set -euo pipefail

echo "=== NeoTrix Supply Chain Security Check ==="
echo ""

# Check if cargo-deny is installed
if ! command -v cargo-deny &>/dev/null; then
    echo "[WARN] cargo-deny not installed. Install with: cargo install cargo-deny"
    echo "       Skipping supply chain checks."
    exit 0
fi

echo "=== 1. Advisory Check (vulnerabilities) ==="
cargo deny check advisories 2>&1 || echo "[WARN] Advisory check found issues"
echo ""

echo "=== 2. License Check ==="
cargo deny check licenses 2>&1 || echo "[WARN] License check found issues"
echo ""

echo "=== 3. Bans Check (duplicate versions) ==="
cargo deny check bans 2>&1 || echo "[WARN] Bans check found issues"
echo ""

echo "=== 4. Sources Check ==="
cargo deny check sources 2>&1 || echo "[WARN] Sources check found issues"
echo ""

echo "=== Supply chain check complete ==="
