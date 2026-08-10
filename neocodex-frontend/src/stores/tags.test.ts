import { describe, it, expect, beforeEach } from 'vitest'
import {
  normalizeTagName,
  tagRoot,
  tagDepth,
  TAG_PALETTE,
  RECOMMENDED_TAGS,
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

  it('TAG_PALETTE 有 9 档色', () => {
    expect(TAG_PALETTE).toHaveLength(9)
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

  it('importSessionTags 后端标签回填：注册 + 规范化 + 不覆盖本地', () => {
    const store = freshStore()
    // 本地已有 a，后端带 [a, B ] → 合并，不丢本地，规范化后端名
    store.addSessionTag('s1', 'a')
    store.importSessionTags('s1', ['a', ' B '])
    expect(store.tagsForSession('s1')).toEqual(['a', 'b'])
    expect(store.state.tags['b']).toBeTruthy()
    // 空数组不动
    store.importSessionTags('s1', [])
    expect(store.tagsForSession('s1')).toEqual(['a', 'b'])
  })

  it('importSessionTags 空/纯垃圾输入安全', () => {
    const store = freshStore()
    store.importSessionTags('s2', ['', '///', '  '])
    expect(store.tagsForSession('s2')).toEqual([])
  })

  it('RECOMMENDED_TAGS 预置集：2 层级根 + 规范名 + 色板色', () => {
    const roots = new Set(RECOMMENDED_TAGS.map((t) => tagRoot(t.name)))
    expect(roots.has('工作')).toBe(true)
    expect(roots.has('领域')).toBe(true)
    for (const t of RECOMMENDED_TAGS) {
      // 预置名必须与规范化幂等（可直接打标）
      expect(normalizeTagName(t.name)).toBe(t.name)
      // 颜色来自色板
      expect(TAG_PALETTE).toContain(t.color)
    }
  })

  it('registerTag 仅注册不绑会话；无效名返回 null', () => {
    const store = freshStore()
    expect(store.registerTag('  Bug #42 ')).toBe('bug-#42')
    expect(store.state.tags['bug-#42']).toBeTruthy()
    expect(store.tagsForSession('s1')).toEqual([]) // 不绑定任何会话
    expect(store.registerTag('   ')).toBeNull()
    expect(store.registerTag('///')).toBeNull()
  })

  it('seedRecommendedTags 幂等 + 预置色覆盖 hash', () => {
    const store = freshStore()
    // 首次：全部添加
    const added = store.seedRecommendedTags()
    expect(added).toBe(RECOMMENDED_TAGS.length)
    for (const t of RECOMMENDED_TAGS) {
      expect(store.state.tags[t.name]).toBe(t.color)
    }
    // 二次：全部已存在 → 0 新增，且色不被 hash 覆盖
    expect(store.seedRecommendedTags()).toBe(0)
    for (const t of RECOMMENDED_TAGS) {
      expect(store.state.tags[t.name]).toBe(t.color)
    }
  })

  it('seedRecommendedTags 不覆盖用户已改色的标签', () => {
    const store = freshStore()
    const target = RECOMMENDED_TAGS[0]
    // 用户手动改了推荐标签颜色
    store.setTagColor(target.name, '#000000')
    store.seedRecommendedTags()
    expect(store.state.tags[target.name]).toBe('#000000')
  })

  it('推荐标签计入 tagTree 层级树', () => {
    const store = freshStore()
    store.seedRecommendedTags()
    store.addSessionTag('s1', '工作/修复')
    const tree = store.tagTree()
    const work = tree.find((t) => t.name === '工作')
    expect(work).toBeTruthy()
    expect(work!.children.some((c) => c.name === '工作/修复')).toBe(true)
  })
})