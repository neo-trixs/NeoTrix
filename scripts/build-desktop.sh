#!/usr/bin/env bash
# NeoTrix 桌面端统一构建脚本 (iPolloWork 式构建阶梯)
#
# 用法:
#   ./scripts/build-desktop.sh <ladder> [--release] [--run]
#
# 阶梯 (由浅入深, 对应 iPolloWork build → package:dir → package):
#   check       类型检查 + 单元测试 + cargo check (最快的验证门)
#   build       前端构建 + cargo build 桌面端二进制 (默认 debug, --release 优化)
#   package:dir 完整 tauri build --no-bundle → 未打包 .app/ 目录 (本地验证)
#   package     完整 tauri build → 原生安装包 (dmg/appimage/msi...) + updater 签名
#
# 说明:
#   Tauri 的资源嵌入发生在编译期 (tauri::generate_context!)。构建前必须
#   先构建前端 dist, 并强制 tauri-build 重新嵌入 (touch tauri.conf.json),
#   否则二进制会嵌入旧的前端资源 → 白屏/内容陈旧。
#
#   package 需要 updater 签名私钥 (TAURI_PRIVATE_KEY / TAURI_PRIVATE_KEY_PATH);
#   无密钥时先用 scripts/setup-updater.sh 生成, 或仅跑 package:dir 验证。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LADDER="${1:-build}"
PROFILE="${2:---debug}"
RUN="${3:-}"

cd "$ROOT"

frontend_build() {
  echo "==> 构建前端 (npm run build)"
  (cd "$ROOT/neocodex-frontend" && npm run build)
  echo "==> 强制 tauri-build 重新嵌入 (touch tauri.conf.json)"
  touch "$ROOT/src-tauri/tauri.conf.json"
}

cmd_check() {
  echo "==> [check] 前端类型检查 + 单元测试"
  (cd "$ROOT/neocodex-frontend" && npx tsc --noEmit && npm test)
  echo "==> [check] Rust cargo check (desktop + core)"
  cargo check --all-targets -p neotrix-tauri
  echo "✅ check 通过"
}

cmd_build() {
  frontend_build
  echo "==> [build] 构建 Tauri 桌面端 ($PROFILE)"
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
}

cmd_package_dir() {
  frontend_build
  echo "==> [package:dir] tauri build --no-bundle (未打包 .app 验证)"
  (cd "$ROOT/neocodex-frontend" && npx tauri build --no-bundle "$PROFILE")
  echo "✅ package:dir 完成 — 未打包应用在 src-tauri/target/release/bundle/macos/ 下"
}

cmd_package() {
  frontend_build
  echo "==> [package] tauri build (原生安装包 + updater 签名)"
  if [ -z "${TAURI_PRIVATE_KEY:-}" ] && [ ! -f "${TAURI_PRIVATE_KEY_PATH:-}" ] \
      && [ ! -f "$HOME/.neotrix/tauri-updater.key" ]; then
    echo "⚠️  未检测到 updater 签名私钥 (TAURI_PRIVATE_KEY / TAURI_PRIVATE_KEY_PATH / ~/.neotrix/tauri-updater.key)"
    echo "   → 先用 scripts/setup-updater.sh 生成密钥, 或改用 ./scripts/build-desktop.sh package:dir 验证构建"
    echo "   → 继续执行无签名 bundle (updater 将不可用)"
  fi
  (cd "$ROOT/neocodex-frontend" && npx tauri build "$PROFILE")
  echo "✅ package 完成 — 安装包在 src-tauri/target/release/bundle/ 下"
}

case "$LADDER" in
  check)        cmd_check ;;
  build)        cmd_build ;;
  package:dir)  cmd_package_dir ;;
  package)      cmd_package ;;
  *)
    echo "用法: $0 <check|build|package:dir|package> [--release] [--run]"
    exit 2
    ;;
esac