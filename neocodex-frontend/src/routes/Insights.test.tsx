import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@solidjs/testing-library'
import { MemoryRouter, Route } from '@solidjs/router'
import { Insights } from './Insights'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

function renderPage() {
  return render(() => (
    <MemoryRouter>
      <Route path="/" component={Insights} />
    </MemoryRouter>
  ))
}

describe('Insights page', () => {
  it('渲染标题、返回按钮与刷新按钮', async () => {
    renderPage()
    expect(screen.getByRole('heading', { name: /洞察/ })).toBeTruthy()
    expect(screen.getByRole('button', { name: '返回对话' })).toBeTruthy()
    expect(screen.getByRole('button', { name: '刷新洞察' })).toBeTruthy()
  })

  it('成本卡渲染预算使用率进度条 (a11y progressbar)', async () => {
    renderPage()
    // onMount 触发 loading 门控，内容异步回归 — 全部用 findBy*
    const bar = await screen.findByRole('progressbar', { name: '预算使用率' })
    expect(bar.getAttribute('aria-valuenow')).toBe('7') // 3.42 / 50 ≈ 6.84 → 7
    expect(screen.getByText('$3.42')).toBeTruthy()
    expect(screen.getByText('cli-session/claude-code')).toBeTruthy()
  })

  it('用量账本带诚实标注且含 cli-session 行', async () => {
    renderPage()
    expect(await screen.findByText('活动记录，非权威账单 — 与服务商实际计费可能存在差异')).toBeTruthy()
    expect(screen.getByText('cli-session')).toBeTruthy()
    expect(screen.getByText('llm7')).toBeTruthy()
  })

  it('周摘要渲染生产力评分徽章与洞察条目', async () => {
    renderPage()
    expect(await screen.findByLabelText('生产力评分 78')).toBeTruthy()
    expect(screen.getByText(/专注时段稳定/)).toBeTruthy()
  })
})
