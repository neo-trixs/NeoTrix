#!/bin/sh
# 哨兵：盯邻居文件，稳定后跑 cargo check --tests（轻量，不过 link）。
# 完整 test 链接吃数 GB，只在显式要求时跑（见 sessions 指令）。
cd /Users/neo/Downloads/neotrix || exit 9
F1="neotrix-core/src/l1_action/nt_io/nt_io_browser_engine.rs"
F2="neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs"
FILTERS="nt_jev_calibration nt_orchestrator nt_eval_loop nt_jev_agentjev nt_train_export nt_awaken_loop backup_plan"
LOGDIR="sessions/logs"
mkdir -p "$LOGDIR"
M1=$(stat -f %m "$F1"); M2=$(stat -f %m "$F2" 2>/dev/null || echo 0)
STABLE=0; TRIES=0
echo "watch start" >> "$LOGDIR/watch.log"
while [ $TRIES -lt 24 ]; do
  sleep 120
  C1=$(stat -f %m "$F1"); C2=$(stat -f %m "$F2" 2>/dev/null || echo 0)
  if [ "$C1" != "$M1" ] || [ "$C2" != "$M2" ]; then
    echo "changed, debounce" >> "$LOGDIR/watch.log"
    M1=$C1; M2=$C2; STABLE=0; continue
  fi
  STABLE=$((STABLE+1))
  if [ $STABLE -lt 2 ]; then continue; fi
  TRIES=$((TRIES+1))
  echo "attempt $TRIES at $(date '+%H:%M:%S')" >> "$LOGDIR/watch.log"
  rm -f "$LOGDIR/verify_auto.done"
  cargo check -j 1 -p neotrix --lib --tests > "$LOGDIR/verify_auto.log" 2>&1
  EC=$?
  echo "attempt $TRIES EXIT:$EC" >> "$LOGDIR/watch.log"
  echo "$EC" > "$LOGDIR/verify_auto.done"
  if [ $EC -eq 0 ]; then echo "GREEN-check" >> "$LOGDIR/watch.log"; exit 0; fi
  if grep -qE "nt_io_browser_engine|handlers_consciousness|consciousness_core|skill_loader" "$LOGDIR/verify_auto.log"; then
    echo "still neighbor-blocked" >> "$LOGDIR/watch.log"
    STABLE=0; continue
  else
    grep -E "^error" "$LOGDIR/verify_auto.log" | head -n 5 >> "$LOGDIR/watch.log"
    echo "NON-NEIGHBOR error, human needed" >> "$LOGDIR/watch.log"; exit 2
  fi
done
echo "GAVE UP" >> "$LOGDIR/watch.log"; exit 3
