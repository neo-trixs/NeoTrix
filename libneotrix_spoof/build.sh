#!/bin/bash
set -e
cd "$(dirname "$0")"
echo "=== Building L3 libneotrix_spoof ==="
make clean 2>/dev/null
make
echo ""
echo "=== Testing (DYLD_INSERT_LIBRARIES) ==="
make test
echo ""
echo "=== Installing ==="
make install
echo ""
echo "=== Verify installed ==="
ls -la ~/.neotrix/libneotrix_spoof.dylib 2>/dev/null
echo "=== Done ==="
