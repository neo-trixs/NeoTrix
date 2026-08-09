import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { Chat } from '../routes/Chat'
import { chatStore } from '../stores/chat'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === 'neocodex_provider_config') {
      return Promise.resolve({ providers: [], active_model: '' })
    }
    return Promise.resolve([])
  }),
}))
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  UnlistenFn: vi.fn(),
}))

describe('Chat 主界面全 UI 冒烟测试（对标 Claude Code 布局）', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('侧栏 seg 只有 对话/协同 两个标签（电脑标签已移除）', () => {
    render(() => <Chat />)
    const tabs = document.querySelectorAll('[role="tab"]')
    expect(tabs.length).toBe(2)
    expect(tabs[0].textContent).toContain('对话')
    expect(tabs[1].textContent).toContain('协同')
  })

  it('功能入口区含 6 个按钮（Git/任务/成本/时间线/侧向对话/电脑控制）', () => {
    render(() => <Chat />)
    const toolbar = document.querySelector('[role="toolbar"]')
    expect(toolbar).toBeTruthy()
    const labels = ['Git 变更', '定时任务', '成本看板', '时间线', '侧向对话', '电脑控制']
    for (const label of labels) {
      expect(toolbar!.querySelector(`[aria-label="${label}"]`)).toBeTruthy()
    }
  })

  it('顶部 ch-top 极简：无功能按钮（仅会话标题）', () => {
    render(() => <Chat />)
    const header = document.querySelector('.ch-top')
    expect(header).toBeTruthy()
    // 顶部不应有 tb-btn（功能按钮已移到侧栏）
    const headerBtns = header!.querySelectorAll('.tb-btn')
    expect(headerBtns.length).toBe(0)
  })

  it('点击功能入口区 Git 按钮打开 Git 面板', () => {
    render(() => <Chat />)
    const toolbar = document.querySelector('[role="toolbar"]')!
    const gitBtn = toolbar.querySelector('[aria-label="Git 变更"]')!
    fireEvent.click(gitBtn)
    // Git 面板应渲染（git-panel 或包含 Git 变更内容的面板）
    const panel = document.querySelector('[class*="panel"]')
    expect(panel).toBeTruthy()
  })

  it('点击电脑控制按钮切换视图', () => {
    render(() => <Chat />)
    const toolbar = document.querySelector('[role="toolbar"]')!
    const computerBtn = toolbar.querySelector('[aria-label="电脑控制"]')!
    fireEvent.click(computerBtn)
    // 电脑视图：chat 视图的 header 应隐藏（activeView 切到 computer）
    expect(document.querySelector('.ch-top')).toBeNull()
  })
})