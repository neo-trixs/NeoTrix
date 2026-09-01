import { createSignal, For, Show, onCleanup } from 'solid-js'
import { invoke } from '@tauri-apps/api/core'
import { Settings, Archive, RotateCcw, ChevronRight, GitFork } from 'lucide-solid'
import { chatStore } from '../stores/chat'
import type { Session } from '../stores/chat'

import { clsx } from 'clsx'
import { neocodex } from '../api'
import { NeoPlus, NeoMessage, NeoSearch, NeoChevronRight, NeoTrash, NeoPencil, NeoClose } from './neo-icons'
import { NeoTag } from './NeoTag'

import { ConfirmModal, type ModalReq } from './ConfirmModal'
import type { NeoCodexSessionInfo } from '../api/types'

interface SidebarProps {
  collapsed?: boolean
  onToggleCollapse?: () => void
  activeView?: 'chat' | 'cowork' | 'computer'
  onSwitchView?: (view: 'chat' | 'cowork' | 'computer') => void
  /** 功能面板入口（对标 Claude Code 侧栏：功能融合到侧栏，顶部极简） */
  activePanel?: string | null
  onTogglePanel?: (id: string) => void
  /** 打开设置（弹窗由 Chat 根级渲染，避免被侧栏 overflow 裁剪） */
  onOpenSettings?: () => void
  /** 标签筛选（对标 Obsidian Tag Pane 多选过滤） */
  activeTags?: string[]
  onToggleTag?: (name: string) => void
  onClearTags?: () => void
  /** 展开态宽度（px），由 Chat 拖拽手柄控制 */
  width?: number
}

// Segmented Tab：已移除（仅对话单态，极简无分段）
const VIEW_ORDER = ['chat'] as const
type ViewKey = (typeof VIEW_ORDER)[number]

export function Sidebar(props: SidebarProps) {
  const collapsed = () => props.collapsed ?? false
  // 折叠态 hover 预览：收起为图标栏时，悬停临时展开为完整侧栏
  const [peek, setPeek] = createSignal(false)
  const visible = () => !collapsed() || peek()
  const view = () => 'chat' as const
  const viewIdx = () => 0

  // 会话操作错误提示：chatStore 会吞掉后端错误（chat.ts catch → console.error），
  // 因此部分操作用后端重查/状态后置条件验证真实结果，失败时给可见内联错误（对标 GitPanel toast）
  const [sidebarError, setSidebarError] = createSignal<string | null>(null)
  let errorTimer: ReturnType<typeof setTimeout> | undefined
  const showError = (msg: string) => {
    setSidebarError(msg)
    if (errorTimer) clearTimeout(errorTimer)
    errorTimer = setTimeout(() => setSidebarError(null), 6000)
  }
  onCleanup(() => { if (errorTimer) clearTimeout(errorTimer) })

  // 后端重查活跃会话列表；失败返回 null（避免假阳性误报）
  const fetchSessions = async (): Promise<NeoCodexSessionInfo[] | null> => {
    try {
      return await neocodex.listSessions()
    } catch {
      return null
    }
  }

  // 删除/归档当前会话后把焦点移回最近邻会话项（列表内首个条目），无条目回落侧栏搜索框
  const focusNearestSession = () => {
    requestAnimationFrame(() => {
      const btn = sessionListRef?.querySelector<HTMLElement>('button')
      if (btn) { btn.focus(); return }
      searchButtonRef?.focus()
    })
  }

  // 会话搜索（前端过滤）
  const [searchOpen, setSearchOpen] = createSignal(false)
  const [searchQuery, setSearchQuery] = createSignal('')
  // 搜索按钮引用：关闭搜索后焦点还原（Bug 3）
  let searchButtonRef: HTMLButtonElement | undefined
  let viewTabsRef: HTMLDivElement | undefined
  // 会话列表容器引用：删除/归档当前会话后焦点回移最近邻条目
  let sessionListRef: HTMLDivElement | undefined

  const toggleSearch = () => {
    const next = !searchOpen()
    setSearchOpen(next)
    if (!next) {
      setSearchQuery('')
      // 关闭后焦点还原到搜索按钮（fallback 重新挂载，下帧聚焦）
      requestAnimationFrame(() => searchButtonRef?.focus())
    }
  }

  // 设置入口：由 Chat 根级渲染弹窗（避免 aside overflow-hidden 裁剪 fixed 弹窗）
  const openSettings = () => {
    props.onOpenSettings?.()
  }

  const switchView = (v: 'chat' | 'cowork' | 'computer') => {
    props.onSwitchView?.(v)
  }

  // 激活视图 + 焦点跟随选中 tab（WAI-ARIA tabs：选择态与焦点同步）
  const activateView = (v: 'chat' | 'cowork' | 'computer') => {
    switchView(v)
    requestAnimationFrame(() => {
      const selected = viewTabsRef?.querySelector<HTMLElement>('[role="tab"][aria-selected="true"]')
      selected?.focus()
    })
  }

  // 方向键在 Tab 间移动：dir=1 前进，dir=-1 后退；边界停止（不环绕）
  const moveTab = (dir: 1 | -1) => {
    const idx = VIEW_ORDER.indexOf(view() as ViewKey)
    const target = idx + dir
    if (target < 0 || target >= VIEW_ORDER.length) return
    activateView(VIEW_ORDER[target])
  }

  const currentSessionId = () => chatStore.state.currentSessionId

  // 会话置顶（本地 pin，未持久化；对标 2026 会话列表置顶）
  const [pinnedIds, setPinnedIds] = createSignal<string[]>([])
  const togglePin = (id: string) =>
    setPinnedIds((ids) => (ids.includes(id) ? ids.filter((x) => x !== id) : [id, ...ids]))

  // 会话拖拽排序（前端本地序；后端固化待并发会话释放）：localStorage 持久化
  const ORDER_KEY = 'nt_session_manual_order'
  const loadOrder = (): string[] => {
    try { return JSON.parse(localStorage.getItem(ORDER_KEY) || '[]') } catch { return [] }
  }
  const [manualOrder, setManualOrder] = createSignal<string[]>(loadOrder())
  const persistOrder = () => localStorage.setItem(ORDER_KEY, JSON.stringify(manualOrder()))
  let dragSessionId: string | null = null
  const reorderSession = (targetId: string) => {
    if (!dragSessionId || dragSessionId === targetId) return
    const allIds = chatStore.state.sessions.map((s) => s.id)
    const valid = manualOrder().filter((id) => allIds.includes(id))
    for (const id of allIds) if (!valid.includes(id)) valid.push(id)
    const from = valid.indexOf(dragSessionId)
    if (from === -1) return
    valid.splice(from, 1)
    const to = valid.indexOf(targetId)
    valid.splice(to === -1 ? valid.length : to, 0, dragSessionId)
    setManualOrder(valid)
    persistOrder()
    // 接线工作树后端：真实持久化（running backend 无该命令时静默降级到本地序）
    void invoke('cmd_reorder_sessions', { ids: valid }).catch(() => {})
    dragSessionId = null
  }
  // 跨项目拖拽：前端本地覆盖会话所属项目（后端固化待并发会话释放）
  const PROJ_OVR_KEY = 'nt_project_override'
  const loadProjOverride = (): Record<string, string> => {
    try { return JSON.parse(localStorage.getItem(PROJ_OVR_KEY) || '{}') } catch { return {} }
  }
  const [projectOverride, setProjectOverride] = createSignal<Record<string, string>>(loadProjOverride())
  const setSessionProject = (id: string, project: string) => {
    setProjectOverride((o) => {
      const n = { ...o, [id]: project }
      localStorage.setItem(PROJ_OVR_KEY, JSON.stringify(n))
      return n
    })
    // 接线工作树后端：真实持久化（无命令时静默降级到本地覆盖）
    void invoke('cmd_set_session_project', { id, project }).catch(() => {})
    dragSessionId = null
  }
  const pinnedSessions = () =>
    pinnedIds()
      .map((id) => chatStore.state.sessions.find((s) => s.id === id))
      .filter((s): s is Session => !!s)

  const groupedSessions = () => {
    const sessions = chatStore.state.sessions
    const q = searchQuery().trim().toLowerCase()
    const activeTags = props.activeTags ?? []
    // 会话标签（从 tags store 实时读取，非响应式 session.tags 兜底）
    const sessionTags = (id: string): string[] => chatStore.tagsForSession(id)
    const filtered = sessions.filter((s) => {
      if (q && !s.title.toLowerCase().includes(q)) return false
      if (activeTags.length > 0) {
        const tags = sessionTags(s.id)
        // 多选 = AND：命中每个激活标签（标签层级根匹配：激活 root 命中全部子标签）
        for (const at of activeTags) {
          const matched = tags.some((t) => t === at || t.startsWith(`${at}/`))
          if (!matched) return false
        }
      }
      return true
    })
    // 项目制对话机制（对标 Claude Code / OpenWebUI：按项目分组，项目名取会话工作目录）
    const groups = new Map<string, typeof filtered>()
    for (const s of filtered) {
      const key = projectOverride()[s.id] ?? s.project ?? '未分类'
      if (!groups.has(key)) groups.set(key, [])
      groups.get(key)!.push(s)
    }
    const arr = [...groups.entries()].map(([key, items]) => ({ key, items }))
    // 组内应用手动拖拽序（未排序的会话回落按更新时间降序）
    const orderIdx = (id: string) => {
      const i = manualOrder().indexOf(id)
      return i === -1 ? Number.MAX_SAFE_INTEGER : i
    }
    arr.forEach((g) => {
      g.items.sort((a, b) => {
        const ia = orderIdx(a.id)
        const ib = orderIdx(b.id)
        if (ia !== ib) return ia - ib
        return b.updatedAt.getTime() - a.updatedAt.getTime()
      })
    })
    // 项目组按最近活跃时间降序
    arr.sort((a, b) => {
      const ma = Math.max(...a.items.map((i) => i.updatedAt.getTime()), 0)
      const mb = Math.max(...b.items.map((i) => i.updatedAt.getTime()), 0)
      return mb - ma
    })
    return arr
  }

  // 项目分组折叠态（参考成熟产品：项目可折叠，默认展开，持久化）
  const PROJ_KEY = 'nt_collapsed_projects'
  const loadCollapsed = (): Set<string> => {
    try { return new Set(JSON.parse(localStorage.getItem(PROJ_KEY) || '[]')) } catch { return new Set() }
  }
  const [collapsedProjects, setCollapsedProjects] = createSignal<Set<string>>(loadCollapsed())
  const toggleProject = (key: string) => {
    const next = new Set(collapsedProjects())
    if (next.has(key)) next.delete(key); else next.add(key)
    setCollapsedProjects(next)
    localStorage.setItem(PROJ_KEY, JSON.stringify([...next]))
  }

  const handleNewChat = async () => {
    const newId = await chatStore.addSession()
    // 新会话必须立即可见：清除标签筛选与搜索词，并退出归档视图（Bug 4/6）
    props.onClearTags?.()
    setSearchQuery('')
    if (searchOpen()) toggleSearch()
    setShowArchived(false)
    // 后端失败时 addSession 会静默回退本地会话（chat.ts catch），重查验证并提示
    const list = await fetchSessions()
    if (list && !list.some(s => s.id === newId)) {
      showError('新建会话失败，本次会话可能仅保存在本地')
    }
  }

  const handleSwitchSession = (id: string) => {
    chatStore.switchSession(id)
  }

  const handleDeleteSession = (e: Event, id: string) => {
    e.stopPropagation()
    // 破坏性操作确认（对标 Codex）— 统一模态
    setPendingDeleteId(id)
    setModalReq({ title: '删除会话', message: '确定删除该会话？此操作不可撤销。', danger: true, confirmLabel: '删除' })
  }

  const handleRenameSession = (e: Event, id: string) => {
    e.stopPropagation()
    // 统一输入模态（替换原生 prompt）
    const current = chatStore.state.sessions.find((s) => s.id === id)?.title ?? ''
    setPendingRenameId(id)
    setModalReq({
      title: '重命名会话',
      inputLabel: '会话名称',
      initialValue: current,
      confirmLabel: '保存',
    })
  }

  // 分支新话题：复制源会话历史为新会话（后端 cmd_fork_session 持久化；失败给可见错误）
  const handleForkSession = async (e: Event, id: string) => {
    e.stopPropagation()
    try {
      const newId = await invoke<string>('cmd_fork_session', { from_id: id })
      await chatStore.loadSessions()
      if (newId) await chatStore.switchSession(newId)
    } catch {
      showError('分叉会话失败，请重试')
    }
  }

  /* ── 归档（对标 Claude Code Archive：归档箱 + 恢复；焦点管理对标搜索关闭还原） ── */
  const [showArchived, setShowArchived] = createSignal(false)
  const [archivedSessions, setArchivedSessions] = createSignal<NeoCodexSessionInfo[]>([])
  const [loadingArchived, setLoadingArchived] = createSignal(false)
  const [archivedError, setArchivedError] = createSignal(false)
  let archiveEntryRef: HTMLButtonElement | undefined
  let archivedBackRef: HTMLButtonElement | undefined

  // 直连后端查询归档：chatStore.listArchived 会吞错返回空列表，无法区分失败与空态，
  // 这里独立捕获并暴露错误态，避免归档加载失败误显示「暂无归档会话」
  const loadArchived = async () => {
    setLoadingArchived(true)
    setArchivedError(false)
    try {
      const list = await neocodex.listArchived()
      setArchivedSessions(list)
    } catch {
      setArchivedError(true)
      setArchivedSessions([])
    } finally {
      setLoadingArchived(false)
    }
  }

  const handleArchiveSession = async (e: Event, id: string) => {
    e.stopPropagation()
    await chatStore.archiveSession(id)
    // 后端失败时 chatStore 吞错（会话仍留在活跃列表），重查验证并提示
    const list = await fetchSessions()
    if (list && list.some(s => s.id === id)) {
      showError('归档会话失败，请重试')
    } else {
      // 成功归档：会话条目已消失，焦点回移最近邻会话项
      focusNearestSession()
    }
  }

  const openArchived = () => {
    setShowArchived(true)
    loadArchived()
    // 焦点进入归档视图：聚焦返回按钮
    requestAnimationFrame(() => archivedBackRef?.focus())
  }

  const closeArchived = () => {
    setShowArchived(false)
    setArchivedSessions([])
    // 焦点还原到归档入口
    requestAnimationFrame(() => archiveEntryRef?.focus())
  }

  const handleRestoreArchived = async (e: Event, id: string) => {
    e.stopPropagation()
    // 恢复成功会重新拉取活跃列表；失败时 chatStore 吞错且不重载列表
    await chatStore.restoreSession(id)
    if (chatStore.state.sessions.some(s => s.id === id)) {
      // 恢复成功：清空残留搜索/标签筛选，否则恢复的会话可能仍被过滤不可见
      if (searchQuery().trim() || (props.activeTags ?? []).length > 0) {
        props.onClearTags?.()
        setSearchQuery('')
        if (searchOpen()) toggleSearch()
      }
      setArchivedSessions(prev => prev.filter(s => s.id !== id))
    } else {
      showError('恢复归档会话失败，请重试')
    }
  }

  const formatRelativeTime = (date: Date) => {
    const d = new Date(date)
    const now = new Date()
    const diff = now.getTime() - d.getTime()
    const hours = diff / (1000 * 60 * 60)

    if (hours < 1) return '刚刚'
    if (hours < 24) return `${Math.floor(hours)}小时前`
    if (hours < 48) return '昨天'
    if (hours < 168) return `${Math.floor(hours / 24)}天前`
    return d.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
  }

  /* ── 打标交互已移除（极简侧栏） ── */
  const [taggingSessionId, setTaggingSessionId] = createSignal<string | null>(null)
  const [tagInput, setTagInput] = createSignal('')

  // 统一确认/输入模态（替换原生 confirm/prompt）
  const [modalReq, setModalReq] = createSignal<ModalReq | null>(null)
  const [pendingDeleteId, setPendingDeleteId] = createSignal<string | null>(null)
  const [pendingRenameId, setPendingRenameId] = createSignal<string | null>(null)
  const closeModal = () => {
    setModalReq(null)
    setPendingDeleteId(null)
    setPendingRenameId(null)
  }
  let tagInputRef: HTMLInputElement | undefined

  const handleOpenTagging = (e: Event, id: string) => {
    e.stopPropagation()
    if (taggingSessionId() === id) {
      setTaggingSessionId(null)
      setTagInput('')
      return
    }
    setTaggingSessionId(id)
    setTagInput('')
    requestAnimationFrame(() => tagInputRef?.focus())
  }

  const handleTagInputKey = (e: KeyboardEvent, id: string) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      const raw = tagInput().trim()
      if (raw) {
        chatStore.tagSession(id, raw)
        setTagInput('')
        requestAnimationFrame(() => tagInputRef?.focus())
      }
    } else if (e.key === 'Escape') {
      e.preventDefault()
      setTaggingSessionId(null)
      setTagInput('')
    }
  }

  const handleRemoveTag = (sessionId: string, tag: string) => {
    chatStore.untagSession(sessionId, tag)
  }

  /* 已有标签建议（输入时联想，对标 Obsidian 标签自动补全） */
  const tagSuggestions = () => [] as string[]

  const handleToggleTag = (_name: string) => {
    props.onToggleTag?.(_name)
  }

  return (
    <aside
      class={clsx(
        'flex-shrink-0 glass-side overflow-hidden flex flex-col relative max-w-[85vw]',
        collapsed() ? 'w-[64px] border-r border-black/5' : 'border-r border-border-primary/40',
        collapsed() && peek() && 'sidebar-peek'
      )}
      style={collapsed() && peek() ? { width: `${props.width ?? 280}px`, 'z-index': '50' } : collapsed() ? undefined : { width: `${props.width ?? 280}px` }}
      onMouseEnter={() => collapsed() && setPeek(true)}
      onMouseLeave={() => setPeek(false)}
    >
      {/* Header: 三色灯占位（macOS 28px 拖拽区） */}
      <div class="h-7 shrink-0" data-tauri-drag-region />
      {/* 折叠标签：仅保留图标（对标 Claude Code 侧栏手柄） */}
      <div class={clsx('pt-2', visible() ? 'px-3' : 'flex justify-center px-0')}>
        <button
          class="p-1.5 rounded-md text-zinc-400 hover:text-orange-600 hover:bg-orange-50/60 transition-colors focus-visible:ring-2 focus-visible:ring-orange-400 focus-visible:outline-none"
          onClick={props.onToggleCollapse}
          aria-label={collapsed() ? '展开侧边栏' : '折叠侧边栏'}
          title={collapsed() ? '展开侧边栏' : '折叠侧边栏'}
        >
          <NeoChevronRight class={clsx('w-4 h-4 transition-transform', !collapsed() && 'rotate-180')} />
        </button>
      </div>

      {visible() && (
        <>
          {/* 标题：单态对话（极简，无分段切换） */}

          {/* 搜索 + 新建 — 极简单行：搜索占满 + 新建图标 */}
          <div class="px-3 pb-2 flex items-center gap-2">
            <Show
              when={searchOpen()}
              fallback={
                <button
                  ref={searchButtonRef}
                  class="flex-1 flex items-center gap-2 px-3 py-2 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors text-left border border-border-primary/40"
                  onClick={toggleSearch}
                  aria-label="搜索会话"
                  title="搜索会话"
                >
                  <NeoSearch class="w-4 h-4" />
                  <span class="text-12px">搜索</span>
                </button>
              }
            >
              <div class="flex-1 flex items-center gap-2 px-2 py-1 rounded-lg border border-nt-io-500/40 bg-white/60" role="search">
                <NeoSearch class="w-4 h-4 text-nt-io-600 flex-shrink-0" />
                <input
                  class="flex-1 min-w-0 bg-transparent border-none outline-none text-12px text-text-primary placeholder-text-muted/60 focus-visible:ring-0"
                  placeholder="搜索会话标题…"
                  aria-label="搜索会话"
                  value={searchQuery()}
                  onInput={(e) => setSearchQuery(e.currentTarget.value)}
                  onKeyDown={(e) => { if (e.key === 'Escape') { e.preventDefault(); toggleSearch() } }}
                  autofocus
                />
                <button
                  class="p-1 text-text-muted hover:text-text-primary flex-shrink-0"
                  onClick={toggleSearch}
                  aria-label="关闭搜索"
                  title="关闭搜索"
                >
                  <NeoClose class="w-4 h-4" />
                </button>
              </div>
            </Show>
            <button
              class="p-2 rounded-lg bg-nt-io-500/10 text-nt-io-600 hover:bg-nt-io-500/20 transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
              onClick={handleNewChat}
              aria-label="新建对话"
              title="新建对话"
            >
              <NeoPlus class="w-4 h-4" />
            </button>
          </div>

          {/* 会话列表（极简扁平，按标签过滤）；showArchived 时切换为归档箱视图 */}
          <div ref={sessionListRef} class="flex-1 overflow-y-auto px-3 pb-4">
            {/* 会话操作错误内联提示（对标 GitPanel toast；6s 自动消失） */}
            <Show when={sidebarError()}>
              {(err) => (
                <div
                  class="mx-1 mt-2 px-2.5 py-1.5 rounded-lg text-[11px] leading-snug bg-nt-shield-500/10 text-nt-shield-600 border border-nt-shield-500/20 flex items-start gap-1.5"
                  role="alert"
                >
                  <svg viewBox="0 0 12 12" class="w-3.5 h-3.5 mt-px flex-shrink-0" fill="none">
                    <circle cx="6" cy="6" r="5" stroke="currentColor" stroke-width="1.1" />
                    <line x1="6" y1="3.5" x2="6" y2="6.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
                    <circle cx="6" cy="8.2" r="0.6" fill="currentColor" />
                  </svg>
                  <span class="min-w-0 flex-1">{err()}</span>
                </div>
              )}
            </Show>
            <Show
              when={!showArchived()}
              fallback={
                <div class="flex flex-col h-full">
                  {/* 归档箱头部：返回 + 标题 + 数量 */}
                  <div class="flex items-center gap-2 py-2">
                    <button
                      ref={archivedBackRef}
                      class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70 transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
                      onClick={closeArchived}
                      onKeyDown={(e) => { if (e.key === 'Escape') { e.preventDefault(); closeArchived() } }}
                      aria-label="返回会话列表"
                      title="返回"
                    >
                      <NeoChevronRight class="w-4 h-4 rotate-180" />
                    </button>
                    <span class="flex-1 min-w-0 truncate text-12px font-medium text-text-primary">已归档</span>
                    <span class="text-11px text-text-muted flex-shrink-0">{archivedSessions().length}</span>
                  </div>
                  <Show
                    when={!loadingArchived()}
                    fallback={<div class="px-3 py-8 text-center text-text-muted text-sm">加载中…</div>}
                  >
                    <Show
                      when={!archivedError()}
                      fallback={
                        <div class="px-3 py-8 text-center text-sm">
                          <div class="text-nt-shield-600">加载归档会话失败，请重试</div>
                          <button
                            class="mt-2 px-2.5 py-1 rounded-md text-[11px] font-medium bg-nt-io-500/10 text-nt-io-600 hover:bg-nt-io-500/20 transition-colors"
                            onClick={loadArchived}
                          >
                            重试
                          </button>
                        </div>
                      }
                    >
                      <Show
                        when={archivedSessions().length > 0}
                        fallback={<div class="px-3 py-8 text-center text-text-muted text-sm">暂无归档会话</div>}
                      >
                      <ul class="space-y-1" role="list" aria-label="已归档会话">
                        <For each={archivedSessions()}>
                          {(archived) => (
                            <li class="group relative">
                              <div class="flex items-center rounded-lg transition-colors hover:bg-white/40">
                                <div class="flex-1 flex items-center gap-3 px-3 py-2 min-w-0 text-left">
                                  <Archive class="w-4 h-4 flex-shrink-0 text-text-muted" />
                                  <span class="flex-1 min-w-0 truncate text-[13px] text-text-secondary" title={archived.name}>
                                    {archived.name}
                                  </span>
                                  <span class="text-11px text-text-muted flex-shrink-0">
                                    {formatRelativeTime(new Date(archived.updated_at * 1000))}
                                  </span>
                                </div>
                                <button
                                  class="p-1.5 mr-2 rounded text-text-muted hover:text-nt-io-600 hover:bg-nt-io-500/10 transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
                                  onClick={(e) => handleRestoreArchived(e, archived.id)}
                                  aria-label={`恢复会话 ${archived.name}`}
                                  title="恢复归档"
                                >
                                  <RotateCcw class="w-3.5 h-3.5" />
                                </button>
                              </div>
                            </li>
                          )}
                        </For>
                      </ul>
                    </Show>
                    </Show>
                  </Show>
                </div>
              }
            >
            <Show
              when={groupedSessions().length > 0}
              fallback={
                chatStore.isLoadingSessions && chatStore.state.sessions.length === 0 ? (
                  <div class="px-3 py-8 text-center text-text-muted text-sm">加载中…</div>
                ) : chatStore.state.sessions.length === 0 ? (
                  <div class="px-3 py-8 text-center text-text-muted text-sm">暂无对话记录</div>
                ) : (
                  <div class="px-3 py-8 text-center text-text-muted text-sm">未找到匹配的对话</div>
                )
              }
            >
              {/* 置顶会话（本地 pin，置顶于列表最前） */}
              <Show when={pinnedSessions().length > 0}>
                <div class="mb-4">
                  <div class="px-2 pb-2 pt-1 text-10px uppercase tracking-widest text-text-muted/70 font-semibold">置顶</div>
                  <ul class="space-y-1" role="list" aria-label="置顶会话">
                    <For each={pinnedSessions()}>
                      {(session) => {
                        const active = currentSessionId() === session.id
                        return (
                          <li class="group relative">
                            <div class={clsx(
                              'rounded-lg transition-colors flex items-center',
                              active ? 'bg-nt-io-500/8 shadow-[inset_0_1px_0_rgba(255,255,255,0.7)]' : 'hover:bg-white/40'
                            )}>
                              <button
                                class="flex-1 flex items-center gap-3 px-3 py-2 min-w-0 text-left"
                                onClick={() => handleSwitchSession(session.id)}
                                aria-current={active ? 'true' : undefined}
                                title={session.title}
                              >
                                <NeoMessage class={clsx('w-4 h-4 flex-shrink-0', active ? 'text-nt-io-600' : 'text-text-muted')} />
                                <span class={clsx('flex-1 min-w-0 truncate text-[13px]', active ? 'text-text-primary font-medium' : 'text-text-secondary')}>
                                  {session.title}
                                </span>
                              </button>
                              <button
                                class="p-1 mr-2 rounded text-nt-io-600 bg-nt-io-500/10 hover:bg-white/70 transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
                                onClick={(e) => { e.stopPropagation(); togglePin(session.id) }}
                                aria-label="取消置顶"
                                title="取消置顶"
                              >
                                <svg viewBox="0 0 16 16" fill="none" class="w-3.5 h-3.5">
                                  <path d="M9.5 2.5l4 4L8.5 11.5 4 13l1.5-4.5L9.5 2.5z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
                                </svg>
                              </button>
                            </div>
                          </li>
                        )
                      }}
                    </For>
                  </ul>
                </div>
              </Show>

              <For each={groupedSessions().map((g) => ({ ...g, items: g.items.filter((s) => !pinnedIds().includes(s.id)) }))}>
                {(group) => (
                  <div class="mb-4 last:mb-0">
                    {/* 项目分组头（对标 Claude Code / OpenWebUI：项目可折叠；拖拽会话到此重设项目） */}
                    <button
                      class="w-full flex items-center gap-1.5 px-2 pb-2 pt-2 text-left group/ph focus-visible:outline-none"
                      classList={{ 'sc-drop-target': dragSessionId !== null }}
                      onClick={() => toggleProject(group.key)}
                      onDragOver={(e) => { if (dragSessionId) e.preventDefault() }}
                      onDrop={(e) => { e.preventDefault(); if (dragSessionId) setSessionProject(dragSessionId, group.key) }}
                      aria-expanded={!collapsedProjects().has(group.key)}
                      aria-label={collapsedProjects().has(group.key) ? `展开 ${group.key}` : `折叠 ${group.key}`}
                    >
                      <ChevronRight class={clsx('w-3 h-3 text-text-muted/70 transition-transform flex-shrink-0', !collapsedProjects().has(group.key) && 'rotate-90')} />
                      <span class="text-10px uppercase tracking-widest text-text-muted/70 font-semibold truncate">{group.key}</span>
                      <span class="text-10px text-text-muted/50 ml-1">{group.items.length}</span>
                    </button>
                    <Show when={!collapsedProjects().has(group.key)}>
                      <ul class="space-y-1" role="list" aria-label={`${group.key}会话`}>
                      <For each={group.items}>
                        {(session: { id: string; title: string; updatedAt: Date }) => {
                          const active = currentSessionId() === session.id
                          const sessionTags = () => chatStore.tagsForSession(session.id)
                          const isTagging = () => taggingSessionId() === session.id
                          return (
                            <li
                              class="group relative"
                              draggable={true}
                              onDragStart={(e) => {
                                dragSessionId = session.id
                                if (e.dataTransfer) {
                                  e.dataTransfer.effectAllowed = 'move'
                                  try { e.dataTransfer.setData('text/plain', session.id) } catch { /* 旧浏览器兼容 */ }
                                }
                              }}
                              onDragOver={(e) => { e.preventDefault(); if (e.dataTransfer) e.dataTransfer.dropEffect = 'move' }}
                              onDrop={(e) => { e.preventDefault(); e.stopPropagation(); reorderSession(session.id) }}
                            >
                              <div class={clsx(
                                'rounded-lg transition-colors',
                                active
                                  ? 'bg-nt-io-500/8 shadow-[inset_0_1px_0_rgba(255,255,255,0.7)]'
                                  : 'hover:bg-white/40'
                              )}>
                                {/* 标题行：主按钮 + hover 操作区 */}
                                <div class="flex items-center">
                                  <button
                                    class="flex-1 flex items-center gap-3 px-3 py-2 min-w-0 text-left"
                                    onClick={() => handleSwitchSession(session.id)}
                                    aria-current={active ? 'true' : undefined}
                                    title={session.title}
                                  >
                                    <NeoMessage class={clsx(
                                      'w-4 h-4 flex-shrink-0',
                                      active ? 'text-nt-io-600' : 'text-text-muted'
                                    )} />
                                    <span class={clsx(
                                      'flex-1 min-w-0 truncate text-[13px]',
                                      active ? 'text-text-primary font-medium' : 'text-text-secondary'
                                    )}>
                                      {session.title}
                                    </span>
                                    <span class="text-11px text-text-muted flex-shrink-0">
                                      {formatRelativeTime(session.updatedAt)}
                                    </span>
                                  </button>
                                  <div class="flex items-center gap-1 pr-2 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity">
                                    <button
                                      class={clsx(
                                        'p-1 rounded transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none',
                                        pinnedIds().includes(session.id)
                                          ? 'text-nt-io-600 bg-nt-io-500/10'
                                          : 'text-text-muted hover:text-text-primary hover:bg-white/70'
                                      )}
                                      onClick={(e) => { e.stopPropagation(); togglePin(session.id) }}
                                      aria-label={pinnedIds().includes(session.id) ? '取消置顶' : '置顶'}
                                      title={pinnedIds().includes(session.id) ? '取消置顶' : '置顶'}
                                    >
                                      <svg viewBox="0 0 16 16" fill="none" class="w-3.5 h-3.5">
                                        <path d="M9.5 2.5l4 4L8.5 11.5 4 13l1.5-4.5L9.5 2.5z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
                                      </svg>
                                    </button>
                                    <button
                                      class={clsx(
                                        'p-1 rounded transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none',
                                        isTagging()
                                          ? 'bg-nt-io-500/10 text-nt-io-600'
                                          : 'text-text-muted hover:text-text-primary hover:bg-white/70'
                                      )}
                                      onClick={(e) => handleOpenTagging(e, session.id)}
                                      aria-label={isTagging() ? '关闭打标' : '打标签'}
                                      title={isTagging() ? '关闭打标' : '打标签'}
                                    >
                                      <svg viewBox="0 0 16 16" fill="none" class="w-3.5 h-3.5">
                                        <line x1="5.5" y1="2.5" x2="4" y2="13.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
                                        <line x1="10.5" y1="2.5" x2="9" y2="13.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
                                        <line x1="2.5" y1="6" x2="13.5" y2="6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
                                        <line x1="2.5" y1="10" x2="13.5" y2="10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
                                      </svg>
                                    </button>
                                    <button
                                      class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70 transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
                                      onClick={(e) => handleRenameSession(e, session.id)}
                                      aria-label="重命名会话"
                                      title="重命名"
                                    >
                                      <NeoPencil class="w-3.5 h-3.5" />
                                    </button>
                                    <button
                                      class="p-1 rounded text-text-muted hover:text-nt-io-600 hover:bg-nt-io-500/10 transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
                                      onClick={(e) => handleForkSession(e, session.id)}
                                      aria-label="分叉会话"
                                      title="分叉（复制历史为新会话）"
                                    >
                                      <GitFork class="w-3.5 h-3.5" />
                                    </button>
                                    <button
                                      class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70 transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
                                      onClick={(e) => handleArchiveSession(e, session.id)}
                                      aria-label="归档会话"
                                      title="归档"
                                    >
                                      <Archive class="w-3.5 h-3.5" />
                                    </button>
                                    <button
                                      class="p-1 rounded text-text-muted hover:text-red-600 hover:bg-red-500/10 transition-colors focus-visible:ring-2 focus-visible:ring-red-500 focus-visible:outline-none"
                                      onClick={(e) => handleDeleteSession(e, session.id)}
                                      aria-label="删除会话"
                                      title="删除"
                                    >
                                      <NeoTrash class="w-3.5 h-3.5" />
                                    </button>
                                  </div>
                                </div>

                                {/* 会话标签行（按钮外，避免 button 嵌 button） */}
                                <Show when={sessionTags().length > 0}>
                                  <div class="px-3 pb-2 -mt-1">
                                    <div class="inline-flex flex-wrap gap-1">
                                      <For each={sessionTags()}>
                                        {(tag) => (
                                          <NeoTag
                                            name={tag}
                                            color="#909098"
                                            size="sm"
                                            active={(props.activeTags ?? []).includes(tag)}
                                          />
                                        )}
                                      </For>
                                    </div>
                                  </div>
                                </Show>
                              </div>
                              {/* 当前会话左侧红色指示条 */}
                              {active && (
                                <div class="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-6 rounded-r-full bg-nt-io-500 shadow-[0_0_8px_rgba(240,145,58,0.6)]" />
                              )}
                            </li>
                          )
                        }}
                      </For>
                      </ul>
                    </Show>
                  </div>
                )}
              </For>
            </Show>
            </Show>
          </div>

          {/* 归档入口：归档箱视图（对标 Claude Code Archive） */}
          <Show when={!showArchived()}>
            <button
              ref={archiveEntryRef}
              class="flex items-center gap-2 mx-3 mb-2 px-3 py-2 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/40 transition-colors text-12px focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
              onClick={openArchived}
              aria-label="已归档会话"
              title="已归档会话"
            >
              <Archive class="w-4 h-4" />
              <span>已归档</span>
            </button>
          </Show>

          {/* Footer: 用户条 sf（设计 v2）—— 头像+信息+设置整合为整体 */}
          <button class="sf" onClick={openSettings} aria-label="用户设置" title="用户设置">
            <div class="sa">N</div>
            <div class="su-info">
              <div class="su-name">Neo</div>
              <div class="su-plan">Free Plan</div>
            </div>
            <Settings class="su-gear" />
          </button>
        </>
      )}

      {collapsed() && !peek() && (
        <div class="flex-1 flex flex-col items-center gap-1 py-2">
          <button
            class="p-2 rounded-lg bg-nt-io-500/10 text-nt-io-600 hover:bg-nt-io-500/20 transition-colors"
            onClick={handleNewChat}
            aria-label="新建对话"
            title="新建对话"
          >
            <NeoPlus class="w-4 h-4" />
          </button>
        </div>
      )}

      {collapsed() && !peek() && (
        <button
          class="flex items-center justify-center py-3 border-t border-border-primary/40 hover:bg-white/40 transition-colors"
          onClick={openSettings}
          aria-label="用户设置"
          title="用户设置"
        >
          <div class="sa">N</div>
        </button>
      )}

      <ConfirmModal
        req={modalReq()}
        onConfirm={async (val) => {
          const deleteId = pendingDeleteId()
          const renameId = pendingRenameId()
          const title = val?.trim()
          if (deleteId) {
            // chatStore.deleteSession 吞错：重查后端验证真实结果，失败给出可见错误
            await chatStore.deleteSession(deleteId)
            const list = await fetchSessions()
            if (list && list.some(s => s.id === deleteId)) {
              showError('删除会话失败，请重试')
            } else if (list) {
              // 删除成功：删除按钮已卸载，焦点回移最近邻会话项
              focusNearestSession()
            }
          } else if (renameId && title) {
            // chatStore.updateSessionTitle 吞错：重查后端验证标题是否落盘
            await chatStore.updateSessionTitle(renameId, title)
            const list = await fetchSessions()
            if (list) {
              const s = list.find(x => x.id === renameId)
              if (s && s.name !== title) {
                showError('重命名会话失败，请重试')
              }
            }
          }
          closeModal()
        }}
        onClose={closeModal}
      />
    </aside>
  )
}
