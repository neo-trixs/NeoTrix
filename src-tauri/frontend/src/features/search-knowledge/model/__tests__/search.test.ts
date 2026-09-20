import { describe, it, expect } from 'vitest'
import { normalizeQuery, buildSearchParams, scoreMatch } from '../index'
import { highlightMatch } from '../../ui/index'

describe('features/search-knowledge', () => {
  it('normalizeQuery 去空格并小写', () => {
    expect(normalizeQuery('  Hello  WORLD  ')).toBe('hello world')
  })

  it('buildSearchParams 构建参数', () => {
    expect(buildSearchParams({ text: '  Rust  ', tags: ['a','b'], limit: 10 })).toEqual({ q: 'rust', tags: 'a,b', limit: '10' })
    expect(buildSearchParams({ text: 'hi' })).toEqual({ q: 'hi' })
  })

  it('scoreMatch 完全匹配 1', () => {
    expect(scoreMatch('hello', 'hello')).toBe(1)
  })

  it('scoreMatch 包含 0.8', () => {
    expect(scoreMatch('hello world', 'hello')).toBe(0.8)
  })

  it('scoreMatch 部分词命中', () => {
    expect(scoreMatch('rust async runtime', 'rust runtime')).toBe(0.5)
  })

  it('highlightMatch 高亮', () => {
    expect(highlightMatch('hello world', 'world')).toBe('hello <mark>world</mark>')
    expect(highlightMatch('hello', '')).toBe('hello')
  })
})
