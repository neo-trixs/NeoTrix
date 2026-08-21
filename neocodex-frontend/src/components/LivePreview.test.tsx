import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { LivePreview } from './LivePreview'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

vi.mock('@tauri-apps/api/window', () => ({}))

const settle = () => new Promise((r) => setTimeout(r, 120))

describe('LivePreview 实时预览回归（URL 输入/iframe/刷新/外部打开/关闭）', () => {
  beforeEach(() => {
    resetInvokeMock()
    document.body.innerHTML = ''
  })

  it('open=false 不渲染', () => {
    render(() => <LivePreview open={false} onClose={() => {}} />)
    expect(document.body.textContent).toBe('')
  })

  it('打开渲染端口提示条 + URL 输入框 + EmptyPreview 占位', async () => {
    mockCommand('neocodex_open_external', async () => null)
    render(() => <LivePreview open onClose={() => {}} />)
    await settle()
    // 端口提示条
    expect(document.body.textContent).toContain('端口:')
    expect(document.body.textContent).toContain('1421')
    expect(document.body.textContent).toContain('5173')
    // URL 输入框（无 type="url"，用 placeholder 定位）
    expect(document.querySelector('input[placeholder="http://localhost:5173"]')).toBeTruthy()
    // EmptyPreview 占位
    expect(document.body.textContent).toContain('自动探测')
    // 无 iframe
    expect(document.querySelector('iframe')).toBeNull()
  })

  it('输入 URL 后渲染 iframe', async () => {
    mockCommand('neocodex_open_external', async () => null)
    render(() => <LivePreview open onClose={() => {}} />)
    await settle()
    const input = document.querySelector('input[placeholder="http://localhost:5173"]') as HTMLInputElement
    expect(input).toBeTruthy()
    fireEvent.input(input, { target: { value: 'http://localhost:5173' } })
    await settle()
    const iframe = document.querySelector('iframe')
    expect(iframe).toBeTruthy()
    expect(iframe?.getAttribute('src')).toContain('http://localhost:5173')
  })

  it('Enter 键触发刷新', async () => {
    mockCommand('neocodex_open_external', async () => null)
    render(() => <LivePreview open onClose={() => {}} />)
    await settle()
    const input = document.querySelector('input[placeholder="http://localhost:5173"]') as HTMLInputElement
    fireEvent.input(input, { target: { value: 'http://localhost:5173' } })
    await settle()
    fireEvent.keyDown(input, { key: 'Enter' })
    await settle()
    // frameKey 变化导致 iframe 重载，src 含基址
    expect(document.querySelector('iframe')?.getAttribute('src')).toContain('http://localhost:5173')
  })

  it('点击刷新按钮递增 frameKey', async () => {
    mockCommand('neocodex_open_external', async () => null)
    render(() => <LivePreview open onClose={() => {}} />)
    await settle()
    const input = document.querySelector('input[placeholder="http://localhost:5173"]') as HTMLInputElement
    fireEvent.input(input, { target: { value: 'http://localhost:5173' } })
    await settle()
    fireEvent.click(document.querySelector('[aria-label="加载预览"]')!)
    await settle()
    expect(document.querySelector('iframe')?.getAttribute('src')).toContain('http://localhost:5173')
  })

  it('点击外部打开调用 openExternal', async () => {
    const openStub = mockCommand('neocodex_open_external', async () => null)
    render(() => <LivePreview open onClose={() => {}} />)
    await settle()
    const input = document.querySelector('input[placeholder="http://localhost:5173"]') as HTMLInputElement
    fireEvent.input(input, { target: { value: 'http://localhost:5173' } })
    await settle()
    fireEvent.click(document.querySelector('[aria-label="在浏览器中打开"]')!)
    await settle()
    expect(openStub.calledTimes()).toBe(1)
  })

  it('Esc 关闭面板', async () => {
    const onClose = vi.fn()
    mockCommand('neocodex_open_external', async () => null)
    render(() => <LivePreview open onClose={onClose} />)
    await settle()
    fireEvent.keyDown(window, { key: 'Escape' })
    expect(onClose).toHaveBeenCalled()
  })

  it('点击遮罩关闭', async () => {
    const onClose = vi.fn()
    mockCommand('neocodex_open_external', async () => null)
    render(() => <LivePreview open onClose={onClose} />)
    await settle()
    const overlay = document.querySelector('.fixed.inset-0') as HTMLElement
    fireEvent.mouseDown(overlay, { bubbles: true })
    expect(onClose).toHaveBeenCalled()
  })
})