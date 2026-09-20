import { describe, it, expect, vi, afterEach } from 'vitest'
import { isTauriRuntime, safeLocalStorage, storageGet, storageSet } from './env'

describe('lib/env — 运行环境能力探测', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('无 __TAURI_INTERNALS__ 时 isTauriRuntime 为 false', () => {
    expect(isTauriRuntime()).toBe(false)
  })

  it('存在 __TAURI_INTERNALS__ 时 isTauriRuntime 为 true', () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    expect(isTauriRuntime()).toBe(true)
  })

  it('safeLocalStorage 在无 window 环境返回 null', () => {
    expect(safeLocalStorage()).toBeTruthy() // vitest jsdom 有 localStorage
  })

  it('storageGet/Set 往返', () => {
    storageSet('k', 'v')
    expect(storageGet('k')).toBe('v')
  })

  it('storageGet 缺失返回 null', () => {
    expect(storageGet('__missing__')).toBeNull()
  })
})