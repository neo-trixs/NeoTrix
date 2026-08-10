import { describe, it, expect } from 'vitest'
import { render } from '@solidjs/testing-library'
import { NeoTag } from './NeoTag'

describe('NeoTag', () => {
  it('渲染标签名与色点', () => {
    const { container } = render(() => (
      <NeoTag name="bug" color="#e85454" />
    ))
    expect(container.textContent).toContain('bug')
    const dot = container.querySelector('.nt-tag-dot')
    expect(dot).toBeTruthy()
  })

  it('嵌套标签显示层级提示', () => {
    const { container } = render(() => (
      <NeoTag name="design/ui" color="#f0913a" showHierarchy />
    ))
    expect(container.querySelector('.nt-tag-hint')).toBeTruthy()
    expect(container.textContent).toContain('design')
    expect(container.textContent).toContain('ui')
    expect(container.querySelector('.nt-tag-slash')).toBeTruthy()
  })

  it('计数徽章渲染', () => {
    const { container } = render(() => (
      <NeoTag name="fix" color="#16a34a" count={3} />
    ))
    expect(container.textContent).toContain('3')
    expect(container.querySelector('.nt-tag-count')).toBeTruthy()
  })

  it('remove 按钮触发 onRemove（不冒泡 onClick）', () => {
    let removed: string | null = null
    let clickCount = 0
    const { container } = render(() => (
      <NeoTag
        name="debug"
        color="#2563eb"
        removable
        onRemove={(n) => { removed = n }}
        onClick={() => { clickCount++ }}
      />
    ))
    const xBtn = container.querySelector('.nt-tag-x') as HTMLButtonElement
    expect(xBtn).toBeTruthy()
    xBtn.click()
    expect(removed).toBe('debug')
    expect(clickCount).toBe(0) // 移除不触发点击筛选
  })

  it('active 态应用选中 class', () => {
    const { container } = render(() => (
      <NeoTag name="active" color="#9333ea" active />
    ))
    const tag = container.querySelector('.nt-tag') as HTMLElement
    expect(tag.classList.contains('nt-tag-active')).toBe(true)
  })
})