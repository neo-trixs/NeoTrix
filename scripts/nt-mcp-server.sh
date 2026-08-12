#!/bin/sh
# NeoTrix MCP server launcher (stdio JSON-RPC 2.0).
#
# 启动策略（防止 opencode MCP 30s 超时）:
#   1. 优先用 target 外的稳定副本 (~/.local/share/neotrix/neotrix-mcp) —
#      不受 `cargo clean` / 并发构建清空 target 影响。
#   2. 副本缺失 → 用 target/debug/neotrix（若已编译）。
#   3. 均缺失 → 离线编译一次再启动（依赖已缓存, 失败则走在线编译）。
#
# 刷新稳定副本: scripts/nt-mcp-server.sh --refresh
set -u

STABLE="$HOME/.local/share/neotrix/neotrix-mcp"
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

if [ "${1:-}" = "--refresh" ]; then
    if [ ! -x "$ROOT/target/debug/neotrix" ]; then
        cargo build -q -p neotrix --bin neotrix --manifest-path "$ROOT/Cargo.toml" || exit 1
    fi
    mkdir -p "$(dirname -- "$STABLE")"
    cp "$ROOT/target/debug/neotrix" "$STABLE"
    echo "refreshed: $STABLE"
    exit 0
fi

start() {
    exec "$1" mcp-server
}

if [ -x "$STABLE" ]; then
    start "$STABLE"
fi
if [ -x "$ROOT/target/debug/neotrix" ]; then
    start "$ROOT/target/debug/neotrix"
fi
# 首次启动: 离线快速编译（依赖已缓存在 ~/.cargo）。
if ! cargo build -q --offline -p neotrix --bin neotrix --manifest-path "$ROOT/Cargo.toml" 2>/dev/null; then
    cargo build -q -p neotrix --bin neotrix --manifest-path "$ROOT/Cargo.toml" || { echo "neotrix build failed" >&2; exit 1; }
fi
mkdir -p "$(dirname -- "$STABLE")"
cp "$ROOT/target/debug/neotrix" "$STABLE"
start "$STABLE"
