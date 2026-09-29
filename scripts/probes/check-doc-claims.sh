#!/bin/bash
# 非空门证明：check-doc-claims
# 契约：注入一个已知违规 → 跑门 → 断言门变红 → 清理。
#
# 违规形态：在 TODO.md 里追加一条「X 零生产消费者」断言，而 X 其实已有消费者。
# 选 noise_ik 是因为它是本仓唯一「已被 C1 引用、且模块名唯一可解析」的
# 真实消费者场景 —— 注入后门必须精确报出 noise_ik 的消费者文件。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

T="TODO.md"
INJECT='| 99 | `noise_ik` 零生产消费者 | 探针 | — |'

cleanup() {
  # 从本探针自己拍的 .bak 整份恢复。
  # ⚠️ 曾经的 bug：用 `grep -v > tmp && mv` 只删注入行，**会改变文件字节**
  #    （实测把 TODO.md 弄出 diff，探针跑完仓库不再干净）。
  #    探针内的 .bak 恢复不是「覆写他人内容」—— 它恢复的就是本次运行开始时
  #    的字节快照，与既有 truth-surface 探针同法。
  if [ -f "$T.probe.bak" ]; then
    mv "$T.probe.bak" "$T"
  fi
  rm -f "$T.probe.tmp"
}
trap cleanup EXIT

[ -f "$T.probe.bak" ] || cp "$T" "$T.probe.bak"

# ⛔ 不要写 `$(grep -c ... || echo 0)`：无匹配时 grep 已输出 `0` 且退出 1，
#    `||` 会再补一个 `0` ⇒ 变量变成 "0\n0"，比较永远不等。
#    用 -q 做存在性判断才是对的。
if grep -q -F -x "$INJECT" "$T" 2>/dev/null; then
  echo "probe: 注入行已存在，疑似上次未清理"; exit 2
fi

printf '\n%s\n' "$INJECT" >> "$T"

rc=0
bash scripts/check-doc-claims.sh >/dev/null 2>&1 || rc=$?
assert_gate_red "check-doc-claims" "$rc"

# 额外确认：门报出的应当就是 noise_ik，而不是别的历史残留断言
if bash scripts/check-doc-claims.sh 2>&1 | grep -q 'noise_ik'; then
  echo "  ✅ 门精确指向注入的断言（noise_ik）"
else
  echo "  ❌ 门红了但未指向注入的断言 ⇒ 可能命中了别的残留，门不精确"
  exit 1
fi
