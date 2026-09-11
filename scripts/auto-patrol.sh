#!/bin/bash
# NeoTrix Auto-Patrol Script v3
# Cross-layer audit + dead code + config sprawl + gateway sprawl + facade + compile check
# Usage: ./scripts/auto-patrol.sh

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC_DIR="$PROJECT_ROOT/neotrix-core/src"

PASS_COUNT=0
WARN_COUNT=0
FAIL_COUNT=0

pass()  { echo -e "${GREEN}  ✓ PASS: $1${NC}"; PASS_COUNT=$((PASS_COUNT + 1)); }
warn()  { echo -e "${YELLOW}  ⚠ WARN: $1${NC}"; WARN_COUNT=$((WARN_COUNT + 1)); }
fail()  { echo -e "${RED}  ✗ FAIL: $1${NC}"; FAIL_COUNT=$((FAIL_COUNT + 1)); }
header(){ echo -e "\n${CYAN}[$1/$2] $3${NC}"; }

echo "=========================================="
echo "NeoTrix Auto-Patrol v3  $(date '+%Y-%m-%d %H:%M')"
echo "=========================================="

# ── 1. Compilation Check (300s timeout) ──────────────────────────────
header 1 8 "Compilation Check (300s timeout)"
COMPILE_OUT=$(timeout 300 cargo check -p neotrix --lib 2>&1) || true
COMPILE_ERRORS=$(echo "$COMPILE_OUT" | grep -c "^error" || true)
if [ "$COMPILE_ERRORS" -gt 0 ]; then
    fail "Compilation errors detected ($COMPILE_ERRORS)"
    echo "$COMPILE_OUT" | grep "^error" | head -10 | sed 's/^/    /'
elif [ -z "$COMPILE_OUT" ]; then
    pass "Clean compilation (300s timeout not reached)"
else
    WARN_COMPILES=$(echo "$COMPILE_OUT" | grep -c "^warning" || true)
    if [ "$WARN_COMPILES" -gt 0 ]; then
        warn "Compiled with $WARN_COMPILES warnings (no errors)"
    else
        pass "Clean compilation"
    fi
fi

# ── 2. Cross-Layer Audit (L1→L2, L2→L3, L3→L4 downward violations) ─
header 2 8 "Cross-Layer Audit"
VIOLATIONS=0

# Helper: files in layer_dir importing higher layers
check_downward() {
    local label=$1 src_layer=$2
    local higher_layers=$3
    local hits=0
    for h in $higher_layers; do
        for file in $(rg "^use crate::l$h" "$SRC_DIR/$src_layer/" --no-heading -l 2>/dev/null | grep -v test); do
            echo -e "${RED}    VIOLATION: $label imports L$h+: $file${NC}"
            hits=$((hits + 1))
        done
    done
    VIOLATIONS=$((VIOLATIONS + hits))
}

check_downward "L1" "l1_action" "2 3 4 5 6"
check_downward "L2" "l2_perception" "3 4 5 6"
check_downward "L3" "l3_embodiment" "4 5 6"

# L4→L5/L6
for file in $(rg "^use crate::l[5-6]" "$SRC_DIR/l4_emotion/" --no-heading -l 2>/dev/null | grep -v test); do
    echo -e "${RED}    VIOLATION: L4 imports higher layer: $file${NC}"
    VIOLATIONS=$((VIOLATIONS + 1))
done

# L5→L6 is acceptable (cognition uses meta services)
L5_TO_L6=$(rg "^use crate::l[6]" "$SRC_DIR/l5_cognition/" --no-heading 2>/dev/null | grep -v test | wc -l | tr -d ' ')
if [ "$L5_TO_L6" -gt 0 ]; then
    echo -e "${YELLOW}    INFO: L5→L6 imports: $L5_TO_L6 (acceptable)${NC}"
fi

if [ "$VIOLATIONS" -eq 0 ]; then
    pass "No cross-layer violations"
else
    fail "$VIOLATIONS cross-layer violations found"
fi

# ── 3. Dead Pub Item Count ──────────────────────────────────────────
header 3 8 "Dead Pub Items"
DEAD_ITEMS=$(rg "^pub (fn|struct|enum|trait) " "$SRC_DIR" --no-heading 2>/dev/null | while IFS=: read -r file line content; do
    item=$(echo "$content" | sed 's/pub [a-z]* \([A-Za-z_]*\).*/\1/')
    if [ -n "$item" ] && [ ${#item} -gt 3 ]; then
        count=$(rg "\b$item\b" "$SRC_DIR" --no-heading 2>/dev/null | grep -v "$file" | wc -l)
        if [ "$count" -eq 0 ]; then
            echo "$item"
        fi
    fi
done 2>/dev/null)
DEAD_COUNT=$(echo "$DEAD_ITEMS" | grep -c . || true)

if [ "$DEAD_COUNT" -eq 0 ]; then
    pass "No dead pub items"
elif [ "$DEAD_COUNT" -le 5 ]; then
    warn "$DEAD_COUNT potentially dead pub items"
elif [ "$DEAD_COUNT" -le 15 ]; then
    warn "$DEAD_COUNT dead pub items (consider cleanup)"
else
    fail "$DEAD_COUNT dead pub items (cleanup recommended)"
fi

# ── 4. Config Sprawl Detection ──────────────────────────────────────
header 4 8 "Config Sprawl"
CONFIG_COUNT=$(rg "^pub struct \w*Config" "$SRC_DIR" --no-heading 2>/dev/null | wc -l | tr -d ' ')
if [ "$CONFIG_COUNT" -gt 250 ]; then
    fail "$CONFIG_COUNT Config structs (threshold: 250)"
elif [ "$CONFIG_COUNT" -gt 200 ]; then
    warn "$CONFIG_COUNT Config structs (approaching 250 threshold)"
else
    pass "$CONFIG_COUNT Config structs"
fi

# ── 5. Gateway File Count ───────────────────────────────────────────
header 5 8 "Gateway File Count"
GATEWAY_DIR="$SRC_DIR/l1_action/nt_io/nt_io_provider/gateway"
if [ -d "$GATEWAY_DIR" ]; then
    GATEWAY_FILES=$(find "$GATEWAY_DIR" -name "*.rs" | wc -l | tr -d ' ')
    GATEWAY_LINES=$(wc -l "$GATEWAY_DIR"/*.rs 2>/dev/null | tail -1 | awk '{print $1}')
    if [ "$GATEWAY_FILES" -gt 20 ]; then
        fail "$GATEWAY_FILES gateway files (${GATEWAY_LINES:-0} lines) — consolidation candidate"
    elif [ "$GATEWAY_FILES" -gt 15 ]; then
        warn "$GATEWAY_FILES gateway files (${GATEWAY_LINES:-0} lines) — watch closely"
    else
        pass "$GATEWAY_FILES gateway files (${GATEWAY_LINES:-0} lines)"
    fi
else
    warn "No gateway directory found at expected path"
fi

# ── 6. Facade Audit ─────────────────────────────────────────────────
header 6 8 "Facade Modules"
FACADES=(
    "l1_action/nt_act/mod.rs:L1 NT-ACT facade"
    "l1_action/nt_io/mod.rs:L1 NT-IO facade"
    "l1_action/nt_memory/mod.rs:L1 NT-MEMORY facade"
    "l3_embodiment/nt_physical/mod.rs:L3 NT-PHYSICAL facade"
    "l3_embodiment/nt_shield/mod.rs:L3 NT-SHIELD facade"
    "l3_embodiment/nt_feel/mod.rs:L3 NT-FEEL facade"
    "l5_cognition/nt_core/mod.rs:L5 NT-CORE facade"
    "l5_cognition/nt_mind/mod.rs:L5 NT-MIND facade"
    "l6_meta/nt_meta/mod.rs:L6 NT-META facade"
    "l6_meta/nt_repair/mod.rs:L6 NT-REPAIR facade"
    "l6_meta/nt_nexus/mod.rs:L6 NT-NEXUS facade"
)
FACADE_FOUND=0
FACADE_MISSING=0
for entry in "${FACADES[@]}"; do
    IFS=: read -r path label <<< "$entry"
    if [ -f "$SRC_DIR/$path" ]; then
        echo -e "${GREEN}    ✓ $label${NC}"
        FACADE_FOUND=$((FACADE_FOUND + 1))
    else
        echo -e "${RED}    ✗ $label MISSING${NC}"
        FACADE_MISSING=$((FACADE_MISSING + 1))
    fi
done
if [ "$FACADE_MISSING" -eq 0 ]; then
    pass "All $FACADE_FOUND facade modules present"
else
    fail "$FACADE_MISSING facade modules missing (${FACADE_FOUND} found)"
fi

# ── 7. Test Count ───────────────────────────────────────────────────
header 7 8 "Test Functions"
TEST_COUNT=$(rg "#\[cfg\(test\)\]|#\[test\]" "$SRC_DIR" --no-heading 2>/dev/null | wc -l | tr -d ' ')
if [ "$TEST_COUNT" -gt 100 ]; then
    pass "$TEST_COUNT test items"
elif [ "$TEST_COUNT" -gt 30 ]; then
    warn "$TEST_COUNT test items (low for codebase size)"
else
    fail "$TEST_COUNT test items (insufficient coverage)"
fi

# ── 8. Module Wiring Check ──────────────────────────────────────────
header 8 8 "Module Wiring"
ORPHAN_COUNT=0
for layer_dir in "$SRC_DIR"/l*; do
    if [ ! -d "$layer_dir" ]; then continue; fi
    layer_name=$(basename "$layer_dir")
    mod_file="$layer_dir/mod.rs"
    if [ -f "$mod_file" ]; then
        for sub in "$layer_dir"/*/; do
            [ -d "$sub" ] || continue
            sub_name=$(basename "$sub")
            if ! grep -q "$sub_name" "$mod_file" 2>/dev/null; then
                echo -e "${YELLOW}    ORPHAN: $layer_name/$sub_name not declared in mod.rs${NC}"
                ORPHAN_COUNT=$((ORPHAN_COUNT + 1))
            fi
        done
    fi
done
if [ "$ORPHAN_COUNT" -eq 0 ]; then
    pass "All modules wired into their parent mod.rs"
else
    warn "$ORPHAN_COUNT subdirectories not declared in parent mod.rs"
fi

# ── Summary ──────────────────────────────────────────────────────────
echo ""
echo "=========================================="
echo -e "${CYAN}          PATROL SUMMARY${NC}"
echo "=========================================="
TOTAL=$((PASS_COUNT + WARN_COUNT + FAIL_COUNT))
echo -e "  Total checks: $TOTAL"
echo -e "  ${GREEN}PASS: $PASS_COUNT${NC}"
echo -e "  ${YELLOW}WARN: $WARN_COUNT${NC}"
echo -e "  ${RED}FAIL: $FAIL_COUNT${NC}"
echo "------------------------------------------"

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo -e "${RED}  RESULT: FAIL — $FAIL_COUNT blocking issue(s)${NC}"
    EXIT_CODE=1
elif [ "$WARN_COUNT" -gt 0 ]; then
    echo -e "${YELLOW}  RESULT: WARN — $WARN_COUNT advisory(s)${NC}"
    EXIT_CODE=0
else
    echo -e "${GREEN}  RESULT: PASS — all checks clean${NC}"
    EXIT_CODE=0
fi
echo "=========================================="
exit $EXIT_CODE
