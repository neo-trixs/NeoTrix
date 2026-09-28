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
# 本门**只让数字可见**，不做阻塞。原因：批量改名是纯 churn，零架构收益，
# 且会掩盖更重要的层归属问题（见 DIR-REMEDY-2026-09-28.md）。
#
# ⚠️ 本门**不能**判定架构正确性：前缀只管「叫什么」，不管「属于哪一层」。
#    `nt_` 命名的文件照样可能是错层的（例：l0_substrate/nt_core_platform/orchestrator.rs
#    是 DIR-AUDIT 认定的第 3 个 orchestrator）。层归属见 .neotrix/layer-map.json。
#
# 用法: bash scripts/check-naming.sh [--strict]
#   默认    : exit 0, 打印计数 + top offenders
#   --strict: 有 offender 时 exit 1（清理到 0 后再接 CI）

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

echo "=== NeoTrix nt_ prefix check (2026-09-28 clean-HEAD baseline: 1646) ==="
echo "Files lacking nt_ prefix under $SRC: $COUNT"

if [ "$COUNT" -gt 0 ]; then
  echo
  echo "Top 15 offenders:"
  head -15 "$LIST" | sed 's/^/  /'
  echo
  echo "This is REPORT-ONLY. Naming != layering:"
  echo "  layer ownership  -> .neotrix/layer-map.json (gate: check-layer-deps.sh)"
  echo "  contract check   -> scripts/check-api-surface.sh"
  echo "  doc coverage     -> scripts/check-doc-drift.sh"
fi

if [ "$STRICT" -eq 1 ] && [ "$COUNT" -gt 0 ]; then
  echo
  echo "STRICT: $COUNT offender(s) remain."
  exit 1
fi
echo
echo "PASS (advisory): naming convention reported, not enforced."
exit 0
