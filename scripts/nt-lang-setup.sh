#!/usr/bin/env bash
# nt-lang development setup
set -euo pipefail

echo "=== nt-lang Setup ==="

# Install pre-commit hook
cp .githooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
echo "[ok] pre-commit hook installed"

# Verify nt-lang compiles
echo "Checking nt-lang compilation..."
cargo check -p nt-lang

# Generate all test binaries
echo "Generating test binaries from .nt specs..."
cargo run -p nt-lang -- build-all neotrix-core/test_suites/ neotrix-core/tests/

echo "=== Done ==="
echo "Test specs: $(ls neotrix-core/test_suites/*.nt 2>/dev/null | wc -l) files"
echo "Test binaries: $(ls neotrix-core/tests/*.rs 2>/dev/null | wc -l) files"
echo ""
echo "Next: 'cargo test --test <name>' or './scripts/test-shard.sh'"
