#!/usr/bin/env bash
# check-evolution-ledger.sh — 进化实验账本活性门（R-P79 接线的最后一环）
#
# ## 要防的病（2026-09-29 实测，本仓第三次同形复发）
#
# 本仓反复出现同一形状：**造了设施，没人调用**。
#   - `nt_jev`：导出 ≠ 调用（已错过 3 次）
#   - `ExperimentRunner`（238 行 + 8 测试）：零生产消费者
#   - `nt_evolution_exp` bin：实测能 ACCEPT 也能三路 REJECT，
#     **但没有进 Makefile 也没进 CI** ⇒ 没有任何门/流程会跑它
#
# 第三次复发说明：**「存在」与「会被运行」是两个独立属性**，
# 前者容易证明（文件在、测试过），后者才是活性的真正含义。
#
# ## 为什么这个门不跑 A/B 实验本身
#
# 跑一次实验 = 建 2×repeats 个 worktree + 跑 4 道门 ≈ 数十秒，
# 放进 CI 每次提交都付 ⇒ 不合适。
# ⇒ 本门只验证**「账本是活的」这一结构性质**：
#   1. results.tsv 存在且有数据行（实验确实跑过）
#   2. 账本含**正负两类**结果（append-only 的真义，不只记成功）
#   3. bin 已注册进 Cargo（可构建，不是一堆散落脚本）
#
# 完整 A/B 仍由人/CI 手动跑 `make evolution-exp`。
#
# ## 退出码
#   0 = 账本活性成立
#   1 = 不成立（详见输出）

set -uo pipefail
fail=0

LEDGER="results.tsv"
BIN_REL="neotrix-core/src/bin/nt_evolution_exp.rs"

if [ ! -f "$LEDGER" ]; then
  echo "[evolution] ⛔ $LEDGER 不存在 —— 从未成功跑过实验"
  fail=1
else
  rows=$(grep -c . "$LEDGER" 2>/dev/null || echo 0)
  # 减 1 = 表头
  data=$((rows > 0 ? rows - 1 : 0))
  echo "[evolution] 账本数据行: $data"

  if [ "$data" -lt 1 ]; then
    echo "[evolution] ⛔ 账本只有表头，没有实验记录"
    fail=1
  else
    # append-only 的真义 = 拒绝也必须留痕。只记成功的账本是自欺。
    # ⛔ 用 awk 一次算完，不用 `grep -c`：无匹配时 `grep -c` 打印 `0` **且退 1**，
    #   `$(... || echo 0)` 于是得到 `0\n0` ⇒ 算术报 "integer expression expected"
    #   ⇒ 判定被静默跳过、门假绿。（2026-09-29 自测抓到的真 bug）
    read -r pos neg <<<"$(awk -F'\t' '
      NR==1 { next }
      { total++
        if ($9 == "true")  pos++
        else if ($9 == "false") neg++
      }
      END { printf "%d %d\n", pos+0, neg+0 }
    ' "$LEDGER")"
    echo "[evolution] 接受 $pos / 拒绝 $neg"
    if [ "$neg" -eq 0 ]; then
      echo "[evolution] ⛔ 账本**只有接受、零拒绝** ⇒ 这不是 append-only，是成功日志"
      fail=1
    fi
  fi
fi

if [ ! -f "$BIN_REL" ]; then
  echo "[evolution] ⛔ $BIN_REL 不存在"
  fail=1
elif ! grep -q 'nt-evolution-exp' neotrix-core/Cargo.toml; then
  echo "[evolution] ⛔ bin 未注册进 Cargo.toml ⇒ 不可构建"
  fail=1
else
  echo "[evolution] ✅ bin 已注册进 Cargo"
fi

[ "$fail" -eq 0 ] && echo "[evolution] ✅ PASS"
exit "$fail"
