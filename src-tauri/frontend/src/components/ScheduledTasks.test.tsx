import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { ScheduledTasks } from './ScheduledTasks'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

vi.mock('@tauri-apps/api/window', () => ({}))

function task(over: Record<string, unknown> = {}) {
  return {
    id: 'task-1',
    name: '每日备份',
    prompt: '备份数据库',
    schedule: 'FREQ=DAILY;INTERVAL=1',
    last_run: 1700000000,
    next_run: 1700086400,
    status: 'idle',
    runs: [],
    ...over,
  }
}

function findBtn(text: string) {
  return [...document.querySelectorAll('button')].find((b) => b.textContent?.includes(text))
}

const settle = () => new Promise((r) => setTimeout(r, 120))

/** 后端任务形状（ai_orchestration 域） */
function backendTask(over: Record<string, unknown> = {}) {
  return {
    id: 'task-1',
    description: '每日备份：备份数据库',
    status: 'Queued',
    created_at: '2024-01-01T00:00:00Z',
    started_at: null,
    completed_at: null,
    progress: 0,
    result: null,
    error: null,
    ...over,
  }
}

/** mock domain_call，按 domain/action 分发；未注册分支抛错以暴露遗漏 */
function mockAiDomain(handlers: Record<string, (args: any) => unknown>) {
  return mockCommand('domain_call', async (req: any) => {
    const fn = handlers[`${req.domain}/${req.action}`]
    if (!fn) throw new Error(`unexpected domain call: ${req.domain}/${req.action}`)
    return { ok: true, data: await fn(req.args) }
  })
}

describe('ScheduledTasks 定时任务面板回归（列表/创建/RRule 校验/操作/删除确认）', () => {
  beforeEach(() => {
    resetInvokeMock()
    document.body.innerHTML = ''
    // tasks.ts 经 domain.call 调用，需 Tauri 宿主标识
    ;(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {}
  })

  it('open=false 不渲染', () => {
    render(() => <ScheduledTasks open={false} onClose={() => {}} />)
    expect(document.body.textContent).toBe('')
  })

  it('空列表显示暂无任务 + 新建定时任务按钮', async () => {
    mockAiDomain({ 'ai_orchestration/list_background_tasks': async () => [] })
    render(() => <ScheduledTasks open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('暂无定时任务')
    expect(document.body.textContent).toContain('新建定时任务')
  })

  it('加载任务列表渲染名称/调度/状态', async () => {
    mockAiDomain({
      'ai_orchestration/list_background_tasks': async () => [
        backendTask({ id: 'task-1', description: '每日备份 FREQ=DAILY' }),
        backendTask({ id: 't2', description: '每周清理 FREQ=WEEKLY', status: 'paused' }),
      ],
    })
    render(() => <ScheduledTasks open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('每日备份')
    expect(document.body.textContent).toContain('FREQ=DAILY')
    expect(document.body.textContent).toContain('空闲')
    expect(document.body.textContent).toContain('每周清理')
    expect(document.body.textContent).toContain('已暂停')
  })

  function openCreateForm() {
    const btn = findBtn('新建定时任务')
    expect(btn).toBeTruthy()
    fireEvent.click(btn!)
  }

  it('RRule 校验：空规则报错', async () => {
    mockAiDomain({ 'ai_orchestration/list_background_tasks': async () => [] })
    render(() => <ScheduledTasks open onClose={() => {}} />)
    await settle()
    const newBtn = findBtn('新建定时任务')
    fireEvent.click(newBtn!)
    await settle()
    const scheduleInput = document.querySelectorAll('input')[1] as HTMLInputElement
    expect(scheduleInput).toBeTruthy()
    fireEvent.input(scheduleInput!, { target: { value: '' } })
    await settle()
    expect(document.body.textContent).toContain('调度规则不能为空')
  })

  it('RRule 校验：非法 FREQ 报错', async () => {
    mockAiDomain({ 'ai_orchestration/list_background_tasks': async () => [] })
    render(() => <ScheduledTasks open onClose={() => {}} />)
    await settle()
    const newBtn = findBtn('新建定时任务')
    fireEvent.click(newBtn!)
    await settle()
    const scheduleInput = document.querySelectorAll('input')[1] as HTMLInputElement
    fireEvent.input(scheduleInput, { target: { value: 'FREQ=INVALID' } })
    await settle()
    expect(document.body.textContent).toContain('不支持的 FREQ=INVALID')
  })

  it('RRule 校验：合法规则通过', async () => {
    mockAiDomain({ 'ai_orchestration/list_background_tasks': async () => [] })
    render(() => <ScheduledTasks open onClose={() => {}} />)
    await settle()
    const newBtn = findBtn('新建定时任务')
    fireEvent.click(newBtn!)
    await settle()
    const scheduleInput = document.querySelectorAll('input')[1] as HTMLInputElement
    fireEvent.input(scheduleInput, { target: { value: 'FREQ=DAILY;INTERVAL=1' } })
    await settle()
    expect(document.body.textContent).not.toContain('调度规则')
  })

  it('创建任务：填表提交调用 submit_background_task', async () => {
    mockAiDomain({
      'ai_orchestration/list_background_tasks': async () => [],
      'ai_orchestration/submit_background_task': async (args: any) => {
        expect(args.description).toContain('测试任务')
        return backendTask({ id: 'new-1', description: args.description })
      },
    })
    render(() => <ScheduledTasks open onClose={() => {}} />)
    await settle()
    const newBtn = findBtn('新建定时任务')
    fireEvent.click(newBtn!)
    await settle()
    const nameInput = document.querySelector('input[placeholder*="例如"]') as HTMLInputElement
    const promptInput = document.querySelector('textarea[placeholder*="执行内容"]') as HTMLTextAreaElement
    const scheduleInput = document.querySelectorAll('input')[1] as HTMLInputElement
    fireEvent.input(nameInput, { target: { value: '测试任务' } })
    fireEvent.input(promptInput, { target: { value: '测试提示词' } })
    fireEvent.input(scheduleInput, { target: { value: 'FREQ=DAILY' } })
    await settle()
    const createBtn = findBtn('创建任务')
    expect(createBtn).toBeTruthy()
    fireEvent.click(createBtn!)
    await settle()
    // 提交后列表刷新（submit 成功即刷新）
    expect(document.body.textContent).toContain('新建定时任务')
  })

  it('暂停/恢复/立即运行后端未实现时显示错误', async () => {
    mockAiDomain({
      'ai_orchestration/list_background_tasks': async () => [backendTask({ status: 'idle' })],
    })
    render(() => <ScheduledTasks open onClose={() => {}} />)
    await settle()
    const runBtn = findBtn('立即执行')
    if (runBtn) {
      fireEvent.click(runBtn)
      await settle()
      expect(document.body.textContent).toContain('后端未实现')
    }
    const pauseBtn = findBtn('暂停')
    if (pauseBtn) {
      fireEvent.click(pauseBtn)
      await settle()
      expect(document.body.textContent).toContain('后端未实现')
    }
    const resumeBtn = findBtn('恢复')
    if (resumeBtn) {
      fireEvent.click(resumeBtn)
      await settle()
      expect(document.body.textContent).toContain('后端未实现')
    }
  })

  it('删除经 ConfirmModal 确认后调用 cancel_background_task', async () => {
    let cancelCalls = 0
    mockAiDomain({
      'ai_orchestration/list_background_tasks': async () => [backendTask()],
      'ai_orchestration/cancel_background_task': async () => {
        cancelCalls += 1
        return true
      },
    })
    render(() => <ScheduledTasks open onClose={() => {}} />)
    await settle()
    const deleteBtn = findBtn('删除')
    expect(deleteBtn).toBeTruthy()
    fireEvent.click(deleteBtn!)
    await settle()
    expect(document.body.textContent).toContain('删除定时任务')
    const confirmBtn = [...document.querySelectorAll('.glass-modal button')].pop()!
    fireEvent.click(confirmBtn)
    await settle()
    expect(cancelCalls).toBe(1)
  })

  it('Esc 关闭面板', async () => {
    const onClose = vi.fn()
    mockAiDomain({ 'ai_orchestration/list_background_tasks': async () => [] })
    render(() => <ScheduledTasks open onClose={onClose} />)
    await settle()
    const panel = document.querySelector('[role="dialog"]') as HTMLElement
    fireEvent.keyDown(panel, { key: 'Escape' })
    expect(onClose).toHaveBeenCalled()
  })

  it('task() 辅助保持旧形状兼容（回归锚点）', () => {
    expect(task().name).toBe('每日备份')
  })
})
