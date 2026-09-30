#!/bin/sh
# 内存门：重型 cargo（全量 test / 大包 build）起跑前检查空闲页，防 OOM 连环杀。
# 教训 2026-09-27：全量 lib 测试二进制(239M)+多线程 + sidecar 权重常驻 + 晶体 + App
# 在 16G 机上触发内核 SIGKILL，连坐杀掉 sidecar/crystal/App/测试进程。
# 用法：sh scripts/ops/nt_mem_gate.sh [阈值页数，默认 100000≈1.6G]；EXIT 0 可跑，2 请让路。
# 不杀任何进程，只报告。
set -u
TH="${1:-100000}"
# 2026-09-30 修：这道门在 macOS 上恒红。
# 原实现 `vm_stat | awk '/Pages free/'` 只读 free，而 macOS 把页面缓存记在
# **inactive**（实测 inactive=405,273 vs free=23,284）⇒ 判据永远看不到那
# ~6.4G 可回收内存，门恒 BLOCKED。
# ⇒ 恒红的门比没有门更坏（它训练人忽略红色）——本仓 R-DISK/门纪律的原话。
# 修法：free + inactive + speculative 才是「可分配」的近似。
# ⚠️ inactive 回收有成本（需写入才能逐出），故阈值按总量口径给，不假装与
# Linux 的 free 完全等价。
free_pages=$(vm_stat | awk '/Pages free/ {gsub(/\./,"",$3); print $3+0}')
inactive_pages=$(vm_stat | awk '/Pages inactive/ {gsub(/\./,"",$3); print $3+0}')
spec_pages=$(vm_stat | awk '/Pages speculative/ {gsub(/\./,"",$3); print $3+0}')
avail_pages=$(( ${free_pages:-0} + ${inactive_pages:-0} + ${spec_pages:-0} ))
swap_used=$(sysctl -n vm.swapusage 2>/dev/null | awk -F'[= ]+' '{print $6}')
echo "[mem-gate] free=$free_pages inactive=$inactive_pages speculative=$spec_pages" \
     "=> avail=$avail_pages  swap_used=${swap_used:-?} threshold=$TH"
if [ "${avail_pages:-0}" -gt "$TH" ]; then
  echo "[mem-gate] OPEN"
  exit 0
else
  echo "[mem-gate] BLOCKED: 可用页不足。先停重活（sidecar/晶体/App 按序）。"
  echo "[mem-gate]   若 free 低但 inactive 高，那是页面缓存可回收 ——"
  echo "[mem-gate]   先试 `sudo purge` 或等缓存自然逐出，再复测本门。"
  exit 2
fi
