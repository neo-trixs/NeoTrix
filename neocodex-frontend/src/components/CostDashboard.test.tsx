import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { render } from '@solidjs/testing-library'
import { CostDashboard } from './CostDashboard'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'
import { invalidateAll } from '../api/query'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

vi.mock('@tauri-apps/api/window', () => ({}))

function baseStatus(over: Record<string, unknown> = {}) {
  return {
    running: true,
    provider_model: 'claude-sonnet-4',
    cost_spent: 0.000042,
    cost_budget: 100,
    tokens_used: 2_500_000,
    turn_count: 42,
    context_usage: 0.6,
    evolution_iterations: 7,
    uptime_secs: 5400,
    current_task: null as string | null,
    ...over,
  }
}

const settle = () => new Promise((r) => setTimeout(r, 250))

describe('CostDashboard 成本/Token 看板回归（P：格式化与百分比显示）', () => {
  beforeEach(() => {
    resetInvokeMock()
    invalidateAll()
  })
  afterEach(() => vi.useRealTimers())

  it('加载后显示成本与 token 格式化输出', async () => {
    mockCommand('neocodex_agent_status', async () => baseStatus())
    render(() => <CostDashboard open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('成本 / Token 看板')
    // token 2.5M 缩写
    expect(document.body.textContent).toContain('2.50M')
    // 极小金额用科学计数（$4.2e-5 → 不显示 $0.0000）
    expect(document.body.textContent).toContain('$')
    expect(document.body.textContent).not.toContain('$0.0000')
    // 上下文 60% + 预算 0%（cost_budget=100, spent=0.000042 → ~0.0%）
    expect(document.body.textContent).toContain('60.0%')
    // uptime 5400s = 1h 30m
    expect(document.body.textContent).toContain('1h 30m')
    // 运行状态
    expect(document.body.textContent).toContain('运行中')
  })

  it('大 token 用 k 缩写', async () => {
    mockCommand('neocodex_agent_status', async () => baseStatus({ tokens_used: 4500 }))
    render(() => <CostDashboard open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('4.5k')
  })

  it('预算超 80% 进度条转红', async () => {
    mockCommand(
      'neocodex_agent_status',
      async () => baseStatus({ cost_spent: 85, cost_budget: 100 }),
    )
    render(() => <CostDashboard open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('85.0%')
    const bars = document.querySelectorAll('.bg-red-500')
    expect(bars.length).toBeGreaterThanOrEqual(1)
  })

  it('空数据（null status）显示空态', async () => {
    mockCommand('neocodex_agent_status', async () => null as unknown as Record<string, unknown>)
    render(() => <CostDashboard open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('暂无成本数据')
  })

  it('错误时显示错误信息而非挂起', async () => {
    mockCommand('neocodex_agent_status', async () => {
      throw new Error('backend down')
    })
    render(() => <CostDashboard open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('backend down')
  })

  it('open=false 不渲染面板', () => {
    render(() => <CostDashboard open={false} onClose={() => {}} />)
    expect(document.querySelector('[role="dialog"]')).toBeNull()
  })

  it('Esc 键关闭面板', async () => {
    const onClose = vi.fn()
    mockCommand('neocodex_agent_status', async () => baseStatus())
    render(() => <CostDashboard open onClose={onClose} />)
    await settle()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    expect(onClose).toHaveBeenCalled()
  })

  it('刷新按钮重新拉取', async () => {
    const stub = mockCommand('neocodex_agent_status', async () => baseStatus())
    render(() => <CostDashboard open onClose={() => {}} />)
    await settle()
    const refresh = document.querySelector('[aria-label="刷新"]') as HTMLButtonElement
    refresh.click()
    await settle()
    expect(stub.calledTimes()).toBeGreaterThanOrEqual(2)
  })
})