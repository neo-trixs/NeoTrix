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

/** mock domain_call，按 domain/action 分发，返回 {ok, data} 信封 */
function mockPluginDomain(handlers: Record<string, (args: any) => unknown>) {
  const merged: Record<string, (args: any) => unknown> = {
    // 组件挂载即拉事件日志：默认空，各用例可覆盖
    'plugin/event_log': async () => [],
    ...handlers,
  }
  return mockCommand('domain_call', async (req: any) => {
    const fn = merged[`${req.domain}/${req.action}`]
    if (!fn) throw new Error(`unexpected domain call: ${req.domain}/${req.action}`)
    return { ok: true, data: await fn(req.args) }
  })
}

/** mock chat_send，动作结果走 actions[0].result（market.ts 经 extractResult 读取） */
function mockChatMarket(installed: unknown[] = []) {
  return mockCommand('chat_send', async () => ({
    ok: true,
    data: {
      message: '插件列表:',
      actions: [{ domain: 'plugin', action: 'list', result: installed }],
      state: null,
    },
  }))
}

describe('PluginMarketplace 插件市场回归（列表/启停/卸载确认）', () => {
  beforeEach(() => {
    resetInvokeMock()
    document.body.innerHTML = ''
    // plugins.ts/market.ts 经 domain/chatSend 调用，需 Tauri 宿主标识
    ;(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {}
  })

  it('open=false 不渲染', () => {
    render(() => <PluginMarketplace open={false} onClose={() => {}} />)
    expect(document.body.textContent).toBe('')
  })

  it('空列表显示暂无插件 + 安装入口', async () => {
    mockPluginDomain({ 'plugin/list': async () => [] })
    mockChatMarket()
    render(() => <PluginMarketplace open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('暂无插件')
    expect(document.body.textContent).toContain('本地安装（选择 manifest.json）')
  })

  it('加载失败显示错误信息', async () => {
    mockPluginDomain({
      'plugin/list': async () => {
        throw new Error('list failed')
      },
    })
    mockChatMarket()
    render(() => <PluginMarketplace open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('list failed')
  })

  it('插件列表渲染名称/版本/启用徽章', async () => {
    mockPluginDomain({
      'plugin/list': async () => [
        plugin(),
        plugin({ id: 'plg-b', name: 'plugin-b', version: '0.9.0', enabled: false }),
      ],
    })
    mockChatMarket()
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
    let disableCalls = 0
    mockPluginDomain({
      'plugin/list': async () => [plugin()],
      // 刷新后返回 disabled 状态
      'plugin/disable': async () => {
        disableCalls += 1
        return null
      },
    })
    mockChatMarket()
    render(() => <PluginMarketplace open onClose={() => {}} />)
    await settle()
    const toggleBtn = [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('禁用'))!
    fireEvent.click(toggleBtn)
    await settle()
    expect(disableCalls).toBe(1)
  })

  it('点击启用调用 plugin_enable', async () => {
    let enableCalls = 0
    mockPluginDomain({
      'plugin/list': async () => [plugin({ enabled: false })],
      'plugin/enable': async () => {
        enableCalls += 1
        return null
      },
    })
    mockChatMarket()
    render(() => <PluginMarketplace open onClose={() => {}} />)
    await settle()
    const toggleBtn = [...document.querySelectorAll('button')].find((b) => b.textContent?.trim() === '启用')!
    fireEvent.click(toggleBtn)
    await settle()
    expect(enableCalls).toBe(1)
  })

  it('卸载经 ConfirmModal 确认后调用 plugin_uninstall', async () => {
    let uninstallCalls = 0
    mockPluginDomain({
      'plugin/list': async () => [plugin()],
      'plugin/uninstall': async () => {
        uninstallCalls += 1
        return null
      },
    })
    // 组件优先走 market 卸载：此处使其失败以覆盖回退到原生 plugin_uninstall 的路径
    mockCommand('chat_send', async () => {
      throw new Error('market unavailable')
    })
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
    expect(uninstallCalls).toBe(1)
  })

  it('事件日志渲染', async () => {
    mockPluginDomain({
      'plugin/list': async () => [],
      'plugin/event_log': async () => [
        { timestamp: 1700000000, kind: 'load', plugin_id: 'plg-a', message: 'loaded ok' },
      ],
    })
    mockChatMarket()
    render(() => <PluginMarketplace open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('loaded ok')
  })
})
