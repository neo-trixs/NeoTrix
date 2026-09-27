#!/bin/sh
# sidecar 按需控制器（2026-09-27 根治内存爆炸：常驻 3G 改跑 eval 才拉）。
# 用法：sh scripts/ops/nt_sidecar.sh {start|stop|status}
# start ＝调已有门脚本（内存自检＋健康轮询）；stop ＝优雅停；status ＝探活。
# 禁止事项：训练/采矿进程一律不动；本脚本只管 :8149 推理服务。
set -u
cd /Users/neo/Downloads/neotrix || exit 1
case "${1:-status}" in
  start)
    sh scripts/ops/nt_mem_gate.sh || exit 2
    nohup sh sessions/logs/sidecar_gate.sh >/dev/null 2>&1 &
    echo "[sidecar] gate launched, polling :8149/health (up to ~10min)"
    ;;
  stop)
    PID=$(pgrep -f "jev_service.server" | head -1)
    if [ -z "$PID" ]; then echo "[sidecar] not running"; exit 0; fi
    kill "$PID" && sleep 5
    pgrep -f "jev_service.server" >/dev/null && echo "[sidecar] STILL_UP" || echo "[sidecar] DOWN_OK"
    ;;
  status)
    curl -s --max-time 3 http://127.0.0.1:8149/health >/dev/null 2>&1 \
      && echo "[sidecar] UP" || echo "[sidecar] DOWN (on-demand, start when needed)"
    ;;
  *)
    echo "usage: $0 {start|stop|status}"; exit 2 ;;
esac
