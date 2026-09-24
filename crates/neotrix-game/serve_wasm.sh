#!/bin/bash
# NT-GAME 本地 WASM 服务器
# 用法: ./serve_wasm.sh [端口号]

PORT=${1:-8080}
DIR="$(cd "$(dirname "$0")/web" && pwd)"

echo "╔══════════════════════════════════════════╗"
echo "║   NeoTrix Game — 本地服务器              ║"
echo "╚══════════════════════════════════════════╝"
echo ""
echo "📂 目录: $DIR"
echo "🌐 地址: http://localhost:$PORT"
echo "🛑 停止: Ctrl+C"
echo ""

cd "$DIR"
python3 -m http.server "$PORT"
