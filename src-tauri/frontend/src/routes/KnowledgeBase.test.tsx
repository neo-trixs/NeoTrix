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

/** mock domain_call（kb 域），返回 {ok, data} 信封 */
function mockKbDomain(handlers: Record<string, (args: any) => unknown>) {
  return mockCommand('domain_call', async (req: any) => {
    const fn = handlers[`${req.domain}/${req.action}`]
    if (!fn) throw new Error(`unexpected domain call: ${req.domain}/${req.action}`)
    return { ok: true, data: await fn(req.args) }
  })
}

function renderPage() {
  return render(() => (
    <MemoryRouter>
      <Route path="/" component={KnowledgeBase} />
    </MemoryRouter>
  ))
}

describe('KnowledgeBase page (tauri 数据源)', () => {
  it('渲染标题、搜索框与新建按钮', () => {
    mockKbDomain({ 'kb/doc_list': async () => [] })
    renderPage()
    expect(screen.getByRole('heading', { name: /知识库/ })).toBeTruthy()
    expect(screen.getByRole('searchbox', { name: '搜索知识库' })).toBeTruthy()
    expect(screen.getByRole('button', { name: '新建知识库' })).toBeTruthy()
  })

  it('空库渲染空态引导', async () => {
    mockKbDomain({ 'kb/doc_list': async () => [] })
    renderPage()
    expect(await screen.findByText(/还没有知识库/)).toBeTruthy()
  })

  it('kb_doc_list 聚合为库卡片 (默认库名映射)', async () => {
    mockKbDomain({
      'kb/doc_list': async () => [
        { doc_id: 'kbdoc-a', title: '设计文档A', library: null, chunk_count: 4, total_chars: 1200, status: 'ready', created_at: Date.now() },
        { doc_id: 'kbdoc-b', title: '规则B', library: 'rules', chunk_count: 2, total_chars: 800, status: 'ready', created_at: Date.now() },
      ],
    })
    renderPage()
    expect(await screen.findByText('默认库')).toBeTruthy()
    expect(screen.getByText('rules')).toBeTruthy()
    expect(screen.getAllByText(/1 文档/).length).toBeGreaterThan(0)
  })

  it('点击库卡片展开文档列表与入库弹层 (B2 文档流)', async () => {
    mockKbDomain({
      'kb/doc_list': async () => [
        { doc_id: 'kbdoc-a', title: '设计文档A', library: null, chunk_count: 4, total_chars: 1200, status: 'ready', created_at: Date.now() },
      ],
      'kb/doc_ingest': async () => ({ doc_id: 'kbdoc-new', title: '新文档', library: 'default', chunk_count: 1, status: 'ready' }),
    })
    renderPage()
    const card = await screen.findByLabelText('知识库 默认库')
    card.click()
    expect(await screen.findByLabelText('文档列表')).toBeTruthy()
    screen.getByRole('button', { name: '添加文档' }).click()
    const dlg = await screen.findByRole('dialog', { name: '入库文档' })
    expect(dlg).toBeTruthy()
  })
})
