#!/bin/bash
# 非空门证明：check-agent-config
# 契约（scripts/probes/_lib.sh）：注入已知违规 → 跑门 → 断言门变红 → 清理。
#
# 注入两个**不同**违规，证明两处独立判据都真的有判别力：
#   #1 C4 工件链棘轮：新增一件缺节的工件 ⇒ 门须红并指名该工件
#   #2 C3 引用完整性：把 review agent 的 REVIEW.md 引用去掉 ⇒ 门须红
#
# ⛔ 为什么不用 `--update-baseline` 兜底：基线是「记录」不是「赦免」。
#   若探针依赖更新基线来变绿，就等于证明「门可以被人随手关掉」——
#   那不是非空证明，是可关闭性证明。
#
# ⛔⛔ 断言必须**指名注入对象**，不能只判 rc：
#   否则门若因别的原因红（存量残留、别窗在途改动），探针会自证循环
#   —— 正是 gate-registry.tsv 里 check-unwrap / check-silent-failure
#   两条 note 记录的历史教训。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

GATE="check-agent-config"
PROBE_PLAN="docs/plans/2999-01-01-probe-missing-sections.md"
AGENT=".opencode/agent/review.md"
AGENT_BAK=".opencode/agent/review.md.probe.bak"

cleanup() {
  rm -f "$PROBE_PLAN"
  if [ -f "$AGENT_BAK" ]; then mv "$AGENT_BAK" "$AGENT"; fi
}
trap cleanup EXIT

# 注入前自检：落点必须干净，否则疑似上次未清理
if [ -e "$PROBE_PLAN" ]; then
  PROBE_FAIL "注入目标已存在: $PROBE_PLAN"
fi
if [ -e "$AGENT_BAK" ]; then
  PROBE_FAIL "备份已存在: $AGENT_BAK"
fi

# ── 注入 #1：一件缺节的新工件 ─────────────────────────────────────
# ⛔ 故意写得有 sections 标题的**一部分**（只 3 节）：若门只查「文件存在」
#    或只查 status 行，本注入就抓不到它 ⇒ 证明门真的在逐节点名。
cat > "$PROBE_PLAN" <<'INNER'
# probe injection: a new artifact that skipped most of the template

status: plan
date: 2999-01-01
owner: probe

## 问题
probe

## 目标
probe

## 影响面
probe
INNER

rc=0
out=$(bash scripts/$GATE.sh --strict 2>&1) || rc=$?
assert_gate_red "$GATE (注入 #1: 缺节工件)" "$rc"

if printf '%s' "$out" | grep -qF "$PROBE_PLAN"; then
  echo "  ✅ 门精确指向注入的工件（${PROBE_PLAN}）"
else
  echo "  ❌ 门红了但未指向注入的工件 ⇒ 可能命中了别的残留，门不精确" >&2
  printf '%s\n' "$out" | sed 's/^/       /' >&2
  exit 1
fi

# 抽一条缺节名，确认报的是**缺哪节**而不只是「有问题」
if printf '%s' "$out" | grep -qE '缺:.*约束'; then
  echo "  ✅ 门点名了缺失小节（约束）⇒ 逐节判定有判别力"
else
  echo "  ❌ 门未点名缺失小节 ⇒ 可能只判了存在性" >&2
  exit 1
fi

rm -f "$PROBE_PLAN"

# ── 注入 #2：去掉 review agent 的 REVIEW.md 引用 ─────────────────
cp "$AGENT" "$AGENT_BAK"
# 用普通引号的字面量替换（⛔ 不用 sed 正则去匹配中文与标点的组合）
python3 - "$AGENT" <<'PY'
import sys, pathlib
p = pathlib.Path(sys.argv[1])
lines = [l for l in p.read_text(encoding='utf-8').splitlines(keepends=True)
         if 'REVIEW.md' not in l]
p.write_text(''.join(lines), encoding='utf-8')
PY

rc=0
out=$(bash scripts/$GATE.sh --strict 2>&1) || rc=$?
assert_gate_red "$GATE (注入 #2: 引用完整性)" "$rc"

if printf '%s' "$out" | grep -qF "$AGENT"; then
  echo "  ✅ 门精确指向断链的消费者（${AGENT}）"
else
  echo "  ❌ 门红了但未指向断链消费者" >&2
  exit 1
fi

echo "PROBE-OK: $GATE 两处注入均变红且均指名注入对象"