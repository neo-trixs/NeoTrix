#!/usr/bin/env bash
# test-shard.sh — Run a subset of neotrix tests from the precompiled binary.
#
# Usage:
#   ./scripts/test-shard.sh [module1 module2 ...]
#
# Examples:
#   ./scripts/test-shard.sh                          # list available test modules
#   ./scripts/test-shard.sh reasoning_distiller       # run one module's tests
#   ./scripts/test-shard.sh foundation rule_engine    # run multiple modules
#
# Before first use, compile the test binary once:
#   cargo test -p neotrix --lib --no-run
#
# The script auto-discovers the compiled test binary hash.

set -euo pipefail

NEOTRIX_DIR="$(cd "$(dirname "$0")/.." && pwd)"

find_test_binary() {
  ls -t "$NEOTRIX_DIR"/target/debug/deps/neotrix-* 2>/dev/null \
    | grep -v '\.d$' \
    | head -1 || true
}

BIN="$(find_test_binary)"

if [ -z "$BIN" ]; then
  echo ":: No compiled test binary found. Building first..."
  cargo test -p neotrix --lib --no-run
  BIN="$(find_test_binary)"
  if [ -z "$BIN" ]; then
    echo "ERROR: Could not find compiled test binary after build." >&2
    exit 1
  fi
fi

echo ":: Test binary: $(basename "$BIN")"
echo ""

if [ $# -eq 0 ]; then
  # List all available test modules
  echo ":: Available test modules:"
  "$BIN" --list 2>/dev/null | sed 's/: test//' | sort
  exit 0
fi

# Build a test filter regex from the module names
# Each argument is a module name prefix that matches test functions in that module
PATTERNS=()
for mod in "$@"; do
  PATTERNS+=("$mod")
done

if [ ${#PATTERNS[@]} -eq 1 ]; then
  FILTER="${PATTERNS[0]}"
  echo ":: Running tests matching: $FILTER"
  exec "$BIN" "$FILTER"
else
  # Run each module group separately
  for pat in "${PATTERNS[@]}"; do
    echo ":: Running tests matching: $pat"
    "$BIN" "$pat"
    echo ""
  done
fi
