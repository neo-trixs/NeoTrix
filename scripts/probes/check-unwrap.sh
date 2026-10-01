#!/bin/bash
# 非空门证明：check-unwrap
# 契约：注入一个已知违规 → 跑门 → 断言门变红 → 清理。
#
# ⚠️ 本门**当前就是恒红**（实测 `--strict` rc=1：NEW 5 / STALE 6，全是已提交的
#    生产代码，见提交说明）。所以「它能变红」**不证明**本探针有效 ——
#    恒红的门本来就会红。**必须断言它指名了本次注入的文件**，
#    否则这个探针是自证循环。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

TARGET="scripts/nt_probe_unwrap.rs"

cleanup() { rm -f "$TARGET"; }
trap cleanup EXIT

inject_unwrap_rs "$TARGET"
out=$(bash scripts/check-unwrap.sh --strict 2>&1); rc=$?
printf '%s\n' "$out" | grep -q "nt_probe_unwrap.rs" \
  || PROBE_FAIL "门红了但没指名注入文件（rc=$rc）—— 本门本就恒红，此断言才排除自证循环"
assert_gate_red "check-unwrap" "$rc"
