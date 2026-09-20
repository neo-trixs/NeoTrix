/* ════════════════════════════════════════════
   api/query.ts — 轻量查询缓存层（对齐 iPolloWork query-client 思想）
   多个组件重复轮询同一后端数据（agent_status / health_report /
   provider_config）时，经此处共享一份缓存 + TTL，避免各自 fetch。
   仅做进程内简单缓存，不做请求去重/重试（那些仍是调用方职责）。
   规则：状态突变后调用 invalidate(key) 主动失效，防脏读。
   ════════════════════════════════════════════ */

export interface QueryOptions<T> {
  /** 缓存有效期毫秒，默认 0 = 不做缓存（等同直接调用） */
  ttlMs?: number
  /** 是否绕过缓存强制刷新（默认 false） */
  force?: boolean
  /** 自定义 key（默认用 command 名） */
  key?: string
}

interface CacheEntry<T> {
  value: T
  expiresAt: number
  inFlight: Promise<T> | null
}

const cache = new Map<string, CacheEntry<unknown>>()

/** 取缓存（读路径） */
export function peek<T>(key: string): T | undefined {
  const e = cache.get(key)
  if (!e) return undefined
  if (Date.now() >= e.expiresAt) {
    cache.delete(key)
    return undefined
  }
  return e.value as T
}

/** 主动失效缓存（状态突变后调用） */
export function invalidate(...keys: string[]): void {
  for (const k of keys) cache.delete(k)
}

/** 清空全部缓存（登出/切换 workspace 等） */
export function invalidateAll(): void {
  cache.clear()
}

/** 带缓存的查询：命中 TTL 内缓存直接返回，否则调 fetcher。
 *  并发同 key 请求共享同一个 in-flight promise（防重复 fetch）。 */
export async function query<T>(
  key: string,
  fetcher: () => Promise<T>,
  options: QueryOptions<T> = {},
): Promise<T> {
  const { ttlMs = 0, force = false } = options
  if (!force && ttlMs > 0) {
    const hit = peek<T>(key)
    if (hit !== undefined) return hit
    const existing = cache.get(key) as CacheEntry<T> | undefined
    if (existing?.inFlight) return existing.inFlight
  }

  const p = fetcher()
  const entry: CacheEntry<T> = {
    value: undefined as unknown as T,
    expiresAt: ttlMs > 0 ? Date.now() + ttlMs : Date.now(),
    inFlight: p,
  }
  cache.set(key, entry as CacheEntry<unknown>)

  try {
    const value = await p
    entry.value = value
    entry.expiresAt = ttlMs > 0 ? Date.now() + ttlMs : Date.now()
    return value
  } finally {
    entry.inFlight = null
  }
}
