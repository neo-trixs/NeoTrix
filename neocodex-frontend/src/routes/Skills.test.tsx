import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, within } from '@solidjs/testing-library'
import { MemoryRouter, Route } from '@solidjs/router'
import { Skills } from './Skills'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

const seed = {
  skills: [
    { name: 'rev-officer', path: '/s/rev/officer', description: '全量审查', line_count: 320, domain: 'shield' },
    { name: 'dev-implementer', path: '/s/dev/implementer', description: 'TDD 实现', line_count: 210, domain: 'act' },
    { name: 'kb-search', path: '/s/kb/search', description: '知识检索', line_count: 88, domain: 'memory' },
  ],
  total: 3,
}

beforeEach(() => resetInvokeMock())

function renderPage() {
  return render(() => (
    <MemoryRouter>
      <Route path="/" component={Skills} />
    </MemoryRouter>
  ))
}

describe('Skills page (B4 直连后端)', () => {
  it('加载后按域分组渲染技能卡片与总数', async () => {
    mockCommand('skill_list', () => seed)
    renderPage()
    expect(await screen.findByText('3 个技能')).toBeTruthy()
    expect(await screen.findByLabelText('技能域 shield')).toBeTruthy()
    expect(screen.getByLabelText('技能 rev-officer')).toBeTruthy()
    expect(screen.getByLabelText('技能域 act')).toBeTruthy()
  })

  it('搜索输入防抖后调用 skill_search 并渲染过滤结果', async () => {
    vi.useFakeTimers()
    try {
      mockCommand('skill_list', () => seed)
      mockCommand('skill_search', (args) => {
        const q = String(args?.query ?? '')
        return seed.skills.filter((s) => s.name.includes(q))
      })
      renderPage()
      await screen.findByText('rev-officer')
      const input = screen.getByRole('searchbox', { name: '搜索技能' })
      // 模拟输入触发 onInput（Solid 需要 dispatch input 事件）
      const setV = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')!.set!
      setV.call(input, 'rev')
      input.dispatchEvent(new Event('input', { bubbles: true }))
      await vi.advanceTimersByTimeAsync(400)
      expect(screen.queryByText('kb-search')).toBeNull()
      expect(screen.getByText('rev-officer')).toBeTruthy()
    } finally {
      vi.useRealTimers()
    }
  })

  it('点击卡片经 skill_get 打开详情侧滑', async () => {
    mockCommand('skill_list', () => seed)
    mockCommand('skill_get', (args) => seed.skills.find((s) => s.name === String(args?.name)))
    renderPage()
    const card = await screen.findByText('dev-implementer')
    card.click()
    const dialog = await screen.findByRole('dialog', { name: '技能详情 dev-implementer' })
    expect(dialog).toBeTruthy()
    expect(screen.getByText('/s/dev/implementer')).toBeTruthy()
    // 描述同时出现在卡片与详情中 — 断言限定在 dialog 内
    expect(within(dialog).getByText('TDD 实现')).toBeTruthy()
  })

  it('后端错误渲染错误态与重试按钮', async () => {
    mockCommand('skill_list', () => {
      throw new Error('skills dir missing')
    })
    renderPage()
    expect(await screen.findByText('skills dir missing')).toBeTruthy()
    expect(screen.getByRole('button', { name: '重试' })).toBeTruthy()
  })
})
