#!/usr/bin/env bash
# check-claims-numbers.sh — claims 数字追溯门的 shell 包装
#
# 为什么要有这层壳（不是多余）：
#   `scripts/check-gate-satisfiable.sh` 按 `grep --strict scripts/check-*.sh`
#   **发现**哪些门需要非空性证明。只放在 `scripts/ops/` 的 py 门它**看不见**
#   ⇒ 有探针也不会被元门跑到，等于证据存在但制度不执行。
#   本仓已有同型先例：`check-orphan-dirs.sh` 就是 `nt_orphan_dir.py` 的壳。
#   ⇒ 逻辑在 scripts/ops/nt_claims_numbers.py，这里只 exec。
#
# 见 scripts/ops/nt_claims_numbers.py 顶部文档（判据、两条设计教训、
# 防空转双判据、为什么注册表**不存字面量**）。
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
exec python3 "$ROOT/scripts/ops/nt_claims_numbers.py" "$@"
