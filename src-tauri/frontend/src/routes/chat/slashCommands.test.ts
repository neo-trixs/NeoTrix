import { describe, it, expect, vi, beforeEach } from 'vitest'

// 实现经 ../../api/domain 直调 domainCall；此处 mock 该层而非 barrel
const mockDomainCall = vi.fn()
vi.mock('../../api/domain', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../api/domain')>()
  return { ...actual, call: (...args: unknown[]) => (mockDomainCall as (...a: unknown[]) => unknown)(...args) }
})

import { runSlashModel, runSlashStatus, runSlashCost, runSlashExport, runSlashDispatch } from './slashCommands'
import type { SlashContext } from './slashCommands'
import type { ChatStore } from '../../stores/chat'
import type { SlashCommandDef } from '../../components/SlashMenu'

function makeCtx(overrides?: Partial<SlashContext>): SlashContext {
  const store = { clearMessages: vi.fn(), addSession: vi.fn() } as unknown as ChatStore
  return {
    store,
    currentSessionId: () => 's1',
    clearInput: vi.fn(),
    showInfo: vi.fn(),
    showError: vi.fn(),
    ...overrides,
  }
}

function cmd(id: string): SlashCommandDef {
  return { id, label: id, desc: '', keywords: [id] }
}

describe('routes/chat/slashCommands — 斜杠命令逻辑', () => {
  beforeEach(() => {
    mockDomainCall.mockReset()
    // 默认成功空回包，避免未设 mock 的路径上 .catch 失效
    mockDomainCall.mockResolvedValue({})
  })

  it('/model 显示当前激活模型（providerConfig）', async () => {
    mockDomainCall.mockResolvedValue({ provider_count: 2, resolvable: true, active_model: 'gpt-x', providers: [] })
    const ctx = makeCtx()
    await runSlashModel(ctx)
    expect(mockDomainCall).toHaveBeenCalledWith('llamacpp', 'provider_config')
    expect(ctx.showInfo).toHaveBeenCalledWith(expect.stringContaining('gpt-x'), 5000)
  })

  it('/status 显示运行状态（agentStatus）', async () => {
    mockDomainCall.mockResolvedValue({
      provider_model: 'm1', context_usage: 0.42, tokens_used: 1234,
      cost_spent: 500, cost_budget: 10000, running: true, current_task: null,
      uptime_secs: 1, turn_count: 1, evolution_iterations: 1,
    })
    const ctx = makeCtx()
    await runSlashStatus(ctx)
    expect(mockDomainCall).toHaveBeenCalledWith('agent', 'status')
    expect(ctx.showInfo).toHaveBeenCalledWith(expect.stringContaining('42%'), 6000)
  })

  it('/cost 显示用量与成本占比', async () => {
    mockDomainCall.mockResolvedValue({
      cost_spent: 5000, cost_budget: 10000, tokens_used: 1000, running: true, current_task: null,
      uptime_secs: 1, turn_count: 1, context_usage: 0, provider_model: '', evolution_iterations: 1,
    })
    const ctx = makeCtx()
    await runSlashCost(ctx)
    expect(ctx.showInfo).toHaveBeenCalledWith(expect.stringContaining('50%'), 5000)
  })

  it('/export 无会话时提示，不调后端', async () => {
    const ctx = makeCtx({ currentSessionId: () => '' })
    await runSlashExport(ctx)
    expect(mockDomainCall).not.toHaveBeenCalled()
    expect(ctx.showInfo).toHaveBeenCalledWith('当前没有激活会话，无法导出', 3000)
  })

  it('/clear 清空消息', () => {
    const ctx = makeCtx()
    runSlashDispatch(ctx, cmd('clear'))
    expect(ctx.store.clearMessages).toHaveBeenCalled()
  })

  it('/new 新建会话', () => {
    const ctx = makeCtx()
    runSlashDispatch(ctx, cmd('new'))
    expect(ctx.store.addSession).toHaveBeenCalled()
  })

  it('/model 经 dispatch 派发到 runSlashModel', async () => {
    mockDomainCall.mockResolvedValue({ provider_count: 0, resolvable: true, active_model: 'x', providers: [] })
    const ctx = makeCtx()
    runSlashDispatch(ctx, cmd('model'))
    await Promise.resolve()
    await Promise.resolve()
    expect(ctx.showInfo).toHaveBeenCalledWith(expect.stringContaining('x'), 5000)
  })

  it('/help 显示快捷键提示', () => {
    const ctx = makeCtx()
    runSlashDispatch(ctx, cmd('help'))
    expect(ctx.showInfo).toHaveBeenCalledWith(expect.stringContaining('Enter 发送'), 5000)
  })
})
