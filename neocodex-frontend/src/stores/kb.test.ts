import { describe, it, expect } from 'vitest'
import { createKbStore } from './kb'

describe('kb store (mock-first data source seam)', () => {
  it('refresh 载入种子库并按更新时间倒序', async () => {
    const kb = createKbStore('mock')
    await kb.refresh()
    expect(kb.loading()).toBe(false)
    expect(kb.libraries().length).toBeGreaterThanOrEqual(3)
    const times = kb.libraries().map((l) => l.updatedAt)
    const sorted = [...times].sort((a, b) => b - a)
    expect(times).toEqual(sorted)
  })

  it('create 新库置顶且字段完整', async () => {
    const kb = createKbStore('mock')
    await kb.refresh()
    const lib = await kb.create('测试库', '测试描述')
    expect(kb.libraries()[0].id).toBe(lib.id)
    expect(lib.name).toBe('测试库')
    expect(lib.docCount).toBe(0)
  })

  it('keyword 过滤名称与描述，空关键字返回全部', async () => {
    const kb = createKbStore('mock')
    await kb.refresh()
    kb.setKeyword('')
    expect(kb.filtered().length).toBe(kb.libraries().length)
    kb.setKeyword('设计文档')
    expect(kb.filtered().every((l) => l.name.includes('设计') || l.description.includes('设计'))).toBe(true)
    kb.setKeyword('不存在的关键字xyz')
    expect(kb.filtered()).toEqual([])
  })

  it('rename 更新目标库并刷新 updatedAt', async () => {
    const kb = createKbStore('mock')
    await kb.refresh()
    const first = kb.libraries()[0]
    const before = first.updatedAt
    await new Promise((r) => setTimeout(r, 5))
    await kb.rename(first.id, '改名后的库')
    const renamed = kb.libraries().find((l) => l.id === first.id)
    expect(renamed?.name).toBe('改名后的库')
    expect((renamed?.updatedAt ?? 0)).toBeGreaterThanOrEqual(before)
  })

  it('remove 移除库；删除活跃库时清空 activeLibraryId', async () => {
    const kb = createKbStore('mock')
    await kb.refresh()
    const first = kb.libraries()[0]
    kb.setActiveLibraryId(first.id)
    await kb.remove(first.id)
    expect(kb.libraries().some((l) => l.id === first.id)).toBe(false)
    expect(kb.activeLibraryId()).toBeNull()
  })
})
