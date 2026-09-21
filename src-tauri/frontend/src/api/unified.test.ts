import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

import { fullCatalog, cliList, tauriList, unifiedCatalog, execCli, cliLookup } from './unified'

describe('api/unified — chat-first 命令桥（经 invokeMock 契约）', () => {
  beforeEach(() => {
    resetInvokeMock()
    // domain.call 要求 Tauri 宿主环境；测试中桩化运行时标识
    ;(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {}
  })

  it('fullCatalog 经 domain_call(plugin/list) 返回目录', async () => {
    const c = mockCommand('domain_call', async (args) => {
      expect(args).toEqual({ domain: 'plugin', action: 'list', args: {} })
      return { ok: true, data: [{ id: 'kb', name: 'kb_search' }] }
    })
    const r = await fullCatalog()
    expect(c.calledTimes()).toBe(1)
    expect(r[0].name).toBe('kb_search')
    expect(r[0].backend).toBe('plugin')
  })

  it('cliList 经 domain_call(cli/list)', async () => {
    mockCommand('domain_call', async () => ({ ok: true, data: [{ name: '/help', description: '', aliases: [] }] }))
    const r = await cliList()
    expect(r[0].backend).toBe('cli')
  })

  it('tauriList 委托 cliList', async () => {
    mockCommand('domain_call', async () => ({ ok: true, data: [] }))
    const r = await tauriList()
    expect(r).toEqual([])
  })

  it('unifiedCatalog 委托 cliList', async () => {
    mockCommand('domain_call', async () => ({ ok: true, data: [] }))
    const r = await unifiedCatalog()
    expect(r).toEqual([])
  })

  it('execCli 传 input 经 domain_call(cli/exec) 并返回结构', async () => {
    const c = mockCommand('domain_call', async (args) => {
      expect(args).toEqual({ domain: 'cli', action: 'exec', args: { command: '/help' } })
      return { ok: true, data: { success: true, message: 'help output' } }
    })
    const r = await execCli('/help')
    expect(c.calledTimes()).toBe(1)
    expect(r.success).toBe(true)
    expect(r.exit_code).toBe(0)
  })

  it('cliLookup 在列表中查找', async () => {
    const c = mockCommand('domain_call', async () => ({ ok: true, data: [{ name: '/config', description: '', aliases: [] }] }))
    const r = await cliLookup('/config')
    expect(c.calledTimes()).toBe(1)
    expect(r?.name).toBe('/config')
  })

  it('cliLookup 未命中返回 null', async () => {
    mockCommand('domain_call', async () => ({ ok: true, data: [] }))
    const r = await cliLookup('config')
    expect(r).toBeNull()
  })

  it('execCli 失败时抛出 (调用方兜底)', async () => {
    mockCommand('domain_call', async () => { throw new Error('empty command input') })
    await expect(execCli('  ')).rejects.toThrow('empty command input')
  })
})
