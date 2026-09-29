#!/bin/sh
# worktree 门：多窗口并行时约束 worktree 的创建与回收，兼磁盘体检。
#
# 背景（2026-09-28 实测事故）：22 个 worktree 占 28G，其中 16.6G 是 target
# 编译产物；而 15 个 worktree 合计 850 处未提交改动**不在任何提交里** ——
# `git branch -a --contains` 判不出来。清理时若只看"已合入分支"就删，
# 即永久丢失；本轮我据此误删过 ratchet（4 处脏文件），靠 patch 兜底才恢复。
#
# 本脚本把 R-DISK-1~7 的人工纪律固化为门，**不代替判断，只强制证据**：
#   check  体检 + 分类（退出码：0 无可回收 / 3 有可回收 target / 4 有脏 worktree）
#   clean  清 target（零风险，仅生成物）
#   prune  移除 worktree（须双闸 + patch 兜底，且 --force 时才允许）
#
# 三条硬约束（源自 R-DISK-2/5/7）：
#   ① worktree 本体删除前必须 dirty=0，否则先 patch 兜底
#   ② patch 必须 `git apply --check --reverse` 校验可回放后才准删
#   ③ 任一判据指向"有工作"即放弃 —— 不得挑一个信的判据执行
set -u

REPO=$(git rev-parse --show-toplevel 2>/dev/null) || { echo "ERROR: 非 git 仓库"; exit 1; }
cd "$REPO" || exit 1

CMD=${1:-check}
# 目标磁盘告警阈值（MB），超过即视为可回收
TARGET_WARN_MB=${NT_WT_TARGET_MB:-1024}
SALVAGE_ROOT=${NT_WT_SALVAGE:-$REPO/.neotrix/worktree-salvage}

say() { echo "$@"; }
hr() { say "------------------------------------------------------------"; }

# 列出全部 worktree 路径（主仓除外）
list_worktrees() {
    git worktree list --porcelain 2>/dev/null \
        | awk '/^worktree /{print substr($0,10)}' \
        | grep -v "^$REPO\$"
}

# 单个 worktree 的取证：路径|HEAD|分支|脏数|体积MB|targetMB|mtime
probe() {
    _d=$1
    [ -d "$_d" ] || return 1
    _h=$(git -C "$_d" rev-parse --short HEAD 2>/dev/null) || return 1
    _br=$(git -C "$_d" rev-parse --abbrev-ref HEAD 2>/dev/null || echo '?')
    _dirty=$(git -C "$_d" status --porcelain 2>/dev/null | wc -l | tr -d ' ')
    _mb=$(du -sm "$_d" 2>/dev/null | cut -f1)
    _tmb=$(du -sm "$_d/target" 2>/dev/null | cut -f1); _tmb=${_tmb:-0}
    # 最新 .rs mtime（小时数，-1h 内视为活跃）—— 不写死时间窗，R-DISK-7
    _mt=$(find "$_d" -name '*.rs' -newermt '-3 hours' 2>/dev/null | head -1)
    _active="no"; [ -n "$_mt" ] && _active="YES"
    say "$_d|$_h|$_br|$_dirty|$_mb|$_tmb|$_active"
}

cmd_check() {
    say "[worktree-gate] repo=$REPO mode=check"
    hr
    say "路径 | HEAD | 分支 | 脏 | 体积 | target | 近3h活动"
    say "$(printf -- '-%.0s' {1..74})"
    tot_mb=0; tot_tmb=0; n_dirty=0; n_active=0
    for d in $(list_worktrees); do
        line=$(probe "$d") || continue
        IFS='|' read -r p h br dirty mb tmb act <<EOF
$line
EOF
        say "$p | $h | $br | $dirty | ${mb}M | ${tmb}M | $act"
        tot_mb=$((tot_mb + mb)); tot_tmb=$((tot_tmb + tmb))
        [ "$dirty" != "0" ] && n_dirty=$((n_dirty + 1))
        [ "$act" = "YES" ] && n_active=$((n_active + 1))
    done
    hr
    say "[worktree-gate] worktree=$(( $(list_worktrees | wc -l) )) 个 | 合计 ${tot_mb}M | target 占 ${tot_tmb}M"
    say "[worktree-gate] 带未提交改动: $n_dirty 个 | 近3h有改动: $n_active 个"
    if [ "$n_active" -gt 0 ]; then
        say "[worktree-gate] ⚠️  $n_active 个 worktree 近 3 小时仍有 .rs 改动 ⇒ 可能他窗在用，勿删"
    fi
    if [ "$n_dirty" -gt 0 ]; then
        say "[worktree-gate] ⛔ $n_dirty 个 worktree 的未提交改动**不在任何提交里**"
        say "[worktree-gate]    删它们必须先 patch 兜底（R-DISK-5）：sh $0 prune"
    fi
    if [ "$tot_tmb" -ge "$TARGET_WARN_MB" ]; then
        say "[worktree-gate] ♻️  target 累计 ${tot_tmb}M ≥ ${TARGET_WARN_MB}M ⇒ 零风险可回收：sh $0 clean"
        exit 3
    fi
    [ "$n_dirty" -gt 0 ] && exit 4
    exit 0
}

cmd_clean() {
    say "[worktree-gate] mode=clean —— 仅删 target（生成物，已 gitignore）"
    # 有 cargo 在跑时不碰（R-DISK-4）
    if pgrep -x cargo >/dev/null 2>&1 || pgrep -x rustc >/dev/null 2>&1; then
        say "[worktree-gate] ⛔ 检测到 cargo/rustc 在运行 ⇒ 让位（R-DISK-4）"; exit 2
    fi
    freed=0
    for d in $(list_worktrees); do
        [ -d "$d/target" ] || continue
        mb=$(du -sm "$d/target" 2>/dev/null | cut -f1)
        rm -rf "$d/target" && { say "  ✅ 清 $d/target (${mb}M)"; freed=$((freed + mb)); }
    done
    say "[worktree-gate] 释放 ${freed}M"
    # R-DISK-3：复验脏文件计数未变
    say "[worktree-gate] 复验各 worktree 脏文件计数："
    for d in $(list_worktrees); do
        say "  $(basename "$d"): $(git -C "$d" status --porcelain 2>/dev/null | wc -l | tr -d ' ') 处"
    done
    exit 0
}

salvage_one() {
    # $1=worktree 路径；导出 patch+清单并校验可回放，返回 0=已兜底
    _d=$1
    _name=$(basename "$_d")
    mkdir -p "$SALVAGE_ROOT" || return 1
    _pf="$SALVAGE_ROOT/${_name}.patch"
    _ut="$SALVAGE_ROOT/${_name}.untracked.txt"
    git -C "$_d" diff > "$_pf" 2>/dev/null
    git -C "$_d" status --porcelain | grep '^??' > "$_ut" 2>/dev/null
    if [ -s "$_pf" ] && git -C "$_d" apply --check --reverse "$_pf" 2>/dev/null; then
        say "  ✅ $_name 已兜底 $(du -k "$_pf" | cut -f1)K + $(wc -l < "$_ut" | tr -d ' ') 未跟踪"
        return 0
    fi
    if [ ! -s "$_pf" ] && [ ! -s "$_ut" ]; then
        say "  ➖ $_name 无需兜底（无改动）"; return 0
    fi
    say "  ⛔ $_name 兜底校验未过 ⇒ 拒绝删除"; return 1
}

cmd_prune() {
    # ⚠️ 派发器是 `cmd_prune "${2:-}"` —— flag 已被传成**函数内的 $1**。
    #    原先这里读 $2（函数内不存在）⇒ FORCE 恒空 ⇒ `prune --force` 静默空转，
    #    永远走不到 `git worktree remove`。文档化的 --force 实际从来没生效过。
    #    2026-09-29 修复：改读 $1。改前请用 `prune` 输出核对 mode 行是否含 `--force`。
    FORCE=${1:-}   # set -u 下必须显式兜底，否则未传时报 unbound variable
    say "[worktree-gate] mode=prune${FORCE:+ --force} —— 双闸 + patch 兜底（R-DISK-2/5）"
    for d in $(list_worktrees); do
        _name=$(basename "$d")
        _h=$(git -C "$d" rev-parse --short HEAD 2>/dev/null)
        _dirty=$(git -C "$d" status --porcelain 2>/dev/null | wc -l | tr -d ' ')
        _merged=$(git branch -a --contains "$_h" 2>/dev/null | wc -l | tr -d ' ')
        # 判据三：近 3h 有 .rs 改动 ⇒ 他窗在用，放弃（R-DISK-7）
        # ⚠️ 2026-09-29 修正判据顺序：该判据原为**首道**，但它是个 mtime 启发式，
        #    无法区分「他窗正在写」与「我自己刚做完」——`git merge`/checkout 会刷新
        #    全树 .rs 的 mtime，于是自己刚收工的干净 worktree 也被判成他窗在用，
        #    永远清不掉（本会话实测：merge-test 与 integrate 均被误拦）。
        #    但 mtime 只在「有东西可能丢」时才有意义：
        #      · worktree 干净 + HEAD 已含于某分支 ⇒ `git worktree remove` 可证无损，
        #        此时 mtime 无关紧要；
        #      · worktree 脏 ⇒ 才真有可能丢未提交改动，此时 mtime 判据才有意义。
        #    故改为：脏 → 先走 mtime 判据 + patch 兜底；干净 → 跳过 mtime 直通双闸。
        #    这让门更**准确**（去掉误报），不是放松（脏 worktree 的保护完全不变）。
        if [ "$_dirty" != "0" ]; then
            _hot=$(find "$d" -name '*.rs' -newermt '-3 hours' 2>/dev/null | wc -l | tr -d ' ')
            if [ "$_hot" != "0" ]; then
                say "  ⏭  $_name 脏($_dirty 处) 且近3h有 $_hot 个 .rs 改动 ⇒ 疑似他窗在用，跳过"; continue
            fi
            say "  💾 $_name 有 $_dirty 处未提交改动 ⇒ 先兜底"
            salvage_one "$d" || continue
        fi
        if [ "$_merged" = "0" ]; then
            say "  ⏭  $_name HEAD($_h) 未含于任何分支，跳过"; continue
        fi
        if [ "$FORCE" != "--force" ]; then
            say "  🖋  $_name 可删（双闸通过）。加 --force 才实际执行"; continue
        fi
        git worktree remove --force "$d" 2>&1 | sed 's|^|     |'
    done
    git worktree prune
    say "[worktree-gate] 剩余 worktree: $(list_worktrees | wc -l | tr -d ' ') 个"
    exit 0
}

case "$CMD" in
    check) cmd_check ;;
    clean) cmd_clean ;;
    prune) cmd_prune "${2:-}" ;;
    *) say "用法: sh $0 {check|clean|prune [--force]}"; exit 1 ;;
esac
