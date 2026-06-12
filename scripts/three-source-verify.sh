#!/usr/bin/env bash
# Three-source verification for Ne bootstrap.
#
# Verifies that all three implementations of the 8 VSA primitives
# produce byte-identical outputs on reference inputs.
#
# Source 1: C reference interpreter (< 300 lines, NEVER changes)
# Source 2: Rust Bridge (bridge.rs → generated Ne compiler)
# Source 3: Ne self-compiler (compiler compiled by itself)
#
# Usage: ./scripts/three-source-verify.sh [source]
#   source: c | rust | ne | all (default: all)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
C_REF="$SCRIPT_DIR/c_reference/vsa"
TMPDIR="${TMPDIR:-/tmp}/neotrix-verify-$$"
mkdir -p "$TMPDIR"
trap 'rm -rf "$TMPDIR"' EXIT

# Reference test vectors (64 bytes each, deterministic)
A_HEX="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f"
B_HEX="000306090c0f1215181b1e2124272a2d303336393c3f4245484b4e5154575a5d606366696c6f7275787b7e8184878a8d909396999c9fa2a5a8abaeb1b4b7babd"

PASS=0
FAIL=0

check() {
    local name="$1" got="$2" expected="$3"
    if [ "$got" = "$expected" ]; then
        echo "  ✅ $name"
        PASS=$((PASS + 1))
    else
        echo "  ❌ $name"
        echo "     got:      $got"
        echo "     expected: $expected"
        FAIL=$((FAIL + 1))
    fi
}

verify_c() {
    echo "── Source 1: C Reference Interpreter ──"
    
    # 1. bind
    RES=$( "$C_REF" bind "$A_HEX" "$B_HEX" )
    check "bind" "$RES" "0002040a080a14121012142a282a24222022242a282a54525052544a484a44424042444a484a5452505254aaa8aaa4a2a0a2a4aaa8aa94929092948a888a8482"

    # 2. negate
    RES=$( "$C_REF" negate "$A_HEX" )
    check "negate" "$RES" "fffefdfcfbfaf9f8f7f6f5f4f3f2f1f0efeeedecebeae9e8e7e6e5e4e3e2e1e0dfdedddcdbdad9d8d7d6d5d4d3d2d1d0cfcecdcccbcac9c8c7c6c5c4c3c2c1c0"

    # 3. permute k=7
    RES=$( "$C_REF" permute "$A_HEX" 7 )
    check "permute" "$RES" "0708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f00010203040506"

    # 4. similarity
    RES=$( "$C_REF" similarity "$A_HEX" "$B_HEX" )
    check "similarity" "$RES" "0.6953125000"

    # 5. cosine
    RES=$( "$C_REF" cosine "$A_HEX" "$B_HEX" )
    check "cosine" "$RES" "1.0000000000"

    # 6. hamming_distance
    RES=$( "$C_REF" hamming_distance "$A_HEX" "$B_HEX" )
    check "hamming_distance" "$RES" "156"

    # 7. random_vector seed=42
    RES=$( "$C_REF" random_vector 42 )
    check "random_vector" "$RES" "aabf7a688223d332f0259fdc335501ef8415bf8c9722548c1761e50252f401fb3e3c12a8afd037354d653923254fb5564684e3e485428cb78855c9b8c53efc07"

    # 8. bundle of two vectors
    RES=$( "$C_REF" bundle "$A_HEX" "$B_HEX" )
    # bundle: majority per byte (sum >= 128 → 0xFF, else 0x00)
    # expected computed by: python3 -c "a=bytes(range(64)); b=bytes((i*3)&0xff for i in range(64)); print(bytes([0xFF if (x+y)>=128 else 0x00 for x,y in zip(a,b)]).hex())"
    check "bundle" "$RES" "0000000000000000000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"

    echo "  C: $PASS passed, $FAIL failed"
}

# For now, only C source is operational.
# Rust bridge and Ne self-compiler will be added when the generated
# compiler binary exists.
#
# verify_rust() {
#     echo "── Source 2: Rust Bridge (bridge.rs) ──"
#     ...
# }
#
# verify_ne() {
#     echo "── Source 3: Ne Self-Compiler ──"
#     ...
# }

SOURCE="${1:-all}"

case "$SOURCE" in
    c|C)
        verify_c
        ;;
    all)
        verify_c
        echo ""
        echo "── Summary ──"
        echo "  C reference:  $PASS/8 passed"
        echo ""
        echo "NOTE: Rust bridge and Ne self-compiler verification will be"
        echo "added after compiler generation. Currently only C source is"
        echo "operational as the trust anchor."
        ;;
    *)
        echo "Usage: $0 [c|all]"
        exit 1
        ;;
esac

if [ "$FAIL" -gt 0 ]; then
    echo "❌ $FAIL verification(s) failed"
    exit 1
fi
echo "✅ All verifications passed"
