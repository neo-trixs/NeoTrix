#!/bin/bash
# 非空门证明：check-silent-failure
# 契约：注入一个已知违规 → 跑门 → 断言门变红 → 清理。
#
# ⭐ 断言**指名注入行**，不只断言 rc≠0：
#   `assert_gate_red` 只证明「有东西红了」。若门因**别的原因**已经红，
#   探针照样通过 ⇒ 什么都没证明。grep 门自己输出的 `+ <path>:<line>`
#   才能把「红」**绑定到本次注入** —— 这是证据，不是巧合。
set -uo pipefail                       # ⛔ 不用 -e（断言要自己控制流）
cd "$(dirname "$0")/../.." || exit 2  # ⛔ 必须先 cd 到仓库根
. scripts/probes/_lib.sh

TARGET="neotrix-core/src/nt_probe_sf.rs"

cleanup() { rm -f "$TARGET"; }
trap cleanup EXIT                       # ⛔ 必须在注入之前

inject_silent_discard "$TARGET"
out=$(bash scripts/check-silent-failure.sh --strict 2>&1); rc=$?
printf '%s\n' "$out" | grep -q "nt_probe_sf.rs" \
  || PROBE_FAIL "门红了但没指名注入文件（rc=$rc）—— 门可能在因别的原因红"
assert_gate_red "check-silent-failure" "$rc"
