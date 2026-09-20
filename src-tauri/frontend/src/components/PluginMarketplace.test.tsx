import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { PluginMarketplace } from './PluginMarketplace'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

vi.mock('@tauri-apps/api/window', () => ({}))

function plugin(over: Record<string, unknown> = {}) {
  return {
    id: 'plg-a',
    name: 'plugin-a',
    version: '1.2.3',
    enabled: true,
    loaded: true,
    load_time_ms: 12,
    error: null,
    ...over,
  }
}

const settle = () => new Promise((r) => setTimeout(r, 120))

describe('PluginMarketplace 插件市场回归（列表/启停/卸载确认）', () => {
  beforeEach(() => {
    resetInvokeMock()
    document.body.innerHTML = ''
  })

  it('open=false 不渲染', () => {
    render(() => <PluginMarketplace open={false} onClose={() => {}} />)
    expect(document.body.textContent).toBe('')
  })

  it('空列表显示暂无插件 + 安装入口', async () => {
    mockCommand('plugin_list', async () => [])
    mockCommand('plugin_event_log', async () => [])
    render(() => <PluginMarketplace open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('暂无插件')
    expect(document.body.textContent).toContain('安装插件（选择 manifest.json）')
  })

  it('加载失败显示错误信息', async () => {
    mockCommand('plugin_list', async () => {
      throw new Error('list failed')
    })
    mockCommand('plugin_event_log', async () => [])
    render(() => <PluginMarketplace open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('list failed')
  })

  it('插件列表渲染名称/版本/启用徽章', async () => {
    mockCommand('plugin_list', async () => [
      plugin(),
      plugin({ id: 'plg-b', name: 'plugin-b', version: '0.9.0', enabled: false }),
    ])
    mockCommand('plugin_event_log', async () => [])
    render(() => <PluginMarketplace open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('plugin-a')
    expect(document.body.textContent).toContain('plugin-b')
    expect(document.body.textContent).toContain('1.2.3')
    // 启用/禁用徽章
    const badges = [...document.querySelectorAll('.badge')].map((b) => b.textContent)
    expect(badges).toContain('启用')
    expect(badges).toContain('禁用')
  })

  it('点击禁用调用 plugin_disable 并刷新列表', async () => {
    mockCommand('plugin_list', async () => [plugin()])
      // 刷新后返回 disabled 状态
    const disableStub = mockCommand('plugin_disable', async () => null)
    mockCommand('plugin_event_log', async () => [])
    render(() => <PluginMarketplace open onClose={() => {}} />)
    await settle()
    const toggleBtn = [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('禁用') && b.getAttribute('aria-pressed') === 'true')!
    fireEvent.click(toggleBtn)
    await settle()
    expect(disableStub.calledTimes()).toBe(1)
  })

  it('点击启用调用 plugin_enable', async () => {
    mockCommand('plugin_list', async () => [plugin({ enabled: false })])
    const enableStub = mockCommand('plugin_enable', async () => null)
    mockCommand('plugin_event_log', async () => [])
    render(() => <PluginMarketplace open onClose={() => {}} />)
    await settle()
    const toggleBtn = [...document.querySelectorAll('button')].find((b) => b.textContent?.trim() === '启用')!
    fireEvent.click(toggleBtn)
    await settle()
    expect(enableStub.calledTimes()).toBe(1)
  })

  it('卸载经 ConfirmModal 确认后调用 plugin_uninstall', async () => {
    mockCommand('plugin_list', async () => [plugin()])
    const uninstallStub = mockCommand('plugin_uninstall', async () => null)
    mockCommand('plugin_event_log', async () => [])
    render(() => <PluginMarketplace open onClose={() => {}} />)
    await settle()
    const uninstallBtn = [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('卸载'))!
    fireEvent.click(uninstallBtn)
    await settle()
    // ConfirmModal 出现
    expect(document.body.textContent).toContain('卸载')
    const confirmBtn = [...document.querySelectorAll('.glass-modal button')].pop()!
    fireEvent.click(confirmBtn)
    await settle()
    expect(uninstallStub.calledTimes()).toBe(1)
  })

  it('事件日志渲染', async () => {
    mockCommand('plugin_list', async () => [])
    mockCommand('plugin_event_log', async () => [
      { timestamp: 1700000000, kind: 'load', plugin_id: 'plg-a', message: 'loaded ok' },
    ])
    render(() => <PluginMarketplace open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('loaded ok')
  })
})