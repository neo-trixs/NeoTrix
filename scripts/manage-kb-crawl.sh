#!/usr/bin/env bash
# NeoTrix KnowledgeBase 自动爬取 — launchd 安装/卸载/状态
set -euo pipefail

PLIST="com.neotrix.kb-crawl"
SRC="$(dirname "$0")/com.neotrix.kb-crawl.plist"
DST="$HOME/Library/LaunchAgents/com.neotrix.kb-crawl.plist"
BIN="/usr/local/bin/neotrix-kb-crawl"
LOG="$HOME/.neotrix/crawl-daemon.log"

case "${1:-status}" in
  install)
    echo "==> Building binary..."
    (cd "$(dirname "$0")/.." && cargo build -p neotrix --bin neotrix-kb-crawl --release)
    cp "../target/release/neotrix-kb-crawl" "$BIN"
    echo "==> Installing launchd plist..."
    mkdir -p "$HOME/Library/LaunchAgents"
    cp "$SRC" "$DST"
    launchctl load "$DST"
    echo "✅ Installed. Log: $LOG"
    ;;
  uninstall)
    echo "==> Uninstalling..."
    launchctl unload "$DST" 2>/dev/null || true
    rm -f "$DST"
    echo "✅ Uninstalled."
    ;;
  status)
    if launchctl list | grep -q "$PLIST"; then
      echo "✅ Running. PID: $(launchctl list | grep "$PLIST" | awk '{print $1}')"
      echo "   Log (tail 5):"
      tail -5 "$LOG" 2>/dev/null || echo "   (no log yet)"
    else
      echo "❌ Not loaded."
    fi
    ;;
  log)
    tail -f "$LOG"
    ;;
  *)
    echo "Usage: $0 {install|uninstall|status|log}"
    exit 1
    ;;
esac
