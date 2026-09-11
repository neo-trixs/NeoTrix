#!/bin/bash
# NeoTrix Auto-Patrol Script v2
# Runs compile check + cross-layer audit + dead code detection + gateway sprawl + config audit
# Usage: ./scripts/auto-patrol.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC_DIR="$PROJECT_ROOT/neotrix-core/src"

echo "=========================================="
echo "NeoTrix Auto-Patrol v2 $(date '+%Y-%m-%d %H:%M')"
echo "=========================================="

# 1. Compile Check
echo -e "\n${YELLOW}[1/6] Compile Check...${NC}"
if cargo check -p neotrix --lib 2>&1 | grep -q "^error"; then
    echo -e "${RED}FAIL: Compilation errors detected${NC}"
    cargo check -p neotrix --lib 2>&1 | grep "^error" | head -10
    exit 1
else
    echo -e "${GREEN}PASS: Clean compilation${NC}"
fi

# 2. Cross-Layer Audit (downward only — L(n) importing L(n+k) where k>0 is violation)
echo -e "\n${YELLOW}[2/6] Cross-Layer Audit...${NC}"
VIOLATIONS=0

# L1 must not import from L2-L6
for file in $(rg "^use crate::l[2-6]" "$SRC_DIR/l1_action/" --no-heading -l 2>/dev/null | grep -v test); do
    echo -e "${RED}  VIOLATION: L1 imports higher layer: $file${NC}"
    VIOLATIONS=$((VIOLATIONS + 1))
done

# L2 must not import from L3-L6
for file in $(rg "^use crate::l[3-6]" "$SRC_DIR/l2_perception/" --no-heading -l 2>/dev/null | grep -v test); do
    echo -e "${RED}  VIOLATION: L2 imports higher layer: $file${NC}"
    VIOLATIONS=$((VIOLATIONS + 1))
done

# L3 must not import from L4-L6 (upward dependency)
for file in $(rg "^use crate::l[4-6]" "$SRC_DIR/l3_embodiment/" --no-heading -l 2>/dev/null | grep -v test); do
    echo -e "${RED}  VIOLATION: L3 imports higher layer: $file${NC}"
    VIOLATIONS=$((VIOLATIONS + 1))
done

# L4 must not import from L5-L6 (upward dependency)
for file in $(rg "^use crate::l[5-6]" "$SRC_DIR/l4_emotion/" --no-heading -l 2>/dev/null | grep -v test); do
    echo -e "${RED}  VIOLATION: L4 imports higher layer: $file${NC}"
    VIOLATIONS=$((VIOLATIONS + 1))
done

# L5→L6 is acceptable (cognition uses meta-cognition services)
L5_TO_L6=$(rg "^use crate::l[6]" "$SRC_DIR/l5_cognition/" --no-heading 2>/dev/null | grep -v test | wc -l | tr -d ' ')
if [ "$L5_TO_L6" -gt 0 ]; then
    echo -e "${YELLOW}  INFO: L5→L6 imports: $L5_TO_L6 (acceptable: cognition uses meta services)${NC}"
fi

if [ "$VIOLATIONS" -eq 0 ]; then
    echo -e "${GREEN}PASS: No cross-layer violations${NC}"
else
    echo -e "${RED}FAIL: $VIOLATIONS cross-layer violations found${NC}"
fi

# 3. Dead Code Detection
echo -e "\n${YELLOW}[3/6] Dead Code Detection...${NC}"
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
echo -e "\n${YELLOW}[4/6] Config Proliferation...${NC}"
CONFIG_COUNT=$(rg "^pub struct \w*Config" "$SRC_DIR" --no-heading 2>/dev/null | wc -l)
if [ "$CONFIG_COUNT" -gt 250 ]; then
    echo -e "${RED}  WARN: $CONFIG_COUNT Config structs (threshold: 250)${NC}"
else
    echo -e "${GREEN}  OK: $CONFIG_COUNT Config structs${NC}"
fi

# 5. Gateway Sprawl Check
echo -e "\n${YELLOW}[5/6] Gateway Sprawl...${NC}"
GATEWAY_DIR="$SRC_DIR/l1_action/nt_io/nt_io_provider/gateway"
if [ -d "$GATEWAY_DIR" ]; then
    GATEWAY_FILES=$(find "$GATEWAY_DIR" -name "*.rs" | wc -l | tr -d ' ')
    GATEWAY_LINES=$(wc -l "$GATEWAY_DIR"/*.rs 2>/dev/null | tail -1 | awk '{print $1}')
    if [ "$GATEWAY_FILES" -gt 20 ]; then
        echo -e "${RED}  WARN: $GATEWAY_FILES gateway files (${GATEWAY_LINES:-0} lines) — consolidation candidate${NC}"
    else
        echo -e "${GREEN}  OK: $GATEWAY_FILES gateway files${NC}"
    fi
else
    echo -e "${GREEN}  OK: No gateway directory found${NC}"
fi

# 6. Facade Audit
echo -e "\n${YELLOW}[6/6] Facade Audit...${NC}"
FACADES=("l1_action/l1_facade" "l3_embodiment/l1_facade" "l5_cognition/kb_facade" "l5_cognition/io_facade" "l5_cognition/act_facade" "l5_cognition/l3_facade" "l6_meta/l1_facade")
for facade in "${FACADES[@]}"; do
    if [ -f "$SRC_DIR/$facade.rs" ] || [ -d "$SRC_DIR/$facade" ]; then
        echo -e "${GREEN}  OK: $facade exists${NC}"
    else
        echo -e "${YELLOW}  MISSING: $facade${NC}"
    fi
done

echo -e "\n=========================================="
echo "Patrol Complete"
echo "=========================================="
