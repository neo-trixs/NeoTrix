import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@solidjs/testing-library'
import { MemoryRouter, Route } from '@solidjs/router'
import { Workflows } from './Workflows'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

beforeEach(() => resetInvokeMock())

const seed = [
  {
    id: 'wf-1', name: '每日吸收', description: 'KB 批量吸收', version: 1,
    steps: [{ id: 's1', kind: 'shell', name: 'run', params: {}, depends_on: [], timeout_secs: 60, retry_count: 0 }],
    created_at: Date.now(), updated_at: Date.now(), tags: ['kb'],
  },
]

function renderPage() {
  return render(() => (
    <MemoryRouter>
      <Route path="/" component={Workflows} />
    </MemoryRouter>
  ))
}

describe('Workflows page (B5 直连后端)', () => {
  it('渲染工作流卡列表与步骤数/标签', async () => {
    mockCommand('workflow_list', () => seed)
    renderPage()
    expect(await screen.findByLabelText('工作流 每日吸收')).toBeTruthy()
    expect(screen.getByText('1 步骤')).toBeTruthy()
    expect(screen.getByText('#kb')).toBeTruthy()
  })

  it('运行按钮触发 workflow_run 并轮询 run_status 至完成恢复按钮', async () => {
    mockCommand('workflow_list', () => seed)
    let polls = 0
    mockCommand('workflow_run', () => 'run-9')
    mockCommand('workflow_run_status', () => {
      polls += 1
      return {
        id: 'run-9', workflow_id: 'wf-1',
        status: polls >= 2 ? 'completed' : 'running',
        current_step: 1, progress_pct: polls >= 2 ? 100 : 40,
        started_at: Date.now(),
      }
    })
    renderPage()
    // 页面轮询间隔 1s, 真实计时器 + 宽松超时
    const btn = await screen.findByRole('button', { name: '运行 每日吸收' }) as HTMLButtonElement
    btn.click()
    await screen.findByLabelText(/运行进度/, {}, { timeout: 4000 })
    // 运行中按钮禁用
    expect(screen.getByRole('button', { name: '运行 每日吸收' }).hasAttribute('disabled')).toBe(true)
    // 完成后轮询停止, 按钮恢复可用
    await vi.waitFor(() => {
      expect((screen.getByRole('button', { name: '运行 每日吸收' }) as HTMLButtonElement).disabled).toBe(false)
    }, { timeout: 6000 })
    expect(polls).toBeGreaterThanOrEqual(2)
  }, 15000)

  it('空列表渲染空态; 错误渲染错误态', async () => {
    mockCommand('workflow_list', () => [])
    const { unmount } = renderPage()
    expect(await screen.findByText('暂无工作流')).toBeTruthy()
    unmount()
    resetInvokeMock()
    mockCommand('workflow_list', () => { throw new Error('state lock') })
    renderPage()
    expect(await screen.findByText('state lock')).toBeTruthy()
  })
})
