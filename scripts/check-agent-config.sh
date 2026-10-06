#!/usr/bin/env bash
# check-agent-config.sh — agent 配置回归门 + 变更工件链棘轮
#
# 吸收源（思想，非代码）：
#   Anthropic《The AI-native SDLC playbook》(2026-08-21) Stage 4 Test §
#   「Continuous evals in CI」第 3 步：「suite runs ... on any change to
#   CLAUDE.md, skills or hooks, **since that configuration steers the agent
#   and deserves the regression testing that code gets**」。
#   原文事实源：docs/architecture/ABSORPTION-AI-NATIVE-SDLC-2026-10-06.md
#
# ── 立门理由（本仓复发型缺陷：agent 配置没有回归门）──────────────────
#   实测（2026-10-06）：ci.yml 的 on: 无 paths 过滤，任何 PR 都会触发 5 分钟
#   的 cargo 套件 —— 但改 skills/index.json 的 PR **不会**跑 check-skill-gate，
#   改 AGENTS.md 的 PR **不会**跑任何配置校验，改 .neotrix/task-index.json 的 PR
#   **不会**跑 nt_find.py --audit（那只在 pre-commit，而 pre-commit 可被
#   --no-verify 跳过，且 AGENTS.md §1 记载共享 index 下它本身不原子）。
#   ⇒ **配置在改，配置无门**：这正是 playbook 点名的「configuration deserves
#   the regression testing that code gets」。
#   姊妹证据：`check-skill-gate.sh` 在 CI 与 Makefile **双双零引用**（本仓自述
#   见 nt_gate_coverage.py 的 EXEMPT 表），而它一旦 strict 就是恒红（存量未清）
#   ⇒ 故本门**不复用**它，只守引用完整性（见 C2/C3）。
#
# ── 判定分工（本门只做「agent 配置」这一类，不越界）────────────────
#   C1 工件模板在位      —— 链的起点存在
#   C2 REVIEW.md 在位    —— 审查策略正典存在
#   C3 引用完整性        —— 消费者必须指向正典（指针守恒的机器判据）
#   C4 工件链棘轮        —— 基线之后新增的工件必须写全六段
#   C5 任务索引自审      —— nt_find.py --audit（索引指向的工具必须存在）
#   C6 防空转            —— 门自己的扫描对象非空（抄 better-sidebar：
#                          「a glob that silently matches nothing would make
#                          this contract vacuous」）
#
# 用法: check-agent-config.sh [--strict] [--update-baseline]
#   默认 advisory：只报告，exit 0（不挡存量）。
#   --strict：任一失败 exit 1（供 CI）。
#   --update-baseline：把当前 docs/plans/*.md 写入基线（**只用于立门当日**，
#     之后调用等于给违规发永久通行证 —— 见文件末尾的警告）。
#
# ⛔ 本门**只读**：不写任何被扫对象（唯一写入是 --update-baseline 写基线本身）。
#    抄 check-disk.sh 的教训（R-SCAN-4）：门脚本里不得有会真的执行的示例命令。
set -uo pipefail

cd "$(dirname "$0")/.." || exit 2

STRICT=0
UPDATE_BASE=0
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    --update-baseline) UPDATE_BASE=1 ;;
    *) echo "usage: $0 [--strict] [--update-baseline]" >&2; exit 2 ;;
  esac
done

TEMPLATE="docs/plans/_TEMPLATE.md"
REVIEW="REVIEW.md"
REVIEW_AGENT=".opencode/agent/review.md"
PLANS_DIR="docs/plans"
BASELINE="scripts/artifact-chain-baseline.txt"

FAIL=0
fail() { echo "  ❌ $*"; FAIL=$((FAIL + 1)); }
ok()   { echo "  ✅ $*"; }

echo "agent-config gate (strict=$STRICT)"

# ── C1 工件模板在位 ────────────────────────────────────────────────
n_tpl=0
if [ -s "$TEMPLATE" ]; then
  ok "C1 工件模板在位: $TEMPLATE"
  n_tpl=1
else
  fail "C1 工件模板缺失或为空: $TEMPLATE —— 变更工件链没有起点"
fi

# ── C2 REVIEW.md 在位 ──────────────────────────────────────────────
if [ -s "$REVIEW" ]; then
  ok "C2 审查策略正典在位: $REVIEW"
else
  fail "C2 审查策略正典缺失或为空: $REVIEW —— severity 分桶与不报清单无处为据"
fi

# ── C3 引用完整性（消费者必须指向正典）─────────────────────────────
#   抄 R-P199 的教训：不按行号引用，按**章节名/文件名**引用。
n_ref=0
if [ -f "$REVIEW_AGENT" ]; then
  if grep -q 'REVIEW\.md' "$REVIEW_AGENT"; then
    # ⛔ 反向断言：自造的 3 级词表若还在，引用完整性是假的（它会同时指向两个正典）
    if grep -qE 'Blocker *[/／] *Warning|Blocker/Warning/Info' "$REVIEW_AGENT"; then
      fail "C3 $REVIEW_AGENT 仍带自造词表 'Blocker/Warning/Info' —— 与 REVIEW.md §1 的 Severity 正典并存 ⇒ 两套口径"
    else
      ok "C3 引用完整: $REVIEW_AGENT → ${REVIEW}（且无自造词表残留）"
      n_ref=1
    fi
  else
    fail "C3 $REVIEW_AGENT 未引用 $REVIEW —— 策略改了消费者不知道（指针守恒缺口）"
  fi
else
  fail "C3 消费者缺失: $REVIEW_AGENT"
fi

# ── C4 工件链棘轮 ──────────────────────────────────────────────────
#   ⛔ 存量 66 件工件一节都不全（实测）⇒ **普查即出生恒红**，恒红的门等于
#   没有门（本仓门纪律）。故只对**基线之后新增**的工件判红。
REQUIRED=("## 问题" "## 目标" "## 影响面" "## 约束" "## 未决问题" \
          "## 改动文件" "## 施工顺序" "## 风险" "## 证据")

if [ "$UPDATE_BASE" -eq 1 ]; then
  # ⛔⛔ 只在立门当日用。之后调用 = 给当前所有违规发永久通行证，
  #    而基线是「记录」不是「赦免」—— 故每次写入都留时间戳，便于事后分辨。
  {
    echo "# 变更工件链棘轮基线 — 每行一件 (path|条数) 形式为 'path'。"
    echo "# 生成于 $(date +%Y-%m-%d)。既有工件早于本约定，不追溯。"
    echo "# ⛔ 绝不要为了让门变绿而删条目（那等于让违规永久合法化）。"
    for f in "$PLANS_DIR"/*.md; do
      [ -f "$f" ] || continue
      case "$(basename "$f")" in _TEMPLATE.md) continue ;; esac
      echo "$f"
    done
  } > "$BASELINE"
  echo "  ⟳ 基线已写入 ${BASELINE}（$(grep -cv '^#' "$BASELINE") 件存量）"
fi

n_artifact=0
n_baselined=0
if [ -d "$PLANS_DIR" ]; then
  for f in "$PLANS_DIR"/*.md; do
    [ -f "$f" ] || continue
    case "$(basename "$f")" in _TEMPLATE.md) continue ;; esac
    n_artifact=$((n_artifact + 1))
    # 基线命中 = 存量，不判
    if [ -f "$BASELINE" ] && grep -qxF "$f" "$BASELINE"; then
      n_baselined=$((n_baselined + 1))
      continue
    fi
    # 新工件：逐节点名，缺哪节报哪节
    missing=""
    for sec in "${REQUIRED[@]}"; do
      if ! grep -qF "$sec" "$f"; then
        missing="$missing ${sec#\#\# }"
      fi
    done
    if ! grep -qE '^status:[[:space:]]*(intent|spec|plan|done)' "$f"; then
      missing="$missing [status:行]"
    fi
    if [ -n "$missing" ]; then
      fail "C4 新工件缺节: $f ⇒ 缺:${missing}（模板见 ${TEMPLATE}）"
    else
      ok "C4 新工件齐备: $f"
    fi
  done
fi

# ── C5 任务索引自审 ────────────────────────────────────────────────
#   nt_find.py --audit 校验索引指向的 scripts/* 路径与 make target 真的存在。
#   ⛔ 索引漂移的后果是**静默**：agent 照着旧路径跑，得到「工具不存在」而
#   不是报错（pre-commit:74-76 原话：静默失败比报错更贵）。
if [ -f .neotrix/task-index.json ] && [ -f scripts/ops/nt_find.py ]; then
  audit_out=$(python3 scripts/ops/nt_find.py --audit 2>&1)
  audit_rc=$?
  if [ "$audit_rc" -ne 0 ]; then
    fail "C5 任务索引自审失败 (rc=$audit_rc):"
    echo "$audit_out" | sed 's/^/       /' | head -20
  else
    ok "C5 任务索引自审通过: $(echo "$audit_out" | tail -1)"
  fi
else
  fail "C5 索引或审计器缺失: .neotrix/task-index.json / scripts/ops/nt_find.py"
fi

# ── C6 防空转（本门自己的扫描对象必须非空）──────────────────────────
#   ⛔ 若 docs/plans/ 整个空了、或者模板路径写错，上面所有 for 循环一次都不进，
#     C4 就会「全绿」—— 那是一个**从来没跑过的门**，不是通过。
if [ "$n_artifact" -eq 0 ]; then
  fail "C6 防空转失败: $PLANS_DIR/*.md 扫描到 0 件工件 ⇒ C4 是空门"
else
  ok "C6 防空转通过: 扫到 $n_artifact 件工件（其中 $n_baselined 件基线存量）"
fi
if [ "$n_tpl" -eq 0 ]; then
  fail "C6 防空转失败: 模板 $TEMPLATE 不可读 ⇒ C1 是空门"
fi
if [ "$n_ref" -eq 0 ]; then
  fail "C6 防空转失败: 消费者 $REVIEW_AGENT 不可读 ⇒ C3 是空门"
fi

# ── 报告（playbook Stage 5/6 的先行 + 滞后指标，见 REVIEW.md §6）──
echo "  ── 指标（先行）────────────────────────────────────────"
total=$n_artifact
echo "     工件存量: $total   基线内(不追溯): $n_baselined   棘轮管辖: $((total - n_baselined))"
echo "  ⛔ 指标口径提醒：棘轮只拦**新增**违规，故「工件合规率」这个数字**永远 100%**，"
echo "     它不是质量指标而是「增量是否守规矩」的指标。别拿它汇报覆盖率。"

echo
if [ "$FAIL" -eq 0 ]; then
  echo "agent-config: PASS（$FAIL 项失败）"
  exit 0
fi
if [ "$STRICT" -eq 1 ]; then
  echo "agent-config: FAIL — $FAIL 项"
  exit 1
fi
echo "agent-config: DONE(advisory) — $FAIL 项，使用 --strict 阻断"
exit 0