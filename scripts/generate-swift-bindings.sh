#!/bin/bash
# Generate Swift bindings for the NeoTrix FFI (iOS bridge)
# Usage: scripts/generate-swift-bindings.sh
#
# NOTE: uniffi 0.28 library-mode invokes `cargo metadata` which, in offline/network-
# restricted environments, can hang on full dependency resolution. The
# `--metadata-no-deps` flag skips full resolution (safe when all FFI types are in
# the `neotrix` namespace, which they are).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$ROOT/neotrix-ios/Sources/Bridge/NeoTrixFFI/Generated"
LIB="$ROOT/target/debug/libneotrix.dylib"
BINDGEN="$ROOT/target/debug/uniffi-bindgen"

echo "==> Building neotrix lib (cdylib) with ios-bridge feature"
cargo build -p neotrix --features ios-bridge --lib --manifest-path "$ROOT/neotrix-core/Cargo.toml"

echo "==> Building uniffi-bindgen bin"
cargo build -p neotrix --features ios-bridge --bin uniffi-bindgen --manifest-path "$ROOT/neotrix-core/Cargo.toml"

echo "==> Generating Swift bindings from $LIB"
mkdir -p "$OUT_DIR"
"$BINDGEN" generate \
    --library "$LIB" \
    --metadata-no-deps \
    --language swift \
    --out-dir "$OUT_DIR"

echo "==> Renaming outputs to match Swift module naming"
mv -f "$OUT_DIR/neotrix.swift" "$OUT_DIR/NeoTrixFFI.swift" 2>/dev/null || true

echo "==> Done. Bindings written to $OUT_DIR"
ls -la "$OUT_DIR"