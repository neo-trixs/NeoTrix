#!/bin/bash
# NeoTrix Auto-Patrol Script
# Runs compile check + cross-layer audit + dead code detection
# Usage: ./scripts/auto-patrol.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC_DIR="$PROJECT_ROOT/neotrix-core/src"

echo "=========================================="
echo "NeoTrix Auto-Patrol $(date '+%Y-%m-%d %H:%M')"
echo "=========================================="

# 1. Compile Check
echo -e "\n${YELLOW}[1/4] Compile Check...${NC}"
if cargo check -p neotrix --lib 2>&1 | grep -q "^error"; then
    echo -e "${RED}FAIL: Compilation errors detected${NC}"
    cargo check -p neotrix --lib 2>&1 | grep "^error" | head -10
    exit 1
else
    echo -e "${GREEN}PASS: Clean compilation${NC}"
fi

# 2. Cross-Layer Audit
echo -e "\n${YELLOW}[2/4] Cross-Layer Audit...${NC}"
VIOLATIONS=0
for layer in l1_action l2_perception l3_embodiment; do
    num=$(echo $layer | sed 's/l\([0-9]\).*/\1/')
    while IFS=: read -r file line content; do
        target=$(echo "$content" | sed 's/.*crate::l\([0-9]\).*/\1/')
        if [ -n "$target" ] && [ "$target" -gt "$num" ] 2>/dev/null; then
            echo -e "${RED}  L${num}→L${target}: $file:$line${NC}"
            VIOLATIONS=$((VIOLATIONS + 1))
        fi
    done < <(rg "^use crate::l[0-9]" "$SRC_DIR/$layer"/ --no-heading 2>/dev/null)
done
if [ "$VIOLATIONS" -eq 0 ]; then
    echo -e "${GREEN}PASS: No cross-layer violations${NC}"
else
    echo -e "${YELLOW}WARN: $VIOLATIONS cross-layer violations found${NC}"
fi

# 3. Dead Code Detection
echo -e "\n${YELLOW}[3/4] Dead Code Detection...${NC}"
DEAD_COUNT=$(rg "^pub (fn|struct|enum|trait) " "$SRC_DIR" --no-heading 2>/dev/null | while IFS=: read -r file line content; do
    item=$(echo "$content" | sed 's/pub [a-z]* \([A-Za-z_]*\).*/\1/')
    if [ -n "$item" ] && [ ${#item} -gt 3 ]; then
        count=$(rg "\b$item\b" "$SRC_DIR" --no-heading 2>/dev/null | grep -v "$file" | wc -l)
        if [ "$count" -eq 0 ]; then
            echo "$item"
        fi
    fi
done 2>/dev/null | wc -l)
echo -e "${YELLOW}  Found ~$DEAD_COUNT potentially dead pub items${NC}"

# 4. Config Proliferation
echo -e "\n${YELLOW}[4/4] Config Proliferation...${NC}"
CONFIG_COUNT=$(rg "^pub struct \w*Config" "$SRC_DIR" --no-heading 2>/dev/null | wc -l)
echo -e "${YELLOW}  Found $CONFIG_COUNT Config structs${NC}"

echo -e "\n=========================================="
echo "Patrol Complete"
echo "=========================================="
