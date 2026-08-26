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

const dataSource: KbDataSource = mockDataSource()

export function createKbStore() {
  const [libraries, setLibraries] = createSignal<KbLibrary[]>([])
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [keyword, setKeyword] = createSignal('')
  const [activeLibraryId, setActiveLibraryId] = createSignal<string | null>(null)

  async function refresh() {
    setLoading(true)
    setError(null)
    try {
      setLibraries(await dataSource.listLibraries())
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  async function create(name: string, description: string) {
    const lib = await dataSource.createLibrary(name, description)
    setLibraries((prev) => [lib, ...prev])
    return lib
  }

  async function rename(id: string, name: string) {
    await dataSource.renameLibrary(id, name)
    setLibraries((prev) => prev.map((l) => (l.id === id ? { ...l, name } : l)))
  }

  async function remove(id: string) {
    await dataSource.deleteLibrary(id)
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
