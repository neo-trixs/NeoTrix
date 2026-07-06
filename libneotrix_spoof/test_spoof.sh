#!/bin/bash
DYLD_INSERT_LIBRARIES=./libneotrix_spoof.dylib
export DYLD_INSERT_LIBRARIES

echo "=== Testing sysctl hw.model ==="
sysctl -n hw.model

echo "=== Testing hostname ==="
hostname

echo "=== Testing uname ==="
uname -a

echo "=== Testing sysctl kern.osrelease ==="
sysctl -n kern.osrelease

echo "=== Testing getlogin ==="
python3 -c "import os; print('login:', os.getlogin())"

echo ""
echo "=== All tests completed ==="
