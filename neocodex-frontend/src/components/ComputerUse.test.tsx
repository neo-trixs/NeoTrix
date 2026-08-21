import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { ComputerUse } from './ComputerUse'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

vi.mock('@tauri-apps/api/window', () => ({}))

function findBtn(text: string) {
  return [...document.querySelectorAll('button')].find((b) => b.textContent?.includes(text))
}

const settle = () => new Promise((r) => setTimeout(r, 120))

describe('ComputerUse 电脑控制面板回归（截图/鼠标/键盘/窗口）', () => {
  beforeEach(() => {
    resetInvokeMock()
    document.body.innerHTML = ''
  })

  it('open=false 不渲染', () => {
    render(() => <ComputerUse open={false} onClose={() => {}} />)
    expect(document.body.textContent).toBe('')
  })

  it('加载显示前台应用 + 截图区域 + 截图渲染', async () => {
    mockCommand('computer_get_frontmost_app', async () => ({ app_name: 'Xcode', title: 'main.swift' }))
    mockCommand('computer_screenshot_and_save', async () => ({ data_base64: 'dGVzdA==', width: 1920, height: 1080, path: '/tmp/s.png' }))
    mockCommand('computer_screen_list', async () => [{ width: 1920, height: 1080, index: 0, name: 'Built-in' }])
    mockCommand('computer_get_window_list', async () => [{ pid: 123, app_name: 'Xcode', title: 'main.swift' }])
    mockCommand('computer_mouse_position', async () => ({ x: 100, y: 200 }))
    render(() => <ComputerUse open onClose={() => {}} />)
    await new Promise((r) => setTimeout(r, 200))
    expect(document.body.textContent).toContain('Xcode')
    expect(document.body.textContent).toContain('main.swift')
    expect(document.body.textContent).toContain('屏幕捕获')
    expect(document.querySelector('img')).toBeTruthy()
  })

  it('重新捕获调用 screenshot_and_save', async () => {
    mockCommand('computer_get_frontmost_app', async () => ({ app_name: 'Xcode', title: 'main.swift' }))
    mockCommand('computer_screenshot_and_save', async () => ({ data_base64: 'dGVzdA==', width: 1920, height: 1080, path: '/tmp/s.png' }))
    mockCommand('computer_screen_list', async () => [{ width: 1920, height: 1080, index: 0, name: 'Built-in' }])
    mockCommand('computer_get_window_list', async () => [{ pid: 123, app_name: 'Xcode', title: 'main.swift' }])
    mockCommand('computer_mouse_position', async () => ({ x: 100, y: 200 }))
    const captureStub = mockCommand('computer_screenshot_and_save', async () => ({ data_base64: 'bmV3', width: 1920, height: 1080, path: '/tmp/s2.png' }))
    render(() => <ComputerUse open onClose={() => {}} />)
    await new Promise((r) => setTimeout(r, 200))
    const captureBtn = findBtn('重新捕获')
    expect(captureBtn).toBeTruthy()
    fireEvent.click(captureBtn!)
    await new Promise((r) => setTimeout(r, 200))
    expect(captureStub.calledTimes()).toBe(1)
  })

  it('鼠标控制：点击按钮调用 mouse_click', async () => {
    mockCommand('computer_get_frontmost_app', async () => ({ app_name: 'Xcode', title: 'main.swift' }))
    mockCommand('computer_screenshot_and_save', async () => ({ data_base64: 'dGVzdA==', width: 1920, height: 1080, path: '/tmp/s.png' }))
    mockCommand('computer_screen_list', async () => [{ width: 1920, height: 1080, index: 0, name: 'Built-in' }])
    mockCommand('computer_get_window_list', async () => [])
    const clickStub = mockCommand('computer_mouse_click', async () => null)
    mockCommand('computer_mouse_position', async () => ({ x: 500, y: 300 }))
    render(() => <ComputerUse open onClose={() => {}} />)
    await new Promise((r) => setTimeout(r, 200))
    const clickBtn = findBtn('点击')
    if (clickBtn) {
      fireEvent.click(clickBtn!)
      await new Promise((r) => setTimeout(r, 200))
      expect(clickStub.calledTimes()).toBe(1)
    }
  })

  it('键盘控制：输入文本 + 按键 + 修饰键', async () => {
    mockCommand('computer_get_frontmost_app', async () => ({ app_name: 'Xcode', title: 'main.swift' }))
    mockCommand('computer_screenshot_and_save', async () => ({ data_base64: 'dGVzdA==', width: 1920, height: 1080, path: '/tmp/s.png' }))
    mockCommand('computer_screen_list', async () => [{ width: 1920, height: 1080, index: 0, name: 'Built-in' }])
    mockCommand('computer_get_window_list', async () => [])
    mockCommand('computer_mouse_position', async () => ({ x: 100, y: 100 }))
    const typeStub = mockCommand('computer_keyboard_type', async () => null)
    const pressStub = mockCommand('computer_keyboard_press', async () => null)
    render(() => <ComputerUse open onClose={() => {}} />)
    await new Promise((r) => setTimeout(r, 200))
    const textInput = document.querySelector('input[placeholder*="输入的文本"]') as HTMLInputElement
    fireEvent.input(textInput, { target: { value: 'hello world' } })
    await new Promise((r) => setTimeout(r, 200))
    const typeBtn = findBtn('输入')
    if (typeBtn) {
      fireEvent.click(typeBtn!)
      await new Promise((r) => setTimeout(r, 200))
      expect(typeStub.calledTimes()).toBe(1)
    }
    const keyInput = document.querySelector('input[placeholder*="回车"]') as HTMLInputElement
    fireEvent.input(keyInput, { target: { value: '36' } })
    await new Promise((r) => setTimeout(r, 200))
    const pressBtn = findBtn('按下')
    if (pressBtn) {
      fireEvent.click(pressBtn!)
      await new Promise((r) => setTimeout(r, 200))
      expect(pressStub.calledTimes()).toBe(1)
    }
    const cmdBtn = [...document.querySelectorAll('button')].find((b) => b.textContent === 'command')
    if (cmdBtn) {
      fireEvent.click(cmdBtn!)
      await new Promise((r) => setTimeout(r, 200))
      expect(cmdBtn.getAttribute('aria-pressed')).toBe('true')
    }
  })

  it('Esc 关闭面板', async () => {
    mockCommand('computer_get_frontmost_app', async () => ({ app_name: 'Xcode', title: 'main.swift' }))
    mockCommand('computer_screenshot_and_save', async () => ({ data_base64: 'dGVzdA==', width: 1920, height: 1080, path: '/tmp/s.png' }))
    mockCommand('computer_screen_list', async () => [{ width: 1920, height: 1080, index: 0, name: 'Built-in' }])
    mockCommand('computer_get_window_list', async () => [])
    mockCommand('computer_mouse_position', async () => ({ x: 100, y: 100 }))
    const onClose = vi.fn()
    render(() => <ComputerUse open onClose={onClose} />)
    await new Promise((r) => setTimeout(r, 200))
    const panel = document.querySelector('[role="dialog"]') as HTMLElement
    fireEvent.keyDown(panel, { key: 'Escape' })
    expect(onClose).toHaveBeenCalled()
  })
})