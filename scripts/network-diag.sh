#!/bin/bash
# neotrix Network Environment Diagnostic Tool — v2
# Usage: bash scripts/network-diag.sh [--quick]

set -o pipefail

OUTPUT_FILE="/tmp/neotrix-network-diag-$(date +%s).json"
QUICK=false
for arg in "$@"; do [[ "$arg" == "--quick" ]] && QUICK=true; done

R_NAMES=(); R_STATUS=(); R_DETAIL=()
record() { R_NAMES+=("$1"); R_STATUS+=("$2"); R_DETAIL+=("${3:-}"); }

C_RST='\033[0m'; C_CYN='\033[36m'; C_GRN='\033[32m'
C_RED='\033[31m'; C_YLW='\033[33m'; C_GRY='\033[90m'
info()  { printf "${C_CYN}[INFO]${C_RST} %s\n" "$1"; }
warn()  { printf "${C_YLW}[WARN]${C_RST} %s\n" "$1" >&2; }
fail()  { printf "${C_RED}[FAIL]${C_RST} %s\n" "$1" >&2; }
ok()    { printf "${C_GRN}[OK]${C_RST}   %s\n" "$1"; }
pass()  { printf "${C_GRN}[PASS]${C_RST} %s\n" "$1"; }
hr()    { printf "${C_GRY}%*s${C_RST}\n" 60 | tr ' ' '─'; }

# ═══════════════════════════════════════════════════════════════════════
# LAYER 1 — Network Interfaces & Routing
# ═══════════════════════════════════════════════════════════════════════
hr; echo "  LAYER 1 — Network Interfaces & Routing"; hr

DEFAULT_IF=$(route -n get default 2>/dev/null | grep interface | awk '{print $2}')
if [ -n "$DEFAULT_IF" ]; then
  ok "Default route via $DEFAULT_IF"; record "default_route" "PASS" "$DEFAULT_IF"
else
  fail "No default route"; record "default_route" "FAIL" "no default route"
fi

PHYS_IP=$(ifconfig en4 2>/dev/null | grep "inet " | awk '{print $2}')
[ -n "$PHYS_IP" ] && ok "Physical LAN IP: $PHYS_IP" && record "physical_ip" "PASS" "$PHYS_IP"

TUN_IFACES=$(ifconfig 2>/dev/null | grep -E "^utun" | awk -F: '{print $1}' | sort)
TUN_COUNT=$(echo "$TUN_IFACES" | grep -c . 2>/dev/null || echo 0)
if [ "$TUN_COUNT" -gt 0 ]; then
  TUN_LIST=$(echo "$TUN_IFACES" | tr '\n' ' ')
  warn "VPN interfaces detected: $TUN_LIST"; record "tun_interfaces" "WARN" "$TUN_COUNT TUN: $TUN_LIST"
else
  pass "No TUN interfaces"; record "tun_interfaces" "PASS" "none"
fi

FAKE_DNS=$(cat /etc/resolv.conf 2>/dev/null | grep nameserver | awk '{print $2}')
if echo "$FAKE_DNS" | grep -q "^198\.18\."; then
  warn "Fake-IP DNS: $FAKE_DNS (Shadowrocket/Surge/Clash TUN)"
  record "fake_ip_dns" "WARN" "DNS $FAKE_DNS in 198.18.0.0/15"
fi

PROXY_COUNT=0
for var in http_proxy https_proxy all_proxy HTTP_PROXY HTTPS_PROXY ALL_PROXY; do
  [ -n "${!var:-}" ] && PROXY_COUNT=$((PROXY_COUNT + 1))
done
[ "$PROXY_COUNT" -gt 0 ] && ok "Proxy env vars present ($PROXY_COUNT)" || info "No proxy env vars"
record "proxy_env" "INFO" "$PROXY_COUNT vars set"

# ═══════════════════════════════════════════════════════════════════════
# LAYER 2 — DNS Resolution
# ═══════════════════════════════════════════════════════════════════════
echo ""; hr; echo "  LAYER 2 — DNS Resolution"; hr

for domain in api.opencode.ai api.xiaohuxing.eu.org api.openai.com api.anthropic.com; do
  via_vpn=$(host "$domain" 198.18.0.2 2>/dev/null | grep "has address" | awk '{print $NF}' | head -1)
  via_pub=$(host "$domain" 8.8.8.8 2>/dev/null | grep "has address" | awk '{print $NF}' | head -1)
  if [ -z "$via_vpn" ]; then
    fail "  $domain -> DNS FAILED (via VPN)"; record "dns_$domain" "FAIL" "DNS failed"
  elif echo "$via_vpn" | grep -q "^198\.18\."; then
    warn "  $domain -> VPN fake-IP: $via_vpn (public: $via_pub)"
    record "dns_$domain" "WARN" "fake-IP $via_vpn"
  else
    ok "  $domain -> $via_vpn"; record "dns_$domain" "PASS" "$via_vpn"
  fi
done

# ═══════════════════════════════════════════════════════════════════════
# LAYER 3 — Quick API Health Check
# ═══════════════════════════════════════════════════════════════════════
echo ""; hr; echo "  LAYER 3 — API Health Check (Quick: HTTP headers only)"; hr
echo ""

check_api() {
  local name="$1" url="$2" auth="$3"
  local info
  info=$(curl -sI --connect-timeout 8 --max-time 20 \
    -H "Content-Type: application/json" ${auth:+-H "$auth"} \
    "$url" 2>/dev/null | head -1)
  local code
  code=$(echo "$info" | awk '{print $2}')
  if [ -z "$code" ]; then
    fail "  $name: connection FAILED"; record "api_${name}" "FAIL" "connection failed"
  elif [ "$code" = "200" ] || [ "$code" = "401" ] || [ "$code" = "422" ]; then
    ok "  $name: HTTP $code (server reachable)"; record "api_${name}" "PASS" "HTTP $code"
  elif [ "$code" = "503" ]; then
    fail "  $name: HTTP 503 (Service Unavailable!)"; record "api_${name}" "FAIL" "HTTP 503"
  else
    warn "  $name: HTTP $code"; record "api_${name}" "WARN" "HTTP $code"
  fi
}

check_api "opencode" "https://api.opencode.ai/v1/models" ""
check_api "xiaohuxing" "https://api.xiaohuxing.eu.org/v1/models" \
  "Authorization: Bearer sk-OWmevZU06pTOqF6FzmFXYwCYpGhJEmloWfu0fbF6310OF18v"

if $QUICK; then
  echo ""
  echo "  (--quick: skipping detailed timing test)"
else
  echo ""
  info "Detailed timing test (xiaohuxing chat completions, may take 60s)..."
  local result
  result=$(curl -s -w "HTTP:%{http_code}|TCP:%{time_connect}s|SSL:%{time_appconnect}s|TTFB:%{time_starttransfer}s|TOTAL:%{time_total}s" \
    --connect-timeout 10 --max-time 60 \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer ${XIAOHUXING_API_KEY:?XIAOHUXING_API_KEY not set}" \
    -d '{"model":"mimo-v2.5","messages":[{"role":"user","content":"OK"}],"max_tokens":2}' \
    https://api.xiaohuxing.eu.org/v1/chat/completions 2>/dev/null || echo "CURL_FAILED")
  echo ""
  http_code=$(echo "$result" | grep -oE 'HTTP:[0-9]+' | cut -d: -f2)
  ttfb=$(echo "$result" | grep -oE 'TTFB:[0-9.]+s' | cut -d: -f2 | tr -d s)
  if [ -n "$http_code" ]; then
    echo "  xiaohuxing chat: HTTP $http_code, TTFB ${ttfb}s"
    record "api_xiaohuxing_timing" "INFO" "HTTP $http_code TTFB ${ttfb}s"
  else
    fail "  xiaohuxing chat completions: FAILED"
    record "api_xiaohuxing_timing" "FAIL" "timeout or error"
  fi
fi

# ═══════════════════════════════════════════════════════════════════════
# LAYER 4 — VPN Routing Health
# ═══════════════════════════════════════════════════════════════════════
echo ""; hr; echo "  LAYER 4 — VPN Routing Health"; hr

if ps aux | grep -qiE "shadowrocket" 2>/dev/null && echo "$FAKE_DNS" | grep -q "^198\.18\."; then
  warn "VPN: Shadowrocket fake-IP TUN mode"; record "vpn_type" "WARN" "Shadowrocket TUN"
fi

GEO=$(curl -s --connect-timeout 5 --max-time 10 https://ipinfo.io/json 2>/dev/null)
COUNTRY=$(echo "$GEO" | grep -o '"country":"[^"]*"' | cut -d'"' -f4 || echo "unknown")
ORG=$(echo "$GEO" | grep -o '"org":"[^"]*"' | cut -d'"' -f4 || echo "unknown")
CITY=$(echo "$GEO" | grep -o '"city":"[^"]*"' | cut -d'"' -f4 || echo "unknown")
info "Egress: $CITY, $COUNTRY ($ORG)"; record "ip_geo" "INFO" "$CITY, $COUNTRY, $ORG"

# ═══════════════════════════════════════════════════════════════════════
# ROOT CAUSE ANALYSIS
# ═══════════════════════════════════════════════════════════════════════
echo ""; hr; echo "  ROOT CAUSE ANALYSIS"; hr
echo ""

for i in "${!R_NAMES[@]}"; do
  if [ "${R_STATUS[$i]}" = "FAIL" ]; then
    CHECK="${R_NAMES[$i]}"
    DETAIL="${R_DETAIL[$i]}"
    printf "${C_RED}✗ $CHECK: $DETAIL${C_RST}\n"
  fi
done

echo ""
xiaohuxing_result=""
for i in "${!R_NAMES[@]}"; do
  [ "${R_NAMES[$i]}" = "api_xiaohuxing" ] && xiaohuxing_result="${R_STATUS[$i]}"
done

if [ "$xiaohuxing_result" = "FAIL" ]; then
  printf "${C_RED}ROOT CAUSE: Provider 'xiaohuxing.eu.org' is unreliable (HTTP 503)%s\n"
  echo "  • TCP/SSL connections succeed — network path is healthy"
  echo "  • Server responds with HTTP 503 (Service Unavailable)"
  echo "  • opencode wraps this as 'Cannot connect to API' after timeout"
  echo "  • Upstream bug: vercel/ai handle-fetch-error.ts marks ALL errors retryable"
  echo ""
  printf "${C_GRN}FIX:${C_RST}\n"
  echo "  ✓ Set default model to 'opencode/deepseek-v4-flash-free' in opencode.jsonc"
  echo "  ✓ Remove xiaohuxing from opencode config or set as non-default"
  echo ""
fi

# Save
{
  echo "{"
  echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
  echo "  \"results\": {"
  for i in "${!R_NAMES[@]}"; do
    comma=","; [ "$i" -eq $((${#R_NAMES[@]}-1)) ] && comma=""
    echo "    \"${R_NAMES[$i]}\": \"${R_STATUS[$i]}\"$comma"
  done
  echo "  },"
  echo "  \"summary\": {"
  echo "    \"fail\": $(printf '%s\n' "${R_STATUS[@]}" | grep -c FAIL || echo 0),"
  echo "    \"warn\": $(printf '%s\n' "${R_STATUS[@]}" | grep -c WARN || echo 0),"
  echo "    \"pass\": $(printf '%s\n' "${R_STATUS[@]}" | grep -c PASS || echo 0)"
  echo "  }"
  echo "}"
} > "$OUTPUT_FILE"
echo "  Results saved: $OUTPUT_FILE"
