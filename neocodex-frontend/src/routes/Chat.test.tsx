import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { Chat } from '../routes/Chat'

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

  it('侧栏 seg 已移除协同（单态对话）', () => {
    render(() => <Chat />)
    const tabs = document.querySelectorAll('.seg [role="tab"]')
    expect(tabs.length).toBe(0)
  })

  it('顶部 ch-top 极简：无功能按钮（仅拖拽区）', () => {
    render(() => <Chat />)
    const header = document.querySelector('.ch-top')
    expect(header).toBeTruthy()
    // 顶部不应有 tb-btn（功能按钮已移除，仅保留拖拽区）
    const headerBtns = header!.querySelectorAll('.tb-btn')
    expect(headerBtns.length).toBe(0)
  })

  it('协同入口已移除（单态对话，ch-top 常驻）', () => {
    render(() => <Chat />)
    const tabs = document.querySelectorAll('.seg [role="tab"]')
    expect(tabs.length).toBe(0)
    expect(document.querySelector('.ch-top')).toBeTruthy()
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

  it('权限模式选择器在空态 hero 输入区渲染（对标 Claude 权限模式可见性），可点击切换', () => {
    render(() => <Chat />)
    const selector = document.querySelector('[aria-label="权限模式"]') as HTMLElement | null
    expect(selector).toBeTruthy()
    fireEvent.click(selector!)
    const opts = document.querySelectorAll('[role="option"]')
    expect(opts.length).toBeGreaterThan(0)
  })

  it('功能面板入口已移除（极简侧栏）', () => {
    render(() => <Chat />)
    const group = document.querySelector('[role="group"][aria-label="功能面板"]')
    expect(group).toBeNull()
  })

  it('折叠标签置于三色灯下方且同按钮复用', () => {
    render(() => <Chat />)
    const btn = document.querySelector('[aria-label="折叠侧边栏"]') as HTMLElement
    expect(btn).toBeTruthy()
    // 三色灯占位 h-7 紧邻折叠行
    expect(document.querySelector('[data-tauri-drag-region]')).toBeTruthy()
  })

  it('斜杠 / 菜单包含 model/status/cost/export 命令（对标 Claude Code 命令菜单）', () => {
    render(() => <Chat />)
    // 在输入区输入 / 激活命令菜单
    const textarea = document.querySelector('textarea')!
    fireEvent.input(textarea, { target: { value: '/' } })
    const menu = document.querySelector('.slash-menu')
    expect(menu).toBeTruthy()
    const items = [...menu!.querySelectorAll('.slash-item')].map((el) => el.textContent ?? '')
    expect(items.length).toBeGreaterThanOrEqual(8)
    expect(items.some((t) => t.includes('切换模型'))).toBe(true)
    expect(items.some((t) => t.includes('运行状态'))).toBe(true)
    expect(items.some((t) => t.includes('成本统计'))).toBe(true)
    expect(items.some((t) => t.includes('导出会话'))).toBe(true)
  })

  it('斜杠 /mo 过滤出模型命令（关键词匹配）', () => {
    render(() => <Chat />)
    const textarea = document.querySelector('textarea')!
    fireEvent.input(textarea, { target: { value: '/mo' } })
    const items = [...document.querySelectorAll('.slash-item')].map((el) => el.textContent ?? '')
    expect(items.length).toBeGreaterThan(0)
    expect(items.some((t) => t.includes('切换模型'))).toBe(true)
  })
})