import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@solidjs/testing-library'
import { MemoryRouter, Route } from '@solidjs/router'
import { KnowledgeBase } from './KnowledgeBase'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

beforeEach(() => resetInvokeMock())

function renderPage() {
  return render(() => (
    <MemoryRouter>
      <Route path="/" component={KnowledgeBase} />
    </MemoryRouter>
  ))
}

describe('KnowledgeBase page (tauri 数据源)', () => {
  it('渲染标题、搜索框与新建按钮', () => {
    mockCommand('kb_doc_list', () => [])
    renderPage()
    expect(screen.getByRole('heading', { name: /知识库/ })).toBeTruthy()
    expect(screen.getByRole('searchbox', { name: '搜索知识库' })).toBeTruthy()
    expect(screen.getByRole('button', { name: '新建知识库' })).toBeTruthy()
  })

  it('空库渲染空态引导', async () => {
    mockCommand('kb_doc_list', () => [])
    renderPage()
    expect(await screen.findByText(/还没有知识库/)).toBeTruthy()
  })

  it('kb_doc_list 聚合为库卡片 (默认库名映射)', async () => {
    mockCommand('kb_doc_list', () => [
      { doc_id: 'kbdoc-a', title: '设计文档A', library: null, chunk_count: 4, total_chars: 1200, status: 'ready', created_at: Date.now() },
      { doc_id: 'kbdoc-b', title: '规则B', library: 'rules', chunk_count: 2, total_chars: 800, status: 'ready', created_at: Date.now() },
    ])
    renderPage()
    expect(await screen.findByText('默认库')).toBeTruthy()
    expect(screen.getByText('rules')).toBeTruthy()
    // docCount 聚合: 默认库 1 文档
    expect(screen.getAllByText('1 文档').length).toBeGreaterThan(0)
  })
})
