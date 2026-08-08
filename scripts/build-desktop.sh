#!/usr/bin/env bash
# NeoTrix 桌面端统一构建脚本
#
# 用法:
#   ./scripts/build-desktop.sh [--release] [--run]
#
# 说明:
#   Tauri 的资源嵌入发生在编译期 (tauri::generate_context!)。构建前必须
#   先构建前端 dist, 并强制 tauri-build 重新嵌入 (touch tauri.conf.json),
#   否则二进制会嵌入旧的前端资源 → 白屏/内容陈旧。
#
# 步骤:
#   1. 构建前端 (neocodex-frontend)
#   2. touch tauri.conf.json 强制 tauri-build 重跑
#   3. cargo build -p neotrix-tauri (debug/release)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROFILE="${1:---debug}"
RUN="${2:-}"

cd "$ROOT/neocodex-frontend"
echo "==> [1/3] 构建前端 (npm run build)"
npm run build

echo "==> [2/3] 强制 tauri-build 重新嵌入 (touch tauri.conf.json)"
touch "$ROOT/src-tauri/tauri.conf.json"

echo "==> [3/3] 构建 Tauri 桌面端 ($PROFILE)"
cd "$ROOT"
case "$PROFILE" in
  --release) cargo build --release -p neotrix-tauri ;;
  *)         cargo build -p neotrix-tauri ;;
esac

BIN="$ROOT/target/$([ "$PROFILE" = "--release" ] && echo release || echo debug)/neotrix-tauri"
echo ""
echo "✅ 构建完成: $BIN"
if [ "$RUN" = "--run" ]; then
  echo "==> 启动桌面端"
  "$BIN"
fi
