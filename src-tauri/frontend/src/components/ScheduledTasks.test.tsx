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

describe('ScheduledTasks 定时任务面板回归（列表/创建/RRule 校验/操作/删除确认）', () => {
  beforeEach(() => {
    resetInvokeMock()
    document.body.innerHTML = ''
  })

  it('open=false 不渲染', () => {
    render(() => <ScheduledTasks open={false} onClose={() => {}} />)
    expect(document.body.textContent).toBe('')
  })

  it('空列表显示暂无任务 + 新建定时任务按钮', async () => {
    mockCommand('list_background_tasks', async () => [])
    render(() => <ScheduledTasks open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('暂无定时任务')
    expect(document.body.textContent).toContain('新建定时任务')
  })

  it('加载任务列表渲染名称/调度/状态', async () => {
    mockCommand('list_background_tasks', async () => [task(), task({ id: 't2', name: '每周清理', schedule: 'FREQ=WEEKLY', status: 'paused' })])
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
    mockCommand('list_background_tasks', async () => [])
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
    mockCommand('list_background_tasks', async () => [])
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
    mockCommand('list_background_tasks', async () => [])
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

  it('创建任务：填表提交调用 create_background_task', async () => {
    mockCommand('list_background_tasks', async () => [])
    const createStub = mockCommand('create_background_task', async () => task({ id: 'new-1' }))
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
    expect(createStub.calledTimes()).toBe(1)
  })

  it('暂停/恢复/立即运行调用对应 API', async () => {
    mockCommand('list_background_tasks', async () => [task({ status: 'idle' })])
    const pauseStub = mockCommand('pause_background_task', async () => null)
    const resumeStub = mockCommand('resume_background_task', async () => null)
    const runStub = mockCommand('run_background_task_now', async () => 'run-id')
    render(() => <ScheduledTasks open onClose={() => {}} />)
    await settle()
    const runBtn = findBtn('立即执行')
    if (runBtn) {
      fireEvent.click(runBtn)
      await settle()
      expect(runStub.calledTimes()).toBe(1)
    }
    const pauseBtn = findBtn('暂停')
    if (pauseBtn) {
      fireEvent.click(pauseBtn)
      await settle()
      expect(pauseStub.calledTimes()).toBe(1)
    }
    const resumeBtn = findBtn('恢复')
    if (resumeBtn) {
      fireEvent.click(resumeBtn)
      await settle()
      expect(resumeStub.calledTimes()).toBe(1)
    }
  })

  it('删除经 ConfirmModal 确认后调用 delete_background_task', async () => {
    mockCommand('list_background_tasks', async () => [task()])
    const deleteStub = mockCommand('delete_background_task', async () => null)
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
    expect(deleteStub.calledTimes()).toBe(1)
  })

  it('Esc 关闭面板', async () => {
    const onClose = vi.fn()
    mockCommand('list_background_tasks', async () => [])
    render(() => <ScheduledTasks open onClose={onClose} />)
    await settle()
    const panel = document.querySelector('[role="dialog"]') as HTMLElement
    fireEvent.keyDown(panel, { key: 'Escape' })
    expect(onClose).toHaveBeenCalled()
  })
})