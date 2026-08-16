import { describe, it, expect, vi, beforeEach } from 'vitest'

const invokeMock = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}))

import { fullCatalog, cliList, tauriList, unifiedCatalog, execCli, cliLookup } from './unified'

describe('api/unified — CLI ↔ app 统一命令桥', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('fullCatalog 调用 unified_tauri_full_catalog', async () => {
    invokeMock.mockResolvedValue([{ name: 'kb_search', backend: 'tauri' }])
    const r = await fullCatalog()
    expect(invokeMock).toHaveBeenCalledWith('unified_tauri_full_catalog', undefined)
    expect(r[0].name).toBe('kb_search')
  })

  it('cliList 调用 unified_cli_list', async () => {
    invokeMock.mockResolvedValue([{ name: '/help', backend: 'cli' }])
    const r = await cliList()
    expect(invokeMock).toHaveBeenCalledWith('unified_cli_list', undefined)
    expect(r[0].backend).toBe('cli')
  })

  it('tauriList 调用 unified_tauri_list', async () => {
    invokeMock.mockResolvedValue([])
    await tauriList()
    expect(invokeMock).toHaveBeenCalledWith('unified_tauri_list', undefined)
  })

  it('unifiedCatalog 调用 unified_command_catalog', async () => {
    invokeMock.mockResolvedValue([])
    await unifiedCatalog()
    expect(invokeMock).toHaveBeenCalledWith('unified_command_catalog', undefined)
  })

  it('execCli 传 input 调用 unified_cli_execute 并返回结构', async () => {
    invokeMock.mockResolvedValue({ success: true, message: 'help output', exit_code: 0, json: null })
    const r = await execCli('/help')
    expect(invokeMock).toHaveBeenCalledWith('unified_cli_execute', { input: '/help' })
    expect(r.success).toBe(true)
    expect(r.exit_code).toBe(0)
  })

  it('cliLookup 调用 unified_cli_lookup', async () => {
    invokeMock.mockResolvedValue({ name: '/config', backend: 'cli' })
    const r = await cliLookup('config')
    expect(invokeMock).toHaveBeenCalledWith('unified_cli_lookup', { name: 'config' })
    expect(r?.name).toBe('/config')
  })

  it('execCli 失败时抛出 (调用方兜底)', async () => {
    invokeMock.mockRejectedValue(new Error('empty command input'))
    await expect(execCli('  ')).rejects.toThrow('empty command input')
  })
})