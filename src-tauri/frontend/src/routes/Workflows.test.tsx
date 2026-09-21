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

/** mock domain_call（workflow 域），返回 {ok, data} 信封 */
function mockWorkflowDomain(handlers: Record<string, (args: any) => unknown>) {
  return mockCommand('domain_call', async (req: any) => {
    const fn = handlers[`${req.domain}/${req.action}`]
    if (!fn) throw new Error(`unexpected domain call: ${req.domain}/${req.action}`)
    return { ok: true, data: await fn(req.args) }
  })
}

function renderPage() {
  return render(() => (
    <MemoryRouter>
      <Route path="/" component={Workflows} />
    </MemoryRouter>
  ))
}

describe('Workflows page (B5 直连后端)', () => {
  it('渲染工作流卡列表与步骤数/标签', async () => {
    mockWorkflowDomain({ 'workflow/list': async () => seed })
    renderPage()
    expect(await screen.findByLabelText('工作流 每日吸收')).toBeTruthy()
    expect(screen.getByText('1 步骤')).toBeTruthy()
    expect(screen.getByText('#kb')).toBeTruthy()
  })

  it('运行按钮触发 workflow_run 并轮询 status 至完成恢复按钮', async () => {
    // 注：同一命令重复注册会覆盖，前面的 mockWorkflowDomain 会被下面的内联 mock 取代，
    // 故此处只保留内联版本（覆盖 list/run/status 三个动作）
    let polls = 0
    mockCommand('domain_call', async (req: any) => {
      if (req.action === 'status') {
        polls += 1
        return {
          ok: true,
          data: {
            id: 'run-9', workflow_id: 'wf-1',
            status: polls >= 2 ? 'completed' : 'running',
            current_step: 1, progress_pct: polls >= 2 ? 100 : 40,
            started_at: Date.now(),
          },
        }
      }
      if (req.action === 'list') return { ok: true, data: seed }
      if (req.action === 'run') {
        return {
          ok: true,
          data: {
            id: 'run-9', workflow_id: 'wf-1', status: 'running',
            current_step: 0, progress_pct: 0, started_at: Date.now(),
          },
        }
      }
      throw new Error(`unexpected domain call: ${req.domain}/${req.action}`)
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
    mockWorkflowDomain({ 'workflow/list': async () => [] })
    const { unmount } = renderPage()
    expect(await screen.findByText('暂无工作流')).toBeTruthy()
    unmount()
    resetInvokeMock()
    mockWorkflowDomain({
      'workflow/list': async () => {
        throw new Error('state lock')
      },
    })
    renderPage()
    expect(await screen.findByText('state lock')).toBeTruthy()
  })
})
