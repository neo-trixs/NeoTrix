import { describe, it, expect } from 'vitest'
import { foldPreview, guessMime, formatSize, estimateTokens, greeting } from './text'

describe('lib/text — 文本工具函数', () => {
  it('foldPreview 短内容不截断', () => {
    expect(foldPreview('hello', 100)).toBe('hello\n\n…')
  })

  it('foldPreview 未闭合围栏自动补闭合', () => {
    const r = foldPreview('```js\ncode', 10)
    expect(r).toContain('```')
    expect(r.endsWith('…')).toBe(true)
  })

  it('guessMime 常见扩展名映射', () => {
    expect(guessMime('a.png')).toBe('image/png')
    expect(guessMime('b.rs')).toBe('text/rust')
    expect(guessMime('c.pdf')).toBe('application/pdf')
    expect(guessMime('d.unknown')).toBe('application/octet-stream')
  })

  it('formatSize 分级', () => {
    expect(formatSize(500)).toBe('500 B')
    expect(formatSize(2048)).toBe('2.0 KB')
    expect(formatSize(5 * 1024 * 1024)).toBe('5.0 MB')
  })

  it('estimateTokens CJK 每字 1 token，拉丁 4 字 1 token', () => {
    expect(estimateTokens('')).toBe(0)
    expect(estimateTokens('你好')).toBe(2)
    expect(estimateTokens('abcd')).toBe(1)
  })

  it('greeting 返回合法问候语', () => {
    const g = greeting()
    expect(['夜深了', '上午好', '中午好', '下午好', '晚上好']).toContain(g)
  })
})