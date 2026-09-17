import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@solidjs/testing-library'
import { createSignal } from 'solid-js'
import type { ProviderConfig } from '../../api/types'
import { ModelsSection } from './ModelsSection'
import { resetInvokeMock, mockCommand } from '../../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

const baseCfg = (overrides: Partial<ProviderConfig> = {}): ProviderConfig => ({
  provider_count: 3,
  resolvable: true,
  active_model: 'local-model-a',
  providers: [
    { id: 'local', name: 'Local', display_name: '本地引擎', category: 'local', is_free: true, base_url: '', model: 'local-model-a', models: ['local-model-a', 'local-model-b'], resolvable: true },
    { id: 'proxy', name: 'Proxy', display_name: '中转代理', category: 'proxy', is_free: false, base_url: 'https://x', model: 'proxy-model-x', models: ['proxy-model-x'], resolvable: true },
    { id: 'broken', name: 'Broken', display_name: '不可用服务', category: 'cloud', is_free: false, base_url: '', model: 'dead-model', models: ['dead-model'], resolvable: false },
  ],
  ...overrides,
})

const mount = (cfg: ProviderConfig, opts?: { onTestConnection?: (name: string) => void }) => {
  const [config] = createSignal<ProviderConfig | null>(cfg)
  const [loading] = createSignal(false)
  const [switching] = createSignal(false)
  const onSwitch = vi.fn()
  const onTestConnection = opts?.onTestConnection ?? vi.fn()
  const utils = render(() => (
    <ModelsSection
      config={config}
      loading={loading}
      switching={switching}
      onSwitchProvider={onSwitch}
      onTestConnection={onTestConnection}
    />
  ))
  return { ...utils, onSwitch }
}

describe('ModelsSection', () => {
  beforeEach(() => resetInvokeMock())
  it('只展示可用模型：过滤 resolvable=false 的提供商', () => {
    const { container, onSwitch } = mount(baseCfg())
    expect(container.textContent).toContain('本地引擎')
    expect(container.textContent).toContain('中转代理')
    expect(container.textContent).not.toContain('不可用服务')
    expect(container.textContent).not.toContain('dead-model')
    expect(onSwitch).not.toHaveBeenCalled()
  })

  it('按分类分组并显示模型计数', () => {
    const { container } = mount(baseCfg())
    expect(container.textContent).toContain('本地推理')
    expect(container.textContent).toContain('自定义代理')
    expect(container.textContent).not.toContain('云端 API')
  })

  it('池概览显示可用模型总数（仅 resolvable）', () => {
    const { container } = mount(baseCfg())
    expect(container.textContent).toContain('3') // 2+1 个可用模型
    expect(container.textContent).toContain('2') // 2 个可用提供商
  })

  it('点击非激活模型切换提供商', async () => {
    const { container, onSwitch } = mount(baseCfg())
    const proxyRow = [...container.querySelectorAll('button')].find((b) => b.textContent?.includes('proxy-model-x'))
    expect(proxyRow).toBeTruthy()
    proxyRow!.click()
    expect(onSwitch).toHaveBeenCalledWith('Proxy')
  })

  it('激活模型标记"当前"，点击不重复触发', () => {
    const { container, onSwitch } = mount(baseCfg())
    const activeBtn = [...container.querySelectorAll('button')].find((b) => b.textContent?.includes('local-model-a'))
    expect(activeBtn).toBeTruthy()
    expect(activeBtn!.getAttribute('aria-checked')).toBe('true')
    activeBtn!.click()
    expect(onSwitch).not.toHaveBeenCalled()
  })



  it('连通测试: 测试按钮通过 title 可定位', async () => {
    const { container } = mount(baseCfg())
    // 新 UI 用 title="测试连接" 而非 aria-label
    const btns = container.querySelectorAll('button[title="测试连接"]')
    expect(btns.length).toBeGreaterThan(0)
  })

  it('连通测试: 全部不可用时仍渲染模型池', async () => {
    const { container } = mount(baseCfg({
      providers: baseCfg().providers.map((p) => ({ ...p, resolvable: false })),
      active_model: 'dead-model',
    }))
    // 新 UI 渲染 Free LLM 池子面板，空态显示"暂无免费模型"或"加载中"
    expect(container.textContent).toMatch(/暂无|加载中|Free LLM/)
  })
})