#!/usr/bin/env bash
set -euo pipefail

NEOTRIX_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$NEOTRIX_ROOT"

SBOM_DIR="${SBOM_DIR:-sbom}"
SBOM_FORMAT="${SBOM_FORMAT:-cyclonedx}"
mkdir -p "$SBOM_DIR"

if ! command -v cargo-cyclonedx &>/dev/null; then
  echo "Installing cargo-cyclonedx..."
  cargo install cargo-cyclonedx
fi

echo "=== Generating SBOM (CycloneDX) ==="
cargo cyclonedx --all --format json --output-dir "$SBOM_DIR" 2>&1 | tee "$SBOM_DIR/generate.log"

SBOM_FILE="${SBOM_DIR}/neotrix-sbom.json"
if [ -f "$SBOM_FILE" ]; then
  python3 -c "
import json
with open('${SBOM_FILE}') as f:
    sbom = json.load(f)
components = sbom.get('components', [])
print(f'✓ SBOM generated: {len(components)} components')
print(f'  Format: {sbom.get(\"bomFormat\", \"unknown\")} v{sbom.get(\"specVersion\", \"?\")}')
print(f'  File: ${SBOM_FILE}')
"
else
  echo "Warning: SBOM file not found at ${SBOM_FILE}"
  ls -la "$SBOM_DIR/"
fi

echo ""
echo "=== SBOM Verification ==="
python3 -c "
import json
with open('${SBOM_FILE}') as f:
    sbom = json.load(f)
comps = sbom.get('components', [])
purls = [c.get('purl', '') for c in comps if c.get('purl')]
print(f'✓ Verified: {len(purls)} components with PURL identifiers')
print(f'✓ Total dependencies tracked: {len(comps)}')
"
