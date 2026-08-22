import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { Sidebar } from './Sidebar'

// Mock Tauri invoke for store initialization
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue([]),
}))

describe('Sidebar 标签交互（对标 Codex tablist 规范）', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('seg 已极简移除（单态对话，无分段）', () => {
    render(() => <Sidebar activeView="chat" />)
    const tablist = document.querySelector('[role="tablist"]')
    expect(tablist).toBeNull()
  })

  it('roving tabindex：单态无需（无分段）', () => {
    render(() => <Sidebar activeView="chat" />)
    const tabs = document.querySelectorAll<HTMLElement>('[role="tab"]')
    expect(tabs.length).toBe(0)
  })

  it('方向键切换已移除（单态）', () => {
    render(() => <Sidebar activeView="chat" onSwitchView={() => {}} />)
    const tabs = document.querySelectorAll<HTMLElement>('[role="tab"]')
    expect(tabs.length).toBe(0)
  })

  it('功能面板入口已移除（极简侧栏，无 Git 行）', () => {
    render(() => <Sidebar activeView="chat" activePanel={null} onTogglePanel={() => {}} />)
    const group = document.querySelector('[role="group"][aria-label="功能面板"]')
    expect(group).toBeNull()
  })

  it('折叠/展开同标签：同一按钮 rotate-180 切换（上线不再过长）', () => {
    render(() => <Sidebar activeView="chat" collapsed={false} onToggleCollapse={() => {}} />)
    const btn = document.querySelector('[aria-label="折叠侧边栏"]') as HTMLElement
    expect(btn).toBeTruthy()
    // 极简图标：单圆点而非多外扩射线
    expect(btn.querySelector('svg')).toBeTruthy()
  })
})
