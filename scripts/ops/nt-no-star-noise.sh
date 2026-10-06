#!/usr/bin/env bash
# ⛔ 只读。本脚本**绝不**修改任何文件（只 grep + 计数）。
#
# 门：源码注释里的装饰性 ⭐ 不得新增。
#
# ## 为什么需要门（2026-10-06 实测）
#
# 一次批量清理（`854d4131`，123 文件、4526→285）**当天**就被证明不够：
#紧随其后的 4 个提交里又新增了 **21 行**带 ⭐ 的注释，其中一条 commit 标题就是
# `fix(l6/approval): ⭐⭐⭐ 审批绑定内容指纹`。
# ⇒ 只要没有门，下一个窗口会照旧写 ⭐，清理就变成永远还债的猫鼠游戏。
#
# ## 判据形状：只挡「新增」，不挡存量（★ 关键设计）
#
# 本门比的是**改动行**（`git diff -U0` 的 `+` 行），不是文件里 ⭐ 的总数。
# 理由：存量里还有 285 处 ⭐（**全在字符串字面量里**，是有意保留的）
# 与 78 处待下轮清理的注释 ⭐。
# - 若按「文件内总数」判 ⇒ 存量直接让门恒红 ⇒ 门被关 ⇒ 彻底没用。
# - 按「新增行」判 ⇒ 存量不阻断，而**每一条新写的 ⭐ 注释**都会被抓住。
#
# ## 为什么不判「代码里有几个 ⭐」
#
# 本仓实测有 51 处 ⭐ 在**字符串字面量**里（`assert_eq!` 断言消息、
# `eprintln!` 跨行串、原始串），它们不是注释、不是噪声。
# 本门只看 `+` 行里 **`//` 之后**的部分 ⇒ 字符串里的 ⭐ 一律不误判。
set -euo pipefail

# ⚠️ 支持 `NT_REPO_ROOT` 覆盖：本门自己的单元测试要在**临时仓库**里跑，
#    否则会在真实仓库上误测（2026-10-06 首次自测就踩了这个：门 `cd` 到本仓，
#    于是「测试新仓库」实际测的是本仓未提交改动 ⇒ 三个场景结论全错）。
ROOT="${NT_REPO_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$ROOT"
REF="${1:-HEAD}"

RED=$'\033[31m'; GRN=$'\033[32m'; RST=$'\033[0m'

# 只扫源码目录；排除 worktrees / node_modules / target / 归档区
# ⛔⛔ `grep` 零命中退 1，而 `set -e` 下它会让**整条管道退 1 ⇒ 脚本静默终止**
#（无任何输出、退出码 1）。本轮实测踩到：「无改动」场景静默 RC=1。
# ⇒ 每个 grep/管道后都显式 `|| true`，让退出码由末尾的判定统一决定。
files=$(git diff "$REF" -U0 -- '*.rs' 2>/dev/null \
        | { grep -E '^\+\+\+ b/' || true; } \
        | sed -E 's|^\+\+\+ b/||' | LC_ALL=C sort -u || true)

if [ -z "$files" ]; then
  printf '%s PASS%s —— 相对 %s 无 .rs 改动\n' "$GRN" "$RST" "$REF"
  exit 0
fi

# 逐文件取「新增行」，只看 `//` 之后的注释部分
hits=""
while IFS= read -r f; do
  [ -f "$f" ] || continue
  case "$f" in
    ./.worktrees/*|*/node_modules/*|*/target/*|./models/*) continue ;;
  esac
  # 该文件的新增行里，注释部分含 ⭐ 的
  found=$(git diff "$REF" -U0 -- "$f" 2>/dev/null \
    | { grep -E '^\+' || true; } \
    | { grep -v '^+++' || true; } \
    | sed -E 's/^\+//' \
    | { grep -E '(//|///|//!).*⭐' || true; })
  # ⚠️ 不可写 `[ -n "$found" ] && ...`：它作为 `while` 末条命令返回 1 时，
  #    配合 `set -e` 会让脚本以 1 退出而**不打印任何东西**（实测 RC=0 但无输出）。
  #    ⇒ 改用 if 块，退出码由末尾统一决定。
  if [ -n "$found" ]; then
    hits="${hits}${f}"$'\n'"$(printf '%s\n' "$found" | sed 's/^/      /')"$'\n'
  fi
done <<< "$files"

if [ -n "$hits" ]; then
  printf '%sFAIL%s —— 注释里新增了装饰性 ⭐：\n' "$RED" "$RST"
  printf '%s' "$hits"
  cat <<'EOF'

  ⛔ 装饰性 ⭐ 是纯噪声（本仓曾一次清理 4526 处）。
  ⛔ 若是**字符串字面量**里的（`assert_eq!` 消息等）⇒ 本门不拦，那是内容不是注释。
  ⛔ 确有必要强调时用 `⛔` 或 `★` 单个标记，不要用 ⭐ 连排。
EOF
  exit 1
fi

printf '%s PASS%s —— 相对 %s 的新增 .rs 行里，注释中无装饰性 ⭐\n' "$GRN" "$RST" "$REF"
exit 0