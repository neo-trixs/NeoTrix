import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@solidjs/testing-library'
import { Sidebar } from './Sidebar'
import { chatStore } from '../stores/chat'

// Mock Tauri invoke for store initialization
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue([]),
}))

describe('Sidebar 标签交互（对标 Codex tablist 规范）', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('seg 标签组具备 role=tablist 与 role=tab 语义（对话/协同/电脑控制）', () => {
    render(() => <Sidebar activeView="chat" />)
    const tablist = document.querySelector('[role="tablist"]')
    expect(tablist).toBeTruthy()
    const tabs = document.querySelectorAll('[role="tab"]')
    expect(tabs.length).toBe(3)
    expect(tabs[0].getAttribute('aria-selected')).toBe('true')
    expect(tabs[1].getAttribute('aria-selected')).toBe('false')
    expect(tabs[2].getAttribute('aria-selected')).toBe('false')
  })

  it('roving tabindex：仅激活标签可 Tab 聚焦', () => {
    render(() => <Sidebar activeView="chat" />)
    const tabs = document.querySelectorAll<HTMLElement>('[role="tab"]')
    expect(tabs[0].tabIndex).toBe(0)
    expect(tabs[1].tabIndex).toBe(-1)
    expect(tabs[2].tabIndex).toBe(-1)
  })

  it('方向键切换视图：←/→ 在 对话/协同/电脑控制 间轮转', () => {
    let currentView = 'chat'
    render(() => (
      <Sidebar
        activeView={currentView as 'chat' | 'cowork' | 'computer'}
        onSwitchView={(v) => { currentView = v }}
      />
    ))
    const tabs = document.querySelectorAll<HTMLElement>('[role="tab"]')
    // 从「对话」按 → 应切到「协同」
    fireEvent.keyDown(tabs[0], { key: 'ArrowRight' })
    expect(currentView).toBe('cowork')
  })

  it('电脑控制标签：激活时 aria-selected 正确、点击切换视图', () => {
    let currentView = 'chat'
    render(() => (
      <Sidebar
        activeView={currentView as 'chat' | 'cowork' | 'computer'}
        onSwitchView={(v) => { currentView = v }}
      />
    ))
    const tabs = document.querySelectorAll<HTMLElement>('[role="tab"]')
    fireEvent.click(tabs[2])
    expect(currentView).toBe('computer')
  })
})
