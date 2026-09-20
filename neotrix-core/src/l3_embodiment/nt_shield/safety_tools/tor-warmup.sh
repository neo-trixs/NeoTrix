#!/bin/bash
# opencode 冷启动预热脚本
# 在启动 opencode 前运行，预热 DNS 缓存 + TUN 接口连接
# 防止 "Cannot connect to API" 错误

set -e

echo "=== opencode Cold-Start Warmup ==="

# 1. 预热 DNS 缓存（强制 Shadowrocket fake-IP DNS 提前解析所有目标域名）
DOMAINS=(
  "api.opencode.ai"
  "api.xiaohuxing.eu.org"
  "api.openai.com"
  "api.anthropic.com"
)

echo "[1/3] Warming DNS cache..."
for domain in "${DOMAINS[@]}"; do
  dig +short "$domain" @"$(echo "198.18.0.2")" > /dev/null 2>&1
  echo "  ✓ $domain"
done

# 2. 预热 TCP/SSL 连接（建立 Shadowrocket TUN 到目标服务器的连接）
echo "[2/3] Warming TCP connections..."
curl -s --connect-timeout 5 --max-time 10 \
  -H "Content-Type: application/json" \
  -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"hi"}],"max_tokens":1}' \
  https://api.opencode.ai/v1/chat/completions \
  -o /dev/null -w "  ✓ opencode.ai: HTTP %{http_code} (%{time_total}s)\n"

# 3. 验证 xiaohuxing 可用性（如果 503，记录到日志但不阻塞）
echo "[3/3] Checking xiaohuxing health..."
HEALTH=$(curl -s --connect-timeout 5 --max-time 15 \
  -H "Authorization: Bearer ${XIAOHUXING_API_KEY:?XIAOHUXING_API_KEY not set}" \
  -H "Content-Type: application/json" \
  -d '{"model":"mimo-v2.5","messages":[{"role":"user","content":"hi"}],"max_tokens":1}' \
  -o /dev/null -w "%{http_code}" \
  https://api.xiaohuxing.eu.org/v1/chat/completions 2>/dev/null || echo "unreachable")

if [ "$HEALTH" = "200" ]; then
  echo "  ✓ xiaohuxing: healthy"
else
  echo "  ⚠ xiaohuxing: HTTP $HEALTH (will use default opencode model)"
fi

echo "=== Warmup complete. Start opencode now. ==="
