#!/usr/bin/env bash
# NeoTrix 桌面端统一构建脚本 (iPolloWork 式构建阶梯)
#
# 用法:
#   ./skills/dev-tools/build-desktop/build-desktop.sh <ladder> [--release|--debug] [--run]
#
# 阶梯 (由浅入深, 对应 iPolloWork build → package:dir → package):
#   check       前端类型检查 + Rust 测试 + cargo check (最快的验证门)
#   build       前端构建 + cargo build 桌面端二进制 (**默认 --release**; --debug 需先起 dev server)
#   package:dir 完整 tauri build --no-bundle → 未打包 .app/ 目录 (本地验证)
#   package     完整 tauri build → 原生安装包 (dmg/appimage/msi...) + updater 签名
#
# ── 2026-09-28 重写：脚本从"就地构建"改为"委派到独立桌面仓" ──
#
# 上一版把桌面端当成本仓的一部分，于是**每一处引用的路径都已不存在**：
#
#   ROOT=…/skills/dev-tools   脚本被移进 skills/ 后，dirname/.. 早已不是仓库根
#   neocodex-frontend/        不存在
#   src-tauri/tauri.conf.json 不存在（5c02e738 归档）
#   cargo … -p neotrix-tauri  该包不在 workspace（5c02e738）
#   scripts/setup-updater.sh  不存在（错误提示里让人去跑的那个脚本也没了）
#
# 也就是说四个阶梯**全都**跑不通，且与本次会话无关 —— 它在 2026-09-28 归档
# src-tauri 时就已失效，只是一直没人跑它所以没被发现。
#
# 现在桌面端在独立仓 `~/Downloads/Neo/neobot`（d5413335 从本仓移除
# apps/neobot-desktop）。阶梯逻辑本身有用，故保留并改为委派。
# **不是把路径机械替换一遍**：新仓的包名（neobot-desktop ≠ neotrix-tauri）、
# 前端目录、tauri.conf 位置、产物名全都不一样。
#
# 覆盖方式（默认按顺序找，优先级从高到低）:
#   $NEOBOT_DIR            显式指定独立仓路径
#   <repo>/../Neo/neobot   本仓旁边的默认位置
#   找不到就报错退出，不猜。
set -euo pipefail

# 独立桌面仓的位置。**不写死 /Users/...** —— 换用户名或换机器就失效。
find_desktop_repo() {
  if [ -n "${NEOBOT_DIR:-}" ]; then
    [ -d "$NEOBOT_DIR" ] || { echo "✗ NEOBOT_DIR 指向的目录不存在: $NEOBOT_DIR" >&2; return 1; }
    printf '%s' "$NEOBOT_DIR"; return 0
  fi
  # 本脚本在 <neotrix>/skills/dev-tools/build-desktop/ 下，故上溯三层才是 neotrix 根。
  local here neotrix_root cand
  here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  neotrix_root="$(cd "$here/../../.." && pwd)"
  for cand in "$neotrix_root/../Neo/neobot" "$HOME/Downloads/Neo/neobot" "$HOME/Neo/neobot"; do
    if [ -f "$cand/apps/neobot-desktop/tauri.conf.json" ]; then
      printf '%s' "$cand"; return 0
    fi
  done
  return 1
}

LADDER="${1:-build}"
# **默认 --release, 不是 --debug。**
#
# 2026-09-28 实测踩出来的坑: `cargo build`(debug) 产出的 Tauri 2 二进制会去
# `tauri.conf.json` 的 `build.devUrl`(http://localhost:1422) 取前端, 而不是用
# 编译期嵌进二进制的 `frontendDist`。1422 上没有 vite dev server 时, webview
# 加载到空页面 —— **窗口开得出来、全白、无任何报错输出**(Rust 侧不崩, stdout
# 零行), 看起来像"前端炸了", 实际是前端根本没被加载过。
#
# `cargo build --release` 走嵌入资源, 不依赖任何 dev server, 才是"拿去就能跑"
# 的那个。debug 保留为显式选项, 但会在启动前检查 1422 有没有在服务。
PROFILE="${2:---release}"
RUN="${3:-}"

warn_debug_needs_devserver() {
  [ "$PROFILE" = "--release" ] && return 0
  if ! lsof -nP -iTCP:1422 -sTCP:LISTEN >/dev/null 2>&1; then
    cat >&2 <<'EOF'
✗ debug 构建需要 vite dev server, 但 1422 端口没有东西在监听。

  症状: 窗口开得出来但**全白**, 且没有任何报错 —— 因为 webview 是去
        http://localhost:1422 取前端, 那里没人应答, 它就渲染出一个空页面。

两个办法:
  1) 用 release (推荐, 走编译期嵌入的 frontendDist, 无需 dev server):
       $0 build --release
  2) 先起 dev server 再跑 debug:
       npm --prefix <桌面仓>/apps/neobot-desktop/frontend run dev
       $0 build --debug
EOF
    exit 2
  fi
}

if ! REPO="$(find_desktop_repo)"; then
  cat >&2 <<'EOF'
✗ 找不到独立桌面仓。

桌面端已于 2026-09-28 从本仓迁出（src-tauri 归档于 5c02e738，
apps/neobot-desktop 移除于 d5413335），现在位于独立仓 Neo/neobot。

请指路:
  NEOBOT_DIR=/path/to/Neo/neobot ./build-desktop.sh build
EOF
  exit 2
fi
FE="$REPO/apps/neobot-desktop/frontend"
APP="$REPO/apps/neobot-desktop"

# 说明:
#   Tauri 的资源嵌入发生在编译期 (tauri::generate_context!)。构建前必须
#   先构建前端 dist, 并强制 tauri-build 重新嵌入 (touch tauri.conf.json),
#   否则二进制会嵌入旧的前端资源 → 白屏/内容陈旧。
#
#   package 需要 updater 签名私钥 (TAURI_PRIVATE_KEY / TAURI_PRIVATE_KEY_PATH);
#   无密钥时先用 package:dir 验证。
frontend_build() {
  echo "==> 构建前端 (npm run build)"
  npm --prefix "$FE" run build
  echo "==> 强制 tauri-build 重新嵌入 (touch tauri.conf.json)"
  touch "$APP/tauri.conf.json"
}

cmd_check() {
  echo "==> [check] 前端类型检查"
  npm --prefix "$FE" run typecheck
  echo "==> [check] Rust 测试 + cargo check (桌面端 + 库)"
  (cd "$REPO" && CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}" cargo check -j "${CARGO_BUILD_JOBS:-2}" --all-targets)
  (cd "$REPO" && CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}" cargo test -j "${CARGO_BUILD_JOBS:-2}")
  echo "✅ check 通过"
}

cmd_build() {
  warn_debug_needs_devserver
  frontend_build
  echo "==> [build] 构建 Tauri 桌面端 ($PROFILE)"
  case "$PROFILE" in
    --release) (cd "$REPO" && cargo build --release -p neobot-desktop) ;;
    *)         (cd "$REPO" && cargo build -p neobot-desktop) ;;
  esac
  BIN="$REPO/target/$([ "$PROFILE" = "--release" ] && echo release || echo debug)/neobot-desktop"
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
  (cd "$APP" && npx tauri build --no-bundle "$PROFILE")
  echo "✅ package:dir 完成 — 未打包应用在 apps/neobot-desktop/target/release/bundle/macos/ 下"
}

cmd_package() {
  frontend_build
  echo "==> [package] tauri build (原生安装包 + updater 签名)"
  if [ -z "${TAURI_PRIVATE_KEY:-}" ] && [ ! -f "${TAURI_PRIVATE_KEY_PATH:-}" ] \
      && [ ! -f "$HOME/.neotrix/tauri-updater.key" ]; then
    echo "⚠️  未检测到 updater 签名私钥 (TAURI_PRIVATE_KEY / TAURI_PRIVATE_KEY_PATH / ~/.neotrix/tauri-updater.key)"
    echo "   → 可先用 $0 package:dir 验证构建，再决定是否生成密钥"
    echo "   → 继续执行将产出无签名 bundle (updater 不可用)"
  fi
  (cd "$APP" && npx tauri build "$PROFILE")
  echo "✅ package 完成 — 安装包在 apps/neobot-desktop/target/release/bundle/ 下"
}

echo "桌面端仓库: $REPO"
case "$LADDER" in
  check)       cmd_check ;;
  build)       cmd_build ;;
  package:dir) cmd_package_dir ;;
  package)     cmd_package ;;
  *)
    echo "用法: $0 <check|build|package:dir|package> [--release|--debug] [--run]"
    exit 2
    ;;
esac
