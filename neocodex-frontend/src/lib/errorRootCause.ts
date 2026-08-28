/**
 * errorRootCause — 结构化错误根因映射（前端）
 * 把 provider/流式阶段错误按已知模式映射到精确的 WHY / NEXT，
 * 避免所有错误都落到同一句通用文案（研究结论：结构化错误需 what/why/next）。
 * 后端可在 StreamErrorPayload 直接携带 what/why/next 覆盖此推导。
 */

export interface RootCause {
  why: string
  next: string
}

const RULES: Array<{ test: RegExp; why: string; next: string }> = [
  { test: /rate.?limit|429|too many requests/i, why: '命中提供商速率限制（HTTP 429）', next: '稍后重试，或切换到更高配额的 key / 模型' },
  { test: /401|unauthor|api.?key|invalid.?key|auth/i, why: '认证失败（HTTP 401）：API key 无效或缺失', next: '在 Settings 中检查并重新填入 API key' },
  { test: /network|econn|timeout|timed out|dns|fetch failed|connection/i, why: '网络层失败：连接超时或 DNS 解析出错', next: '检查网络连通性；企业内网可能需配置代理 / egress 策略' },
  { test: /context|too long|max.?token|length|truncat/i, why: '上下文超出模型最大长度', next: '运行 /compact 压缩会话，或切换到更长上下文的模型' },
  { test: /model|not.?found|does not exist|deprecated/i, why: '请求模型不存在或无权限', next: '在 Settings 中选择当前账号可用的模型' },
  { test: /quota|exceed|credit|余额|insufficient/i, why: '账号配额 / 额度耗尽', next: '充值或切换到其他账号 / 提供商' },
  { test: /cancelled|stopped|abort/i, why: '用户在生成中途取消', next: '可重新发送以继续，或调整提示后重试' },
]

const FALLBACK: RootCause = {
  why: 'provider/流式阶段错误（F1），回复未落盘',
  next: '可重试；若持续出现，检查网络连通性或 API key',
}

export function rootCause(message: string): RootCause {
  const m = (message || '').toLowerCase()
  for (const r of RULES) {
    if (r.test.test(m)) return { why: r.why, next: r.next }
  }
  return FALLBACK
}
