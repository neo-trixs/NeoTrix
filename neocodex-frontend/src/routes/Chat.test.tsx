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

  it('侧栏 seg 有 对话/协同/电脑 三个标签', () => {
    render(() => <Chat />)
    const tabs = document.querySelectorAll('[role="tab"]')
    expect(tabs.length).toBe(3)
    expect(tabs[0].textContent).toContain('对话')
    expect(tabs[1].textContent).toContain('协同')
    expect(tabs[2].textContent).toContain('电脑')
  })

  it('顶部 ch-top 极简：无功能按钮（仅拖拽区）', () => {
    render(() => <Chat />)
    const header = document.querySelector('.ch-top')
    expect(header).toBeTruthy()
    // 顶部不应有 tb-btn（功能按钮已移除，仅保留拖拽区）
    const headerBtns = header!.querySelectorAll('.tb-btn')
    expect(headerBtns.length).toBe(0)
  })

  it('点击协同 seg 标签切换到协同视图（CoworkView）', () => {
    render(() => <Chat />)
    const coworkTab = document.querySelectorAll('[role="tab"]')[1]
    fireEvent.click(coworkTab)
    // 协同视图应渲染（chat header 隐藏）
    expect(document.querySelector('.ch-top')).toBeNull()
  })

  it('侧栏用户条点击打开设置弹窗，含插件 section（技能插件市场放设置）', () => {
    render(() => <Chat />)
    const sfBtn = document.querySelector('[aria-label="用户设置"]')!
    fireEvent.click(sfBtn)
    // 设置弹窗应渲染（role=dialog + aria-label）
    const dialog = document.querySelector('[role="dialog"][aria-label="设置"]')
    expect(dialog).toBeTruthy()
    // 点击插件 section，应渲染插件市场
    const pluginTab = [...dialog!.querySelectorAll('[role="tab"]')].find(t => t.textContent?.includes('插件'))
    expect(pluginTab).toBeTruthy()
    fireEvent.click(pluginTab!)
    expect(document.querySelector('.panel-title')?.textContent).toContain('插件市场')
  })
})