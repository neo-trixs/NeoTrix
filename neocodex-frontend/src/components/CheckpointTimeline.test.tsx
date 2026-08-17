import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, waitFor } from '@solidjs/testing-library'
import { CheckpointTimeline } from './CheckpointTimeline'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === 'neocodex_checkpoint_list') {
      return Promise.resolve([
        { id: 'cp-1', created_at: Date.now() - 60000, message_count: 4 },
        { id: 'cp-2', created_at: Date.now() - 3000, message_count: 2 },
      ])
    }
    return Promise.resolve([])
  }),
}))

describe('CheckpointTimeline 渲染回归（P1-2：快照列表必须渲染）', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('加载完成后显示快照列表而非永久 spinner', async () => {
    render(() => <CheckpointTimeline open sessionId="s-1" onClose={() => {}} />)
    // 初始加载中：spinner 出现
    expect(document.querySelector('.animate-spin')).toBeTruthy()
    // 加载完成：快照条目出现，spinner 消失
    await waitFor(() => {
      expect(document.body.textContent).toContain('最新快照')
      expect(document.body.textContent).toContain('2 条')
    })
    expect(document.querySelector('.animate-spin')).toBeNull()
    expect(document.body.textContent).not.toContain('暂无快照')
  })

  it('无快照时显示空态而非 spinner', async () => {
    const mockedInvoke = (await import('@tauri-apps/api/core')).invoke as ReturnType<typeof vi.fn>
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'neocodex_checkpoint_list') return Promise.resolve([])
      return Promise.resolve([])
    })
    render(() => <CheckpointTimeline open sessionId="s-1" onClose={() => {}} />)
    await waitFor(() => {
      expect(document.body.textContent).toContain('暂无快照')
    })
  })
})