import { describe, it, expect } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ProviderIcon, CategoryBadge, FreeBadge } from './ProviderIcon'

describe('ProviderIcon 六边形几何标识（E8 Hexagram 语义）', () => {
  it('知名提供商渲染 SVG 六边形', () => {
    render(() => <ProviderIcon name="openai" category="cloud" />)
    const svg = document.querySelector('svg')
    expect(svg).toBeTruthy()
    const hex = svg!.querySelector('path')
    expect(hex).toBeTruthy()
  })

  it('本地提供商渲染菱形内纹', () => {
    render(() => <ProviderIcon name="ollama" category="local" />)
    const svg = document.querySelector('svg')
    expect(svg).toBeTruthy()
    // local 类型应有内嵌菱形路径
    const innerPaths = svg!.querySelectorAll('path')
    expect(innerPaths.length).toBeGreaterThan(1)
  })

  it('代理提供商渲染同心环', () => {
    render(() => <ProviderIcon name="openrouter" category="proxy" />)
    const svg = document.querySelector('svg')
    expect(svg).toBeTruthy()
    const circles = svg!.querySelectorAll('circle')
    expect(circles.length).toBeGreaterThanOrEqual(3)
  })

  it('云端提供商渲染点阵', () => {
    render(() => <ProviderIcon name="anthropic" category="cloud" />)
    const svg = document.querySelector('svg')
    expect(svg).toBeTruthy()
    const dots = svg!.querySelectorAll('circle')
    expect(dots.length).toBeGreaterThan(0)
  })

  it('未知分类渲染默认样式', () => {
    render(() => <ProviderIcon name="weird" />)
    const svg = document.querySelector('svg')
    expect(svg).toBeTruthy()
  })

  it('大小写不敏感', () => {
    const { unmount } = render(() => <ProviderIcon name="OPENAI" category="cloud" />)
    const svg1 = document.querySelector('svg')
    expect(svg1).toBeTruthy()
    unmount()
    render(() => <ProviderIcon name="openai" category="cloud" />)
    const svg2 = document.querySelector('svg')
    expect(svg2).toBeTruthy()
  })

  it('size 变体切换 sm/md', () => {
    const { unmount } = render(() => <ProviderIcon name="ollama" category="local" />)
    const svg1 = document.querySelector('svg')
    expect(svg1!.getAttribute('width')).toBe('32')
    unmount()
    render(() => <ProviderIcon name="ollama" size="sm" category="local" />)
    const svg2 = document.querySelector('svg')
    expect(svg2!.getAttribute('width')).toBe('24')
  })
})

describe('CategoryBadge 分类徽章回归', () => {
  it('local 显示本地', () => {
    render(() => <CategoryBadge category="local" />)
    expect(document.body.textContent).toBe('本地')
  })

  it('proxy 显示代理', () => {
    render(() => <CategoryBadge category="proxy" />)
    expect(document.body.textContent).toBe('代理')
  })

  it('cloud 显示云端', () => {
    render(() => <CategoryBadge category="cloud" />)
    expect(document.body.textContent).toBe('云端')
  })

  it('未知分类显示未知 + title', () => {
    render(() => <CategoryBadge category="weird" />)
    expect(document.body.textContent).toBe('未知')
    expect(document.querySelector('span')!.getAttribute('title')).toContain('未知')
  })
})

describe('FreeBadge 免费徽章回归', () => {
  it('free=true 显示免费', () => {
    render(() => <FreeBadge free />)
    expect(document.body.textContent).toBe('免费')
  })

  it('free=false 不渲染', () => {
    const { unmount } = render(() => <FreeBadge free={false} />)
    expect(document.body.textContent).toBe('')
    unmount()
  })
})
