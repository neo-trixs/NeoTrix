#!/usr/bin/env bash
set -euo pipefail

# NeoTrix Documentation System — check & repair
# Usage: scripts/neotrix-doc-system.sh [check|repair]

DOCS_DIR="$(cd "$(dirname "$0")/../docs" && pwd)"
BLUEPRINT="$DOCS_DIR/../.blueprint/manifest.json"
ANCHOR="$DOCS_DIR/../.anchor/session.md"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
pass=0; fail=0

check() {
  local desc="$1" cond="$2"
  if eval "$cond"; then
    echo -e "${GREEN}  PASS${NC} $desc"
    ((pass++))
  else
    echo -e "${RED}  FAIL${NC} $desc"
    ((fail++))
  fi
}

echo "=== NeoTrix Documentation System Check ==="
echo ""

check "blueprint manifest exists"  "test -f '$BLUEPRINT'"
check "anchor session exists"      "test -f '$ANCHOR'"

for layer in 0-ARCHITECTURE 1-DESIGN 2-PLANS 3-API 4-GUIDES 5-LEARNING 6-REFERENCE; do
  check "$layer/ directory exists"  "test -d '$DOCS_DIR/$layer'"
  check "$layer/00-INDEX.md exists" "test -f '$DOCS_DIR/$layer/00-INDEX.md'"
done

check "docs/index.md exists"       "test -f '$DOCS_DIR/index.md'"
check ".vitepress/config.ts exists" "test -f '$DOCS_DIR/.vitepress/config.ts'"
check "public/ assets exist"        "test -d '$DOCS_DIR/public'"

echo ""
echo "=== Results: $pass passed, $fail failed ==="

if [ "$fail" -gt 0 ]; then
  echo -e "${YELLOW}Run with 'repair' to attempt fixes${NC}"
  exit 1
fi
}

repair() {
  echo "=== Repairing Documentation Structure ==="
  for layer in 0-ARCHITECTURE 1-DESIGN 2-PLANS 3-API 4-GUIDES 5-LEARNING 6-REFERENCE; do
    mkdir -p "$DOCS_DIR/$layer"
    if [ ! -f "$DOCS_DIR/$layer/00-INDEX.md" ]; then
      echo "# $layer" > "$DOCS_DIR/$layer/00-INDEX.md"
      echo "" >> "$DOCS_DIR/$layer/00-INDEX.md"
      echo "*This directory is part of the NeoTrix documentation system.*" >> "$DOCS_DIR/$layer/00-INDEX.md"
      echo "Created $layer/00-INDEX.md"
    fi
  done
  mkdir -p "$(dirname "$BLUEPRINT")" "$(dirname "$ANCHOR")"
  echo "Repair complete. Run with no args to verify."
}

case "${1:-check}" in
  check) check ;;
  repair) repair ;;
  *) echo "Usage: $0 [check|repair]" >&2; exit 1 ;;
esac
