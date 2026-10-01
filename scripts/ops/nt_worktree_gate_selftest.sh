#!/usr/bin/env bash
# nt_worktree_gate_selftest.sh —— worktree 门（check 子命令）行为自测
#
# ============================ 它测什么 ============================
# 在 /tmp 下用 mktemp -d 造**隔离的临时 git 仓库 + 真实 worktree**，
# 然后**执行真正的** scripts/ops/nt_worktree_gate.sh check，
# 断言它的**真实进程退出码**与**真实 stdout**。8 个用例：
#
#   1 rc=0   干净 worktree（无未提交改动、target 很小）
#   2 rc=3   worktree 干净但 target 累计超阈值（NT_WT_TARGET_MB=1）
#   3 rc=4   worktree 有已跟踪文件的未提交修改，且输出含**具体路径**
#   4 rc=4   ⭐脏 + 体积**同时**超阈值 ⇒ 4（不是 3）
#   5 rc=4   未跟踪新文件也算脏
#   6 rc=0   ⭐主树脏但 worktree 干净 ⇒ 0，且 stdout 必须出现
#              主树未提交处数与「只报告，不影响退出码」
#   7        ⭐cwd 无关性：在 worktree 内部执行 check，被检查集合
#              （"worktree=N 个" + 表格行）**不得含真正的主树**
#   8        ⭐告警措辞契约：rc=3 与 rc=4 时 stdout 必须仍含
#              `worktree=` 与 `未提交改动` 两个子串
#              （.githooks/pre-push:59 依赖它们做 grep；措辞改了会静默失效）
#
# ================== 为什么必须测「可达性」而非逻辑 ==================
# `RUST-STANDARDS.md` §17.7 的 **R-DISK-8** 记录了一次真实事故：
# 有人把判定逻辑抽出来喂 mock 数据，把 0/3/4 三档退出码**全部测对**，
# 却没发现那段代码**根本不可达** —— 前面有一行 `exit 0` 提前返回。
# ⇒ mock 只能证明「这段表达式算得对」，证不了「这行真的被执行」。
# ⛔ 所以本脚本**不 mock 任何判定逻辑**：唯一被测物是真实脚本的真实进程，
#    唯一被断言的是真实 rc + 真实 stdout。任何一段分支被 `exit 0`
#    掐掉、或改成不可达，本脚本都会**变红**。
#
# ============================ 退出码约定 ============================
#   nt_worktree_gate.sh check:
#     0 = 干净
#     3 = **仅** target 累计超阈值（NT_WT_TARGET_MB，默认 1024）
#     4 = 有 worktree 带未提交改动（**含**体积也超阈值的情形）
#     2 = 有 cargo 在跑（**仅 clean 模式**）
#   本自测脚本:
#     0 = 8 个用例全通过
#     1 = 有失败（末尾会打印逐用例 PASS/FAIL 汇总）
#
# ⛔ 只测 check。**绝不执行 clean / prune**（它们会 rm -rf 目录）。
# ⛔ 造脏 worktree 用普通文件（echo 追加），**绝不跑 cargo build**
#    造 target（太慢）；体积一律用 NT_WT_TARGET_MB=1 小阈值控制。
# ⛔ 绝不在真实仓库里造 worktree：全部 fixture 在 mktemp -d 里，退出即删。
#
# ==================== 关于真实仓库的当前预期 rc ====================
# 写这份自测时（2026-10-01），**真实** neotrix 仓库跑
#   `sh scripts/ops/nt_worktree_gate.sh check`
# 的预期退出码是 **4**（确有 2 个带未提交改动的 worktree）。
# ⛔ 但那是**真实仓库的时点状态**，会随别人的窗口变动。
#    **本自测完全不依赖它**：8 个用例全部在临时仓库里自造自销，
#    结论与真实仓库脏不脏、几个 worktree 都无关。

set -u
set -o pipefail
# ⛔ 故意**不** `set -e`：测试脚本要在断言失败时继续跑完剩下的用例，
#    一次性收集全部失败项，而不是第一个失败就哑掉。

# --------------------------------------------------------------------------
# 定位真实被测脚本（同目录）
# --------------------------------------------------------------------------
HERE=$(cd "$(dirname "$0")" && pwd) || exit 1
GATE="$HERE/nt_worktree_gate.sh"
if [ ! -r "$GATE" ]; then
    echo "FATAL: 找不到被测脚本 $GATE" >&2
    exit 1
fi

# ⛔ /tmp 在 macOS 是指向 /private/tmp 的**符号链接**，而 `git rev-parse
#    --show-toplevel` 返回**物理路径** ⇒ 不做物理化的话，下面按路径断言
#    （表格行、具体路径）会因前缀不同而假失败。故统一 physpath 归一。
TMPROOT=$(mktemp -d /tmp/nt_wt_gate_selftest.XXXXXX) || exit 1
TMPROOT=$(cd "$TMPROOT" && pwd -P) || exit 1
physpath() { (cd "$1" 2>/dev/null && pwd -P) || printf '%s\n' "$1"; }
cleanup() { rm -rf "$TMPROOT"; }
trap cleanup EXIT INT TERM

PASS=0
FAIL=0
CASE="?"

banner() {
    CASE="$1"
    printf '\n=== [%s] %s\n' "$1" "$2"
}
ok() {
    PASS=$((PASS + 1))
    printf '  ✅ PASS  %s\n' "$1"
}
bad() {
    FAIL=$((FAIL + 1))
    printf '  ❌ FAIL  %s\n' "$1"
    [ -n "${2:-}" ] && printf '         ---- 实际输出 ----\n' && printf '%s\n' "$2" | sed 's/^/         | /'
    return 0
}
dump() { printf '%s\n' "$1" | sed 's/^/         | /'; }

# 断言 rc
expect_rc() { # $1=期望 rc  $2=实际 rc  $3=说明
    if [ "$1" = "$2" ]; then ok "$3（rc=$2）"; else bad "$3：期望 rc=$1，实得 rc=$2" "$LAST_OUT"; fi
}
# 断言 stdout 含子串
expect_has() { # $1=子串 $2=说明
    if printf '%s\n' "$LAST_OUT" | grep -F -q -- "$1"; then
        ok "$2（含「$1」）"
    else
        bad "$2：stdout 缺子串「$1」" "$LAST_OUT"
    fi
}
expect_hasnt() { # $1=子串 $2=说明
    if printf '%s\n' "$LAST_OUT" | grep -F -q -- "$1"; then
        bad "$2：stdout 不该含子串「$1」，但含了" "$LAST_OUT"
    else
        ok "$2（不含「$1」）"
    fi
}

# --------------------------------------------------------------------------
# fixture 助手：全部落在 $TMPROOT 之下
# --------------------------------------------------------------------------
# make_repo <name> —— 建一个已提交一次的干净仓库，返回其路径
make_repo() {
    _d="$TMPROOT/$1"
    mkdir -p "$_d" || return 1
    git -C "$_d" init -q || return 1
    git -C "$_d" config user.email 'selftest@example.invalid'
    git -C "$_d" config user.name 'wt gate selftest'
    git -C "$_d" config commit.gpgsign false
    printf 'target/\n' > "$_d/.gitignore"
    printf 'seed\n' > "$_d/seed.txt"
    git -C "$_d" add -A || return 1
    git -C "$_d" commit -qm seed || return 1
    printf '%s\n' "$_d"
}

# add_wt <repo> <name> —— 挂一个 detached worktree（路径与主树不同前缀，避免误匹配）
add_wt() {
    _p="$TMPROOT/$2"
    git -C "$1" worktree add --detach -q "$_p" HEAD || return 1
    printf '%s\n' "$_p"
}

# bloat <path> —— 造 ~3MB target（.gitignore 已忽略 target/，不引入脏）
bloat() {
    mkdir -p "$1/target" || return 1
    dd if=/dev/zero of="$1/target/blob.bin" bs=1048576 count=3 2>/dev/null
}

# run_gate <cwd> <target_mb_threshold> —— 跑真实门，LAST_OUT / LAST_RC
run_gate() {
    LAST_OUT=$(cd "$1" && NT_WT_TARGET_MB="$2" sh "$GATE" check 2>&1)
    LAST_RC=$?
    return 0
}

printf 'worktree 门自测开始（被测：%s）\n' "$GATE"
printf '临时沙箱：%s\n' "$TMPROOT"

# ==========================================================================
banner 1 'rc=0：干净 worktree'
R1=$(make_repo r1) || { echo 'FATAL: make_repo 失败' >&2; exit 1; }
W1=$(add_wt "$R1" wt-clean) || { echo 'FATAL: add_wt 失败' >&2; exit 1; }
run_gate "$R1" 1024
expect_rc 0 "$LAST_RC" '干净 worktree 应为 rc=0'
expect_has 'worktree=1 个' '被检查集合应恰好 1 个 worktree'

# ==========================================================================
banner 2 'rc=3：worktree 干净但 target 累计超阈值'
R2=$(make_repo r2) || exit 1
W2=$(add_wt "$R2" wt-bloat) || exit 1
bloat "$W2"
run_gate "$R2" 1
expect_rc 3 "$LAST_RC" '仅体积超阈值应为 rc=3'
expect_has 'target 累计' '体积告警行必须打印'

# ==========================================================================
banner 3 'rc=4：worktree 有已跟踪文件的未提交修改'
R3=$(make_repo r3) || exit 1
W3=$(add_wt "$R3" wt-dirty) || exit 1
printf 'dirty\n' >> "$W3/seed.txt"
run_gate "$R3" 1024
expect_rc 4 "$LAST_RC" '脏 worktree 应为 rc=4'
expect_has "$W3" '告警必须含该 worktree 的**具体路径**（不是只报个数）'

# ==========================================================================
banner 4 '⭐ rc=4 优先于 3：脏 + 体积同时超阈值（掩盖回归的核心用例）'
R4=$(make_repo r4) || exit 1
W4=$(add_wt "$R4" wt-both) || exit 1
printf 'dirty\n' >> "$W4/seed.txt"
bloat "$W4"
run_gate "$R4" 1
expect_rc 4 "$LAST_RC" '脏优先于体积 ⇒ 必须 rc=4（旧实现这里给 3，掩盖了脏判据）'
expect_has 'target 累计' '体积信息在 rc=4 时仍应打印（两条判据都报，不互相吞）'

# ==========================================================================
banner 5 'rc=4：未跟踪新文件也算脏'
R5=$(make_repo r5) || exit 1
W5=$(add_wt "$R5" wt-untracked) || exit 1
printf 'brand new\n' > "$W5/untracked.txt"
run_gate "$R5" 1024
expect_rc 4 "$LAST_RC" '未跟踪文件应被判脏 ⇒ rc=4'

# ==========================================================================
banner 6 '⭐ 主树只报告不阻断：主树脏 + worktree 干净 ⇒ rc=0'
R6=$(make_repo r6) || exit 1
W6=$(add_wt "$R6" wt-clean2) || exit 1
printf 'main-dirty\n' >> "$R6/seed.txt"
run_gate "$R6" 1024
expect_rc 0 "$LAST_RC" '主树脏不得计入退出码 ⇒ rc=0'
expect_has '1 处未提交' '必须打印主树的未提交处数'
expect_has '只报告，不影响退出码' '必须显式声明「只报告，不影响退出码」'
expect_has '带未提交改动: 0 个' 'worktree 集合必须判为 0 脏（主树那 1 处不得算进去）'

# ==========================================================================
banner 7 '⭐ cwd 无关性：在 worktree 内部执行 check，主树不得进入被检查集合'
R7=$(make_repo r7) || exit 1
W7=$(add_wt "$R7" wt-cwd) || exit 1
printf 'wt-dirty\n' >> "$W7/seed.txt"   # worktree 脏 ⇒ 预期 rc=4
printf 'main-dirty\n' >> "$R7/seed.txt" # 主树也脏 ⇒ 若被误当 worktree 检查，rc 仍是 4
run_gate "$W7" 1024                    # ⭐ cwd = worktree 内部
expect_rc 4 "$LAST_RC" '在 worktree 内执行仍应正确识别脏 worktree ⇒ rc=4'
expect_has 'worktree=1 个' '被检查集合恒为 1 个（主树若混入会变成 2 个）'
# 表格行形如 "<path> | <head> | <branch> | ..."；主树只允许出现在「**主树**」那一行
if printf '%s\n' "$LAST_OUT" | grep -F -q -- "$R7 | "; then
    bad '表格里出现了主树行 ⇒ 主树被当成 worktree 检查了' "$LAST_OUT"
else
    ok '表格里没有主树行（主树始终被排除）'
fi
expect_has '**主树**' '主树信息只应出现在「主树」那一行'

# ==========================================================================
banner 8 '⭐ 告警措辞契约（.githooks/pre-push 依赖的子串）'
R8a=$(make_repo r8a) || exit 1
W8a=$(add_wt "$R8a" wt-warn3) || exit 1
bloat "$W8a"
run_gate "$R8a" 1
expect_rc 3 "$LAST_RC" 'rc=3 场景（体积）'
expect_has 'worktree=' 'rc=3 时仍须含「worktree=」'
expect_has '未提交改动' 'rc=3 时仍须含「未提交改动」'

R8b=$(make_repo r8b) || exit 1
W8b=$(add_wt "$R8b" wt-warn4) || exit 1
printf 'dirty\n' >> "$W8b/seed.txt"
run_gate "$R8b" 1024
expect_rc 4 "$LAST_RC" 'rc=4 场景（脏）'
expect_has 'worktree=' 'rc=4 时仍须含「worktree=」'
expect_has '未提交改动' 'rc=4 时仍须含「未提交改动」'

# ==========================================================================
printf '\n============================================================\n'
printf '汇总：PASS=%d  FAIL=%d（共 8 个用例）\n' "$PASS" "$FAIL"
if [ "$FAIL" -eq 0 ]; then
    printf '✅ 全部通过 —— 真实脚本的 0/3/4 三档退出码均**可达**且语义正确。\n'
    exit 0
fi
printf '❌ 有失败：上面标 ❌ 的断言不成立。\n'
printf '   ⭐ 若失败项是 rc 相关，请先怀疑「分支不可达」（R-DISK-8）。\n'
exit 1
