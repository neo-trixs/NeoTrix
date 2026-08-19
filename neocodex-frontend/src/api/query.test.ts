import { describe, it, expect, vi, beforeEach } from 'vitest'
import { query, peek, invalidate, invalidateAll } from './query'

describe('api/query — 轻量查询缓存', () => {
  beforeEach(() => {
    invalidateAll()
  })

  it('ttl>0 时命中缓存不重复调用 fetcher', async () => {
    const fetcher = vi.fn().mockResolvedValue('v1')
    const a = await query('k', fetcher, { ttlMs: 1000 })
    const b = await query('k', fetcher, { ttlMs: 1000 })
    expect(a).toBe('v1')
    expect(b).toBe('v1')
    expect(fetcher).toHaveBeenCalledTimes(1)
  })

  it('force 绕过缓存强制刷新', async () => {
    const fetcher = vi.fn().mockResolvedValueOnce('v1').mockResolvedValueOnce('v2')
    await query('k', fetcher, { ttlMs: 1000 })
    const b = await query('k', fetcher, { ttlMs: 1000, force: true })
    expect(b).toBe('v2')
    expect(fetcher).toHaveBeenCalledTimes(2)
  })

  it('ttl 过期后重新调用 fetcher', async () => {
    vi.useFakeTimers()
    try {
      const fetcher = vi.fn().mockResolvedValue('v')
      await query('k', fetcher, { ttlMs: 100 })
      vi.advanceTimersByTime(150)
      await query('k', fetcher, { ttlMs: 100 })
      expect(fetcher).toHaveBeenCalledTimes(2)
    } finally {
      vi.useRealTimers()
    }
  })

  it('并发同 key 共享 in-flight（防重复 fetch）', async () => {
    let resolveFn: ((v: string) => void) | undefined
    const fetcher = vi.fn().mockImplementation(() => new Promise<string>((r) => (resolveFn = r)))
    const p1 = query('k', fetcher, { ttlMs: 1000 })
    const p2 = query('k', fetcher, { ttlMs: 1000 })
    resolveFn?.('v')
    const [r1, r2] = await Promise.all([p1, p2])
    expect(r1).toBe('v')
    expect(r2).toBe('v')
    expect(fetcher).toHaveBeenCalledTimes(1)
  })

  it('invalidate 主动失效', async () => {
    const fetcher = vi.fn().mockResolvedValueOnce('v1').mockResolvedValueOnce('v2')
    await query('k', fetcher, { ttlMs: 5000 })
    invalidate('k')
    const b = await query('k', fetcher, { ttlMs: 5000 })
    expect(b).toBe('v2')
    expect(fetcher).toHaveBeenCalledTimes(2)
  })

  it('peek 未命中返回 undefined', () => {
    expect(peek('absent')).toBeUndefined()
  })

  it('ttl=0 不做缓存（等同直接调用）', async () => {
    const fetcher = vi.fn().mockResolvedValue('v')
    await query('k', fetcher)
    await query('k', fetcher)
    expect(fetcher).toHaveBeenCalledTimes(2)
  })
})
