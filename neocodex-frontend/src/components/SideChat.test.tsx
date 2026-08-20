import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { SideChat } from './SideChat'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})
vi.mock('@tauri-apps/api/window', () => ({}))

function msg(over: Record<string, unknown> = {}) {
  return {
    id: 'm1',
    role: 'assistant',
    content: '你好',
    timestamp: 1700000000,
    ...over,
  }
}

const settle = () => new Promise((r) => setTimeout(r, 100))

describe('SideChat 侧向对话面板回归（加载/发送/IME/焦点还原）', () => {
  beforeEach(() => {
    resetInvokeMock()
    document.body.innerHTML = ''
  })
  afterEach(() => vi.useRealTimers())

  it('open=false 不渲染面板', () => {
    render(() => <SideChat open={false} sessionId="s1" onClose={() => {}} />)
    expect(document.querySelector('[role="dialog"]')).toBeNull()
  })

  it('打开后加载侧聊消息', async () => {
    mockCommand('neocodex_get_side_chat', async () => [msg(), msg({ role: 'user', content: '问题?' })])
    render(() => <SideChat open sessionId="s1" onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('你好')
    expect(document.body.textContent).toContain('问题?')
  })

  it('无消息时显示空态说明', async () => {
    mockCommand('neocodex_get_side_chat', async () => [])
    render(() => <SideChat open sessionId="s1" onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('侧向对话与主上下文隔离')
  })

  it('加载失败显示错误', async () => {
    mockCommand('neocodex_get_side_chat', async () => {
      throw new Error('load failed')
    })
    render(() => <SideChat open sessionId="s1" onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('load failed')
  })

  it('输入 + 发送调用 sendSideChat 并清空输入', async () => {
    mockCommand('neocodex_get_side_chat', async () => [])
    const sendStub = mockCommand('neocodex_send_side_chat', async () => [msg({ role: 'user', content: '测试' })])
    render(() => <SideChat open sessionId="s1" onClose={() => {}} />)
    await settle()
    const ta = document.querySelector('textarea') as HTMLTextAreaElement
    fireEvent.input(ta, { target: { value: '测试' } })
    fireEvent.keyDown(ta, { key: 'Enter', shiftKey: false })
    await settle()
    expect(sendStub.calledTimes()).toBe(1)
    // 发送后清空输入
    expect((document.querySelector('textarea') as HTMLTextAreaElement).value).toBe('')
    // 发送后消息列表出现新内容
    expect(document.body.textContent).toContain('测试')
  })

  it('发送按钮 disabled 当输入为空', async () => {
    mockCommand('neocodex_get_side_chat', async () => [])
    render(() => <SideChat open sessionId="s1" onClose={() => {}} />)
    await settle()
    const btn = document.querySelector('[aria-label="发送"]') as HTMLButtonElement
    expect(btn.disabled).toBe(true)
  })

  it('发送按钮 enabled 当有输入', async () => {
    mockCommand('neocodex_get_side_chat', async () => [])
    render(() => <SideChat open sessionId="s1" onClose={() => {}} />)
    await settle()
    const ta = document.querySelector('textarea') as HTMLTextAreaElement
    fireEvent.input(ta, { target: { value: 'hi' } })
    const btn = document.querySelector('[aria-label="发送"]') as HTMLButtonElement
    expect(btn.disabled).toBe(false)
  })

  it('IME 组合态 Enter 不发送（isComposing/229 守卫）', async () => {
    const sendStub = mockCommand('neocodex_send_side_chat', async () => [msg()])
    mockCommand('neocodex_get_side_chat', async () => [])
    render(() => <SideChat open sessionId="s1" onClose={() => {}} />)
    await settle()
    const ta = document.querySelector('textarea') as HTMLTextAreaElement
    fireEvent.input(ta, { target: { value: '你好' } })
    // IME 组合中按 Enter：keyCode 229
    fireEvent.keyDown(ta, { key: 'Enter', keyCode: 229, isComposing: true })
    await settle()
    expect(sendStub.calledTimes()).toBe(0) // 不发送
  })

  it('Esc 关闭面板', async () => {
    const onClose = vi.fn()
    mockCommand('neocodex_get_side_chat', async () => [])
    render(() => <SideChat open sessionId="s1" onClose={onClose} />)
    await settle()
    // 面板容器 onKeyDown 处理 Esc
    const dialog = document.querySelector('[role="dialog"]') as HTMLElement
    fireEvent.keyDown(dialog, { key: 'Escape' })
    expect(onClose).toHaveBeenCalled()
  })

  it('刷新按钮重新加载', async () => {
    const stub = mockCommand('neocodex_get_side_chat', async () => [msg()])
    render(() => <SideChat open sessionId="s1" onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('你好')
    const refresh = document.querySelector('[aria-label="刷新"]') as HTMLButtonElement
    refresh.click()
    await settle()
    expect(stub.calledTimes()).toBeGreaterThanOrEqual(2)
  })

  it('assistant 消息用 Markdown 渲染, user 消息纯文本', async () => {
    mockCommand('neocodex_get_side_chat', async () => [
      msg({ role: 'user', content: 'plain text' }),
      msg({ role: 'assistant', content: '**bold** text' }),
    ])
    render(() => <SideChat open sessionId="s1" onClose={() => {}} />)
    await settle()
    // user 纯文本回显
    expect(document.body.textContent).toContain('plain text')
    // assistant Markdown 加粗
    expect(document.body.querySelector('strong')).toBeTruthy()
  })
})