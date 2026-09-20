import { describe, it, expect } from 'vitest'
import { rootCause } from './errorRootCause'

describe('rootCause', () => {
  it('速率限制映射到 429 根因', () => {
    const r = rootCause('HTTP 429 Too Many Requests')
    expect(r.why).toContain('速率限制')
  })
  it('认证失败映射到 key 提示', () => {
    const r = rootCause('401 Unauthorized: invalid api key')
    expect(r.why).toContain('认证失败')
    expect(r.next).toContain('API key')
  })
  it('网络错误映射到连通性提示', () => {
    const r = rootCause('fetch failed: connection timeout')
    expect(r.why).toContain('网络层')
  })
  it('未知错误回退到通用文案', () => {
    const r = rootCause('some weird thing')
    expect(r.next).toContain('重试')
  })
})
