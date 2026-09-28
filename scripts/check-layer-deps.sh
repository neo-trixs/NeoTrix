#!/bin/bash
# Layer dependency check — enforces L0->L1->L2->L3->L4->L5->L6 unidirectional deps.
#
# 2026-09-28 修正：本文件原第 2 行写 "mirrors neotrix-core/tests/architecture_constraints.rs"，
# 但那个 Rust 文件**从未存在**（`git log --all` 查无，`neotrix-core/tests/` 实有
# `evolution_integration.rs` + `g3_metrics.rs`）。即注释在指一个不存在的"孪生实现"，
# 会让下一个 agent 以为 Rust 侧已有等价门而不敢动它。已改为如实描述。
#
# 唯一层归属真源：`.neotrix/layer-map.json`（见 RUST-STANDARDS.md R-P199 口径）。
#
# Usage:
#   bash scripts/check-layer-deps.sh              # advisory: report, never fail
#   bash scripts/check-layer-deps.sh --strict     # fail on NEW violations only
#   bash scripts/check-layer-deps.sh --update-baseline
#
# Why a baseline (2026-09-28):
#   The gate had no ratchet, so it failed unconditionally: 148 pre-existing
#   L1->L2/L3/L4/L5/L6 references across 11 categories. A gate that is always
#   red is a gate nobody reads — it had been sitting red in CI while the real
#   signal (a *new* violation) was indistinguishable from the standing debt.
#   So the standing debt is recorded as a LIST and the gate blocks on NEW debt,
#   ratcheting down as violations are actually fixed.
#
#   This mirrors scripts/check-truth-surface.sh by design. It does NOT claim the
#   debt is resolved — see TODO.md "分层依赖" for the outstanding work.
#
# Baseline entries are `pattern<TAB>file` (deliberately NO line numbers: lines
# shift on any edit, which would churn the ledger and report phantom new debt).
# Trade-off: swapping one violation for another in the same file+pattern is
# invisible to the ledger. That is the lesser evil versus constant false alarms.
#
# Exit: 0 pass, 1 new violations (--strict), 2 bad usage.

set -uo pipefail

SRC="neotrix-core/src"
BASELINE="scripts/layer-deps-baseline.txt"
STRICT=0
UPDATE=0

case "${1:-}" in
  "")                STRICT=0 ;;
  --strict)          STRICT=1 ;;
  --update-baseline) UPDATE=1 ;;
  -h|--help)         sed -n '2,30p' "$0"; exit 0 ;;
  *) echo "unknown arg: $1 (use --strict | --update-baseline)" >&2; exit 2 ;;
esac

command -v rg >/dev/null 2>&1 || { echo "rg (ripgrep) not found in PATH" >&2; exit 2; }
[ -d "$SRC" ] || { echo "source dir not found: $SRC (run from repo root)" >&2; exit 2; }

CUR=$(mktemp)
trap 'rm -f "$CUR"' EXIT

echo "=== NeoTrix layer dependency check ==="

# check_layer <layer_dir> <layer_label> <forbidden-pattern>...
# Appends every hit to $CUR as `pattern<TAB>file`; prints at most 20 for humans.
check_layer() {
  local layer_dir="$1"
  local layer_label="$2"
  shift 2
  local pattern hits n
  for pattern in "$@"; do
    # Exclude facade bridge files and cross-layer traits (sanctioned channels).
    # SIM-27: drop full-line comments (//, ///, //!, /*) — migration notes like
    # "migrated from cli::" are not dependencies. Trailing inline comments and
    # string literals still count (conservative: may over-report, never under-).
    hits=$(rg --no-heading -n "$pattern" "$SRC/$layer_dir" \
      -g '!*facade*' -g '!*l1_facade*' -g '!traits.rs' 2>/dev/null \
      | rg -v ':[0-9]+:\s*//' | rg -v ':[0-9]+:\s*/\*' || true)
    [ -n "$hits" ] || continue
    n=$(printf '%s\n' "$hits" | wc -l | tr -d ' ')
    echo "VIOLATION: $layer_label must not reference $pattern  (${n} site(s))"
    printf '%s\n' "$hits" | head -n 20 | sed 's/^/  /'
    [ "$n" -gt 20 ] && echo "  ... and $((n - 20)) more (full set in the ledger)"
    echo "---"
    # Full set (NOT the truncated print) feeds the ledger.
    printf '%s\n' "$hits" | cut -d: -f1 | sort -u | while read -r f; do
      printf '%s\t%s\n' "$pattern" "$f"
    done >> "$CUR"
  done
}

check_layer "l1_action"     "L1(action)"     "l2_perception" "l3_embodiment" "l4_emotion" "l5_cognition" "l6_meta"
check_layer "l2_perception" "L2(perception)" "l3_embodiment" "l4_emotion" "l5_cognition" "l6_meta"
check_layer "l3_embodiment" "L3(embodiment)" "l4_emotion" "l5_cognition" "l6_meta"
check_layer "l4_emotion"    "L4(emotion)"    "l5_cognition" "l6_meta"
check_layer "l5_cognition"  "L5(cognition)"  "l6_meta"

# --- neotrix/ second tree (2026-09-28) -----------------------------------------
# `neotrix-core/src/neotrix/` is 129 files / 43,834 lines that do NOT live under
# any l*_ dir, so the check_layer calls above never see it. Its layer ownership is
# declared in .neotrix/layer-map.json; we enforce the same rule against that
# declared layer. Rationale: docs/architecture/DIR-REMEDY-2026-09-28.md
#   - decoupling layer ownership from directory location is what lets us enforce
#     architecture here WITHOUT moving 129 files (a large regression for zero
#     architectural gain).
#   - Everything here is ratcheted: the tree is live (nt_crystal_core has 6 L1
#     consumers), so it enters the same baseline ledger, not a hard failure.
LAYER_MAP=".neotrix/layer-map.json"
if [ -f "$LAYER_MAP" ]; then
  mapfile_layers=$(python3 - "$LAYER_MAP" <<'PY' 2>/dev/null
import json, sys
try:
    d = json.load(open(sys.argv[1]))
except Exception:
    sys.exit(0)
ORDER = ["l0_substrate", "l1_action", "l2_perception",
         "l3_embodiment", "l4_emotion", "l5_cognition", "l6_meta"]
for path, meta in d.get("trees", {}).items():
    if meta.get("role") != "primary":
        continue
    layer = meta.get("layer")
    if layer not in ORDER:
        continue
    # It may not reference any STRICTLY HIGHER layer.
    for higher in ORDER[ORDER.index(layer) + 1:]:
        print(f"{path}\t{higher}")
PY
)
  if [ -n "$mapfile_layers" ]; then
    echo
    echo "--- neotrix/ second tree (layer declared in $LAYER_MAP) ---"
    while IFS=$'\t' read -r tree higher; do
      [ -n "$tree" ] && [ -n "$higher" ] || continue
      [ -d "$SRC/$tree" ] || { echo "WARN: declared tree missing: $SRC/$tree"; continue; }
      # Recover the declared layer from the map for the human-readable message.
      decl=$(python3 -c "import json,sys;d=json.load(open(sys.argv[1]));print(d['trees'][sys.argv[2]].get('layer','?'))" \
             "$LAYER_MAP" "$tree" 2>/dev/null || echo "?")
      # Match either the bare layer name or its absolute crate path.
      hits=$(rg --no-heading -n "(crate::)?$higher" "$SRC/$tree" \
        -g '!*facade*' -g '!traits.rs' 2>/dev/null \
        | rg -v ':[0-9]+:\s*//' | rg -v ':[0-9]+:\s*/\*' || true)
      [ -n "$hits" ] || continue
      n=$(printf '%s\n' "$hits" | wc -l | tr -d ' ')
      echo "VIOLATION: $tree (declared $decl) must not reference $higher  (${n} site(s))"
      printf '%s\n' "$hits" | head -n 10 | sed 's/^/  /'
      [ "$n" -gt 10 ] && echo "  ... and $((n - 10)) more"
      echo "---"
      printf '%s\n' "$hits" | cut -d: -f1 | sort -u | while read -r f; do
        printf '%s\t%s\n' "$higher" "$f"
      done >> "$CUR"
    done <<< "$mapfile_layers"
  fi
else
  echo
  echo "NOTE: $LAYER_MAP not found — the neotrix/ second tree is UNCHECKED."
  echo "      See docs/architecture/DIR-REMEDY-2026-09-28.md"
fi

sort -u "$CUR" -o "$CUR"
TOTAL=$(wc -l < "$CUR" | tr -d ' ')

if [ ! -f "$BASELINE" ]; then
  mkdir -p "$(dirname "$BASELINE")"
  : > "$BASELINE"
fi
BASE_N=$(grep -c . "$BASELINE" 2>/dev/null || echo 0)
BASE_N=${BASE_N:-0}

if [ "$UPDATE" -eq 1 ]; then
  cp "$CUR" "$BASELINE"
  echo "baseline updated: $TOTAL entr(ies) -> $BASELINE"
  echo "NOTE: ratchet only downward. Re-run and confirm the count moved in the"
  echo "      intended direction; a baseline that grew hides new debt."
  exit 0
fi

echo
echo "violation sites: $TOTAL   baseline entries: $BASE_N"

if [ "$BASE_N" -eq 0 ]; then
  if [ "$TOTAL" -eq 0 ]; then
    echo "PASS: no layer violations detected."
    exit 0
  fi
  echo "NOTE: baseline is empty. Run --update-baseline to record the standing"
  echo "      debt, or fix the $TOTAL site(s) outright."
  [ "$STRICT" -eq 1 ] && exit 1
  exit 0
fi

NEW=$(comm -23 "$CUR" "$BASELINE" 2>/dev/null | grep . || true)
GONE=$(comm -13 "$CUR" "$BASELINE" 2>/dev/null | grep . || true)
N_NEW=$(printf '%s' "$NEW" | grep -c . || true); N_NEW=${N_NEW:-0}
N_GONE=$(printf '%s' "$GONE" | grep -c . || true); N_GONE=${N_GONE:-0}

if [ "$N_GONE" -gt 0 ]; then
  echo "resolved since baseline: $N_GONE  (re-run --update-baseline to ratchet down)"
fi

if [ "$N_NEW" -gt 0 ]; then
  echo
  echo "NEW layer violations (not in baseline): $N_NEW"
  printf '%s\n' "$NEW" | sed 's/^/  /'
  if [ "$STRICT" -eq 1 ]; then
    echo
    echo "FAIL(strict): $N_NEW new layer violation(s). Fix them, or extend the"
    echo "  baseline only if the new dependency is a sanctioned channel."
    exit 1
  fi
  echo "(advisory mode; --strict would fail here)"
  exit 0
fi

if [ "$TOTAL" -eq 0 ]; then
  echo "PASS: no layer violations at all. Delete the baseline when it empties."
  exit 0
fi
echo "PASS: $N_NEW new violation(s); $TOTAL known/recorded."
echo "  The standing debt is real and unresolved — see TODO.md. This gate now"
echo "  protects against regression; it does not certify the architecture."
exit 0
