#!/usr/bin/env bash
# check-cli-plugin-descriptors.sh — 外部 CLI descriptor 探活门的 shell 包装（TODO.md F3）
#
# 为什么要有这层壳（不是多余）：
#   `scripts/check-gate-satisfiable.sh` 按 `grep --strict scripts/check-*.sh`
#   **发现**哪些门需要非空性证明。只放在 `scripts/ops/` 的 py 门它**看不见**
#   ⇒ 有证据但制度不执行。本仓已有同型先例：`check-claims-numbers.sh` 是
#   `nt_claims_numbers.py` 的壳、`check-orphan-dirs.sh` 是 `nt_orphan_dir.py`
#   的壳。⇒ 逻辑在 scripts/ops/nt_cli_plugin_probe.py，这里只 exec。
#
# 判据（必填字段 / 未知字段 / mode 枚举 / 类型 / 可解析 / 防空转 / 探活）
# **不复刻**，而是从 Rust 真源 external_cli_plugins.rs 的 struct 定义里
# **正则抽出**（真源漂移 ⇒ 门自己会红）。见 py 文件顶部文档。
#
# 用法: bash scripts/check-cli-plugin-descriptors.sh [--strict] [--list] [--self-test]
#   默认 advisory（报告，exit 0）；--strict 任一失败 exit 1。
#   ⛔ 刻意不加 -e：本壳只有一条 exec，-e 在这里其实是 no-op，加了只会给人
#      「错误由 shell 兜住」的错觉（真实成败全部由 py 侧的 exit code 表达）。
#      与 check-claims-numbers.sh / check-orphan-dirs.sh 同族保持一致。
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
exec python3 "$ROOT/scripts/ops/nt_cli_plugin_probe.py" "$@"