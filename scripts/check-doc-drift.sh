#!/bin/bash
# Doc-drift check — R-P232 advisory implementation.
# Verifies every nt_*.rs module file (excluding mod.rs/lib.rs/main.rs)
# carries inner module docs (//! in the first 3 lines).
# Baseline (2026-09-21): 111 files missing. Goal: ratchet to 0, then --strict.
# Usage: bash scripts/check-doc-drift.sh [--strict]
#   default : advisory, always exit 0, prints count + top offenders.
#   --strict: exit 1 if any offender remains (wire into CI only after cleanup).
# Note: written in bash-3.2-safe style (macOS system bash).

set -e

SRC="neotrix-core/src"
STRICT=0
if [ "${1:-}" = "--strict" ]; then
  STRICT=1
fi

LIST=$(mktemp)
OUT=$(mktemp)
trap 'rm -f "$LIST" "$OUT"' EXIT

rg --files -g 'nt_*.rs' -g '!mod.rs' -g '!lib.rs' -g '!main.rs' "$SRC" > "$LIST"

while IFS= read -r f; do
  if head -n 3 "$f" | rg -q '^//!'; then
    :
  else
    echo "$f" >> "$OUT"
  fi
done < "$LIST"

COUNT=$(rg -c . "$OUT" || true)
echo "=== NeoTrix doc-drift check (R-P232) ==="
echo "Files missing //! module docs: $COUNT (baseline 111 on 2026-09-21)"

if [ "$COUNT" -gt 0 ]; then
  echo "--- top offenders (first 20) ---"
  head -n 20 "$OUT"
fi

if [ "$STRICT" -eq 1 ] && [ "$COUNT" -gt 0 ]; then
  echo "FAIL(strict): $COUNT files still undocumented."
  exit 1
fi

echo "DONE(advisory)."
