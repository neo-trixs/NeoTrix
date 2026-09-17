import { describe, it, expect } from 'vitest'
import { createDecision, isValidDecision, labelForAction } from '../index'

describe('features/approve-tool', () => {
  it('createDecision approve 无需 reason', () => {
    const d = createDecision('t1', 'approve')
    expect(d.action).toBe('approve')
    expect(d.toolCallId).toBe('t1')
  })

  it('createDecision reject 需 reason', () => {
    expect(() => createDecision('t1', 'reject')).toThrow()
    expect(() => createDecision('t1', 'reject', '   ')).toThrow()
    const d = createDecision('t1', 'reject', 'unsafe')
    expect(d.reason).toBe('unsafe')
  })

  it('isValidDecision 校验', () => {
    expect(isValidDecision({ toolCallId: '', action: 'approve', decidedAt: Date.now() })).toBe(false)
    expect(isValidDecision({ toolCallId: 't1', action: 'reject', decidedAt: Date.now() })).toBe(false)
    expect(isValidDecision({ toolCallId: 't1', action: 'approve', decidedAt: Date.now() })).toBe(true)
  })

  it('labelForAction 映射', () => {
    expect(labelForAction('approve')).toBe('批准')
    expect(labelForAction('reject')).toBe('拒绝')
    expect(labelForAction('defer')).toBe('稍后')
  })
})
