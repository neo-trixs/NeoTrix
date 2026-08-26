import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@solidjs/testing-library'
import { MemoryRouter, Route } from '@solidjs/router'
import { KnowledgeBase } from './KnowledgeBase'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

function renderPage() {
  return render(() => (
    <MemoryRouter>
      <Route path="/" component={KnowledgeBase} />
    </MemoryRouter>
  ))
}

describe('KnowledgeBase page', () => {
  it('渲染标题、搜索框与新建按钮', async () => {
    renderPage()
    expect(screen.getByRole('heading', { name: /知识库/ })).toBeTruthy()
    expect(screen.getByRole('searchbox', { name: '搜索知识库' })).toBeTruthy()
    expect(screen.getByRole('button', { name: '新建知识库' })).toBeTruthy()
    expect(screen.getByRole('button', { name: '返回对话' })).toBeTruthy()
  })

  it('mock 数据源加载后渲染库卡片', async () => {
    const { findByText } = renderPage()
    // 种子库之一出现在卡片网格中
    expect(await findByText('NeoTrix 设计文档')).toBeTruthy()
    expect(screen.getByLabelText(/知识库 NeoTrix 设计文档/)).toBeTruthy()
  })
})
