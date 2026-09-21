#!/bin/bash
# Build-surface inventory — R-P248 advisory implementation.
# Rust's highest-risk supply-chain surface is compile-time code execution:
# build.rs scripts + proc-macro crates (Safeguard 2026 program, gh-guard).
# This script inventories the WORKSPACE-OWNED surface (fast, no network).
# Transitive registry proc-macros are covered by the cargo-vet bake plan (P3).
# Baseline (2026-09-21): 2 build.rs (both benign), 0 workspace proc-macro crates.
# Usage: bash scripts/check-build-surface.sh [--strict]
# Note: bash-3.2-safe style. `bash -n` before commit.

set -e

FAIL=0
if [ "${1:-}" = "--strict" ]; then
  STRICT=1
else
  STRICT=0
fi

echo "=== NeoTrix build-surface inventory (R-P248) ==="
echo "--- workspace build.rs files (excluding target/) ---"
BUILDRS=$(rg --files -g 'build.rs' . 2>/dev/null | rg -v "^./target/" || true)
if [ -z "$BUILDRS" ]; then
  echo "(none)"
else
  echo "$BUILDRS"
fi

echo "--- workspace proc-macro crates ---"
PROCMACRO=$(rg -n "proc-macro *= *true" --glob 'Cargo.toml' . 2>/dev/null | rg -v "^./target/" || true)
if [ -z "$PROCMACRO" ]; then
  echo "(none)"
else
  echo "$PROCMACRO"
fi

echo "--- workspace [build-dependencies] sections ---"
BUILDDEPS=$(rg -n "\[build-dependencies\]" --glob 'Cargo.toml' . 2>/dev/null | rg -v "^./target/" || true)
if [ -z "$BUILDDEPS" ]; then
  echo "(none)"
else
  echo "$BUILDDEPS"
fi

echo "--- network/download behavior in build.rs (must be empty) ---"
SUSPICIOUS=""
for f in $BUILDRS; do
  hits=$(rg -n "curl|wget|http://|https://|Command::new|fs::write|OUT_DIR.*write|include_bytes" "$f" 2>/dev/null || true)
  if [ -n "$hits" ]; then
    echo "REVIEW: $f"
    echo "$hits" | head -n 10
    SUSPICIOUS="$SUSPICIOUS $f"
  fi
done
if [ -z "$SUSPICIOUS" ]; then
  echo "(clean)"
fi

if [ "$STRICT" -eq 1 ] && [ -n "$SUSPICIOUS" ]; then
  echo "FAIL(strict): build.rs files need human review (see above)."
  exit 1
fi

echo "DONE(advisory). Baseline: 2 benign build.rs, 0 proc-macro crates."
