#!/usr/bin/env bash
# check-disk.sh — 构建产物膨胀 advisory 门
#
# ## ⚠️ 本脚本自身在 2026-09-29 制造过一次真实事故，先读这段
#
# 初版在「建议」文案里写了**反引号包裹的示例命令**：
#     echo "  建议：优先 `cargo clean -p <crate>` 局部回收"
# 干跑时 bash **真的执行了** `cargo clean` ⇒ **删掉 115.2 GiB**，
# 且当时有 4 个他窗 cargo 进程正在构建。
#
# 教训（比本门本身重要）：
#   ① **反引号在 bash 里是命令替换，不是排版**。写示例命令用普通引号。
#      `bash -n` 抓不到这个 —— 语法完全合法，执行才有后果。
#   ② 门脚本的**干跑本身可能有副作用**。新写门必须先审一遍有无
#      副作用命令，再跑 —— 不能「先跑跑看」。R-SCAN-2 说「判断行为要
#      喂真实输入跑」，但那前提是**先读代码确认无副作用**。
#   ③ 判别「无副作用」的标准：门内除 `du`/`df`/`ps`/`git status` 等
#      **只读**命令外，不得出现任何写操作。
#
# ## 动机
#
# `target/` 曾涨到 **87G**，其中 `target/debug/incremental` 独占 **64G**。
# 根因：`incremental` 是 cargo **debug 档**的增量编译缓存，而本仓
# `neotrix-core` 单 crate 就 74 万行 + 9 个 workspace member ⇒ 每次
# `cargo check`/`cargo test` 都在增量缓存上重写，历史版本不回收。
#
# 本仓此前**没有任何门看磁盘**（`check-truth-surface.sh` 里的 'disk' 是
# 误命中）。⇒ 87G 是静默涨出来的。
#
# ## ⛔ 为什么只 advisory，不阻断
#
# R-DISK-1：**有 cargo 在跑时不碰主 `target/`**。本门在多窗口共享工作树里
# 若 exit 1，会把「磁盘紧张」变成「别人不能构建」——而磁盘紧张时正确
# 动作是**人决定何时清**，不是门替人决定。故：advisory 恒 exit 0。
#
# ## 清理判据（门只报数，不代劳）
#
#   ⛔ 绝不在有 cargo 进程时删 'target/'（会打断他窗构建）：
#       ps aux | grep -c '[c]argo'   # >0 则等
#   ✅ 'target/' 已 gitignore ⇒ 是纯生成物，删零风险：
#       git check-ignore -q target && rm -rf target/debug/incremental
#       （只删 incremental，保留 'deps/' 已编译产物，省一次全量重编）
#   ⛔ 绝不可 'rm -rf' worktree 目录 —— 本体可能带未提交工作，走
#      sh scripts/ops/nt_worktree_gate.sh prune
#
# 用法：bash scripts/check-disk.sh [--strict]
#   默认 advisory（恒 exit 0）；--strict 仅当超 WARN 阈值时 exit 1。

set -uo pipefail

STRICT=0
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    -h|--help) sed -n '2,28p' "$0"; exit 0 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

WARN_GB=40
CRIT_GB=80

# ── 1. target 体积（不跟随符号链接，du -s 抗跨设备）──────────────────
if [ -d target ]; then
  T_BYTES=$(du -sk target 2>/dev/null | awk '{print $1}')
  T_GB=$(( ${T_BYTES:-0} / 1048576 ))
  I_BYTES=$(du -sk target/debug/incremental 2>/dev/null | awk '{print $1}')
  I_GB=$(( ${I_BYTES:-0} / 1048576 ))
else
  T_GB=0; I_GB=0
fi

# ── 2. 根卷可用空间 ────────────────────────────────────────────────
AVAIL_GB=$(df -g / 2>/dev/null | awk 'NR==2{print $4}')
[ -n "${AVAIL_GB:-}" ] || AVAIL_GB=$(df -k / 2>/dev/null | awk 'NR==2{printf "%d", $4/1048576}')

echo "=== NeoTrix disk check (advisory) ==="
echo "target/: ${T_GB}G   (debug/incremental: ${I_GB}G)   root free: ${AVAIL_GB:-?}G"

# ── 3. 构建中检测 —— 有则只提示，不建议动手 ──────────────────────────
NCARGO=$(ps aux 2>/dev/null | grep -c '[c]argo')
if [ "$NCARGO" -gt 0 ]; then
  echo "⚠️  $NCARGO 个 cargo 进程在跑 ⇒ R-DISK-1：**不要**现在删 target/（会打断他窗构建）"
else
  if [ "$I_GB" -ge 5 ]; then
    echo "可回收（无 cargo 在跑时）: rm -rf target/debug/incremental   # 省 ${I_GB}G，deps/ 保留"
  fi
fi

# ── 4. 判定 ────────────────────────────────────────────────────────
LEVEL="ok"
[ "$T_GB" -ge "$WARN_GB" ] && LEVEL="warn"
[ "$T_GB" -ge "$CRIT_GB" ] && LEVEL="crit"

if [ "$T_GB" -ge "$WARN_GB" ]; then
  echo "level: $LEVEL (target ${T_GB}G ≥ WARN ${WARN_GB}G)"
  echo "  根因：debug 档 incremental 缓存不回收。本仓单 crate 74 万行。"
  # ⛔⛔ 绝不用反引号写示例命令 —— bash 会**执行**它们。
  #    2026-09-29 实测事故：本行原写作 `cargo clean -p <crate>`，
  #    干跑该脚本时 shell 真的执行了 cargo clean ⇒ 删掉 115.2GiB，
  #    且当时有 4 个他窗 cargo 进程在跑。示例一律用普通引号。
  echo '  建议：优先局部回收（cargo clean -p <crate>）；全量 cargo clean 会让'
  echo '        下一轮全量重编（16G 机上约数十分钟，勿在他窗构建时做）。'
  # 长期处方：`profile.dev` 设 incremental = false 或 debug = 0，可省 58% target 体积。
  #   [profile.dev]  incremental = false
  #
  # ⚠️ **2026-09-29 实测：此刻不要改**。数据如下，disk 紧张时再看：
  #   · incremental 16G / target 28G = **58%**（实测构成，非估算）
  #   · 磁盘可用 210Gi、占用仅 8% ⇒ **当前无 disk 压力**
  #   · 改单文件 `cargo check --lib` = **33s**（开 incremental）
  #   · 关掉后每次改码需全量重编 74 万行 ≈ 2–4 min ⇒ **慢 4–7 倍**
  # ⇒ 判定：拿日常开发速度换一个暂时用不上的磁盘空间，是**净损失**。
  #   正确触发条件：磁盘可用 < 80Gi 且 target > 80G 时再改，那时是净收益。
  #   ⛔ 不要为了「省磁盘」这个数字本身去改 —— 它此刻不省任何东西。
  echo '  长期处方（本会话实测：此刻不划算，见门内注释）：profile.dev 设 incremental=false'
fi

# advisory 恒绿（见头注：磁盘紧张时的正确动作由人决定，不是门替人决定）
exit 0
