#!/bin/bash
# 门可满足性元门 —— 检查**门本身**是否可信。
#
# ## 为什么需要这道元门
#
# 依据 `i-have-adhd`（51.9k★）的回测报告：它的发布门有一条绝对规则
# *"It has no blocking findings."*，而**相邻规则是比较性的**。后果：
# 一个把阻断项从 7 降到 3 的候选**仍然失败**。原文：
#
#   "The gate fails on one rule: 'It has no blocking findings.' The candidate
#    has three. The rule is absolute where the neighbouring rules are
#    comparative, so a candidate that more than halves the blocker count
#    (7 → 3) still fails."
#   "as written, **no candidate can ever pass** while any blocker survives
#    anywhere in the case set … This is a property of the gate worth deciding
#    on deliberately rather than discovering during a release."
#
# **关键**：它「永远无法通过」这件事，**是跑出来的，不是看出来的**。
#
# ⇒ 本门对每道 `--strict` 门问**两个**问题，缺一即红：
#
#   **Q1 非空门（gate is not vacuous）**
#     注入一个**已知违规**，该门必须变红。
#     ⛔ 从不红的门 = 一个永远绿的假门。它比没有门更坏，因为没人知道它坏了。
#
#   **Q2 可满足性（gate is satisfiable）**
#     存在一个**已知合规**的输入，该门必须变绿。
#     ⛔ 永远红的门同样有害 —— 它训练人忽略红色。
#     依据 `awesome-dsh-plugin/.github/workflows/pr-gate.yml:44-60`：
#     "A gate that dies before posting is indistinguishable from one that never
#      needed to run."
#
# ## 本门刻意不做的事
#
# ⛔ **不替每道门写注入逻辑。** 14 道门结构各异，逐一注入要么写 14 份
#   易腐的脚本，要么在元门里堆一堆 case 语句 —— 两者都会烂尾
#   （`Agentero` 与本仓的 dependency 门都只有 3 条规则，正是因为规则一多就没人维护）。
#   ⇒ **改为「登记制」**：每道门在 `scripts/gate-registry.tsv` 里声明
#   它的注入方式与合规方式；本门只负责**执行登记并核对声明与现实一致**。
#   未登记的门 = **未审**（不是失败，是缺证据）。
#
# 用法：
#   bash scripts/check-gate-satisfiable.sh            # advisory, exit 0
#   bash scripts/check-gate-satisfiable.sh --strict   # 有「未登记 / 恒红 / 假绿」即 exit 1
#   bash scripts/check-gate-satisfiable.sh --list     # 只打印登记表
#
# Note: bash-3.2-safe style (macOS system bash). `bash -n` before commit.

set -uo pipefail

STRICT=0
LIST=0
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    --list) LIST=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

REG="scripts/gate-registry.tsv"
GATE_DIR="scripts"

# 列出所有 --strict 门
GATES=$(for s in "$GATE_DIR"/check-*.sh; do
  [ -f "$s" ] || continue
  if grep -q -- "--strict" "$s" 2>/dev/null; then
    echo "$s"
  fi
done | sort)

if [ "$LIST" -eq 1 ]; then
  echo "=== --strict 门清单 ==="
  echo "$GATES" | sed 's/^/  /'
  exit 0
fi

echo "=== 门可满足性元门 ==="

# ---------- 读登记表 ----------
# 列：gate<TAB>class<TAB>probe_cmd<TAB>expect_violation_rc<TAB>note
# class: injectable（可注入违规）/ opaque（无法在无副作用下注入）
# probe_cmd: 注入违规后运行什么；用 {GATE} 占位脚本路径
#
# ⚠️ 不用 bash 关联数组：macOS 系统 bash 是 3.2，`declare -A` 不存在。
# （这是本门开发时实测撞到的，注释留档以免下一个人再写一次。）
reg_field() {  # reg_field <gate> <列号 1..4>
  [ -f "$REG" ] || return 1
  awk -F'\t' -v g="$1" -v col="$2" '
    $1 == g && $0 !~ /^#/ { print $col; exit }
  ' "$REG"
}

TOTAL=0; REGISTERED=0; UNREGISTERED=0
OPAQUE=0; SELF_SKIP=0
ALWAYS_RED=0; ALWAYS_GREEN_PROVEN=0; PROBE_FAILED=0
RED_LIST=""; UNREG_LIST=""; FAIL_LIST=""

for gate in $GATES; do
  TOTAL=$((TOTAL+1))
  name=$(basename "$gate")

  # ── Q0：登记了吗？ ──
  klass=$(reg_field "$gate" 2)
  if [ -z "$klass" ]; then
    UNREGISTERED=$((UNREGISTERED+1))
    UNREG_LIST="$UNREG_LIST  $name\n"
    continue
  fi
  REGISTERED=$((REGISTERED+1))

  # opaque 类：声明「无法在无副作用下注入」—— 不算失败，但要显式可见
  if [ "$klass" = "opaque" ]; then
    OPAQUE=$((OPAQUE+1))
    continue
  fi

  # ⛔ 本门**不探测自己**：`{GATE} --strict` 会递归调用本门，
  #   而本门又会探测自己 ⇒ 无限递归（第一版实测 exit=124 超时）。
  #   本门的自证方式不同：见文件末尾的「自检」段。
  if [ "$name" = "check-gate-satisfiable.sh" ]; then
    SELF_SKIP=$((SELF_SKIP+1))
    continue
  fi

  # ── Q2：当前状态下该门是绿的吗？（恒红检测） ──
  timeout 120 bash "$gate" --strict >/dev/null 2>&1
  cur_rc=$?
  if [ "$cur_rc" -ne 0 ]; then
    ALWAYS_RED=$((ALWAYS_RED+1))
    RED_LIST="$RED_LIST  $name (exit=$cur_rc)\n"
  fi

  # ── Q1：注入违规后必须红（非空门证明） ──
  #
  # ⚠️ 语义（2026-09-29 修正，本门第一版把这里写反了）：
  #   **门变红 = 探针成功**。门返回 0 反而说明「注入的违规没被抓住」⇒ 门是空的。
  # 第一版用 `if ! eval ...; then PROBE_FAILED` 判断，等于把「门抓到了违规」
  # 记成失败 —— 结果 7 道门全部报 probe failed，而它们的门其实是对的。
  # ⇒ 改为比对**期望 rc**。
  probe=$(reg_field "$gate" 3)
  want_rc=$(reg_field "$gate" 4)
  [ -n "$probe" ] && [ "$probe" != "—" ] || probe="{GATE} --strict"
  probe_cmd=${probe//\{GATE\}/$gate}
  [ -n "$want_rc" ] && [ "$want_rc" != "—" ] || want_rc=1

  # 超时：cargo 型门需 >3min（实测 check-test-baseline 单独跑 174s）
  eval "$probe_cmd" >/dev/null 2>&1
  got_rc=$?
  if [ "$got_rc" -ne "$want_rc" ]; then
    # 返回 0 = 门没抓住注入的违规 ⇒ 假门
    # 返回 124 = 超时 ⇒ 探针无法在合理时间内判定
    PROBE_FAILED=$((PROBE_FAILED+1))
    FAIL_LIST="$FAIL_LIST  $name (want rc=$want_rc, got $got_rc ← 0=假门 124=超时)\n"
  else
    ALWAYS_GREEN_PROVEN=$((ALWAYS_GREEN_PROVEN+1))
  fi
done

echo "  --strict 门总数:        $TOTAL"
echo "  已登记:                $REGISTERED"
echo "  未登记（缺证据）:      $UNREGISTERED"
echo "  opaque（声明不可注入）:$OPAQUE"
echo "  本门自身（自检，不探测）:$SELF_SKIP"
echo "  ⛔ 当前恒红:            $ALWAYS_RED"
echo "  探针执行失败:          $PROBE_FAILED"

if [ -n "$UNREG_LIST" ]; then
  echo
  echo "--- 未登记的门（不是失败，是**缺审**）---"
  printf "%b" "$UNREG_LIST"
  echo "  ⇒ 在 $REG 加一行即可。格式："
  echo "    <gate><TAB>injectable|opaque<TAB>{GATE} --strict<TAB><rc><TAB><note>"
fi

if [ -n "$RED_LIST" ]; then
  echo
  echo "--- ⛔ 恒红的门（比没有门更坏：它训练人忽略红色）---"
  printf "%b" "$RED_LIST"
  echo "  ⇒ 两种可能，必须二选一："
  echo "    ① 门真的坏了        → 修门"
  echo "    ② 存量债已记账      → 把存量搬进 baseline 账本，--strict 只挡新增"
  echo "  ⛔ 绝不要为了让门变绿而**调大 baseline**或**删掉 --strict**。"
  echo "     那是把症状变成谎言 —— 见 N-1（2026-09-29 拆掉的 5 个幻影门）。"
fi

if [ -n "$FAIL_LIST" ]; then
  echo
  echo "--- 探针自身失败（门有效性未获证）---"
  printf "%b" "$FAIL_LIST"
fi

if [ "$STRICT" -eq 1 ] && { [ "$UNREGISTERED" -gt 0 ] || [ "$ALWAYS_RED" -gt 0 ] || [ "$PROBE_FAILED" -gt 0 ]; }; then
  echo
  echo "FAIL(strict): $((UNREGISTERED + ALWAYS_RED + PROBE_FAILED)) 门不可信。"
  echo "  未登记=$UNREGISTERED 恒红=$ALWAYS_RED 探针失败=$PROBE_FAILED"
  exit 1
fi

echo "DONE(advisory)."
