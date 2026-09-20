import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { PermissionModeSelector, type PermissionMode } from './PermissionModeSelector'

describe('PermissionModeSelector 权限模式下拉回归（compact + 完整版）', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  const openBtn = () =>
    document.querySelector<HTMLButtonElement>('[aria-label="权限模式"]')

  const options = () => document.querySelectorAll('[role="option"]')

  it('compact 渲染当前模式', () => {
    render(() => (
      <PermissionModeSelector value="auto" onChange={() => {}} compact />
    ))
    expect(openBtn()!.textContent).toContain('自动')
    expect(openBtn()!.getAttribute('aria-expanded')).toBe('false')
  })

  it('点击展开下拉并显示 4 个模式', () => {
    render(() => (
      <PermissionModeSelector value="auto" onChange={() => {}} compact />
    ))
    fireEvent.click(openBtn()!)
    expect(openBtn()!.getAttribute('aria-expanded')).toBe('true')
    expect(options().length).toBe(4)
    expect(document.body.textContent).toContain('手动')
    expect(document.body.textContent).toContain('接受编辑')
    expect(document.body.textContent).toContain('规划模式')
  })

  it('选择模式触发 onChange 并关闭下拉', () => {
    const onChange = vi.fn()
    render(() => (
      <PermissionModeSelector value="auto" onChange={onChange} compact />
    ))
    fireEvent.click(openBtn()!)
    const manual = [...options()].find((o) => o.textContent?.includes('手动'))
    fireEvent.click(manual!)
    expect(onChange).toHaveBeenCalledWith('manual')
    expect(openBtn()!.getAttribute('aria-expanded')).toBe('false')
  })

  it('当前选中模式带 aria-selected + 对勾', () => {
    render(() => (
      <PermissionModeSelector value="plan" onChange={() => {}} compact />
    ))
    fireEvent.click(openBtn()!)
    const plan = [...options()].find((o) => o.textContent?.includes('规划模式'))
    expect(plan!.getAttribute('aria-selected')).toBe('true')
    expect(document.querySelector('svg')).toBeTruthy() // Check 图标
  })

  it('Esc 关闭下拉（onMount 全局监听）', () => {
    render(() => (
      <PermissionModeSelector value="auto" onChange={() => {}} compact />
    ))
    fireEvent.click(openBtn()!)
    expect(options().length).toBe(4)
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    expect(options().length).toBe(0)
    expect(openBtn()!.getAttribute('aria-expanded')).toBe('false')
  })

  it('方向键导航高亮（ArrowDown 循环）', async () => {
    render(() => (
      <PermissionModeSelector value="auto" onChange={() => {}} compact />
    ))
    fireEvent.click(openBtn()!)
    const first = options()[0] as HTMLButtonElement
    first.focus()
    fireEvent.keyDown(first, { key: 'ArrowDown' })
    // rAF 后焦点移到下一项
    await new Promise((r) => requestAnimationFrame(r))
    const second = options()[1] as HTMLButtonElement
    expect(document.activeElement).toBe(second)
  })

  it('disabled 时点击不展开', () => {
    render(() => (
      <PermissionModeSelector value="auto" onChange={() => {}} compact disabled />
    ))
    expect(openBtn()!.disabled).toBe(true)
    fireEvent.click(openBtn()!)
    expect(options().length).toBe(0)
  })

  it('完整版渲染当前模式并展开列表', () => {
    render(() => (
      <PermissionModeSelector value="accept_edits" onChange={() => {}} />
    ))
    expect(openBtn()!.textContent).toContain('接受编辑')
    fireEvent.click(openBtn()!)
    expect(options().length).toBe(4)
  })

  it('遮罩点击关闭下拉', () => {
    render(() => (
      <PermissionModeSelector value="auto" onChange={() => {}} compact />
    ))
    fireEvent.click(openBtn()!)
    expect(options().length).toBe(4)
    // 遮罩是 fixed inset-0 元素（aria-hidden）
    const overlay = document.querySelector('.fixed.inset-0') as HTMLElement
    fireEvent.click(overlay)
    expect(options().length).toBe(0)
  })

  it('选择后遮罩隐藏（.hidden class）', () => {
    render(() => (
      <PermissionModeSelector value="auto" onChange={() => {}} compact />
    ))
    fireEvent.click(openBtn()!)
    const overlay = document.querySelector('.fixed.inset-0') as HTMLElement
    expect(overlay.className).not.toContain('hidden')
    fireEvent.click([...options()][0]!)
    expect(overlay.className).toContain('hidden')
  })
})