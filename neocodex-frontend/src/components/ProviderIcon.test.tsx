import { describe, it, expect } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ProviderIcon, CategoryBadge, FreeBadge } from './ProviderIcon'

describe('ProviderIcon 品牌标识回归（monogram/品牌色/回退）', () => {
  it('知名提供商显示品牌 monogram', () => {
    render(() => <ProviderIcon name="openai" />)
    expect(document.body.textContent).toBe('O')
  })

  it('anthropic 显示 C', () => {
    render(() => <ProviderIcon name="anthropic" />)
    expect(document.body.textContent).toBe('C')
  })

  it('deepseek-free 归一到 deepseek 品牌 (D)', () => {
    render(() => <ProviderIcon name="deepseek-free" />)
    expect(document.body.textContent).toBe('D')
  })

  it('openrouter 显示双字 monogram (OR)', () => {
    render(() => <ProviderIcon name="openrouter" />)
    expect(document.body.textContent).toBe('OR')
  })

  it('未知名提供商回退取前两字符（去分隔符）', () => {
    render(() => <ProviderIcon name="unknown-vendor" />)
    expect(document.body.textContent).toBe('Un')
  })

  it('未知名提供商回退色来自名称 hash（确定性）', () => {
    const { unmount } = render(() => <ProviderIcon name="weird" />)
    const el = document.querySelector('span') as HTMLElement
    expect(el.style.background).toMatch(/^rgb\(/)
    unmount()
    // 同名称两次渲染背景色一致
    render(() => <ProviderIcon name="weird" />)
    const el2 = document.querySelector('span') as HTMLElement
    expect(el2.style.background).toBe(el.style.background)
  })

  it('大小写不敏感（OPENAI 与 openai 同 glyph）', () => {
    const { unmount } = render(() => <ProviderIcon name="OPENAI" />)
    expect(document.body.textContent).toBe('O')
    unmount()
  })

  it('size 变体切换类名（md 默认 / sm 小）', () => {
    const { unmount } = render(() => <ProviderIcon name="ollama" />)
    expect(document.querySelector('span')!.className).toContain('w-8')
    unmount()
    render(() => <ProviderIcon name="ollama" size="sm" />)
    expect(document.querySelector('span')!.className).toContain('w-6')
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