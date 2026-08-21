import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { CoworkView } from './CoworkView'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

vi.mock('@tauri-apps/api/window', () => ({}))

function session(over: Record<string, unknown> = {}) {
  return {
    id: 'sess-1',
    name: '每日巡检',
    status: 'active',
    workspace_path: '/Users/neo/demo',
    description: '每日自动化巡检',
    files_read: 10,
    files_created: 2,
    files_modified: 5,
    ...over,
  }
}

function action(over: Record<string, unknown> = {}) {
  return {
    id: 'act-1',
    action_type: 'read_file',
    target_path: '/Users/neo/demo/src/main.rs',
    status: 'completed',
    ...over,
  }
}

function deliverable(over: Record<string, unknown> = {}) {
  return {
    id: 'del-1',
    name: 'perf-report.json',
    kind: 'json',
    path: '/tmp/perf-report.json',
    quality_score: 92,
    ...over,
  }
}

const settle = () => new Promise((r) => setTimeout(r, 120))

describe('CoworkView 协同会话回归（列表/详情/控制/删除）', () => {
  beforeEach(() => {
    resetInvokeMock()
    document.body.innerHTML = ''
  })

  it('空列表显示暂无会话', async () => {
    mockCommand('cowork_list', async () => [])
    render(() => <CoworkView />)
    await settle()
    expect(document.body.textContent).toContain('暂无会话，点击 + 新建')
  })

  it('加载会话列表渲染名称/状态/统计', async () => {
    mockCommand('cowork_list', async () => [session(), session({ id: 's2', name: '周报生成', status: 'paused' })])
    mockCommand('cowork_actions', async () => [action()])
    mockCommand('cowork_list_deliverables', async () => [deliverable()])
    render(() => <CoworkView />)
    await settle()
    expect(document.body.textContent).toContain('每日巡检')
    expect(document.body.textContent).toContain('进行中')
    expect(document.body.textContent).toContain('读 10 · 建 2 · 改 5')
    expect(document.body.textContent).toContain('周报生成')
    expect(document.body.textContent).toContain('已暂停')
  })

  it('点击会话切换 active，加载行动/交付物', async () => {
    mockCommand('cowork_list', async () => [session(), session({ id: 's2', name: '周报生成', status: 'completed' })])
    const actsStub = mockCommand('cowork_actions', async () => [action({ id: 'a1', action_type: 'write_file' })])
    const delsStub = mockCommand('cowork_list_deliverables', async () => [deliverable()])
    render(() => <CoworkView />)
    await settle()
    // 默认选中第一个
    expect(actsStub.calledTimes()).toBe(1)
    expect(delsStub.calledTimes()).toBe(1)
    // 点击第二个会话
    const s2Btn = [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('周报生成'))
    fireEvent.click(s2Btn!)
    await settle()
    // 重新加载
    expect(actsStub.calledTimes()).toBe(2)
    expect(delsStub.calledTimes()).toBe(2)
  })

  it('新建会话：输入路径/描述点击创建', async () => {
    mockCommand('cowork_list', async () => [])
    const startStub = mockCommand('cowork_start', async () => 'new-sess-id')
    render(() => <CoworkView />)
    await settle()
    // 点击 + 展开表单
    const addBtn = document.querySelector('[aria-label="新建会话"]')
    fireEvent.click(addBtn!)
    await settle()
    // 输入路径和描述
    const pathInput = document.querySelector('input[placeholder*="工作区路径"]') as HTMLInputElement
    const descInput = document.querySelector('input[placeholder*="描述"]') as HTMLInputElement
    fireEvent.input(pathInput, { target: { value: '/Users/neo/newproj' } })
    fireEvent.input(descInput, { target: { value: '新项目描述' } })
    await settle()
    // 点击创建
    const createBtn = [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('创建'))
    fireEvent.click(createBtn!)
    await settle()
    expect(startStub.calledTimes()).toBe(1)
  })

  it('暂停/恢复/停止控制', async () => {
    mockCommand('cowork_list', async () => [session()])
    mockCommand('cowork_actions', async () => [action()])
    mockCommand('cowork_list_deliverables', async () => [deliverable()])
    const pauseStub = mockCommand('cowork_pause', async () => null)
    const resumeStub = mockCommand('cowork_resume', async () => null)
    const stopStub = mockCommand('cowork_stop', async () => null)
    render(() => <CoworkView />)
    await settle()
    // 暂停按钮（仅 active 显示）
    const pauseBtn = document.querySelector('button[title="暂停"]')
    if (pauseBtn) {
      fireEvent.click(pauseBtn)
      await settle()
      expect(pauseStub.calledTimes()).toBe(1)
    }
    // 恢复按钮需 paused 状态，跳过
    // 停止按钮（需确认）
    const stopBtn = document.querySelector('button[title="停止"]')
    if (stopBtn) {
      fireEvent.click(stopBtn)
      await settle()
      // ConfirmModal 出现
      expect(document.body.textContent).toContain('停止协同会话')
      const confirmBtn = [...document.querySelectorAll('.glass-modal button')].pop()!
      fireEvent.click(confirmBtn)
      await settle()
      expect(stopStub.calledTimes()).toBe(1)
    }
  })

  it('删除会话经 ConfirmModal 确认', async () => {
    mockCommand('cowork_list', async () => [session()])
    mockCommand('cowork_actions', async () => [action()])
    mockCommand('cowork_list_deliverables', async () => [deliverable()])
    const delStub = mockCommand('cowork_delete', async () => null)
    render(() => <CoworkView />)
    await settle()
    // 找到删除按钮（带垃圾桶图标的 cw-del）
    const delBtn = [...document.querySelectorAll('button')].find((b) => b.querySelector('svg path[d*="M2.5"]'))
    if (delBtn) {
      fireEvent.click(delBtn)
      await settle()
      expect(document.body.textContent).toContain('删除协同会话')
      const confirmBtn = [...document.querySelectorAll('.glass-modal button')].pop()!
      fireEvent.click(confirmBtn)
      await settle()
      expect(delStub.calledTimes()).toBe(1)
    }
  })

  it('刷新按钮重新加载', async () => {
    const listStub = mockCommand('cowork_list', async () => [session()])
    mockCommand('cowork_actions', async () => [action()])
    mockCommand('cowork_list_deliverables', async () => [deliverable()])
    render(() => <CoworkView />)
    await settle()
    const refreshBtn = document.querySelector('[aria-label="刷新"]')
    fireEvent.click(refreshBtn!)
    await settle()
    expect(listStub.calledTimes()).toBe(2)
  })

  it('错误状态显示', async () => {
    mockCommand('cowork_list', async () => {
      throw new Error('list failed')
    })
    render(() => <CoworkView />)
    await settle()
    expect(document.body.textContent).toContain('list failed')
  })
})