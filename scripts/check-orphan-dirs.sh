#!/usr/bin/env bash
# check-orphan-dirs.sh — orphan-dir gate 的 shell 包装
#
# 见 scripts/ops/nt_orphan_dir.py 顶部文档：防的是
# **「目录里有 .rs 但父 mod.rs 从不引用它」⇒ 代码从未编译、测试从未跑，
#   而 CI 全绿。**
#
# 默认 advisory（exit 0 + 清单）；--strict 才对**新增**孤儿非零。
# 既有债记在 scripts/orphan-dir-baseline.txt，棘轮式只拦新增。
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
exec python3 "$ROOT/scripts/ops/nt_orphan_dir.py" "$@"
