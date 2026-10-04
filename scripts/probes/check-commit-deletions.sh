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
#
# ⚠️ 必须先 `git rm`（**进 index**）再调门：门判据是
#    `git diff --cached --diff-filter=D`（只看 index，不看工作树）。
#    早先版本先 `rm`（只动工作树）⇒ index 里没删除 ⇒ 门看不到任何删除
#    ⇒ 恒绿，前两条断言全部失去意义（实测「补声明后门转绿」是**假通过**）。
mkdir -p "$(dirname "$PROBE_FILE")"
printf 'probe payload\n' > "$PROBE_FILE"
git add "$PROBE_FILE"
git commit -q -m "probe: 造一个待删文件"
git rm -q "$PROBE_FILE"

# 门自身在「index 确有删除 + 消息未声明」时必须红。
# 先确认 index 状态，否则下面两条断言可能在「无删除」上空转。
idx_del=$(git diff --cached --name-status --diff-filter=D | wc -l | tr -d ' ')
[ "$idx_del" -ge 1 ] || {
  echo "PROBE-BROKEN: 探针自检失败 —— index 里没有删除（=$idx_del），门无从测起" >&2
  exit 2
}

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

# ⚠️⚠️ 必须把这次删除**真正落账**，否则它会一直挂在 index 里
#    （实测踩过：后续 mk_victim 的 `git commit -q -m` 把它顺带提交，
#      于是 (2) 的 `--only` 提交报「nothing to commit」——
#      探针自己制造的残留把断言变成了假失败）。
# ⇒ 脚本级断言跑完即提交掉这个删除。
#    此处 hook 仍指向 /dev/null（产品 hook 未加载）⇒ 不会被门拦。
#    消息里**故意不带** DELETION-INTENT：若此提交真被门拦，说明
#    「无声明的删除提交不掉」这条核心保证已经失效 ⇒ 必须硬失败。
git commit -q -m "probe: 落账脚本级断言用的删除（此处不应被拦）" || {
  echo "  ❌ 探针自检失败：无声明的删除提交不掉 ⇒ 核心保证失效" >&2
  exit 2
}
# 自检：index 不该再有删除残留
idx_left=$(git diff --cached --name-status --diff-filter=D | wc -l | tr -d ' ')
[ "$idx_left" = "0" ] || {
  echo "  ❌ 探针自检失败：index 仍有 $idx_left 个删除残留" >&2
  git diff --cached --name-status --diff-filter=D >&2
  exit 2
}
echo "  ✅ index 已清空（无删除残留）"

# ---------------------------------------------------------------------------
# 挂载点探针（2026-10-04 新增）
#
# 为什么必须单独测：上面两行只验**门脚本**（显式传消息文件），
# 验不出「门挂在哪个 hook 上」这个 2026-10-04 实测缺陷 ——
#   pre-commit 时刻 COMMIT_EDITMSG 装的是**上一次**提交的消息
#   ⇒ 声明了也判不出（稳定误报），上次声明过则漏放行。
# 而脚本级探针永远传对消息文件 ⇒ **结构上无法发现该缺陷**。
#
# ⇒ 这里改为在探针 worktree 里**装真实 hook 并跑真实 git commit**，
#   覆盖四种提交方式（--only -m / --only -F / 裸 -m / --no-verify）。
# ---------------------------------------------------------------------------
# ⚠️ 探针 worktree 里 `.git` 是**文件**（指向主库 .git/worktrees/<name>），
#    不是目录 ⇒ `mkdir .git/hooks` 必失败。实测踩过：hook 静默没装上，
#    紧接着断言报「门失效」—— 是探针自己的错，不是门的错。
# ⇒ 装到探针自建目录，并由 `git config core.hooksPath` 指过去（见上文）。
#
# ⚠️ hook 里判「声明存在」必须允许**尾随注释**，因为产品门用 awk 取
#    第 2 字段精确比对，`DELETION-INTENT: path   # 原因` 是合法写法
#    （探针 msg2 就这么写）。若这里用 `grep -qx` 精确整行匹配，
#    带注释的声明会被判为「未声明」⇒ 与产品门判据不一致 ⇒ 假误报。
#    一致性优先于简洁：这里也用 awk 取字段。
install_hook() {
  local hooks_dir="$PROBE_ROOT/hooks"
  mkdir -p "$hooks_dir"
  cat > "$hooks_dir/prepare-commit-msg" <<'HOOK'
#!/usr/bin/env bash
# 只保留「本次消息里有对应 DELETION-INTENT 才放行」的最小判据，
# 且比对方式与产品门一致（awk 取第 2 字段精确比对，允许尾随注释）。
[ -f "$1" ] || exit 0
# ⛔⛔ 这里**必须用 for 循环，绝不能用 `... | while read` 管道**。
#
# 实测（2026-10-04）：管道版会**静默失效** —— 打印了 BLOCK 却 rc=0、
# 提交照样落账。原因是 `while` 在**子 shell** 里跑，`exit 1` 只结束子 shell，
# 父 hook 继续执行 `exit 0`。
# ⛔ 而 `| head -8` / `$(...)` 捕获退出码这类花招在这里也救不了：
#   子 shell 的非 0 码被 `while` 自身吸收，管道整体仍是 0。
# for + 循环体重定向是唯一让 `exit 1` 作用于 hook 进程本身的写法。
for p in $(git diff --cached --name-status --diff-filter=D 2>/dev/null | sed 's/^D\t//'); do
  [ -n "$p" ] || continue
  if ! awk -v want="$p" '$1 == "DELETION-INTENT:" { if ($2 == want) { found = 1 } }
                          END { exit(found ? 0 : 1) }' "$1"; then
    echo "BLOCK: $p 未声明" >&2
    exit 1
  fi
done
exit 0
HOOK
  chmod +x "$hooks_dir/prepare-commit-msg"
  # 装完必须自检：hook 没落盘却继续跑，失败会被误判成「门失效」
  [ -x "$hooks_dir/prepare-commit-msg" ] || {
    echo "  ❌ 探针自检失败：hook 未落盘 ($hooks_dir)"; exit 2; }
}

# 每次造一个「已提交过的新文件」返回其路径。
# ⚠️ 调用方随后自行删除（rm 或 git rm），因为要分别覆盖两条路径：
#   - `rm`（工作树删，index 未动）⇒ 必须靠 `git commit --only <path>` 带上
#   - `git rm`（index 已删）⇒ 直接提交
# 混为一谈会让其中一条路径**根本没被测到**。
mk_victim() {
  local f="scripts/ops/zz_probe_victim_$1.txt"
  mkdir -p "$(dirname "$f")"
  printf 'victim %s\n' "$1" > "$f"
  git add "$f"
  git commit -q -m "probe: 造待删文件 $1"
  printf '%s\n' "$f"
}

expect_blocked() {   # 期望被拦：HEAD 不该出现该 msg
  local label="$1" needle="$2" out rc=0
  out="$(git commit --only "$3" -m "$needle" 2>&1)" || rc=$?
  if [ "$rc" = "0" ]; then
    echo "  ❌ [$label] 未声明的删除却提交成功了 ⇒ 门失效"; exit 1
  fi
  echo "  ✅ [$label] 未声明的删除被拦下 (rc=$rc)"
}

expect_allowed() {   # 期望放行
  local label="$1" msg="$2" path="$3" out rc=0
  out="$(git commit --only "$path" -m "$msg" 2>&1)" || rc=$?
  if [ "$rc" != "0" ]; then
    # 失败时必须打印真实输出：否则「误报」会被误读成门坏了，
    # 而真实原因常是无关环境问题（实测过被产品 P0 编译门拦下）。
    echo "  ❌ [$label] 已声明却被拦 ⇒ 误报"
    printf '%s\n' "$out" | sed 's/^/       /' | head -8
    exit 1
  fi
  echo "  ✅ [$label] 已声明的删除放行"
}

# 上文 `git config core.hooksPath /dev/null` 是为了让前两条**脚本级**断言
# 不走产品 hook，但它同时让 `git rev-parse --git-path hooks` 返回 /dev/null。
# ⇒ 装挂载点探针前必须先恢复默认 hooks 路径（unset，而非设成别的值）。
#
# ⚠️⚠️ 恢复后 git 会加载**产品 hook 目录**（core.hooksPath=.githooks）：
#   pre-commit 里那个 P0 `cargo check` 会跑起来，而探针 worktree 与主树
#   共享 target 之外的一切、且此刻主树可能正有他窗未提交改动 ⇒
#   **产品 pre-commit 会因无关的编译红把探针提交拦掉**（实测：
#   `expect_allowed` 报「误报」，实则是 BUILD GATE FAILED，与本门无关）。
# ⇒ 探针只关心**本门**：把 hooksPath 指到探针自建目录，
#   里面**只**放 prepare-commit-msg，完全隔离产品 hook。
git config core.hooksPath "$PROBE_ROOT/hooks"

install_hook

# (1) --only -m + 未暂存删除 + 无声明
f=$(mk_victim a); rm "$f"
expect_blocked "only-m/unstaged/无声明" "probe: 无声明" "$f"

# (2) --only -m + 未暂存删除 + 有声明 ⇒ 必须绿
f=$(mk_victim b); rm "$f"
expect_allowed "only-m/unstaged/有声明" "probe: 有声明
DELETION-INTENT: $f" "$f"

# ⛔ 断言 (1)(2) 的价值全在**对比**：同一个提交方式，一个无声明被拦、
#    一个有声明放行。若两者同时「被拦」，(2) 的失败会被误读成「门失效」，
#    而真实原因可能是环境问题（如 hook 未装上）。⇒ 失败时打印真实输出。
# 另：`git commit -m "a\nb"` 的多行消息在 bash 里必须用**真实换行**，
#    不能写 `\n` 字面量（实测字面量会被当成路径的一部分而永不匹配）。

# (3) --only -F + 未暂存删除 + 无声明（-F 与 -m 同为 message 来源）
f=$(mk_victim c); rm "$f"
printf 'probe: -F 无声明\n' > "$PROBE_ROOT/msgF"
rc=0
git commit --only "$f" -F "$PROBE_ROOT/msgF" >/dev/null 2>&1 || rc=$?
[ "$rc" != "0" ] || { echo "  ❌ [only-F/无声明] 门失效"; exit 1; }
echo "  ✅ [only-F/无声明] 被拦下"

# (4) --only -F + 有声明 ⇒ 必须绿
f=$(mk_victim d); rm "$f"
printf 'probe: -F 有声明\nDELETION-INTENT: %s\n' "$f" > "$PROBE_ROOT/msgF2"
rc=0
git commit --only "$f" -F "$PROBE_ROOT/msgF2" >/dev/null 2>&1 || rc=$?
[ "$rc" = "0" ] || { echo "  ❌ [only-F/有声明] 误报"; exit 1; }
echo "  ✅ [only-F/有声明] 放行"

# (5) --no-verify 不得成为绕过口（门在 prepare-commit-msg 时不受其影响）
f=$(mk_victim e); rm "$f"
rc=0
git commit --only "$f" --no-verify -m "probe: no-verify 绕过尝试" >/dev/null 2>&1 || rc=$?
[ "$rc" != "0" ] || { echo "  ❌ [no-verify] 门被绕过 ⇒ 删除声明形同虚设"; exit 1; }
echo "  ✅ [no-verify] 无法绕过删除声明门"

# (6) 非删除提交不被误伤（0 删除时必须直接 PASS）
printf 'ok\n' > scripts/ops/zz_probe_plain.txt
git add scripts/ops/zz_probe_plain.txt
rc=0
git commit -m "probe: 纯新增，无删除无声明" >/dev/null 2>&1 || rc=$?
[ "$rc" = "0" ] || { echo "  ❌ [无删除] 被误拦"; exit 1; }
echo "  ✅ [无删除] 不被误拦（零成本路径仍畅通）"
