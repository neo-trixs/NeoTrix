#!/usr/bin/env bash
# 跨 worktree 的**重型命令串行锁**。
#
# ## 为什么需要它（2026-09-30 实测）
# 吸收 `GetBrew/growth-engineer`（MIT）时读到它的一句设计说明：
#
#   > Heavy commands wait their turn behind a lock, so several worktrees can
#   > run checks without running out of memory.
#
# 这正是本仓**反复踩到**的坑：AGENTS.md §2 写着「⛔ 禁多窗口同时跑
# `--all-targets` / `--test`」，但那是**散文纪律**，没有任何机制保证。
# 2026-09-30 一个会话内因此**两次**因他窗 cargo 而构建超时/被中断。
#
# 本仓已有两个相关设施，但**都不闭合**：
# · `nt_mem_gate.sh` —— 只报告内存是否充足，不串行。
#   「内存充足」不等于「没人在编译」：16G 机上两个 `--all-targets` 并跑照样爆。
# · `scripts/wait-for-cargo.sh` —— 会 `pgrep` 等 cargo 空闲（已存在，本轮**先查到它**
#   才动手，避免重复造轮子）。但它是**「检查后再执行」**，
#   存在**竞态窗口**：两个窗口可同时通过检查、随后同时开跑。
# ⇒ 本脚本用**原子 `mkdir`** 把「检查 + 占位」合并成一步，闭合那个窗口。
#   两者可叠加使用（先 wait-for-cargo 再加锁），互不冲突。
#
# ## 设计取舍（都为了「绝不能把仓库卡死」）
# * **不用 `flock`**：macOS 默认没有，且 BSD `flock(1)` 语义与 Linux 不同 ⇒ 不可移植。
# * **用 `mkdir` 作锁**：POSIX 规定的原子操作，`mkdir` 失败即表示锁被占。
# * **带 PID + 时间戳 + 陈旧回收**：进程已死则自动清锁，避免「进程被 kill 后
#   锁永远留着」这种最坏情况。
# * **带超时**：拿不到锁时按 `--timeout` 等待；超时后**默认仍然执行**
#   （可用 `--strict` 改为放弃）⇒ 绝不会因为锁而让工作流永久停住。
# * **trap 清理**：正常退出/中断都释放（`EXIT INT TERM HUP`）。
#
# ## 用法
#   bash scripts/ops/nt_build_lock.sh -- <重命令...>      # 包裹执行
#   bash scripts/ops/nt_build_lock.sh --strict -- cargo test -p neotrix --lib
#   bash scripts/ops/nt_build_lock.sh --status            # 只看锁状态
#
# 例：把 Makefile 的重型目标改成经由本脚本串行。

set -uo pipefail

LOCK_DIR="${NT_BUILD_LOCK:-${TMPDIR:-/tmp}/nt-heavy-lock}"
STALE_SEC="${NT_BUILD_LOCK_STALE_SEC:-1800}"   # 30 分钟后视为陈旧
TIMEOUT="${NT_BUILD_LOCK_TIMEOUT:-1800}"     # 等锁上限
STRICT=0

while [ $# -gt 0 ]; do
  case "$1" in
    --) shift; break ;;
    --strict) STRICT=1; shift ;;
    --timeout) TIMEOUT="${2:-}"; shift 2 ;;
    --status)
      if [ -d "$LOCK_DIR" ]; then
        echo "[build-lock] 已被占用: $(cat "$LOCK_DIR/pid" 2>/dev/null | tr -d '\n') (since $(cat "$LOCK_DIR/since" 2>/dev/null))"
        exit 1
      fi
      echo "[build-lock] 空闲（$LOCK_DIR 不存在）"
      exit 0 ;;
    -h|--help) sed -n '2,30p' "$0"; exit 0 ;;
    *) break ;;
  esac
done

pid_alive() {
  # macOS: kill -0 可探活；不假设 `ps` 的输出格式（跨平台且无副作用）
  [ -n "${1:-}" ] && kill -0 "$1" 2>/dev/null
}

try_acquire() {
  if mkdir "$LOCK_DIR" 2>/dev/null; then
    echo "$$" > "$LOCK_DIR/pid"
    date +%s > "$LOCK_DIR/since"
    echo "cmd: $*" > "$LOCK_DIR/cmd"
    return 0
  fi
  return 1
}

reap_if_stale() {
  [ -d "$LOCK_DIR" ] || return 1
  local lp ls_
  lp="$(cat "$LOCK_DIR/pid" 2>/dev/null | tr -d '[:space:]')"
  ls_="$(cat "$LOCK_DIR/since" 2>/dev/null | tr -d '[:space:]')"
  # 锁持有者已死 ⇒ 立刻回收
  if [ -n "$lp" ] && ! pid_alive "$lp"; then
    echo "[build-lock] 回收陈旧锁（持有者 pid=$lp 已不存在）" >&2
    rm -rf "$LOCK_DIR"; return 1
  fi
  # 超龄 ⇒ 回收（防「pid 被复用」导致永久锁）
  if [ -n "$ls_" ] && [ $(( $(date +%s) - ls_ )) -gt "$STALE_SEC" ]; then
    echo "[build-lock] 回收超龄锁（持有时长 > ${STALE_SEC}s）" >&2
    rm -rf "$LOCK_DIR"; return 1
  fi
  return 0
}

# ⛔ 只删**自己持有的**锁（比对 pid），不能无条件 `rm -rf`。
# 这是自证用例 ⑦ 抓到的**真 bug**：原实现无条件删，
# 于是「A 超时 → exec 绕过 trap → B 拿到锁 → A 的后续清理把 B 的锁删了」
# ⇒ 第三个进程能同时进入 ⇒ 互斥失效（实测 A-end 与 B-start 重叠 1 秒）。
release() {
  [ -d "$LOCK_DIR" ] || return 0
  local lp
  lp="$(cat "$LOCK_DIR/pid" 2>/dev/null | tr -d '[:space:]')"
  if [ "$lp" = "$$" ]; then
    rm -rf "$LOCK_DIR"
  fi
  # pid 不匹配 ⇒ 锁已被别人接管，**绝不能删**
}

if [ $# -eq 0 ]; then
  echo "用法: nt_build_lock.sh [--strict] [--timeout N] -- <命令...> | --status" >&2
  exit 2
fi

waited=0
until try_acquire; do
  reap_if_stale
  try_acquire && break
  holder="$(cat "$LOCK_DIR/pid" 2>/dev/null | tr -d '[:space:]')"
  held="$(cat "$LOCK_DIR/cmd" 2>/dev/null | cut -c1-70)"
  echo "[build-lock] 等待锁… 已等 ${waited}s（持有者 pid=${holder:-?} ${held}）" >&2
  if [ "$waited" -ge "$TIMEOUT" ]; then
    if [ "$STRICT" = "1" ]; then
      echo "[build-lock] 超时(${TIMEOUT}s)且 --strict ⇒ 放弃执行（避免并行重编译）" >&2
      exit 75   # EX_TEMPFAIL：调用方可重试
    fi
    echo "[build-lock] 超时(${TIMEOUT}s) ⇒ 仍继续执行（内存门是最终裁决）" >&2
    exec "$@"
  fi
  sleep 5
  waited=$((waited + 5))
done

echo "[build-lock] 已获得锁 [$LOCK_DIR]" >&2
trap release EXIT INT TERM HUP
"$@"
rc=$?
exit $rc