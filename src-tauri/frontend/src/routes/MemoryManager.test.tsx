import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@solidjs/testing-library'
import { MemoryRouter, Route } from '@solidjs/router'
import { MemoryManager } from './MemoryManager'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

beforeEach(() => resetInvokeMock())

const stats = { total_entries: 42, total_categories: 5, avg_confidence: 0.87, memory_usage_bytes: 2048 }
const timeline = [
  { date: '2026-08-24', entries_created: 6, entries_accessed: 12, top_topic: 'absorption' },
  { date: '2026-08-25', entries_created: 9, entries_accessed: 20, top_topic: 'app-features' },
]

function renderPage() {
  return render(() => (
    <MemoryRouter>
      <Route path="/" component={MemoryManager} />
    </MemoryRouter>
  ))
}

describe('MemoryManager page (B6 直连后端)', () => {
  it('渲染统计四格与时间线柱状', async () => {
    mockCommand('memory_stats', () => stats)
    mockCommand('memory_timeline', () => timeline)
    renderPage()
    expect(await screen.findByLabelText('记忆统计')).toBeTruthy()
    expect(screen.getByText('42')).toBeTruthy()
    expect(screen.getByText('87%')).toBeTruthy()
    expect(screen.getByText('2.0 KB')).toBeTruthy()
    const tl = screen.getByLabelText('记忆时间线')
    expect(tl.querySelectorAll('[title^="2026-08-25"]').length).toBe(1)
  })

  it('搜索提交调用 memory_search 并渲染结果与 kind 徽章', async () => {
    mockCommand('memory_stats', () => stats)
    mockCommand('memory_timeline', () => timeline)
    mockCommand('memory_search', (args) => {
      const query = String(args?.query ?? '')
      return query === '吸收' ? [{
        id: 'e1', kind: 'pattern', content: '吸收纪律要点', summary: '', source: 'self-session',
        confidence: 0.9, created_at: Date.now(), last_accessed_at: Date.now(),
        access_count: 3, tags: ['R-P79'], is_pinned: true,
      }] : []
    })
    renderPage()
    await screen.findByLabelText('记忆统计')
    const input = screen.getByRole('searchbox', { name: '搜索记忆' })
    const setV = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')!.set!
    setV.call(input, '吸收')
    input.dispatchEvent(new Event('input', { bubbles: true }))
    screen.getByRole('button', { name: /搜索/ }).click()
    expect(await screen.findByText('1 条结果')).toBeTruthy()
    expect(screen.getByText('吸收纪律要点')).toBeTruthy()
    expect(screen.getByText('pattern')).toBeTruthy()
    expect(screen.getByText('#R-P79')).toBeTruthy()
  })

  it('后端错误渲染错误态', async () => {
    mockCommand('memory_stats', () => { throw new Error('db locked') })
    mockCommand('memory_timeline', () => [])
    renderPage()
    expect(await screen.findByText('db locked')).toBeTruthy()
    expect(screen.getByRole('button', { name: '重试' })).toBeTruthy()
  })
})
