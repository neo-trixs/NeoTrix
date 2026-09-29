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
# ## 第二类：根文档死链（2026-09-29 新增）
#
# 原实现只扫 `neotrix-core/src` 的 `nt_*.rs` 模块文档 ⇒ **根目录 13 个 .md/.toml
# 完全在门外**。而根文档互引极密（AGENTS.md 一个文件就是 94 处引用的中枢），
# 一份被删/被改名的文档会让下一个 agent 拿着死链去查。
#
# 扫法：抽根文档正文里的仓库内路径（`docs/…` `scripts/…` `.neotrix/…` 等），
# 逐个 `-e` 验证存在。⛔ 不扫 http(s) 链接（本门无网络）。
#
# ⚠️ **两种提及必须区分**（2026-09-29 实测踩到）：
#   · 「指示去读」—— 路径该存在，不存在 = 坏链 ⇒ 报。
#   · 「告知已删」—— 路径**故意**不存在。`TODO.md` 的「已废止」清单、
#     `RUST-STANDANCES.md` 的「已完全删除」记录、`CONTRIBUTING.md` 对一条
#     失效命令的说明，都是这类。它们若被门判红，会逼人去删真实历史 ——
#     **恒红的门比没有门更坏**（它训练人忽略红色）。
# ⇒ 故跳过命中「已删/已废止/已归档/已失效/已消失/不要再」语境的行。
# 实测 17 条初始命中全部属此类，跳过后归 0。
#
# Note: bash-3.2-safe style (macOS system bash). `bash -n` before commit.

set -uo pipefail

SRC="neotrix-core/src"
BASELINE="scripts/doc-drift-baseline.txt"
ROOTDOCS="${ROOTDOCS:-AGENTS.md README.md DOCUMENTATION-MAP.md CONTRIBUTING.md RUST-STANDARDS.md TODO.md}"
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

# ─────────────────────────────────────────────────────────────
# 第二类：根文档死链（2026-09-29 新增）
# ─────────────────────────────────────────────────────────────
LINKTMP=$(mktemp)
trap 'rm -f "$CUR" "$NEW" "$GONE" "$BASE_C" "$LINKTMP"' EXIT

# 只取形如 `path/with.ext` 或 `dir/` 的反引号片段；排除 URL、绝对路径、glob
for d in $ROOTDOCS; do
  [ -f "$d" ] || { echo "root-doc-missing: $d"; continue; }
  # 抽出反引号内 / 行内 code span 中以已知仓库前缀开头的路径 token
  rg -o '\b(docs|scripts|crates|skills|sessions|config|models|neotrix-core|src-tauri|\.neotrix|\.githooks|\.github)/[A-Za-z0-9_./-]+' "$d" 2>/dev/null \
    | sed 's/[.,;:)]*$//' | sort -u >> "$LINKTMP" || true
done
# 去重并验证存在
[ -f "$LINKTMP" ] && sort -u "$LINKTMP" -o "$LINKTMP"

DEAD=0
SKIPPED=0
if [ -s "$LINKTMP" ]; then
  # DEADTMP 记「该路径在哪些根文档里被当作**指示**提及」
  DEADTMP=$(mktemp)
  trap 'rm -f "$CUR" "$NEW" "$GONE" "$BASE_C" "$LINKTMP" "$DEADTMP"' EXIT
  while IFS= read -r p; do
    [ -n "$p" ] || continue
    [ -e "$p" ] && continue
    # 逐个根文档找它；只要有一处是指示语境就算死链，全是「已删」语境则跳过
    is_dead=0
    for d in $ROOTDOCS; do
      [ -f "$d" ] || continue
      while IFS= read -r line; do
        [ -n "$line" ] || continue
        case "$line" in
          *已删*|*已废止*|*已归档*|*已失效*|*已消失*|*不要再*|*已完全*|*归档*|*不要删*)
            : ;;   # 历史告知语境 → 不计
          *) is_dead=1; break ;;
        esac
      done <<EOF
$(rg -F "$p" "$d" 2>/dev/null | head -5)
EOF
      [ "$is_dead" -eq 1 ] && break
    done
    if [ "$is_dead" -eq 1 ]; then
      echo "$p" >> "$DEADTMP"
    else
      SKIPPED=$((SKIPPED+1))
    fi
  done < "$LINKTMP"
  if [ -s "$DEADTMP" ]; then
    while IFS= read -r p; do echo "root-doc-deadlink: $p"; DEAD=$((DEAD+1)); done < "$DEADTMP"
  fi
  rm -f "$DEADTMP"
fi
echo "root-doc deadlinks: $DEAD  (skipped $SKIPPED intentional '已删' mentions; scanned $(grep -c . "$LINKTMP" 2>/dev/null || echo 0) path refs across: $ROOTDOCS)"

if [ "$STRICT" -eq 1 ] && [ "$DEAD" -gt 0 ]; then
  echo "FAIL(strict): $DEAD dead path reference(s) in root docs."
  echo "  These are real paths the docs tell the next agent to read."
  echo "  Either restore the file, or fix the reference."
  exit 1
fi
