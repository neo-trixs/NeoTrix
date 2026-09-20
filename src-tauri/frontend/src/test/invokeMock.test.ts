import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mockCommand, mockCommandOnce, resetInvokeMock } from './invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('./invokeMock')
  return { invoke: mockInvokeImpl }
})

import { call } from '../api/client'

describe('test/invokeMock — 契约 mock 基础设施', () => {
  beforeEach(() => resetInvokeMock())
  afterEach(() => resetInvokeMock())

  it('已注册命令走 handler 并记录调用', async () => {
    const h = mockCommand('agent_status', async (args) => ({ ok: true, args }))
    const r = await call('agent_status', { force: true })
    expect(r).toEqual({ ok: true, args: { force: true } })
    expect(h.calledTimes()).toBe(1)
    expect(h.lastArgs()).toEqual({ force: true })
  })

  it('未注册命令抛「未 mock」错误（防遗漏）', async () => {
    await expect(call('some_unregistered')).rejects.toThrow('未 mock 的命令: some_unregistered')
  })

  it('mockCommandOnce 单次生效后失效', async () => {
    mockCommand('agent_status', async () => ({ stable: true }))
    mockCommandOnce('agent_status', async () => ({ once: true }))
    expect(await call('agent_status')).toEqual({ once: true })
    expect(await call('agent_status')).toEqual({ stable: true })
  })

  it('重复注册覆盖 handler', async () => {
    mockCommand('cmd', async () => 1)
    mockCommand('cmd', async () => 2)
    expect(await call('cmd')).toBe(2)
  })
})