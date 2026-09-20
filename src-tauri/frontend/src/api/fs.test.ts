import { describe, it, expect, vi, beforeEach } from 'vitest'

const saveMock = vi.fn()
const openMock = vi.fn()
const writeTextFileMock = vi.fn()
vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: (...a: unknown[]) => saveMock(...a),
  open: (...a: unknown[]) => openMock(...a),
}))
vi.mock('@tauri-apps/plugin-fs', () => ({
  writeTextFile: (...a: unknown[]) => writeTextFileMock(...a),
}))

import { saveFileDialog, openFileDialog, writeTextFileAt } from './fs'

describe('api/fs — 文件对话框封装', () => {
  beforeEach(() => {
    vi.unstubAllGlobals()
    saveMock.mockReset()
    openMock.mockReset()
    writeTextFileMock.mockReset()
  })

  it('非 Tauri 宿主 saveFileDialog 返回 null（不调 tauri）', async () => {
    const r = await saveFileDialog()
    expect(r).toBeNull()
    expect(saveMock).not.toHaveBeenCalled()
  })

  it('非 Tauri 宿主 openFileDialog 返回 null', async () => {
    const r = await openFileDialog()
    expect(r).toBeNull()
    expect(openMock).not.toHaveBeenCalled()
  })

  it('非 Tauri 宿主 writeTextFileAt 抛错', async () => {
    await expect(writeTextFileAt('p', 'c')).rejects.toThrow()
  })

  it('Tauri 宿主 saveFileDialog 传 defaultPath/filters', async () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    saveMock.mockResolvedValue('/tmp/x.json')
    const r = await saveFileDialog({ defaultPath: 'a.json', filters: [{ name: 'JSON', extensions: ['json'] }] })
    expect(r).toBe('/tmp/x.json')
    expect(saveMock).toHaveBeenCalledWith(expect.objectContaining({ defaultPath: 'a.json' }))
  })

  it('Tauri 宿主用户取消 save 返回 null', async () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    saveMock.mockResolvedValue(null)
    expect(await saveFileDialog()).toBeNull()
  })
})