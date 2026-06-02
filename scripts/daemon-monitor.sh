#!/bin/bash
# NeoTrix Daemon Monitor
# Usage: ./scripts/daemon-monitor.sh [status|restart|log|watch]

DAEMON_BIN="$(dirname "$0")/../target/debug/daemon"
PIDFILE="/tmp/neotrix_daemon.pid"
LOGFILE="/tmp/neotrix/daemon.log"
HEALTHFILE="/tmp/neotrix_daemon.health"

case "${1:-status}" in
  status)
    if [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
      echo "✅ Daemon running (PID: $(cat "$PIDFILE"))"
      [ -f "$HEALTHFILE" ] && cat "$HEALTHFILE"
      echo "---"
      echo "Memory: $(ps -o rss= -p "$(cat "$PIDFILE")" 2>/dev/null) KB"
      echo "Uptime: $(ps -o etime= -p "$(cat "$PIDFILE")" 2>/dev/null)"
    else
      echo "❌ Daemon not running"
    fi
    ;;
  restart)
    "$0" stop
    sleep 1
    "$0" start
    ;;
  start)
    if [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
      echo "Already running (PID: $(cat "$PIDFILE"))"
      exit 0
    fi
    mkdir -p /tmp/neotrix
    nohup "$DAEMON_BIN" > "$LOGFILE" 2>&1 &
    echo $! > "$PIDFILE"
    echo "✅ Started (PID: $!)"
    sleep 2
    [ -f "$HEALTHFILE" ] && cat "$HEALTHFILE"
    ;;
  stop)
    if [ -f "$PIDFILE" ]; then
      PID=$(cat "$PIDFILE")
      kill "$PID" 2>/dev/null && echo "✅ Stopped (PID: $PID)" || echo "❌ Not running"
      rm -f "$PIDFILE"
    else
      pkill -f "target/debug/daemon" 2>/dev/null && echo "✅ Stopped" || echo "❌ Not running"
    fi
    ;;
  log)
    tail -f "$LOGFILE"
    ;;
  watch)
    watch -n 5 '[ -f /tmp/neotrix_daemon.health ] && cat /tmp/neotrix_daemon.health; echo "---"; ps -o rss,etime -p $(cat /tmp/neotrix_daemon.pid 2>/dev/null) 2>/dev/null || echo "not running"'
    ;;
  *)
    echo "Usage: $0 {status|start|stop|restart|log|watch}"
    exit 1
    ;;
esac
