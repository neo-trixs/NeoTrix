import { describe, it, expect, beforeEach } from 'vitest'
import {
  normalizeTagName,
  tagRoot,
  tagDepth,
  TAG_PALETTE,
  tagsStore,
} from './tags'

/* 隔离 store：清空 localStorage + reset 单例（每次测试独立态） */
function freshStore() {
  localStorage.clear()
  tagsStore.reset()
  return tagsStore
}

describe('tags store', () => {
  beforeEach(() => {
    localStorage.clear()
    tagsStore.reset()
  })

  it('normalizeTagName 规范化输入', () => {
    expect(normalizeTagName('  Design UI  ')).toBe('design-ui')
    expect(normalizeTagName('#Backend')).toBe('backend')
    expect(normalizeTagName('  parent // child  ')).toBe('parent/child')
    expect(normalizeTagName('/lead/trail/')).toBe('lead/trail')
    expect(normalizeTagName('')).toBe('')
  })

  it('tagRoot / tagDepth 层级解析', () => {
    expect(tagRoot('design/ui')).toBe('design')
    expect(tagRoot('design')).toBe('design')
    expect(tagDepth('design/ui')).toBe(2)
    expect(tagDepth('a/b/c')).toBe(3)
  })

  it('TAG_PALETTE 有 8 档色', () => {
    expect(TAG_PALETTE).toHaveLength(8)
    expect(TAG_PALETTE[0]).toBe('#e85454')
  })

  it('addSessionTag 打标并自动注册（hash 分配色）', () => {
    const store = freshStore()
    store.addSessionTag('s1', '前端')
    expect(store.tagsForSession('s1')).toEqual(['前端'])
    expect(store.state.tags['前端']).toBeTruthy()
    // 重复打标幂等
    store.addSessionTag('s1', 'frontend')
    expect(store.tagsForSession('s1').length).toBe(2)
  })

  it('removeSessionTag 移除单个标签', () => {
    const store = freshStore()
    store.addSessionTag('s1', 'a')
    store.addSessionTag('s1', 'b')
    store.removeSessionTag('s1', 'a')
    expect(store.tagsForSession('s1')).toEqual(['b'])
  })

  it('tagCounts 跨会话计数', () => {
    const store = freshStore()
    store.addSessionTag('s1', 'bug')
    store.addSessionTag('s2', 'bug')
    store.addSessionTag('s2', 'fix')
    const counts = store.tagCounts()
    expect(counts['bug']).toBe(2)
    expect(counts['fix']).toBe(1)
  })

  it('tagTree 层级聚合（父标签聚合子计数）', () => {
    const store = freshStore()
    store.addSessionTag('s1', 'design/ui')
    store.addSessionTag('s1', 'design/ux')
    store.addSessionTag('s2', 'design/ui')
    const tree = store.tagTree()
    const design = tree.find((t) => t.name === 'design')
    expect(design).toBeTruthy()
    expect(design!.count).toBe(3) // ui×2 + ux×1 聚合到根
    expect(design!.children).toHaveLength(2)
    expect(design!.children.find((c) => c.name === 'design/ui')!.count).toBe(2)
  })

  it('renameTag 迁移注册表 + 会话引用', () => {
    const store = freshStore()
    store.addSessionTag('s1', 'old/name')
    store.renameTag('old/name', 'new/name')
    expect(store.tagsForSession('s1')).toEqual(['new/name'])
    expect(store.state.tags['old/name']).toBeUndefined()
    expect(store.state.tags['new/name']).toBeTruthy()
  })

  it('deleteTag 全局清除', () => {
    const store = freshStore()
    store.addSessionTag('s1', 'gone')
    store.addSessionTag('s2', 'gone')
    store.deleteTag('gone')
    expect(store.tagsForSession('s1')).toEqual([])
    expect(store.tagsForSession('s2')).toEqual([])
    expect(store.state.tags['gone']).toBeUndefined()
  })

  it('setTagColor 覆盖颜色', () => {
    const store = freshStore()
    store.addSessionTag('s1', 'x')
    const before = store.state.tags['x']
    store.setTagColor('x', '#2563eb')
    expect(store.state.tags['x']).toBe('#2563eb')
    expect(store.state.tags['x']).not.toBe(before)
  })

  it('clearSessionTags 清空会话全部标签', () => {
    const store = freshStore()
    store.addSessionTag('s1', 'a')
    store.addSessionTag('s1', 'b')
    store.clearSessionTags('s1')
    expect(store.tagsForSession('s1')).toEqual([])
  })
})