#!/bin/bash
# Wait for other cargo/rustc processes to release the build lock.
# NEVER kills anything (contrast scripts/cargo-clean-locks.sh which does).
# Use before launching full-crate builds on memory-starved boxes: two parallel
# rustc on neotrix-core = OOM kill (Killed: 9, SIM-23) + poisoned incremental
# cache (E0689 ghost, SIM-17).
# Usage: bash scripts/wait-for-cargo.sh [max_seconds]  (default 1200)
# Exit 0 = lock free, 1 = timeout (caller decides: abort, not force).
# Note: bash-3.2-safe style. `bash -n` before commit.

MAX_WAIT="${1:-1200}"
ELAPSED=0
STEP=15

while true; do
  # Match build-driving cargo/rustc only; exclude this helper itself.
  HOLDERS=$(pgrep -fl "cargo (check|test|build|bench)|rustc --crate-name" 2>/dev/null || true)
  if [ -z "$HOLDERS" ]; then
    echo "cargo lock free after ${ELAPSED}s."
    exit 0
  fi
  if [ "$ELAPSED" -ge "$MAX_WAIT" ]; then
    echo "TIMEOUT after ${ELAPSED}s; holders still present:"
    echo "$HOLDERS" | head -n 3
    exit 1
  fi
  sleep "$STEP"
  ELAPSED=$((ELAPSED + STEP))
done
