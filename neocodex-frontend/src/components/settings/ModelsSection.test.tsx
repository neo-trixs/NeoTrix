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

const mount = (cfg: ProviderConfig) => {
  const [config] = createSignal<ProviderConfig | null>(cfg)
  const [loading] = createSignal(false)
  const [switching] = createSignal(false)
  const onSwitch = vi.fn()
  const utils = render(() => (
    <ModelsSection
      config={config}
      loading={loading}
      switching={switching}
      onSwitchProvider={onSwitch}
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



  it('连通测试: provider_test 真契约返回延迟 (P3-M3 已接线)', async () => {
    mockCommand('provider_test', () => ({ ok: true, status_code: 200, latency_ms: 137 }))
    const cfg = baseCfg()
    cfg.providers[0].base_url = 'https://api.local.example'
    const utils = mount(cfg)
    const btn = await utils.findByLabelText('测试 本地引擎 连通')
    fireEvent.click(btn)
    const ok = await screen.findByLabelText('本地引擎 延迟')
    expect(ok.textContent).toBe('● 137ms')
  })

  it('连通测试: 无 endpoint 探测失败 → 不通 (fail-closed)', async () => {
    const cfg = baseCfg()
    cfg.providers.push({ id: 'abcd', name: 'Abcd', display_name: '四字服务', category: 'cloud', is_free: false, base_url: '', model: 'm4', models: ['m4'], resolvable: true })
    mount(cfg)
    const btn = await screen.findByLabelText('测试 四字服务 连通')
    fireEvent.click(btn)
    expect(await screen.findByLabelText('四字服务 不可达')).toBeTruthy()
  })

  it('全部不可用时的空态提示', () => {
    const { container } = mount(baseCfg({ providers: baseCfg().providers.map((p) => ({ ...p, resolvable: false })), active_model: 'dead-model' }))
    expect(container.textContent).toContain('当前没有可用的模型')
  })
})