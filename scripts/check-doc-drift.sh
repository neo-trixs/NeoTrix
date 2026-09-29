#!/bin/bash
# Doc-drift check — R-P232（2026-09-29 改：常量 baseline → 账本棘轮）
#
# 校验每个 nt_*.rs 模块文件（排除 mod.rs / lib.rs / main.rs）在前 3 行内
# 带内层模块文档（//!）。
#
# ## 为什么从「常量 baseline」改成「账本棘轮」（2026-09-29）
#
# 旧实现把基线写死在**注释**里：
#
#     echo "Files missing //! module docs: $COUNT (baseline 111 on 2026-09-21)"
#     if [ "$STRICT" -eq 1 ] && [ "$COUNT" -gt 0 ]; then exit 1; fi
#
# 后果：`COUNT > 0` 恒真 ⇒ **`--strict` 从设计之日起就恒红**（实测 127）。
# 而它同时**已接进 pre-commit** ⇒ 每次提交都报红。
#
# 这是 `awesome-dsh-plugin/.github/workflows/pr-gate.yml` 记的那类病：
#   "A gate that dies before posting is indistinguishable from one that never
#    needed to run."
# **恒红的门比没有门更坏 —— 它训练人忽略红色。**
#
# ⇒ 改成与 `check-truth-surface.sh` / `check-layer-deps.sh` 一致的账本棘轮：
#   - 存量进 `scripts/doc-drift-baseline.txt`（**文件列表，不是数字**）
#   - `--strict` 只拦**新增**
#   - 修好一个就 `--update-baseline` 棘轮下降
#
# ⛔ **绝不要为了让门变绿而调大 baseline。** 那是把症状变成谎言。
#    参照 N-1（2026-09-29 拆掉的 5 个恒红幻影门）。
#
# 用法：
#   bash scripts/check-doc-drift.sh                  # advisory, exit 0, 打印计数
#   bash scripts/check-doc-drift.sh --strict         # 仅当有**新增**未文档化文件时 exit 1
#   bash scripts/check-doc-drift.sh --update-baseline# 把当前存量写入账本（棘轮）
#
# Note: bash-3.2-safe style (macOS system bash). `bash -n` before commit.

set -uo pipefail

SRC="neotrix-core/src"
BASELINE="scripts/doc-drift-baseline.txt"
STRICT=0
UPDATE=0
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    --update-baseline) UPDATE=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

CUR=$(mktemp); NEW=$(mktemp); GONE=$(mktemp); BASE_C=$(mktemp)
trap 'rm -f "$CUR" "$NEW" "$GONE" "$BASE_C"' EXIT

# 收集缺 module doc 的文件（**排序**，保证同一状态同一列表）
rg --files -g 'nt_*.rs' -g '!mod.rs' -g '!lib.rs' -g '!main.rs' "$SRC" 2>/dev/null | sort > "$CUR.all"
: > "$CUR"
while IFS= read -r f; do
  [ -f "$f" ] || continue
  if ! head -n 3 "$f" | rg -q '^//!'; then
    echo "$f" >> "$CUR"
  fi
done < "$CUR.all"
rm -f "$CUR.all"
sort -u "$CUR" -o "$CUR"

TOTAL=$(grep -c . "$CUR" 2>/dev/null); TOTAL=${TOTAL:-0}

if [ "$UPDATE" -eq 1 ]; then
  if [ -f "$BASELINE" ]; then
    grep '^#' "$BASELINE" > "$BASELINE.tmp" || : > "$BASELINE.tmp"
  else
    : > "$BASELINE.tmp"
  fi
  cat "$BASELINE.tmp" "$CUR" > "$BASELINE"
  rm -f "$BASELINE.tmp"
  echo "baseline updated: $BASELINE now has $(grep -vc '^#' "$BASELINE") entries"
  exit 0
fi

# 与账本求差
if [ -f "$BASELINE" ]; then
  grep -v '^#' "$BASELINE" 2>/dev/null | sort -u > "$BASE_C" || : > "$BASE_C"
fi
comm -23 "$CUR" "$BASE_C" > "$NEW"    # 在树里、不在账本 => 新增
comm -13 "$CUR" "$BASE_C" > "$GONE"   # 在账本、树里已修好 => 已解决

N_NEW=$(grep -c . "$NEW" 2>/dev/null); N_NEW=${N_NEW:-0}
N_GONE=$(grep -c . "$GONE" 2>/dev/null); N_GONE=${N_GONE:-0}
N_BASE=$(grep -vc '^#' "$BASELINE" 2>/dev/null); N_BASE=${N_BASE:-0}

echo "=== NeoTrix doc-drift check (R-P232) ==="
echo "missing //! module docs: $TOTAL   ledger: $N_BASE   resolved since ledger: $N_GONE"
echo "NEW (not in ledger): $N_NEW"

if [ "$N_GONE" -gt 0 ]; then
  echo "--- resolved (drop from ledger via --update-baseline) ---"
  head -n 10 "$GONE"
  [ "$N_GONE" -gt 10 ] && echo "  ... and $((N_GONE-10)) more"
fi

if [ "$N_NEW" -gt 0 ]; then
  echo "--- NEW offenders (regression, not in ledger) ---"
  cat "$NEW"
fi

if [ "$STRICT" -eq 1 ] && [ "$N_NEW" -gt 0 ]; then
  echo "FAIL(strict): $N_NEW new undocumented file(s). Ledger holds $N_BASE standing."
  echo "  Fix: add a //! module doc, or accept it into the ledger via --update-baseline"
  echo "  (only if the file is genuinely archival — a new file with no doc is a real gap)."
  exit 1
fi

echo "DONE(advisory)."
