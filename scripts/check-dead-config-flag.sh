#!/usr/bin/env bash
# check-dead-config-flag.sh — 「声明了默认值却从未被读取」的配置开关门
#
# 见 scripts/ops/nt_dead_flag.py 顶部文档：要防的是比编译错误更隐蔽的一类缺陷 ——
# 开关**看起来是活的**（默认值存在、`deny(warnings)` 干净），
# 但代码里从未读取，该能力实际不存在（A55 的 enable_semantic_mapping 实例）。
#
# 默认 advisory（exit 0 + 清单）；--strict 才对基线外的新增非零。
# 既有债记在 scripts/dead-flag-baseline.txt，棘轮式只拦新增。
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
exec python3 "$ROOT/scripts/ops/nt_dead_flag.py" --root "$ROOT" "$@"
