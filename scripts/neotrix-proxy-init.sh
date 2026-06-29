#!/bin/bash
# NeoTrix Proxy Launcher v3
set -uo pipefail

NEOTRIX_DIR="${NEOTRIX_DIR:-$HOME/Downloads/neotrix}"
NEOTRIX_PROXY_BIN=""
for candidate in "$NEOTRIX_DIR/target/debug/nt-proxy-daemon" "$NEOTRIX_DIR/target/release/nt-proxy-daemon"; do
    if [ -x "$candidate" ]; then
        NEOTRIX_PROXY_BIN="$candidate"
        break
    fi
done
if [ -z "$NEOTRIX_PROXY_BIN" ]; then
    echo "[neotrix] ERROR: nt-proxy-daemon binary not found (tried debug and release)"
    exit 1
fi

NEOTRIX_HTTP_PORT=11080

log() { echo "[neotrix] $*"; }

# Shadowrocket VPN runs alongside (handles system routing).
# NeoTrix daemon pool supplements for geo-diverse routing.
# No system proxy manipulation needed — Shadowrocket is the router.

# Phase 1: 启动NeoTrix代理守护进程（纯路由层，无MITM）
start_neotrix_proxy() {
    if [ ! -f "$NEOTRIX_PROXY_BIN" ]; then
        log "proxy daemon not compiled: $NEOTRIX_PROXY_BIN"
        return 1
    fi
    if pgrep -x nt-proxy-daemon >/dev/null 2>&1; then
        local pid
        pid=$(pgrep -x nt-proxy-daemon)
        log "proxy daemon already running (PID: $pid)"
        return 0
    fi

    log "starting proxy daemon..."
    nohup "$NEOTRIX_PROXY_BIN" \
        > "/tmp/neotrix-proxy.out.log" \
        2> "/tmp/neotrix-proxy.err.log" & disown

    for i in $(seq 1 5); do
        sleep 1
        if pgrep -x nt-proxy-daemon >/dev/null 2>&1; then
            local pid
            pid=$(pgrep -x nt-proxy-daemon)
            if nc -z 127.0.0.1 "$NEOTRIX_HTTP_PORT" 2>/dev/null; then
                log "proxy daemon ready (PID: $pid)"
                return 0
            fi
        fi
    done
    log "WARNING: proxy daemon not ready after 5s"
    return 1
}

# System routing is handled by Shadowrocket VPN (always-on).
# NeoTrix daemon is a per-app proxy pool — not a system proxy.

# Phase 4: 验证代理端口
verify_proxy_port() {
    if nc -z 127.0.0.1 "$NEOTRIX_HTTP_PORT" 2>/dev/null; then
        log "proxy port $NEOTRIX_HTTP_PORT confirmed open"
        return 0
    else
        log "WARNING: proxy port $NEOTRIX_HTTP_PORT not listening"
        return 1
    fi
}

# 状态
status() {
    echo "=== NeoTrix Proxy ==="
    echo ""
    echo "Services:"
    echo "  NeoTrix daemon: $(pgrep -x nt-proxy-daemon >/dev/null && echo '✓ running' || echo '✗ stopped')"
    echo "  Shadowrocket VPN: $(scutil --nc list 2>/dev/null | grep -qi shadowrocket && echo '✓ active' || echo '✗ inactive')"
    echo "  Port $NEOTRIX_HTTP_PORT:    $(nc -z 127.0.0.1 "$NEOTRIX_HTTP_PORT" 2>/dev/null && echo '✓ listening' || echo '✗')"
    echo ""
    echo "Pool:"
    if [ -f "$HOME/.neotrix/proxy-upstreams.conf" ]; then
        echo "  Upstreams: $(wc -l < "$HOME/.neotrix/proxy-upstreams.conf" 2>/dev/null || echo 0)"
    else
        echo "  Upstreams: none"
    fi
    echo ""
    echo "TLS verification (via proxy):"
    for domain in opencode.ai api.anthropic.com api.openai.com api.github.com; do
        local code
        code=$(http_proxy="http://127.0.0.1:$NEOTRIX_HTTP_PORT" curl -sI --connect-timeout 5 "https://$domain" 2>/dev/null | head -1 | awk '{print $2}')
        if [ -n "$code" ]; then
            echo "  ✓ $domain → HTTP $code"
        else
            echo "  ✗ $domain → unreachable"
        fi
    done
}

# shutdown
shutdown() {
    log "shutting down..."
    if pgrep -x nt-proxy-daemon >/dev/null 2>&1; then
        pkill -x nt-proxy-daemon 2>/dev/null || true
        sleep 1
        log "proxy daemon stopped"
    fi
    cleanup_logs
    log "done"
}

cleanup_logs() {
    rm -f /tmp/neotrix-proxy.{out,err}.log 2>/dev/null || true
}

case "${1:-start}" in
    start)
        start_neotrix_proxy
        verify_proxy_port
        echo ""
        status
        ;;
    stop)
        shutdown
        ;;
    status)
        status
        ;;
    *)
        echo "Usage: $0 {start|stop|status}"
        echo ""
        echo "  start    Start proxy daemon (Shadowrocket VPN runs alongside)"
        echo "  stop     Stop proxy daemon"
        echo "  status   Current state"
        exit 1
        ;;
esac
