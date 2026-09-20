import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, waitFor } from '@solidjs/testing-library'
import { GitPanel } from './GitPanel'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    switch (cmd) {
      case 'neocodex_git_status':
        return Promise.resolve({ branch: 'main', status: [] })
      case 'neocodex_get_diff':
        return Promise.resolve({
          files: [
            {
              path: 'src/foo.rs',
              hunks: [{ lines: [
                { t: 'add', o: null, n: 1, s: '+fn new() {}' },
                { t: 'del', o: 1, n: null, s: '-fn old() {}' },
              ] }],
            },
          ],
        })
      case 'neocodex_git_staged_files':
        return Promise.resolve([])
      case 'neocodex_git_branch':
        return Promise.resolve(['main'])
      default:
        return Promise.resolve(null)
    }
  }),
}))

describe('GitPanel 渲染回归（P1-1：diff 列表必须渲染）', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('加载完成后显示 diff 文件列表而非永久 spinner', async () => {
    render(() => <GitPanel open onClose={() => {}} />)
    // 初始加载中：显示 spinner
    expect(document.querySelector('.animate-spin')).toBeTruthy()
    // 加载完成：diff 文件路径出现，spinner 消失
    await waitFor(() => {
      expect(document.body.textContent).toContain('src/foo.rs')
    })
    expect(document.querySelector('.animate-spin')).toBeNull()
    // 空态文案不应出现
    expect(document.body.textContent).not.toContain('工作区干净')
  })

  it('工作区干净时显示空态而非 spinner', async () => {
    const mockedInvoke = (await import('@tauri-apps/api/core')).invoke as ReturnType<typeof vi.fn>
    mockedInvoke.mockImplementation((cmd: string) => {
      switch (cmd) {
        case 'neocodex_get_diff':
          return Promise.resolve({ files: [] })
        case 'neocodex_git_status':
          return Promise.resolve({ branch: 'main', status: [] })
        case 'neocodex_git_staged_files':
          return Promise.resolve([])
        case 'neocodex_git_branch':
          return Promise.resolve(['main'])
        default:
          return Promise.resolve(null)
      }
    })
    render(() => <GitPanel open onClose={() => {}} />)
    await waitFor(() => {
      expect(document.body.textContent).toContain('工作区干净')
    })
  })
})