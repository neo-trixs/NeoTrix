#!/bin/bash
# Fuzz readiness probe — R-P253 advisory implementation.
# cargo-fuzz needs nightly ONLY for building fuzz targets; production stays stable.
# Bake plan: init fuzz/ -> thin harness + invariant oracle -> seed corpus ->
# corpus cache in CI (depot pattern) -> unsafe-first targets.
# Baseline (2026-09-21): no fuzz/ dir, readiness 0/4. Advisory, always exit 0.
# Note: bash-3.2-safe style. `bash -n` before commit.

echo "=== NeoTrix fuzz readiness (R-P253) ==="

if rustup toolchain list 2>/dev/null | rg -q "^nightly"; then
  echo "[ok] nightly toolchain present"
else
  echo "[missing] nightly toolchain (need: rustup toolchain install nightly)"
fi

if command -v cargo-fuzz >/dev/null 2>&1 || cargo fuzz --version >/dev/null 2>&1; then
  echo "[ok] cargo-fuzz installed"
else
  echo "[missing] cargo-fuzz (need: cargo install cargo-fuzz)"
fi

if [ -d fuzz ]; then
  echo "[ok] fuzz/ workspace exists"
  TARGETS=$(rg --files -g '*.rs' fuzz/fuzz_targets/ 2>/dev/null | wc -l | tr -d ' ')
  echo "       targets: $TARGETS"
else
  echo "[missing] fuzz/ workspace (need: cargo +nightly fuzz init)"
fi

if [ -d fuzz/corpus ]; then
  echo "[ok] seed corpus present"
else
  echo "[missing] seed corpus (need: fuzz/corpus/<target>/ with sample inputs)"
fi

echo "Next per bake plan: harness for parser targets first (nt_io_web/api, tiles, keyframe_motion)."
echo "DONE(advisory)."
