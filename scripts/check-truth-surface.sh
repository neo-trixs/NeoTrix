#!/bin/bash
# Truth-surface gate — keeps "code that exists" from diverging from
# "code the compiler and test runner can actually see".
#
# Motivation (audit 2026-09-27): `cargo test` was fully green while
# 311 tests in 3 files under neotrix-core/src/l1_action/nt_act/nt_act_trade/tests/
# were never compiled, and 54 zero-byte .rs files were re-exported as
# if they were real public API. Green signals meant nothing because the
# offending code was invisible to the toolchain.
#
# Three classes of truth-drift, all structural (not style, so not clippy's job):
#   1) EMPTY     0-byte .rs file declared `pub mod` — compiles as an empty
#                namespace, so every downstream `use` succeeds and yields nothing.
#   2) UNDECLARED a sibling .rs in a tests/ dir with no `mod` declaration in
#                tests/mod.rs — written, reviewed, never executed.
#   3) TRACKED   build artifacts / runtime DB under version control.
#
# Baseline is a LIST (scripts/truth-surface-baseline.txt), not a count, so that
# deleting one offender and adding another cannot hide the new one. Ratchet down
# with --update-baseline once entries are genuinely resolved.
#
# Usage:
#   bash scripts/check-truth-surface.sh              # advisory, always exit 0
#   bash scripts/check-truth-surface.sh --strict     # exit 1 if any NEW offender
#   bash scripts/check-truth-surface.sh --update-baseline
#
# Note: bash-3.2-safe style (macOS system bash). `bash -n` before commit.

set -uo pipefail

BASELINE="scripts/truth-surface-baseline.txt"
STRICT=0
UPDATE=0
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    --update-baseline) UPDATE=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

# Roots to scan. Excludes target/, .worktrees/ (5 stale full copies), thirdparty/.
SCAN_ROOTS="neotrix-core/src crates apps src-tauri/src"
TRACK_GLOBS='\.(rlib|so|dylib|a|o|wasm)$|^\.neotrix/.*\.(db|sqlite|sqlite3)$'

CUR=$(mktemp)
CUR_C=$(mktemp)
BASE_C=$(mktemp)
NEW=$(mktemp)
GONE=$(mktemp)
trap 'rm -f "$CUR" "$CUR_C" "$BASE_C" "$NEW" "$GONE"' EXIT

# ---------- 1) EMPTY: zero-byte .rs files ----------
for root in $SCAN_ROOTS; do
  [ -d "$root" ] || continue
  find "$root" -name '*.rs' -type f -size 0 2>/dev/null | sed 's|^\./||' | sort
done | sed 's/^/EMPTY /' >> "$CUR"

# ---------- 2) UNDECLARED: tests/*.rs with no `mod` in tests/mod.rs ----------
# Scoped to directories literally named `tests` to avoid false positives on
# regular module trees. Handles `mod x;`, inline `mod x {`, and #[path=..].
for mod_rs in $(find $SCAN_ROOTS -type f -name 'mod.rs' -path '*/tests/*' 2>/dev/null); do
  dir=$(dirname "$mod_rs")
  for sib in "$dir"/*.rs; do
    [ -f "$sib" ] || continue
    base=$(basename "$sib")
    [ "$base" = "mod.rs" ] && continue
    stem="${base%.rs}"
    if grep -qE "(^|[^A-Za-z0-9_])mod[[:space:]]+$stem[[:space:]]*[;{]" "$mod_rs" 2>/dev/null; then
      continue
    fi
    # #[path = "..."] attributed module pointing at this file
    if grep -q "#\[path" "$mod_rs" 2>/dev/null && \
       grep -q "\"$base\"" "$mod_rs" 2>/dev/null; then
      continue
    fi
    echo "UNDECLARED $sib"
  done
done | sort -u >> "$CUR"

# ---------- 3) TRACKED: build artifacts / runtime DB in git ----------
git ls-files 2>/dev/null | grep -E "$TRACK_GLOBS" | sed 's/^/TRACKED /' >> "$CUR"

sort -u "$CUR" -o "$CUR"

# ---------- diff against baseline ----------
# Baseline lines starting with '#' are comments (rationale), stripped before diff
# so the file can self-document WHY a known offender is tolerated.
CUR_C=$(mktemp)
BASE_C=$(mktemp)
grep -v '^#' "$CUR" 2>/dev/null | sort -u > "$CUR_C" || : > "$CUR_C"
if [ -f "$BASELINE" ]; then
  grep -v '^#' "$BASELINE" | sort -u > "$BASE_C" || : > "$BASE_C"
  comm -23 "$CUR_C" "$BASE_C" > "$NEW"    # in tree, not in baseline => NEW
  comm -13 "$CUR_C" "$BASE_C" > "$GONE"   # in baseline, gone from tree => RESOLVED
else
  cp "$CUR_C" "$NEW"
  : > "$GONE"
fi

N_EMPTY=$(grep -c '^EMPTY ' "$NEW" || true)
N_UNDECL=$(grep -c '^UNDECLARED ' "$NEW" || true)
N_TRACK=$(grep -c '^TRACKED ' "$NEW" || true)
N_GONE=$(grep -c . "$GONE" || true)
N_BASE=$(grep -vc '^#' "$BASELINE" 2>/dev/null || true)

echo "=== NeoTrix truth-surface gate ==="
echo "baseline entries: $N_BASE   resolved since baseline: $N_GONE"
echo "NEW offenders  -> EMPTY:$N_EMPTY  UNDECLARED:$N_UNDECL  TRACKED:$N_TRACK"

if [ "$N_GONE" -gt 0 ]; then
  echo "--- resolved (drop from baseline via --update-baseline) ---"
  cat "$GONE"
fi

if [ "$N_EMPTY" -gt 0 ] || [ "$N_UNDECL" -gt 0 ] || [ "$N_TRACK" -gt 0 ]; then
  echo "--- NEW offenders (regression, not in baseline) ---"
  cat "$NEW"
fi

if [ "$UPDATE" -eq 1 ]; then
  # preserve any leading comment block, replace the entry list
  if [ -f "$BASELINE" ]; then
    grep '^#' "$BASELINE" > "$BASELINE.tmp" || : > "$BASELINE.tmp"
  else
    : > "$BASELINE.tmp"
  fi
  cat "$BASELINE.tmp" "$CUR_C" > "$BASELINE"
  rm -f "$BASELINE.tmp"
  echo "baseline updated: $BASELINE now has $(grep -vc '^#' "$BASELINE") entries"
  exit 0
fi

if [ "$STRICT" -eq 1 ] && [ -s "$NEW" ]; then
  echo "FAIL(strict): $((N_EMPTY + N_UNDECL + N_TRACK)) new truth-drift offenders."
  echo "  EMPTY     -> delete the file, or implement it (a 0-byte 'pub mod' is a lie)"
  echo "  UNDECLARED-> add 'mod <name>;' to the sibling tests/mod.rs, or delete the file"
  echo "  TRACKED   -> git rm --cached <path> (and fix the matching .gitignore rule)"
  exit 1
fi

echo "DONE(advisory)."
