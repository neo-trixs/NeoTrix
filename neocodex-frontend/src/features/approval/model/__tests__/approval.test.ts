import { describe, it, expect } from 'vitest'
import { isExpired, canApprove, transition, autoExpire, type ApprovalRequest } from '../index'

const base: ApprovalRequest = { id: '1', title: 'Test', description: 'desc', status: 'pending', createdAt: Date.now(), expiresAt: Date.now() + 10000 }

describe('features/approval', () => {
  it('isExpired 判断过期', () => {
    expect(isExpired({ ...base, expiresAt: Date.now() - 1000 })).toBe(true)
    expect(isExpired(base)).toBe(false)
    expect(isExpired({ ...base, expiresAt: undefined })).toBe(false)
  })

  it('canApprove 仅 pending 且未过期', () => {
    expect(canApprove(base)).toBe(true)
    expect(canApprove({ ...base, status: 'approved' })).toBe(false)
    expect(canApprove({ ...base, expiresAt: Date.now() - 1000 })).toBe(false)
  })

  it('transition pending -> approved', () => {
    const next = transition(base, 'approved')
    expect(next.status).toBe('approved')
  })

  it('transition 非 pending 抛错', () => {
    expect(() => transition({ ...base, status: 'approved' }, 'rejected')).toThrow()
  })

  it('transition 过期抛错', () => {
    expect(() => transition({ ...base, expiresAt: Date.now() - 1000 }, 'approved')).toThrow()
  })

  it('autoExpire 自动过期', () => {
    const expired = autoExpire({ ...base, expiresAt: Date.now() - 1000 })
    expect(expired.status).toBe('expired')
    expect(autoExpire(base).status).toBe('pending')
  })
})
