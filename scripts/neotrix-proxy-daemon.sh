#!/bin/bash
# Wrapper for launchd — finds the nt-proxy-daemon binary (debug first, then release)
set -e
NEOTRIX_DIR="$HOME/Downloads/neotrix"
for candidate in "$NEOTRIX_DIR/target/debug/nt-proxy-daemon" "$NEOTRIX_DIR/target/release/nt-proxy-daemon"; do
    if [ -x "$candidate" ]; then
        exec "$candidate"
    fi
done
echo "FATAL: nt-proxy-daemon not found (tried debug and release)" >&2
exit 1
