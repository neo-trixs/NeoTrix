import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { ProviderSelector } from './ProviderSelector'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

function cfg(over: Record<string, unknown> = {}) {
  return {
    provider_count: 2,
    resolvable: true,
    active_model: 'gpt-4o',
    providers: [
      {
        name: 'openai', display_name: 'OpenAI', category: 'cloud', is_free: false,
        base_url: 'https://api.openai.com', model: 'gpt-4o', models: ['gpt-4o'], resolvable: true,
      },
      {
        name: 'ollama', display_name: 'Ollama', category: 'local', is_free: true,
        base_url: 'http://localhost:11434', model: 'llama3', models: ['llama3'], resolvable: true,
      },
    ],
    ...over,
  }
}

const settle = () => new Promise((r) => setTimeout(r, 120))

describe('ProviderSelector 提供商选择器回归（下拉/切换/广播）', () => {
  beforeEach(() => {
    resetInvokeMock()
    document.body.innerHTML = ''
  })

  it('加载后触发按钮显示当前提供商', async () => {
    mockCommand('neocodex_provider_config', async () => cfg())
    render(() => <ProviderSelector />)
    await settle()
    const btn = document.querySelector('[aria-label="选择模型提供商"]') as HTMLElement
    expect(btn.textContent).toContain('OpenAI')
    expect(btn.getAttribute('aria-expanded')).toBe('false')
  })

  it('点击展开下拉并列出提供商', async () => {
    mockCommand('neocodex_provider_config', async () => cfg())
    render(() => <ProviderSelector />)
    await settle()
    fireEvent.click(document.querySelector('[aria-label="选择模型提供商"]')!)
    expect(document.querySelector('[role="listbox"]')).toBeTruthy()
    expect(document.querySelector('[aria-label="选择模型提供商"]')!.getAttribute('aria-expanded')).toBe('true')
    const opts = document.querySelectorAll('[role="option"]')
    expect(opts.length).toBe(2)
    expect(opts[0].getAttribute('aria-selected')).toBe('true') // gpt-4o 激活
    expect(opts[1].getAttribute('aria-selected')).toBe('false')
  })

  it('当前激活项 disabled（不可重复选择）', async () => {
    mockCommand('neocodex_provider_config', async () => cfg())
    render(() => <ProviderSelector />)
    await settle()
    fireEvent.click(document.querySelector('[aria-label="选择模型提供商"]')!)
    const opts = document.querySelectorAll('[role="option"]') as NodeListOf<HTMLButtonElement>
    expect(opts[0].disabled).toBe(true)
    expect(opts[1].disabled).toBe(false)
  })

  it('选择新提供商调用 set_provider + 广播变更 + 关闭下拉', async () => {
    mockCommand('neocodex_provider_config', async () => cfg())
    const setStub = mockCommand('neocodex_set_provider', async () => null)
    const changedSpy = vi.fn()
    window.addEventListener('neotrix:provider-changed', changedSpy)
    render(() => <ProviderSelector />)
    await settle()
    fireEvent.click(document.querySelector('[aria-label="选择模型提供商"]')!)
    fireEvent.click(document.querySelectorAll('[role="option"]')[1])
    await settle()
    expect(setStub.calledTimes()).toBe(1)
    expect(changedSpy).toHaveBeenCalled()
    // 下拉关闭
    expect(document.querySelector('[role="listbox"]')).toBeNull()
    window.removeEventListener('neotrix:provider-changed', changedSpy)
  })

  it('Esc 关闭下拉', async () => {
    mockCommand('neocodex_provider_config', async () => cfg())
    render(() => <ProviderSelector />)
    await settle()
    fireEvent.click(document.querySelector('[aria-label="选择模型提供商"]')!)
    expect(document.querySelector('[role="listbox"]')).toBeTruthy()
    fireEvent.keyDown(window, { key: 'Escape' })
    expect(document.querySelector('[role="listbox"]')).toBeNull()
  })

  it('点击外部区域关闭下拉', async () => {
    mockCommand('neocodex_provider_config', async () => cfg())
    render(() => <ProviderSelector />)
    await settle()
    fireEvent.click(document.querySelector('[aria-label="选择模型提供商"]')!)
    expect(document.querySelector('[role="listbox"]')).toBeTruthy()
    const overlay = document.querySelector('.fixed.inset-0') as HTMLElement
    fireEvent.click(overlay)
    expect(document.querySelector('[role="listbox"]')).toBeNull()
  })

  it('iconOnly 模式渲染紧凑图标按钮 + title', async () => {
    mockCommand('neocodex_provider_config', async () => cfg())
    render(() => <ProviderSelector iconOnly />)
    await settle()
    const btn = document.querySelector('[aria-label="选择模型提供商"]') as HTMLElement
    expect(btn.className).toContain('w-8 h-8')
    expect(btn.getAttribute('title')).toBe('OpenAI')
  })

  it('配置加载失败显示错误 toast 可关闭', async () => {
    mockCommand('neocodex_provider_config', async () => {
      throw new Error('config failed')
    })
    render(() => <ProviderSelector />)
    await settle()
    expect(document.body.textContent).toContain('config failed')
    // 关闭错误
    fireEvent.click([...document.querySelectorAll('button')].find((b) => b.textContent === '×')!)
    expect(document.body.textContent).not.toContain('config failed')
  })

  it('空提供商列表显示暂无可用', async () => {
    mockCommand('neocodex_provider_config', async () => cfg({ providers: [], provider_count: 0 }))
    render(() => <ProviderSelector />)
    await settle()
    fireEvent.click(document.querySelector('[aria-label="选择模型提供商"]')!)
    expect(document.body.textContent).toContain('暂无可用提供商')
  })
})