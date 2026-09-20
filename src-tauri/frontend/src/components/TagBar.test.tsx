import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { TagBar } from './TagBar'
import { tagsStore } from '../stores/tags'

describe('TagBar 标签树回归（层级折叠/多选/空态引导）', () => {
  beforeEach(() => {
    // 重置 store 为持久化快照（测试隔离）；随后按用例显式 seed 或清空
    tagsStore.reset()
  })

  it('空态：无标签时显示推荐标签引导 + 一键添加', () => {
    // reset 后 localStorage 为空（setup 初始 Map）→ tags 注册表空
    const { unmount } = render(() => (
      <TagBar activeTags={[]} onToggleTag={() => {}} onClearTags={() => {}} />
    ))
    expect(document.body.textContent).toContain('暂无标签')
    expect(document.body.textContent).toContain('一键添加推荐标签')
    unmount()
  })

  it('点击一键添加推荐标签后显示标签树', () => {
    const { unmount } = render(() => (
      <TagBar activeTags={[]} onToggleTag={() => {}} onClearTags={() => {}} />
    ))
    expect(document.querySelector('[role="list"]')).toBeNull() // 空态无树
    fireEvent.click(document.querySelector('[aria-label="一键添加推荐标签"]')!)
    // seed 后立即显示完整树（fallback 按钮随空态消失）
    expect(document.querySelector('[role="list"]')).toBeTruthy()
    expect(document.body.textContent).toContain('工作/功能')
    unmount()
  })

  it('根标签显示在树中', () => {
    tagsStore.seedRecommendedTags()
    const { unmount } = render(() => (
      <TagBar activeTags={[]} onToggleTag={() => {}} onClearTags={() => {}} />
    ))
    expect(document.body.textContent).toContain('工作')
    expect(document.body.textContent).toContain('领域')
    unmount()
  })

  it('点击根标签触发 onToggleTag', () => {
    tagsStore.seedRecommendedTags()
    const onToggle = vi.fn()
    const { unmount } = render(() => (
      <TagBar activeTags={[]} onToggleTag={onToggle} onClearTags={() => {}} />
    ))
    // 点击 "工作" 根标签（树中的 NeoTag）
    const workTag = [...document.querySelectorAll('[role="option"], span, button')].find((el) =>
      el.textContent?.includes('工作') && !el.textContent?.includes('功能')
    )
    fireEvent.click(workTag!)
    expect(onToggle).toHaveBeenCalledWith('工作')
    unmount()
  })

  it('折叠/展开根标签：点击箭头隐藏子标签', () => {
    tagsStore.seedRecommendedTags()
    const { unmount } = render(() => (
      <TagBar activeTags={[]} onToggleTag={() => {}} onClearTags={() => {}} />
    ))
    // 初始展开：显示子标签（功能/修复）
    expect(document.body.textContent).toContain('功能')
    // 点击折叠箭头
    const collapseBtn = document.querySelector('[aria-label="折叠 工作"]') as HTMLButtonElement
    fireEvent.click(collapseBtn)
    // 折叠后子标签隐藏
    expect(document.body.textContent).not.toContain('功能')
    // aria-expanded 更新
    expect(collapseBtn.getAttribute('aria-expanded')).toBe('false')
    unmount()
  })

  it('活跃标签计数与清除筛选', () => {
    tagsStore.seedRecommendedTags()
    const onClear = vi.fn()
    const { unmount } = render(() => (
      <TagBar activeTags={['工作', '领域']} onToggleTag={() => {}} onClearTags={onClear} />
    ))
    expect(document.body.textContent).toContain('清除 (2)')
    fireEvent.click(document.querySelector('[aria-label="清除标签筛选"]')!)
    expect(onClear).toHaveBeenCalled()
    unmount()
  })

  it('无活跃标签时隐藏清除按钮', () => {
    tagsStore.seedRecommendedTags()
    const { unmount } = render(() => (
      <TagBar activeTags={[]} onToggleTag={() => {}} onClearTags={() => {}} />
    ))
    expect(document.querySelector('[aria-label="清除标签筛选"]')).toBeNull()
    unmount()
  })

  it('活跃标签高亮（NeoTag active 态）', () => {
    tagsStore.seedRecommendedTags()
    const { unmount } = render(() => (
      <TagBar activeTags={['领域']} onToggleTag={() => {}} onClearTags={() => {}} />
    ))
    const activeTag = document.querySelector('.nt-tag-active')
    expect(activeTag).toBeTruthy()
    expect(activeTag!.textContent).toContain('领域')
    unmount()
  })
})