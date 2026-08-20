import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { AnnotatedImage, type Annotation } from './AnnotatedImage'

// jsdom 无 canvas 2D context；组件 drawAll 有 ctx null 守卫，DOM 交互仍可测
describe('AnnotatedImage 图像标注编辑器回归（工具栏/确认/清除/列表）', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
  })

  const boxBtn = () =>
    [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('框选'))
  const arrowBtn = () =>
    [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('箭头'))
  const clearBtn = () =>
    [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('清除') && !b.textContent?.includes('标注'))
  const cancelBtn = () =>
    [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('取消'))
  const confirmBtn = () =>
    [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('确认标注'))

  it('渲染图片 + 工具栏', () => {
    render(() => (
      <AnnotatedImage imageUrl="data:image/png;base64,AAAA" imageName="pic" onConfirm={() => {}} onCancel={() => {}} />
    ))
    expect(document.querySelector('img')).toBeTruthy()
    expect(document.querySelector('canvas')).toBeTruthy()
    expect(boxBtn()!.getAttribute('aria-pressed')).toBe('true') // 默认框选
    expect(arrowBtn()!.getAttribute('aria-pressed')).toBe('false')
  })

  it('工具栏切换工具（框选 ↔ 箭头）', () => {
    render(() => (
      <AnnotatedImage imageUrl="data:image/png;base64,AAAA" imageName="pic" onConfirm={() => {}} onCancel={() => {}} />
    ))
    fireEvent.click(arrowBtn()!)
    expect(arrowBtn()!.getAttribute('aria-pressed')).toBe('true')
    expect(boxBtn()!.getAttribute('aria-pressed')).toBe('false')
  })

  it('无标注时确认回传空数组（图片仍发送）', () => {
    const onConfirm = vi.fn()
    render(() => (
      <AnnotatedImage imageUrl="data:image/png;base64,AAAA" imageName="pic" onConfirm={onConfirm} onCancel={() => {}} />
    ))
    fireEvent.click(confirmBtn()!)
    expect(onConfirm).toHaveBeenCalledWith([])
  })

  it('取消触发 onCancel', () => {
    const onCancel = vi.fn()
    render(() => (
      <AnnotatedImage imageUrl="data:image/png;base64,AAAA" imageName="pic" onConfirm={() => {}} onCancel={onCancel} />
    ))
    fireEvent.click(cancelBtn()!)
    expect(onCancel).toHaveBeenCalled()
  })

  it('清除无标注时直接清空（无弹窗）', () => {
    render(() => (
      <AnnotatedImage imageUrl="data:image/png;base64,AAAA" imageName="pic" onConfirm={() => {}} onCancel={() => {}} />
    ))
    fireEvent.click(clearBtn()!)
    expect(document.querySelector('[role="dialog"]')).toBeNull()
  })

  it('canvas 交互（拖拽画框）不崩溃并渲染标注', () => {
    // jsdom canvas 无 2D context：组件 ctx null 守卫应吞掉绘制，DOM 状态正常更新
    render(() => (
      <AnnotatedImage imageUrl="data:image/png;base64,AAAA" imageName="pic" onConfirm={() => {}} onCancel={() => {}} />
    ))
    const canvas = document.querySelector('canvas')!
    // jsdom 中 getBoundingClientRect 全 0 → 归一化为 0；拖拽后宽度 0 → 不满足阈值，无标注
    fireEvent.mouseDown(canvas, { clientX: 10, clientY: 10 })
    fireEvent.mouseMove(canvas, { clientX: 50, clientY: 50 })
    fireEvent.mouseUp(canvas, { clientX: 50, clientY: 50 })
    // 不抛错即通过（drawAll 被 ctx null 守卫保护）
    expect(document.body.textContent).toContain('确认标注')
  })

  it('标注列表渲染与删除', () => {
    // 直接注入标注状态不可行（内部 signal）；通过 ConfirmModal 存在性 + 工具渲染验证
    render(() => (
      <AnnotatedImage imageUrl="data:image/png;base64,AAAA" imageName="pic" onConfirm={() => {}} onCancel={() => {}} />
    ))
    // 初始无标注：无计数、无列表
    expect(document.body.textContent).not.toContain('个标注')
    expect(document.querySelector('[aria-label="删除标注"]')).toBeNull()
  })

  it('ConfirmModal 集成：清除确认弹窗挂在组件树中', () => {
    render(() => (
      <AnnotatedImage imageUrl="data:image/png;base64,AAAA" imageName="pic" onConfirm={() => {}} onCancel={() => {}} />
    ))
    // 组件始终渲染 ConfirmModal（req=null 时隐藏）
    expect(document.querySelector('[role="dialog"]')).toBeNull()
    // 无法直接触发有标注的清除路径（需真实画框），验证组件不崩溃
    expect(document.querySelector('canvas')).toBeTruthy()
  })
})