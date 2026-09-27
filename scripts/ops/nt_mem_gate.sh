#!/bin/sh
# 内存门：重型 cargo（全量 test / 大包 build）起跑前检查空闲页，防 OOM 连环杀。
# 教训 2026-09-27：全量 lib 测试二进制(239M)+多线程 + sidecar 权重常驻 + 晶体 + App
# 在 16G 机上触发内核 SIGKILL，连坐杀掉 sidecar/crystal/App/测试进程。
# 用法：sh scripts/ops/nt_mem_gate.sh [阈值页数，默认 100000≈1.6G]；EXIT 0 可跑，2 请让路。
# 不杀任何进程，只报告。
set -u
TH="${1:-100000}"
free_pages=$(vm_stat | awk '/Pages free/ {gsub(/\./,"",$3); print $3+0}')
swap_used=$(sysctl -n vm.swapusage 2>/dev/null | awk -F'[= ]+' '{print $6}')
echo "[mem-gate] free_pages=$free_pages swap_used=${swap_used:-?} threshold=$TH"
if [ "${free_pages:-0}" -gt "$TH" ]; then
  echo "[mem-gate] OPEN"
  exit 0
else
  echo "[mem-gate] BLOCKED: 先停重活（sidecar/晶体/App 按序）或等 sidecar 加载完成再跑"
  exit 2
fi
