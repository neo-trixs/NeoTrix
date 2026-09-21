#!/bin/bash
# Layer dependency check — mirrors neotrix-core/tests/architecture_constraints.rs
# Enforces L0->L1->L2->L3->L4->L5->L6 unidirectional dependencies.
# Usage: bash scripts/check-layer-deps.sh
# Exit 0 = pass, Exit 1 = violation found.

set -e

SRC="neotrix-core/src"
FAIL=0

check_layer() {
  local layer_dir="$1"
  local layer_label="$2"
  shift 2
  # Remaining args: forbidden patterns
  for pattern in "$@"; do
    # Exclude facade bridge files and cross-layer traits (sanctioned channels).
    # SIM-27: drop full-line comments (//, ///, //!, /*) — migration notes like
    # "migrated from cli::" are not dependencies. Trailing inline comments and
    # string literals still count (conservative: may over-report, never under-).
    hits=$(rg --no-heading -n "$pattern" "$SRC/$layer_dir" \
      -g '!*facade*' -g '!*l1_facade*' -g '!traits.rs' 2>/dev/null \
      | rg -v ':[0-9]+:\s*//' | rg -v ':[0-9]+:\s*/\*' || true)
    if [ -n "$hits" ]; then
      echo "VIOLATION: $layer_label must not reference $pattern"
      echo "$hits" | head -n 20
      echo "---"
      FAIL=1
    fi
  done
}

echo "=== NeoTrix layer dependency check ==="

check_layer "l1_action"     "L1(action)"     "l2_perception" "l3_embodiment" "l4_emotion" "l5_cognition" "l6_meta"
check_layer "l2_perception" "L2(perception)" "l3_embodiment" "l4_emotion" "l5_cognition" "l6_meta"
check_layer "l3_embodiment" "L3(embodiment)" "l4_emotion" "l5_cognition" "l6_meta"
check_layer "l4_emotion"    "L4(emotion)"    "l5_cognition" "l6_meta"
check_layer "l5_cognition"  "L5(cognition)"  "l6_meta"

if [ "$FAIL" -eq 0 ]; then
  echo "PASS: no layer violations detected."
else
  echo "FAIL: layer violations detected (see above)."
  exit 1
fi
