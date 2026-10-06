#!/usr/bin/env bash
# ⛔ 只读。本脚本**绝不**写/删仓库任何文件（只读 `git status` / `du` / `find`）。
#
# 门：交接文档与「无界文件」卫生。2026-10-06 建。
#
# ## 为什么要有这个门（不是「再来一遍人工盘点」）
#
# 「梳理所有交接文档 + 清理垃圾与无限文件」这个请求在 2026-10-06 **被提了三遍**。
# 人工做的话每轮都是全量取证（sessions 全扫 + 磁盘全盘 du），慢且容易漏。
# 更糟的是：**我连续两轮把自己的新交接漏在索引之外** ——
# 因为习惯是「先更新索引、后写自己的交接」，于是自己那份永远晚一步。
# ⇒ 索引与实际文件数的**一致性**必须机械化判定，不能靠记性。

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

# ⛔ 不用 `2>/dev/null` 吞 stderr —— 零命中要先确认退出码是 1 而非 2
#（见 AGENTS.md §4.2：假阴性比没判据更坏）。
RED=$'\033[31m'; GRN=$'\033[32m'; YEL=$'\033[33m'; RST=$'\033[0m'
fail=0
note() { printf '%s\n' "$*"; }
bad()  { fail=1; printf '%s FAIL:%s %s\n' "$RED" "$RST" "$*"; }
warn() { printf '%s WARN:%s %s\n' "$YEL" "$RST" "$*"; }
ok()   { printf '%s  OK:%s %s\n' "$GRN" "$RST" "$*"; }

note "══════ 交接文档卫生门 ══════"

# ── 判据 1：sessions/*.md 与 README 索引必须逐份对齐（本门的主判据）──────
if [ ! -f sessions/README.md ]; then
  bad "缺 sessions/README.md（索引入口）"
else
  actual_list=$(mktemp); listed_list=$(mktemp)
  trap 'rm -f "$actual_list" "$listed_list"' EXIT
  find sessions -maxdepth 1 -name '*.md' \
    ! -name 'README.md' ! -name 'OPEN-DEFECTS.md' -exec basename {} \; \
    | LC_ALL=C sort > "$actual_list"
  # 索引里的条目：表格行首为 `| \`handoff-xxx.md\`` 或 `| \`HANDOFF-xxx.md\``
  grep -oE '^\| `[A-Za-z0-9_.-]+\.md`' sessions/README.md \
    | sed -E 's/^\| `([^`]+)`$/\1/' | LC_ALL=C sort -u > "$listed_list"
  a=$(wc -l < "$actual_list" | tr -d ' '); l=$(wc -l < "$listed_list" | tr -d ' ')
  if [ "$a" != "$l" ]; then
    bad "交接文档数($a) ≠ 索引条目数($l)"
    note "  仅在磁盘上（索引漏了，会导致「刚写的交接自己查不到」）："
    comm -23 "$actual_list" "$listed_list" | sed 's/^/    + /'
    note "  仅在索引里（文件已不在）："
    comm -13 "$actual_list" "$listed_list" | sed 's/^/    - /'
  else
    ok "交接文档 $a 份，与索引逐份对齐"
  fi
fi

# ── 判据 2：OPEN-DEFECTS.md 必须存在（缺陷单一入口）────────────────────
[ -f sessions/OPEN-DEFECTS.md ] \
  && ok "OPEN-DEFECTS.md 在（缺陷单一入口）" \
  || bad "缺 sessions/OPEN-DEFECTS.md ⇒ 缺陷会退回散落正文、随窗口沉底"

# ── 判据 3：无界增长：target/debug/incremental ──────────────────────────
# 这条**只报阈值**不当阻断（照「演进指标不当阻断」的纪律）：它会随正常编辑
# 增长，硬阻断会让门被关掉。
if [ -d target/debug/incremental ]; then
  inc=$(du -sm target/debug/incremental 2>/dev/null | cut -f1)
  tot=$(du -sm target 2>/dev/null | cut -f1)
  if [ "${inc:-0}" -gt 8192 ]; then
    warn "target/debug/incremental = ${inc}M（占 target ${tot}M 的 $((${inc:-0}*100/${tot:-1}))%）"
    note "      清理方式（比 cargo clean 正确，保留依赖产物）：rm -rf target/debug/incremental"
    note "      实测代价：仅重编 workspace crate，依赖不重编（2026-10-06 实测 10.17s）"
  else
    ok "target/debug/incremental = ${inc}M（阈值 8192M 内）"
  fi
else
  ok "target/debug/incremental 不存在"
fi

# ── 判据 4：遗留备份文件（.bak/.orig/.rej/~）──────────────────────────
# ⛔ 排除 `models/`（AGENTS.md：gitignored、git 保护不到、删了只能重下）
#    与 `.neotrix/patches/`（他窗 R-DISK-5 兜底）。
junk=$(find . -maxdepth 5 \( -name target -o -name node_modules -o -name .git \
        -o -name models -o -path './.neotrix/patches' \) -prune \
     -o -type f \( -name '*.bak' -o -name '*.orig' -o -name '*.rej' -o -name '*~' \
                 -o -name '*.bak-legacy' \) -print 2>/dev/null | LC_ALL=C sort)
if [ -n "$junk" ]; then
  warn "遗留备份文件（多为垃圾，但**逐一确认后再删**）："
  printf '%s\n' "$junk" | sed 's/^/    /'
else
  ok "无遗留备份文件（.bak/.orig/.rej/~）"
fi

# ── 判据 5：未提交改动数（只报，不判红：共享树上他窗 WIP 是常态）─────────
dirty=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
if [ "${dirty:-0}" -gt 40 ]; then
  warn "主树 ${dirty} 处未提交（共享树常态；若含你自己的改动，收工前必须入库）"
else
  ok "主树未提交 ${dirty} 处"
fi

note ""
note "══════ 收尾 ══════"
if [ "$fail" -ne 0 ]; then
  printf '%sFAIL%s —— 索引与磁盘不一致，见上面带 FAIL 的行\n' "$RED" "$RST"
  exit 1
fi
printf '%sPASS%s —— 交接索引自洽（判据 3/4/5 为报告项）\n' "$GRN" "$RST"