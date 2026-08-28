#!/usr/bin/env bash
# NeoTrix build watchdog
# 周期检测 neotrix lib 是否可编译, 把健康状态写入日志, 并在状态由绿转红时告警。
# 用于发现 openhands 等后台自治循环 churn 引入的红构建 / target 缓存失效
# (此类 churn 会让 cargo 缓存失稳、每次 check 耗时剧增甚至超时)。
#
# 用法:
#   scripts/build-watchdog.sh [interval_sec]     默认 300s
#   WATCHDOG_LOG=/path/log WATCHDOG_STATE=/path/state scripts/build-watchdog.sh
#
# 退出: Ctrl-C。本脚本只观测/告警, 不修改任何文件或强制回滚。
set -uo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

INTERVAL="${1:-300}"
LOG="${WATCHDOG_LOG:-/tmp/neotrix-build-watchdog.log}"
STATE="${WATCHDOG_STATE:-/tmp/neotrix-build-state}"
CHECK_TIMEOUT="${WATCHDOG_TIMEOUT:-280}"   # 单次 cargo check 超时(秒), 避免被 churn 拖死

echo "build-watchdog started: interval=${INTERVAL}s log=$LOG state=$STATE"

PREV=""
while true; do
  TS="$(date '+%Y-%m-%d %H:%M:%S')"
  if timeout "$CHECK_TIMEOUT" cargo check -p neotrix --lib >/tmp/neotrix-watchdog-check.log 2>&1; then
    STATE="ok"
    echo "$TS OK" >> "$LOG"
  else
    # 区分"编译失败"与"超时"(超时也可能因 churn 致缓存失效)
    if [ -s /tmp/neotrix-watchdog-check.log ]; then
      echo "$TS RED" >> "$LOG"
    else
      echo "$TS TIMEOUT" >> "$LOG"
    fi
    STATE="red"
    if [ "$PREV" != "red" ]; then
      echo "[build-watchdog] ⚠️  $TS neotrix lib 编译失败/超时 — 可能后台循环引入红构建或 target 缓存失效" >&2
      echo "[build-watchdog]   详见 /tmp/neotrix-watchdog-check.log" >&2
    fi
  fi
  echo "$STATE" > "$STATE"
  PREV="$STATE"
  sleep "$INTERVAL"
done
