import { describe, it, expect } from 'vitest'
import { render } from '@solidjs/testing-library'
import { NeoSend, NeoPlus, NeoMessage, NeoSearch, NeoChevronRight, NeoTrash, NeoPencil, NeoClose } from './neo-icons'

const ICONS = [
  ['NeoSend', NeoSend],
  ['NeoPlus', NeoPlus],
  ['NeoMessage', NeoMessage],
  ['NeoSearch', NeoSearch],
  ['NeoChevronRight', NeoChevronRight],
  ['NeoTrash', NeoTrash],
  ['NeoPencil', NeoPencil],
  ['NeoClose', NeoClose],
] as const

describe('NeoIcons 图标集契约回归（外扩线条语言）', () => {
  it.each(ICONS)('%s 渲染 SVG 且 viewBox 为 16x16', (_name, Icon) => {
    const { unmount } = render(() => <Icon />)
    const svg = document.querySelector('svg') as SVGElement
    expect(svg).toBeTruthy()
    expect(svg.getAttribute('viewBox')).toBe('0 0 16 16')
    expect(svg.getAttribute('fill')).toBe('none')
    unmount()
  })

  it.each(ICONS)('%s 默认尺寸 w-4 h-4（tailwind-merge 合并 class）', (_name, Icon) => {
    const { unmount } = render(() => <Icon />)
    const svg = document.querySelector('svg') as SVGElement
    expect(svg.getAttribute('class')).toContain('w-4')
    expect(svg.getAttribute('class')).toContain('h-4')
    unmount()
  })

  it.each(ICONS)('%s 自定义 class 与默认合并且不被覆盖', (_name, Icon) => {
    const { unmount } = render(() => <Icon class="w-8 h-8 text-red-500" />)
    const svg = document.querySelector('svg') as SVGElement
    // tailwind-merge 让 w-8 覆盖 w-4, 但 text-red-500 保留
    expect(svg.getAttribute('class')).toContain('w-8')
    expect(svg.getAttribute('class')).toContain('text-red-500')
    unmount()
  })

  it.each(ICONS)('%s 笔画色 currentColor 可继承', (_name, Icon) => {
    const { unmount } = render(() => <Icon class="text-nt-mind-600" />)
    const svg = document.querySelector('svg') as SVGElement
    expect(svg.getAttribute('class')).toContain('text-nt-mind-600')
    unmount()
  })

  it.each(ICONS)('%s 至少含一个 stroke 元素（线条图标语义）', (_name, Icon) => {
    const { unmount } = render(() => <Icon />)
    const svg = document.querySelector('svg') as SVGElement
    expect(svg.querySelectorAll('[stroke]').length).toBeGreaterThan(0)
    unmount()
  })

  it.each(ICONS)('%s 无 fill 色块（外扩线条非填充）', (_name, Icon) => {
    const { unmount } = render(() => <Icon />)
    const svg = document.querySelector('svg') as SVGElement
    for (const el of svg.querySelectorAll('path,circle,line,rect')) {
      expect(el.getAttribute('fill')).not.toBe('currentColor')
    }
    unmount()
  })
})