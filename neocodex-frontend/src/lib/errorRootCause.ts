/**
 * errorRootCause — 结构化错误根因映射 + 自动 fallback 策略（前端）
 *
 * 把 provider/流式阶段错误按已知模式映射到精确的 WHY / NEXT / FALLBACK，
 * 避免所有错误都落到同一句通用文案（研究结论：结构化错误需 what/why/next）。
 *
 * P1 增强: 吸收 cumora "deterministic fallback" 原则 —
 * classifier 不可用时不静默放弃, 而是 carving out 最窄确定性 case:
 * - 429 → 自动 retry-after 倒计时
 * - 503 → 提示切换备用模型
 * - timeout → 提示网络检查 + 自动重试
 * - context overflow → 提示 /compact
 *
 * 后端可在 StreamErrorPayload 直接携带 what/why/next 覆盖此推导。
 */

export interface RootCause {
  why: string
  next: string
  /** 自动重试延迟 (ms)。undefined = 不自动重试 */
  retryAfterMs?: number
  /** 是否建议切换备用模型 */
  suggestFallback?: boolean
  /** 是否建议压缩上下文 */
  suggestCompact?: boolean
}

interface Rule {
  test: RegExp
  why: string
  next: string
  retryAfterMs?: number
  suggestFallback?: boolean
  suggestCompact?: boolean
}

const RULES: Rule[] = [
  {
    test: /rate.?limit|429|too many requests/i,
    why: '命中提供商速率限制（HTTP 429）',
    next: '正在自动重试…',
    retryAfterMs: 5_000, // 5s 后自动重试
  },
  {
    test: /401|unauthor|api.?key|invalid.?key|auth/i,
    why: '认证失败（HTTP 401）：API key 无效或缺失',
    next: '在 Settings 中检查并重新填入 API key',
  },
  {
    test: /503|overloaded|unavailable|service unavailable/i,
    why: '模型服务暂时不可用（HTTP 503）',
    next: '正在尝试备用模型…',
    suggestFallback: true,
    retryAfterMs: 3_000,
  },
  {
    test: /network|econn|timeout|timed out|dns|fetch failed|connection/i,
    why: '网络层失败：连接超时或 DNS 解析出错',
    next: '正在自动重试…',
    retryAfterMs: 3_000,
  },
  {
    test: /context|too long|max.?token|length|truncat/i,
    why: '上下文超出模型最大长度',
    next: '运行 /compact 压缩会话，或切换到更长上下文的模型',
    suggestCompact: true,
  },
  {
    test: /model|not.?found|does not exist|deprecated/i,
    why: '请求模型不存在或无权限',
    next: '在 Settings 中选择当前账号可用的模型',
    suggestFallback: true,
  },
  {
    test: /quota|exceed|credit|余额|insufficient/i,
    why: '账号配额 / 额度耗尽',
    next: '充值或切换到其他账号 / 提供商',
    suggestFallback: true,
  },
  {
    test: /cancelled|stopped|abort/i,
    why: '用户在生成中途取消',
    next: '可重新发送以继续，或调整提示后重试',
  },
]

const FALLBACK: RootCause = {
  why: 'provider/流式阶段错误（F1），回复未落盘',
  next: '可重试；若持续出现，检查网络连通性或 API key',
  retryAfterMs: 3_000,
}

export function rootCause(message: string): RootCause {
  const m = (message || '').toLowerCase()
  for (const r of RULES) {
    if (r.test.test(m)) {
      return {
        why: r.why,
        next: r.next,
        retryAfterMs: r.retryAfterMs,
        suggestFallback: r.suggestFallback,
        suggestCompact: r.suggestCompact,
      }
    }
  }
  return FALLBACK
}
