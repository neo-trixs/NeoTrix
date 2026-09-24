#!/bin/bash
# NT-GAME WASM 构建脚本（macroquad 官方流：cargo 打 BIN + miniquad gl.js 加载）
# 用法: ./build_wasm.sh
#
# 前置条件:
#   - rustup (非 Homebrew Rust) + wasm32-unknown-unknown 目标
#   - 不需要 wasm-pack（macroquad 不走 wasm-bindgen；打空 lib 是已知坑）
#
# 问题排查:
#   如果系统装了 Homebrew Rust，需要确保 rustup 的 cargo/rustc 在 PATH 前面：
#   export PATH="$HOME/.cargo/bin:$PATH"

set -e
cd "$(dirname "$0")"

echo "╔══════════════════════════════════════════╗"
echo "║   NeoTrix Game Engine — WASM Build       ║"
echo "╚══════════════════════════════════════════╝"

# 确保 rustup 的工具链在 PATH 前面（解决 Homebrew Rust 冲突）
export PATH="$HOME/.cargo/bin:$PATH"

# 验证 rustc 来源
RUSTC_PATH=$(which rustc)
RUSTC_VERSION=$(rustc --version)
echo "→ rustc: $RUSTC_PATH ($RUSTC_VERSION)"

# 检查 rustup
if ! command -v rustup &> /dev/null; then
    echo "❌ 需要安装 rustup: https://rustup.rs"
    exit 1
fi

# 检查 wasm32 目标
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "→ 安装 wasm32-unknown-unknown 目标..."
    rustup target add wasm32-unknown-unknown
fi

echo "→ 构建 WASM BIN (release)..."
cargo build --release --target wasm32-unknown-unknown --bin neotrix-game

echo "→ 落盘 web/pkg/neotrix-game.wasm..."
mkdir -p web/pkg
cp ../../target/wasm32-unknown-unknown/release/neotrix-game.wasm web/pkg/neotrix-game.wasm
ls -la web/pkg/neotrix-game.wasm

echo ""
echo "✅ 构建完成!"
echo ""
echo "运行方式:"
echo "  ./serve_wasm.sh"
echo "  打开浏览器: http://localhost:8080/index.html"
