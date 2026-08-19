import { describe, it, expect, beforeEach } from 'vitest'
import { mockInvokeImpl, mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvokeImpl,
}))

import { fullCatalog, cliList, tauriList, unifiedCatalog, execCli, cliLookup } from './unified'

describe('api/unified — CLI ↔ app 统一命令桥（经 invokeMock 契约）', () => {
  beforeEach(() => resetInvokeMock())

  it('fullCatalog 调用 unified_tauri_full_catalog', async () => {
    const c = mockCommand('unified_tauri_full_catalog', async () => [{ name: 'kb_search', backend: 'tauri' }])
    const r = await fullCatalog()
    expect(c.calledTimes()).toBe(1)
    expect(r[0].name).toBe('kb_search')
  })

  it('cliList 调用 unified_cli_list', async () => {
    mockCommand('unified_cli_list', async () => [{ name: '/help', backend: 'cli' }])
    const r = await cliList()
    expect(r[0].backend).toBe('cli')
  })

  it('tauriList 调用 unified_tauri_list', async () => {
    mockCommand('unified_tauri_list', async () => [])
    const r = await tauriList()
    expect(r).toEqual([])
  })

  it('unifiedCatalog 调用 unified_command_catalog', async () => {
    mockCommand('unified_command_catalog', async () => [])
    const r = await unifiedCatalog()
    expect(r).toEqual([])
  })

  it('execCli 传 input 调用 unified_cli_execute 并返回结构', async () => {
    const c = mockCommand('unified_cli_execute', async (args) => {
      expect(args).toEqual({ input: '/help' })
      return { success: true, message: 'help output', exit_code: 0, json: null }
    })
    const r = await execCli('/help')
    expect(c.calledTimes()).toBe(1)
    expect(r.success).toBe(true)
    expect(r.exit_code).toBe(0)
  })

  it('cliLookup 调用 unified_cli_lookup', async () => {
    const c = mockCommand('unified_cli_lookup', async (args) => {
      expect(args).toEqual({ name: 'config' })
      return { name: '/config', backend: 'cli' }
    })
    const r = await cliLookup('config')
    expect(c.calledTimes()).toBe(1)
    expect(r?.name).toBe('/config')
  })

  it('execCli 失败时抛出 (调用方兜底)', async () => {
    mockCommand('unified_cli_execute', async () => { throw new Error('empty command input') })
    await expect(execCli('  ')).rejects.toThrow('empty command input')
  })
})