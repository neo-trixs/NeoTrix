#!/bin/bash
# 非空门证明：check-commit-deletions
# 契约：造一个未声明的删除 ⇒ 门红；补声明 ⇒ 门绿；清理干净。
#
# ## 为什么必须自我隔离
#
# 本门操作 index（git rm --cached / 提交）。而主工作树的 index 是**多窗口共享**的。
# 2026-09-29 我在主树跑自测，`git reset -q` 掉的是**别人**的 staged 内容。
# ⇒ 探针只能在自己的一次性 worktree 里跑。
#
# ## 注入对象为什么自造而不用真实文件
#
# 初版拿事故文件 `scripts/ops/nt_evolution_exp.py` 当注入对象，结果它很快被
# 另一窗口的 `2f11389f` 合法替换（Python CLI → Rust bin）⇒ 探针当天就失效。
# 探针依赖仓库某个具体文件存活，就是给自己埋一颗地雷。
# ⇒ 自造文件：探针只依赖 git 本身，不依赖任何仓库文件的历史存亡。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

PROBE_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/nt-delprobe.XXXXXX") || exit 2
PROBE_WT="$PROBE_ROOT/wt"
PROBE_FILE="scripts/ops/zz_probe_delete_me.txt"

cleanup() {
  git worktree remove --force "$PROBE_WT" >/dev/null 2>&1 || true
  git worktree prune >/dev/null 2>&1 || true
  rm -rf "$PROBE_ROOT"
}
trap cleanup EXIT

git worktree add --detach "$PROBE_WT" HEAD >/dev/null 2>&1 || {
  echo "probe: 无法创建探针 worktree"; exit 2
}
cd "$PROBE_WT" || exit 2
git config core.hooksPath /dev/null   # 验门脚本本身，不走产品 hook

# 造一个文件并提交，然后删掉它但**不声明**
mkdir -p "$(dirname "$PROBE_FILE")"
printf 'probe payload\n' > "$PROBE_FILE"
git add "$PROBE_FILE"
git commit -q -m "probe: 造一个待删文件"
git rm -q "$PROBE_FILE"

printf 'probe: 未声明的删除\n' > "$PROBE_ROOT/msg1"
rc=0
bash scripts/check-commit-deletions.sh "$PROBE_ROOT/msg1" >/dev/null 2>&1 || rc=$?
assert_gate_red "check-commit-deletions(未声明)" "$rc"

# 同一删除，补声明 ⇒ 必须绿（验豁免不是「永远红」）
printf 'probe: 已声明的删除\nDELETION-INTENT: %s   # 探针注入\n' "$PROBE_FILE" > "$PROBE_ROOT/msg2"
rc=0
bash scripts/check-commit-deletions.sh "$PROBE_ROOT/msg2" >/dev/null 2>&1 || rc=$?
[ "$rc" = "0" ] || { echo "  ❌ 补声明后门仍红 ⇒ 豁免逻辑坏了"; exit 1; }
echo "  ✅ 补声明后门转绿（豁免生效，非恒红）"
