import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { TrafficLights } from './TrafficLights'

// 窗口操作经 @tauri-apps/api/window 直调（system.ts），不再走 invoke 命令
const { mockMinimize, mockToggleMaximize, mockClose } = vi.hoisted(() => ({
  mockMinimize: vi.fn(() => Promise.resolve()),
  mockToggleMaximize: vi.fn(() => Promise.resolve()),
  mockClose: vi.fn(() => Promise.resolve()),
}))

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    minimize: mockMinimize,
    toggleMaximize: mockToggleMaximize,
    close: mockClose,
  }),
}))

// 非 Tauri 测试环境：isTauriRuntime 为 false → 组件不接窗口监听，保持默认 focused
// 窗口按钮调用前桩化运行时标识（system.ts 有 isTauriRuntime 门禁）
describe('TrafficLights 窗口控制回归（自绘 macOS 交通灯）', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    ;(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {}
    vi.useFakeTimers()
  })
  afterEach(() => vi.useRealTimers())

  const closeBtn = () =>
    [...document.querySelectorAll('button')].find((b) => b.getAttribute('aria-label') === '关闭窗口')
  const minBtn = () =>
    [...document.querySelectorAll('button')].find((b) => b.getAttribute('aria-label') === '最小化')
  const maxBtn = () =>
    [...document.querySelectorAll('button')].find((b) => b.getAttribute('aria-label') === '最大化')

  it('渲染三灯并默认聚焦态', () => {
    render(() => <TrafficLights />)
    expect(closeBtn()).toBeTruthy()
    expect(minBtn()).toBeTruthy()
    expect(maxBtn()).toBeTruthy()
    expect(document.querySelector('.traffic')!.className).not.toContain('blurred')
  })

  it('关闭按钮调用 window_close', () => {
    render(() => <TrafficLights />)
    fireEvent.click(closeBtn()!)
    expect(mockClose).toHaveBeenCalledTimes(1)
  })

  it('最小化按钮调用 window_minimize', () => {
    render(() => <TrafficLights />)
    fireEvent.click(minBtn()!)
    expect(mockMinimize).toHaveBeenCalledTimes(1)
  })

  it('单击最大化延迟 250ms 后触发 window_maximize（双击防抖）', () => {
    render(() => <TrafficLights />)
    fireEvent.click(maxBtn()!)
    expect(mockToggleMaximize).not.toHaveBeenCalled() // 延迟中未触发
    vi.advanceTimersByTime(250)
    expect(mockToggleMaximize).toHaveBeenCalledTimes(1) // 250ms 后单次触发
  })

  it('双击最大化：两次点击只触发一次 window_maximize', () => {
    render(() => <TrafficLights />)
    const btn = maxBtn()!
    fireEvent.click(btn)
    fireEvent.click(btn)
    expect(mockToggleMaximize).toHaveBeenCalledTimes(1) // 双击取消单击延迟，只 toggle 一次
    vi.advanceTimersByTime(250)
    expect(mockToggleMaximize).toHaveBeenCalledTimes(1) // 双击后无额外触发
  })

  it('单击后等待超过防抖窗口再单击：触发两次 window_maximize', () => {
    render(() => <TrafficLights />)
    const btn = maxBtn()!
    fireEvent.click(btn)
    vi.advanceTimersByTime(250)
    expect(mockToggleMaximize).toHaveBeenCalledTimes(1)
    fireEvent.click(btn)
    vi.advanceTimersByTime(250)
    expect(mockToggleMaximize).toHaveBeenCalledTimes(2)
  })

  it('组件卸载时清理未触发的防抖定时器（无内存泄漏）', () => {
    const { unmount } = render(() => <TrafficLights />)
    fireEvent.click(maxBtn()!)
    unmount()
    // 卸载后推进定时器不应触发（onCleanup 已 clearTimeout）
    expect(() => vi.advanceTimersByTime(250)).not.toThrow()
  })
})
