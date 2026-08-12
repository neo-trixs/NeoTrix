import { createStore, produce } from 'solid-js/store'

/* ════════════════════════════════════════════
   tags store — 会话标签系统（对标 Obsidian 标签体系）
   数据模型：
   - tags:     标签注册表 { name → color }（name 含层级: parent/child）
   - sessionTags: { sessionId → string[] } 会话打标映射
   持久化：localStorage (neotrix:tags)，与后端 session 并存（前端侧挂载）
   ════════════════════════════════════════════ */

/** 标签色板：9 档离散色（索引色），新标签按 name hash 分配 */
export const TAG_PALETTE: string[] = [
  '#e85454', // 红 NT-IO 品牌
  '#f0913a', // 浅橙 NT-ACT
  '#d97706', // 琥珀
  '#16a34a', // 绿 NT-CORE
  '#0d9488', // 青 NT-REPAIR
  '#2563eb', // 蓝 NT-MEMORY
  '#9333ea', // 紫 NT-MIND
  '#db2777', // 玫红
  '#64748b', // 石板灰（中性，GitHub labels 风格）
]

/** 推荐标签（对标 Linear/GitHub 默认 label + Obsidian 层级树）
    2 层级根 → 侧栏天然折叠为工作/领域两棵树。
    颜色从 TAG_PALETTE 显式指定，保证预置观感统一（而非 hash 乱序）。 */
export const RECOMMENDED_TAGS: { name: string; color: string }[] = [
  { name: '工作/功能', color: '#16a34a' },
  { name: '工作/修复', color: '#e85454' },
  { name: '工作/重构', color: '#9333ea' },
  { name: '工作/调研', color: '#2563eb' },
  { name: '工作/测试', color: '#0d9488' },
  { name: '工作/文档', color: '#db2777' },
  { name: '领域/前端', color: '#f0913a' },
  { name: '领域/后端', color: '#d97706' },
  { name: '领域/数据', color: '#2563eb' },
  { name: '领域/devops', color: '#64748b' },
]

export interface TagsState {
  tags: Record<string, string>
  sessionTags: Record<string, string[]>
}

const STORAGE_KEY = 'neotrix:tags'

/** 安全 localStorage 访问（jsdom 等环境可能无 Storage 实现；异常回退空态） */
const storage = (): Storage | null => {
  try {
    if (typeof window !== 'undefined' && window.localStorage) return window.localStorage
  } catch { /* 无 window */ }
  return null
}

function loadPersisted(): TagsState {
  try {
    const ls = storage()
    if (!ls) return { tags: {}, sessionTags: {} }
    const raw = ls.getItem(STORAGE_KEY)
    if (raw) {
      const p = JSON.parse(raw)
      if (p && typeof p === 'object') {
        return {
          tags: p.tags ?? {},
          sessionTags: p.sessionTags ?? {},
        }
      }
    }
  } catch { /* 解析失败走默认 */ }
  return { tags: {}, sessionTags: {} }
}

function persist(state: TagsState) {
  try {
    const ls = storage()
    if (!ls) return
    ls.setItem(STORAGE_KEY, JSON.stringify(state))
  } catch { /* 持久化失败静默（隐私模式等） */ }
}

/** 标签名 hash → 色板索引（确定性，同标签同色） */
function colorForName(name: string): string {
  let h = 0
  for (let i = 0; i < name.length; i++) {
    h = (h * 31 + name.charCodeAt(i)) >>> 0
  }
  return TAG_PALETTE[h % TAG_PALETTE.length]
}

/** 规范化标签名：小写、去首尾空白、去前导 #、空格转连字符、保留层级斜杠 */
export function normalizeTagName(raw: string): string {
  let name = raw.trim().replace(/^#+/, '')
  // 先将斜杠保护（/ 周围空格移除），再把其余空白转连字符
  name = name.replace(/\s*\/\s*/g, '/').replace(/\s+/g, '-').toLowerCase()
  name = name.replace(/\/{2,}/g, '/').replace(/^\/+|\/+$/g, '').replace(/(-)+/g, '-')
  return name
}

/** 提取标签的顶层分组（层级根，对标 Obsidian nested tag 折叠） */
export function tagRoot(name: string): string {
  return name.split('/')[0]
}

/** 标签层级深度（parent/child → 2） */
export function tagDepth(name: string): number {
  return name.split('/').length
}

function createTagsStore() {
  const init = loadPersisted()
  const [state, setState] = createStore<TagsState>(init)

  /** 确保标签在注册表（无则按 hash 分配色） */
  const ensureRegistered = (name: string): void => {
    if (!state.tags[name]) {
      setState('tags', produce(t => { t[name] = colorForName(name) }))
    }
  }

  /** 给会话打标签（自动注册；会话已挂同名则跳过） */
  const addSessionTag = (sessionId: string, rawName: string): void => {
    const name = normalizeTagName(rawName)
    if (!name) return
    ensureRegistered(name)
    setState('sessionTags', produce(st => {
      const cur = st[sessionId] ?? []
      if (!cur.includes(name)) {
        st[sessionId] = [...cur, name]
      }
    }))
    persist(state)
  }

  /** 移除会话的单个标签 */
  const removeSessionTag = (sessionId: string, name: string): void => {
    setState('sessionTags', produce(st => {
      const cur = st[sessionId] ?? []
      st[sessionId] = cur.filter(t => t !== name)
    }))
    persist(state)
  }

  /** 从后端合并会话标签：注册表补色；本地缺该会话标签时才回填（不覆盖本地） */
  const importSessionTags = (sessionId: string, names: string[]): void => {
    if (!names || names.length === 0) return
    const normalized = names.map(normalizeTagName).filter(Boolean)
    setState('tags', produce(t => {
      for (const n of normalized) {
        if (!t[n]) t[n] = colorForName(n)
      }
    }))
    setState('sessionTags', produce(st => {
      const cur = st[sessionId] ?? []
      const merged = [...cur]
      for (const n of normalized) {
        if (!merged.includes(n)) merged.push(n)
      }
      st[sessionId] = merged
    }))
    persist(state)
  }

  /** 移除会话全部标签（会话删除时调用） */
  const clearSessionTags = (sessionId: string): void => {
    setState('sessionTags', produce(st => { delete st[sessionId] }))
    persist(state)
  }

  /** 设置标签颜色 */
  const setTagColor = (name: string, color: string): void => {
    ensureRegistered(name)
    setState('tags', produce(t => { t[name] = color }))
    persist(state)
  }

  /** 仅注册标签（不绑定会话）—— 设置面板快速新建用。返回规范化名，无效返回 null */
  const registerTag = (rawName: string): string | null => {
    const name = normalizeTagName(rawName)
    if (!name) return null
    ensureRegistered(name)
    persist(state)
    return name
  }

  /** 一键添加推荐标签（幂等：仅注册缺失项，用推荐色而非 hash）。返回新增数 */
  const seedRecommendedTags = (): number => {
    let added = 0
    setState('tags', produce(t => {
      for (const { name, color } of RECOMMENDED_TAGS) {
        if (!t[name]) {
          t[name] = color
          added++
        }
      }
    }))
    if (added > 0) persist(state)
    return added
  }

  /** 重命名标签（含层级改名；同步会话引用）。
      目标名已存在 → 返回冲突错误且不覆盖（避免覆盖合并丢数据）；成功返回 null */
  const renameTag = (oldName: string, newName: string): string | null => {
    const target = normalizeTagName(newName)
    if (!target || target === oldName) return null
    // 重名冲突防护：目标标签已注册则拒绝重命名（不覆盖不合并）
    if (state.tags[target]) return `标签 #${target} 已存在，请改用其他名称`
    // 迁移注册表
    const color = state.tags[oldName] ?? colorForName(target)
    setState('tags', produce(t => {
      delete t[oldName]
      t[target] = color
    }))
    // 迁移会话引用
    setState('sessionTags', produce(st => {
      for (const sid of Object.keys(st)) {
        const cur = st[sid]
        if (cur.includes(oldName)) {
          st[sid] = [...cur.filter(t => t !== oldName), target]
        }
      }
    }))
    persist(state)
    return null
  }

  /** 删除标签（全局：注册表 + 所有会话引用） */
  const deleteTag = (name: string): void => {
    setState('tags', produce(t => { delete t[name] }))
    setState('sessionTags', produce(st => {
      for (const sid of Object.keys(st)) {
        st[sid] = st[sid].filter(t => t !== name)
      }
    }))
    persist(state)
  }

  /** 会话的全部标签 */
  const tagsForSession = (sessionId: string): string[] => {
    return state.sessionTags[sessionId] ?? []
  }

  /** 统计每个标签被使用的会话数（用于侧栏计数徽章） */
  const tagCounts = (): Record<string, number> => {
    const counts: Record<string, number> = {}
    for (const sid of Object.keys(state.sessionTags)) {
      for (const t of state.sessionTags[sid]) {
        counts[t] = (counts[t] ?? 0) + 1
      }
    }
    return counts
  }

  /** 侧栏标签树：层级折叠视图（对标 Obsidian Tag Pane）
      根 count = 该根下全部子标签会话数之和（含根自身标签） */
  const tagTree = (): { name: string; color: string; count: number; children: { name: string; color: string; count: number }[] }[] => {
    const counts = tagCounts()
    const allTags = Object.keys(state.tags).sort((a, b) => a.localeCompare(b))
    // 每根聚合计数（该根前缀匹配的所有 tag 计数和）
    const rootCounts = new Map<string, number>()
    // 真实存在的根（要么被显式注册，要么有子标签）
    const rootColor = new Map<string, string>()
    for (const name of allTags) {
      const root = tagRoot(name)
      const c = counts[name] ?? 0
      rootCounts.set(root, (rootCounts.get(root) ?? 0) + c)
      if (!rootColor.has(root)) rootColor.set(root, state.tags[name])
    }
    const roots = new Map<string, { name: string; color: string; count: number; children: { name: string; color: string; count: number }[] }>()
    for (const name of allTags) {
      const root = tagRoot(name)
      if (!roots.has(root)) {
        roots.set(root, {
          name: root,
          color: rootColor.get(root) ?? state.tags[root] ?? '#909098',
          count: rootCounts.get(root) ?? 0,
          children: [],
        })
      }
      if (name !== root) {
        roots.get(root)!.children.push({ name, color: state.tags[name], count: counts[name] ?? 0 })
      }
    }
    const sort = <T extends { name: string; count: number }>(arr: T[]) =>
      [...arr].sort((a, b) => b.count - a.count || a.name.localeCompare(b.name))
    const out = [...roots.values()].map(r => ({ ...r, children: sort(r.children) }))
    return sort(out)
  }

  return {
    get state() { return state },
    /** 重置为持久化快照（测试隔离 / 清除全部标签数据）；传空可用空态 */
    reset(): void {
      const fresh = loadPersisted()
      setState(fresh)
    },
    addSessionTag,
    removeSessionTag,
    clearSessionTags,
    importSessionTags,
    setTagColor,
    registerTag,
    seedRecommendedTags,
    renameTag,
    deleteTag,
    tagsForSession,
    tagCounts,
    tagTree,
  }
}

export const tagsStore = createTagsStore()
