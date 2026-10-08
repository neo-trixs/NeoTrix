#!/bin/bash
# nt_ 前缀规约检查 — 报告式（默认 exit 0），与 check-doc-drift.sh 同风格。
#
# 背景（2026-09-28）：AGENTS.md 规定「所有模块名用 nt_ 前缀」，但实测
# neotrix-core/src 下 1,646 个 .rs 文件不以 nt_ 开头（排除 mod.rs/lib.rs/main.rs）。
# 规约与现实差 1,646 个文件 ⇒ 规约不产生约束力。
#
# ⚠️ 基线数字的测量台：1,646 是在 `git worktree add --detach HEAD` 的**干净检出**
#    上量的。首版写的 1,644 是在**主工作树（脏，含他窗未提交 .rs）**上量的 ——
#    脏树不是合法测量台。改代码后基线会变，请在干净检出上重测再改此处。
#
# ── 2026-10-08：默认模式由「恒绿」改为**递减棘轮** ──────────────────
# 改前两个模式都不诚实，且都零约束：
#   默认    : COUNT=1612 却打印 `PASS (advisory)` ⇒ **报绿而实际判红**（L8 绿色≠有效）
#   --strict: 失败条件是 `COUNT > 0`，而 COUNT 恒为正 ⇒ **永久恒红** ⇒ 无法接 CI
# ⇒ 两条路都是「门在，但读者学不到任何东西」。
#
# 改后（与本仓 dead-flag-baseline.txt / silent-failure-baseline.txt 同一套机制）：
#   COUNT >  BASELINE ⇒ **新增违规** ⇒ 两个模式都判红 ⇒ 可接 CI，真正有约束力
#   COUNT <= BASELINE ⇒ 无回归 ⇒ exit 0，且输出明写「PASS ≠ 合规」
#   COUNT <  BASELINE ⇒ 有进展 ⇒ 提示下调基线（债务只能单调不增）
# 基线与测法见 scripts/naming-baseline.txt（含干净检出的测量记录）。
#
# ⚠️ 本门密不透风的**唯一缺口**（如实记账，别当它不存在）：
#    「同时上调基线 + 新增文件」这一组合无法与「无变化」区分 ——
#    基线=1613 且新增 1 个文件 ⇒ COUNT=1613，仍判绿（实测 ③）。
#    ⇒ **防线就是基线文件本身的编辑纪律**：基线只许下调，不许上调
#      （下调需附干净检出重测证据，上调需独立裁决记录）。
#      这与 dead-flag/silent-failure 的基线是同一套信任模型。
#
# ⛔ 仍然**不做**批量改名：那是纯 churn，零架构收益，且会掩盖更重要的层归属问题
#    （见 DIR-REMEDY-2026-09-28.md）。本门管的是「别再欠新债」，不是「还清旧债」。
#
# ⚠️ 本门**不能**判定架构正确性：前缀只管「叫什么」，不管「属于哪一层」。
#    `nt_` 命名的文件照样可能是错层的（例：l0_substrate/nt_core_platform/orchestrator.rs
#    是 DIR-AUDIT 认定的第 3 个 orchestrator）。层归属见 .neotrix/layer-map.json。
#
# 用法: bash scripts/check-naming.sh [--strict]
#   两者对**回归**的判定一致（都判红）；--strict 保留给「期望恒 0」的调用方
#   （本仓当前无人这么用，见 gate-registry.tsv 的 opaque 登记）。

set -uo pipefail

SRC="neotrix-core/src"
STRICT=0
case "${1:-}" in
  "")          STRICT=0 ;;
  --strict)    STRICT=1 ;;
  -h|--help)   sed -n '2,20p' "$0"; exit 0 ;;
  *)           echo "unknown arg: $1 (use --strict)" >&2; exit 2 ;;
esac

LIST=$(mktemp)
trap 'rm -f "$LIST"' EXIT

# 与 check-doc-drift.sh 同口径：排除 mod.rs / lib.rs / main.rs
rg --files -g '*.rs' -g '!mod.rs' -g '!lib.rs' -g '!main.rs' "$SRC" \
  | while IFS= read -r f; do
      base=$(basename "$f")
      case "$base" in
        nt_*|mod.rs|lib.rs|main.rs) ;;
        *) echo "$f" ;;
      esac
    done > "$LIST"

COUNT=$(grep -c . "$LIST" 2>/dev/null || echo 0)

BASELINE_FILE="$(dirname "$0")/naming-baseline.txt"
BASELINE=$(grep -vE '^\s*(#|$)' "$BASELINE_FILE" 2>/dev/null | tail -1 | tr -d '[:space:]')
if [ -z "$BASELINE" ] || ! [ "$BASELINE" -eq "$BASELINE" ] 2>/dev/null; then
  echo "⛔ 基线不可用或不是整数：${BASELINE_FILE}（读到 '$BASELINE'）⇒ 拒绝静默放过" >&2
  exit 2
fi

echo "=== NeoTrix nt_ prefix check ==="
echo "Files lacking nt_ prefix under $SRC: $COUNT   (baseline: $BASELINE)"

if [ "$COUNT" -gt 0 ]; then
  echo
  echo "Top 15 offenders:"
  head -15 "$LIST" | sed 's/^/  /'
  echo
  echo "Naming != layering:"
  echo "  layer ownership  -> .neotrix/layer-map.json (gate: check-layer-deps.sh)"
  echo "  contract check   -> scripts/check-api-surface.sh"
  echo "  doc coverage     -> scripts/check-doc-drift.sh"
fi

# ── 棘轮判定 ─────────────────────────────────────────────
if [ "$COUNT" -gt "$BASELINE" ]; then
  DELTA=$((COUNT - BASELINE))
  echo
  echo "⛔ 新增 $DELTA 个无 nt_ 前缀的 .rs（$COUNT > baseline ${BASELINE}）⇒ 判红。"
  echo "   规约本身仍不合规（存量 $COUNT 个），但**不许再欠新债**。"
  echo "   合法出路二选一：① 把文件改名；② 若这是有意为之，先改规约再改基线。"
  exit 1
fi

if [ "$COUNT" -lt "$BASELINE" ]; then
  echo
  echo "✅ 存量从 baseline $BASELINE 降到 ${COUNT}（-$((BASELINE - COUNT))）。"
  echo "   ⬇ 请在干净检出上重测后下调 scripts/naming-baseline.txt，让棘轮跟着收紧。"
  exit 0
fi

echo
echo "BASELINE: 无回归（$COUNT == baseline ${BASELINE}）。"
echo "⛔ PASS ≠ 合规：本门当前**不允许新增**，但存量 $COUNT 个仍未改名。"
exit 0
