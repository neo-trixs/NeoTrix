#!/usr/bin/env bash
# Three-source verification for Ne bootstrap.
# Verifies all 3 implementations produce byte-identical outputs.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
C_REF="$SCRIPT_DIR/c_reference/vsa"
TMPDIR="${TMPDIR:-/tmp}/neotrix-verify-$$"
mkdir -p "$TMPDIR"
trap 'rm -rf "$TMPDIR"' EXIT

# VSA-like binary test vectors (balanced 0/1, deterministic seed=42)
R1="00000100000000000100000000000000010001010000010101000001000001000101010001000100010100000000010000000101010100010100010000000001"
R2="01010001000000010101000001000101010001000001010101000001000000000001000101010100000101000101000100010000010000010000010100000101"

RUST_BIND_EXPECTED="00000100000100000001000100000000010100000100000100000100000001010100010100010001010100000001010000000000000001000001000001000001"
RUST_UNBIND_EXPECTED="01010001010100010100000001010101010001000001010101000001000001000001000101010101000101010101010101010101010000010000010000010000"

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
    
    RES=$( "$C_REF" bind "$R1" "$R2" )
    check "bind (FFT-HRR)" "$RES" "$RUST_BIND_EXPECTED"

    RES=$( "$C_REF" unbind "$RUST_BIND_EXPECTED" "$R1" )
    check "unbind (FFT-HRR)" "$RES" "$RUST_UNBIND_EXPECTED"

    RES=$( "$C_REF" negate "$R1" )
    check "negate" "$RES" "fffffefffffffffffefffffffffffffffefffefefffffefefefffffefffffefffefefefffefffefffefefffffffffefffffffefefefefffefefffefffffffffe"

    RES=$( "$C_REF" permute "$R1" 7 )
    check "permute" "$RES" "00010000000000000001000101000001010100000100000100010101000100010001010000000001000000010101010001010001000000000100000100000000"

    RES=$( "$C_REF" similarity "$R1" "$R2" )
    check "similarity" "$RES" "0.9433593750"

    RES=$( "$C_REF" cosine "$R1" "$R2" )
    check "cosine" "$RES" "0.5120915565"

    RES=$( "$C_REF" hamming_distance "$R1" "$R2" )
    check "hamming_distance" "$RES" "29"

    RES=$( "$C_REF" random_vector 42 )
    check "random_vector" "$RES" "aabf7a688223d332f0259fdc335501ef8415bf8c9722548c1761e50252f401fb3e3c12a8afd037354d653923254fb5564684e3e485428cb78855c9b8c53efc07"

    RES=$( "$C_REF" bundle "$R1" "$R2" )
    check "bundle" "$RES" "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"

    RES=$( "$C_REF" xor_bind "$R1" "$R2" )
    check "xor_bind (preserved)" "$RES" "01010101000000010001000001000101000000010001000000000000000001000100010100010000010001000101010100010101000100000100000100000100"

    echo "  C: $PASS passed, $FAIL failed"
}

verify_rust() {
    echo "── Source 2: Rust Bridge (not yet operational) ──"
    echo "  (will be added after Stage 0 compiler generation)"
}

verify_ne() {
    echo "── Source 3: Ne Self-Compiler (not yet operational) ──"
    echo "  (will be added after Stage 1 self-hosting)"
}

SOURCE="${1:-all}"
case "$SOURCE" in
    c|C) verify_c ;;
    all)
        verify_c
        echo ""
        echo "── Summary ──"
        echo "  C reference:  $PASS/10 passed"
        echo ""
        echo "NOTE: Rust bridge and Ne self-compiler verification will be"
        echo "added after compiler generation. Currently only C source is"
        echo "operational as the trust anchor."
        ;;
    *) echo "Usage: $0 [c|all]"; exit 1 ;;
esac

if [ "$FAIL" -gt 0 ]; then echo "❌ $FAIL verification(s) failed"; exit 1; fi
echo "✅ All verifications passed"
