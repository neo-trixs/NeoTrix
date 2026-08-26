/* ════════════════════════════════════════════
   stores/kb.ts — 知识库前端状态（Phase 1: mock-first）

   吸收 Cherry Studio / LobeChat 知识库页模式：
   - 库卡片（文档数 / 切片数 / 更新时间）
   - 本地 CRUD + 搜索过滤，全部走 dataSource seam

   ⚠️ 后端对接策略（"先完善功能，最后再对接后端"）：
   mockDataSource 为当前唯一实现；后端接线时新增
   tauriDataSource（invoke kb_list/kb_create/...）并在此
   文件内切换引用 — 组件层零改动。
   ════════════════════════════════════════════ */
import { createSignal } from 'solid-js'

export interface KbLibrary {
  id: string
  name: string
  description: string
  docCount: number
  chunkCount: number
  updatedAt: number // epoch ms
}

export interface KbDoc {
  id: string
  libraryId: string
  title: string
  status: 'ready' | 'indexing' | 'failed'
  sizeKb: number
  addedAt: number
}

/** 数据源契约 — mock 与未来 Tauri invoke 实现共同遵守 */
export interface KbDataSource {
  listLibraries(): Promise<KbLibrary[]>
  createLibrary(name: string, description: string): Promise<KbLibrary>
  renameLibrary(id: string, name: string): Promise<void>
  deleteLibrary(id: string): Promise<void>
  listDocs(libraryId: string): Promise<KbDoc[]>
}

/* ── Mock 实现：内存态种子数据 ── */

const now = Date.now()
const seedLibraries: KbLibrary[] = [
  {
    id: 'kb-neotrix-docs',
    name: 'NeoTrix 设计文档',
    description: 'DESIGN.md、架构决策记录与设计语言 token 规范',
    docCount: 12,
    chunkCount: 842,
    updatedAt: now - 3_600_000,
  },
  {
    id: 'kb-dev-rules',
    name: '开发规则库',
    description: 'dev-rules 全量 R-P 条款与审查维度 D1-D50',
    docCount: 5,
    chunkCount: 310,
    updatedAt: now - 86_400_000,
  },
  {
    id: 'kb-absorbed',
    name: '外部吸收语料',
    description: '已吸收仓库的机制摘要与能力映射',
    docCount: 28,
    chunkCount: 2_140,
    updatedAt: now - 7 * 86_400_000,
  },
]

const seedDocs: KbDoc[] = [
  { id: 'doc-1', libraryId: 'kb-neotrix-docs', title: 'DESIGN v1.3 Consciousness Glass', status: 'ready', sizeKb: 96, addedAt: now - 3_600_000 },
  { id: 'doc-2', libraryId: 'kb-neotrix-docs', title: 'UCN 能力网络 ADR', status: 'ready', sizeKb: 41, addedAt: now - 7_200_000 },
  { id: 'doc-3', libraryId: 'kb-neotrix-docs', title: '图标系统规范 (待重建索引)', status: 'indexing', sizeKb: 18, addedAt: now - 60_000 },
]

let seq = 100
function mockDataSource(): KbDataSource {
  const libs = [...seedLibraries]
  const docs = [...seedDocs]
  return {
    async listLibraries() {
      return libs.map((l) => ({ ...l }))
    },
    async createLibrary(name, description) {
      const lib: KbLibrary = {
        id: `kb-${++seq}`,
        name,
        description,
        docCount: 0,
        chunkCount: 0,
        updatedAt: Date.now(),
      }
      libs.unshift(lib)
      return { ...lib }
    },
    async renameLibrary(id, name) {
      const l = libs.find((x) => x.id === id)
      if (!l) throw new Error(`知识库不存在: ${id}`)
      l.name = name
      l.updatedAt = Date.now()
    },
    async deleteLibrary(id) {
      const i = libs.findIndex((x) => x.id === id)
      if (i >= 0) libs.splice(i, 1)
    },
    async listDocs(libraryId) {
      return docs.filter((d) => d.libraryId === libraryId).map((d) => ({ ...d }))
    },
  }
}

/* ── 响应式 store ── */

import { kbDocList, kbDocIngest, kbDocDelete, type KbDocSummary as TauriDoc } from '../api/kb'

const dataSource: KbDataSource = mockDataSource()

/** 真实后端数据源 (B2 接线): 库 = 文档 metadata.library 聚合 (v1 虚拟分组) */
const tauriDataSource: KbDataSource = {
  async listLibraries() {
    const docs = await kbDocList()
    const groups = new Map<string, KbLibrary>()
    for (const d of docs) {
      const lib = d.library || 'default'
      const g = groups.get(lib) ?? {
        id: `lib-${lib}`, name: lib === 'default' ? '默认库' : lib,
        description: `${docs.filter((x) => (x.library || 'default') === lib).length} 个文档`,
        docCount: 0, chunkCount: 0, updatedAt: 0,
      }
      g.docCount += 1
      g.chunkCount += d.chunk_count
      g.updatedAt = Math.max(g.updatedAt, d.created_at)
      groups.set(lib, g)
    }
    return [...groups.values()]
  },
  // v1 限制: 分组为派生视图 — create/rename/delete 仅作用于文档层,
  // 空组不持久化 (诚实标注, 待 kb_library 表后再实体化)
  async createLibrary(name) {
    return { id: `lib-${name}`, name, description: '(虚拟分组 — 入库第一个文档后固化)', docCount: 0, chunkCount: 0, updatedAt: Date.now() }
  },
  async renameLibrary() { /* v1: 派生分组无实体 */ },
  async deleteLibrary() { /* v1: 派生分组无实体 */ },
  async listDocs(libraryId) {
    const lib = libraryId.replace(/^lib-/, '')
    const docs = await kbDocList()
    return docs
      .filter((d) => (d.library || 'default') === lib)
      .map((d): KbDoc => ({
        id: d.doc_id,
        libraryId: `lib-${d.library || 'default'}`,
        title: d.title,
        status: d.status === 'ready' ? 'ready' : 'indexing',
        sizeKb: Math.max(1, Math.round(d.total_chars / 1024)),
        addedAt: d.created_at,
      }))
  },
}

export { kbDocIngest, kbDocDelete }
export type { TauriDoc }

/** 默认 mock; 传 'tauri' 切真实后端 (组件零改动) */
export function createKbStore(source: 'mock' | 'tauri' = 'tauri') {
  const ds = source === 'tauri' ? tauriDataSource : dataSource
  const [libraries, setLibraries] = createSignal<KbLibrary[]>([])
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [keyword, setKeyword] = createSignal('')
  const [activeLibraryId, setActiveLibraryId] = createSignal<string | null>(null)

  async function refresh() {
    setLoading(true)
    setError(null)
    try {
      setLibraries(await ds.listLibraries())
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  async function create(name: string, description: string) {
    const lib = await ds.createLibrary(name, description)
    setLibraries((prev) => [lib, ...prev])
    return lib
  }

  async function rename(id: string, name: string) {
    await ds.renameLibrary(id, name)
    setLibraries((prev) => prev.map((l) => (l.id === id ? { ...l, name } : l)))
  }

  async function remove(id: string) {
    await ds.deleteLibrary(id)
    setLibraries((prev) => prev.filter((l) => l.id !== id))
    if (activeLibraryId() === id) setActiveLibraryId(null)
  }

  /** 关键字过滤（名称+描述不区分大小写）；空关键字返回全部 */
  function filtered() {
    const kw = keyword().trim().toLowerCase()
    if (!kw) return libraries()
    return libraries().filter(
      (l) => l.name.toLowerCase().includes(kw) || l.description.toLowerCase().includes(kw),
    )
  }

  return {
    libraries,
    loading,
    error,
    keyword,
    setKeyword,
    activeLibraryId,
    setActiveLibraryId,
    filtered,
    refresh,
    create,
    rename,
    remove,
  }
}

export type KbStore = ReturnType<typeof createKbStore>
