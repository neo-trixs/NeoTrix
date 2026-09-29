#!/usr/bin/env bash
# check-commit-deletions.sh — 删除声明门
#
# ## 要防的具体事故（2026-09-29 实测，本仓真实发生过）
#
# 多个 agent 窗口共享同一个工作树，也就**共享同一个 index**。于是
# `git add <显式文件> && git commit` 有一个致命性质：
#   显式 add 只约束「我 add 什么」，**约束不了「暂存区里已经有什么」**。
# 事故实录：我在 `343a346d` 只 add 了自己的 5 个文件，提交里却带上了
# 另一窗口暂存区里的 `D scripts/ops/nt_evolution_exp.py` —— 339 行、
# 那是那扇窗刚启动的进化实验 CLI、并被文档标注为「流程已启动」。
# 从那扇窗看：删除随我的提交落账、working tree 不再提示 pending，
# 等于**凭空消失且无迹可寻**。
#
# ## 为什么能机械地防住
#
# 我无法在事后区分「谁 stage 的这个改动」—— git 只记录最终 tree。
# 但**那类事故的签名是确定的：提交里有删除**。删除的爆炸半径远大于普通改动
# （本次 339 行、且是他人在途工作），所以：
#   **任何 staged 删除都必须在 commit message 里显式声明。**
# 声明这个动作本身就是防护 —— 写不出 `DELETION-INTENT:` 就必须去看一眼
# 到底删了什么。
#
# ## 规则
#
# 若 `git diff --cached --name-status` 中存在 `D` 项，则 commit message
# 必须对**每一个**被删路径含一行：
#     DELETION-INTENT: <path>
# 否则 exit 1。
#
# ## 边界
#
# · 只看 `--cached`（本次提交内容），不扫历史。
# · 纯新增/修改的提交成本为 0（无删除即直接 PASS），不拖慢正常路径。
# · 绕过需要 `--no-verify`，与仓库既有 P0 门同一处置口径。
# · 路径含空格/制表符时用 NUL 分隔处理，不做字面拆分。
set -uo pipefail

ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || {
  echo "[deletions] ERROR: 不在 git 仓库内"; exit 2
}
cd "$ROOT" || exit 2

# commit message 来源：pre-commit 时 git 把它作为 $1 传入；退回到 COMMIT_EDITMSG。
MSG_FILE="${1:-}"
if [ -z "$MSG_FILE" ] || [ ! -f "$MSG_FILE" ]; then
  MSG_FILE="$(git rev-parse --git-path COMMIT_EDITMSG)"
fi
[ -f "$MSG_FILE" ] || MSG_FILE=/dev/null

# 用 NUL 分隔拿 name-status，路径含空格也不会被拆坏。
deletions=$(
  git diff --cached --name-status -z --diff-filter=D 2>/dev/null \
  | tr '\0' '\n' | awk 'NF { sub(/^D\t/, ""); sub(/^D/, ""); print }'
)

if [ -z "$deletions" ]; then
  echo "[deletions] PASS: 本次提交无删除，无需声明。"
  exit 0
fi

n=$(printf '%s\n' "$deletions" | grep -c .)
echo "[deletions] 本次提交删除 $n 个文件："
printf '%s\n' "$deletions" | sed 's/^/    /'

missing=""
while IFS= read -r p; do
  [ -n "$p" ] || continue
  # 用 awk 做**精确字段比较**，不用 grep + 正则。
  # ⚠️ 曾经的 bug：`${p//./\\.}` 转义后，含数字的路径（如 D2551_xxx.md）会让
  #    grep 报 "invalid backreference number" —— 反斜杠+数字被当成反向引用。
  # 声明格式固定为 `DELETION-INTENT: <path>`，取第 2 个字段精确比对即可。
  if ! awk -v want="$p" '
      $1 == "DELETION-INTENT:" { if ($2 == want) { found = 1 } }
      END { exit(found ? 0 : 1) }
    ' "$MSG_FILE" 2>/dev/null; then
    missing="$missing$p"$'\n'
  fi
done <<EOF
$deletions
EOF

if [ -n "$missing" ]; then
  echo ""
  echo "❌ [deletions] 下列删除未在 commit message 中声明："
  printf '%s' "$missing" | sed 's/^/    /'
  echo ""
  echo "   每次删除都必须在 commit message 里加一行（可带原因注释）："
  while IFS= read -r p; do
    [ -n "$p" ] || continue
    echo "     DELETION-INTENT: $p"
  done <<EOF
$deletions
EOF
  echo ""
  echo "   为什么：共享工作树 ⇒ 共享 index。显式 git add 约束不了"
  echo "   「暂存区里已有什么」，删除可能来自**别的窗口**。"
  echo "   2026-09-29 实测事故：误删他窗 339 行在途工作，见本脚本头注。"
  exit 1
fi

echo "[deletions] PASS: 全部 $n 个删除均已声明。"
exit 0
