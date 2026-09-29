#!/usr/bin/env bash
# check-push-deletions.sh — 推送级删除声明兜底
#
# ## 为什么要第二层
#
# `scripts/check-commit-deletions.sh` 挂在 pre-commit，能拦住**走 hook** 的提交。
# 但 `--no-verify`、libgit2、部分 GUI 客户端会绕过 hook —— 本仓历史上已有
# 绕过 P0 编译门的先例（见 .githooks/pre-commit 头注）。
# 既然本门守的是「别把别人的在途工作删进主干」这种**不可逆后果**，
# 就不该只有单点防御：推送前再查一次本地未推送的提交。
#
# ## 检查什么
#
# 对 `upstream..HEAD`（无 upstream 时退化为全部本地提交）里的每个提交，
# 若它有删除的文件，则该提交 message 必须含对应的 `DELETION-INTENT:` 行。
# 任何一条缺失 ⇒ exit 1。
#
# ## 边界
#
# · 只查删除，不查内容（内容正确性属 review 范畴）。
# · 历史提交若未声明（本次门上线前的老提交）**不追溯**——只查将要推送的增量，
#   否则等于要求重写全部历史。
# · 只读，无副作用，不改任何 ref。
set -uo pipefail

ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || {
  echo "[push-deletions] ERROR: 不在 git 仓库内"; exit 2
}
cd "$ROOT" || exit 2

# 无 upstream 时退化：只看当前分支上的本地提交（最近的 30 个封顶，避免
# 在长期无 remote 的仓上把几千个提交全扫一遍）。
# ⛔ 锚点：只查**本门上线之后**的提交，绝不追溯历史。
#
#    初版把范围写成 `HEAD~30..HEAD`，结果一上线就报 50 个「门之前的历史提交
#    未声明删除」⇒ 门从第一天起恒红。而**恒红的门等于没有门** —— 本仓
#    check-layer-deps 当初就是因为「无条件红 92 处既存违规」才改成棘轮式，
#    同一个教训（见 .github/workflows/ci.yml 的注释）。别重复这个错。
#
#    正确做法：锚点 = 本脚本**首次入库**的那笔提交，从它开始算（含它自己）。
#    门还没入库时（开发中自测）没有锚点，此时只做只读报告并 exit 0，
#    不能因为「找不到锚点」就退化成一个必然红的门。
ANCHOR=$(git log --diff-filter=A --format=%H -1 -- scripts/check-push-deletions.sh 2>/dev/null || true)

if [ -z "$ANCHOR" ]; then
  echo "[push-deletions] SKIP: 本脚本尚未入库（无锚点提交），跳过历史检查。"
  echo "[push-deletions]       提交后此门自动生效，起算点 = 该提交本身。"
  exit 0
fi

if ANCHOR_COMMIT=$(git rev-parse -q --verify "$ANCHOR^{commit}" 2>/dev/null); then
  if P=$(git rev-parse -q --verify "$ANCHOR_COMMIT^" 2>/dev/null); then
    RANGE="$P..HEAD"
  else
    RANGE="$ANCHOR_COMMIT..HEAD"   # 根提交：只能查它之后
  fi
  echo "[push-deletions] 起算点：${ANCHOR_COMMIT:0:8}（本门首次入库），不追溯此前历史。"
else
  RANGE="HEAD"
  echo "[push-deletions] 锚点提交不可解析，范围退化为单个 HEAD。"
fi

COMMITS=$(git rev-list "$RANGE" 2>/dev/null || true)
if [ -z "$COMMITS" ]; then
  echo "[push-deletions] PASS: 待推送范围内无提交。"
  exit 0
fi

bad=0
nchecked=0
for c in $COMMITS; do
  del=$(git diff-tree --no-commit-id --name-status -r --diff-filter=D "$c" 2>/dev/null \
        | awk '{ sub(/^D\t/, ""); sub(/^D/, ""); print }' | grep . )
  [ -n "$del" ] || continue
  nchecked=$((nchecked + 1))
  msg=$(git log -1 --format=%B "$c" 2>/dev/null)
  # 字面提取声明（见 check-commit-deletions.sh 头注：不能用正则匹配路径）
  declared=$(printf '%s\n' "$msg" | grep -E '^DELETION-INTENT:[[:space:]]*[^[:space:]]' 2>/dev/null \
    | sed -E 's/^DELETION-INTENT:[[:space:]]*([^[:space:]]+).*$/\1/')
  miss=""
  while IFS= read -r p; do
    [ -n "$p" ] || continue
    printf '%s\n' "$declared" | grep -qxF -- "$p" \
      || miss="$miss    $p"$'\n'
  done <<EOF
$del
EOF
  if [ -n "$miss" ]; then
    echo "[push-deletions] ❌ $(git log -1 --format='%h %s' "$c") 的删除未声明："
    printf '%s' "$miss"
    bad=$((bad + 1))
  fi
done

if [ "$bad" -gt 0 ]; then
  echo ""
  echo "   $bad 个提交含未声明删除，拒绝推送。"
  echo "   补声明的办法（不改历史）：若这些提交尚未推送且你确认删除无误，"
  echo "   可用 git rebase -i 补 message；若是误删，先恢复文件再提交。"
  echo "   详见 scripts/check-commit-deletions.sh 头注的事故记录。"
  exit 1
fi

echo "[push-deletions] PASS: 待推送范围内 $nchecked 个含删除的提交均已声明。"
exit 0
