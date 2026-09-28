#!/bin/bash
# NeoBot CLI 独立分发 — 与桌面端同源 (~/.neobot).
# 用法: bash apps/neobot-desktop/scripts/install-cli.sh
set -euo pipefail
TOP="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$TOP"
cargo install --path crates/neotrix-neobot --bin neobot --locked
echo "--- parity (7 命令 EC:0) ---"
neobot init
neobot doctor
neobot run -t "install check" --text "hello neobot"
neobot task list
neobot audit list
echo "models (echo 模式下失败是正常的, 需 NEOBOT_* http 引擎):"
neobot models || true
echo "neobot CLI ok — 数据目录 ~/.neobot (与 NeoBot 桌面端共享)"
