#!/bin/bash
# Test-failure baseline gate — "no NEW test failures", with the standing failures
# recorded as a ledger instead of a permanently red CI.
#
# Usage:
#   bash scripts/check-test-baseline.sh              # run + report, never fail
#   bash scripts/check-test-baseline.sh --strict     # fail on NEW failures
#   bash scripts/check-test-baseline.sh --update-baseline
#   bash scripts/check-test-baseline.sh --list       # just show the ledger
#
# Why this exists (2026-09-28):
#   HEAD did not compile for a long time, so `cargo test --lib -p neotrix` had
#   **no verifiable baseline at all**. Once the build was fixed, the suite ran:
#   12093 passed / 56 failed. Those 56 are genuine behavioural assertion
#   failures spread across every layer (l6_meta 20, l4_emotion 10, l1_action 8,
#   l3_embodiment 7, l5_cognition 5, l0_substrate 3, ...) — pre-existing debt
#   that was invisible precisely because nothing could be built.
#
#   Each one needs a product decision (fix the code, or fix an assertion that
#   encoded behaviour never implemented). This gate does NOT decide that. It
#   makes the debt *visible and countable* while blocking any NEW failure, so
#   the signal is real again.
#
#   Same pattern as scripts/check-truth-surface.sh and check-layer-deps.sh.
#
# Exit: 0 pass, 1 new failures (--strict), 2 bad usage / build failure.

set -uo pipefail

BASELINE="scripts/test-failures-baseline.txt"
STRICT=0
UPDATE=0
LIST=0
PKG="neotrix"

for arg in "$@"; do
  case "$arg" in
    --strict)          STRICT=1 ;;
    --update-baseline) UPDATE=1 ;;
    --list)            LIST=1 ;;
    -h|--help)         sed -n '2,28p' "$0"; exit 0 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

if [ "$LIST" -eq 1 ]; then
  if [ -f "$BASELINE" ]; then
    echo "=== recorded test failures ($(grep -c . "$BASELINE")) ==="
    sed 's/^/  /' "$BASELINE"
  else
    echo "no baseline yet: $BASELINE"
  fi
  exit 0
fi

command -v cargo >/dev/null 2>&1 || { echo "cargo not found" >&2; exit 2; }

if [ "$UPDATE" -eq 1 ]; then
  RAW=$(mktemp)
  echo "=== running suite to record the ledger (this takes a few minutes) ==="
  cargo test --lib -p "$PKG" -- --test-threads=2 >"$RAW" 2>&1
  # Distinguish a BUILD error from a test failure. `error: test failed, to rerun
  # pass ...` is the suite's own failure summary, not a compile error — matching
  # a bare `^error` would abort on exactly the thing this gate exists to record.
  if grep -qE "^error\[E[0-9]+\]|^error: could not compile" "$RAW"; then
    echo "ABORT: the suite does not build; refusing to record a baseline of a"
    echo "  broken build (a ledger built now would encode the breakage)."
    grep -E "^error" -A3 "$RAW" | head -12 | sed 's/^/  /'
    rm -f "$RAW"; exit 2
  fi
  grep -E "^test .* FAILED" "$RAW" | sed 's/^test //; s/ \.\.\. FAILED$//' \
    | sort -u > "$BASELINE"
  echo "baseline updated: $(grep -c . "$BASELINE" || echo 0) failing test(s) -> $BASELINE"
  echo "NOTE: ratchet only downward. Each entry is a real assertion failure that"
  echo "      needs a product decision; this gate does not decide it for you."
  rm -f "$RAW"; exit 0
fi

RAW=$(mktemp)
CUR=$(mktemp)
trap 'rm -f "$RAW" "$CUR"' EXIT

echo "=== NeoTrix test-failure baseline gate ==="
echo "running: cargo test --lib -p $PKG  (a few minutes)"
cargo test --lib -p "$PKG" -- --test-threads=2 >"$RAW" 2>&1
RC=$?

SUMMARY=$(grep -E "^test result:" "$RAW" | tail -1)
if [ -z "$SUMMARY" ]; then
  echo "FAIL: the suite produced no result line (build error?)."
  grep -E "^error" -A3 "$RAW" | head -12 | sed 's/^/  /'
  exit 2
fi
echo "  $SUMMARY"

if [ "$RC" -eq 0 ]; then
  echo
  echo "PASS: the whole suite is green."
  if [ -s "$BASELINE" ]; then
    echo "  The ledger ($BASELINE) can be deleted now."
  fi
  exit 0
fi

grep -E "^test .* FAILED" "$RAW" | sed 's/^test //; s/ \.\.\. FAILED$//' \
  | sort -u > "$CUR"
TOTAL=$(grep -c . "$CUR" || true); TOTAL=${TOTAL:-0}

if [ ! -f "$BASELINE" ]; then
  : > "$BASELINE"
  echo
  echo "no baseline yet. $TOTAL failing test(s) recorded in the run above."
  echo "Run --update-baseline to acknowledge them as standing debt, or fix them."
  printf '%s\n' "$(sed 's/^/  /' "$CUR")"
  [ "$STRICT" -eq 1 ] && exit 1
  exit 0
fi

NEW=$(comm -23 "$CUR" "$BASELINE" | grep . || true)
GONE=$(comm -13 "$CUR" "$BASELINE" | grep . || true)
N_NEW=$(printf '%s' "$NEW" | grep -c . || true); N_NEW=${N_NEW:-0}
N_GONE=$(printf '%s' "$GONE" | grep -c . || true); N_GONE=${N_GONE:-0}
BASE_N=$(grep -c . "$BASELINE" || true); BASE_N=${BASE_N:-0}

echo "  failing now: $TOTAL   in ledger: $BASE_N"
[ "$N_GONE" -gt 0 ] && echo "  FIXED since ledger: $N_GONE  (run --update-baseline to ratchet down)"

if [ "$N_NEW" -gt 0 ]; then
  echo
  echo "NEW failing tests (not in the ledger): $N_NEW"
  printf '%s\n' "$NEW" | sed 's/^/  /'
  if [ "$STRICT" -eq 1 ]; then
    echo
    echo "FAIL(strict): $N_NEW new test failure(s)."
    exit 1
  fi
  echo "(advisory mode; --strict would fail here)"
  exit 0
fi

echo
echo "PASS: $N_NEW new failure(s); $TOTAL recorded in the ledger."
echo "  This gate protects against NEW test failures. It does NOT mean the suite"
echo "  is healthy — $TOTAL assertions are still red and tracked in TODO.md."
exit 0
