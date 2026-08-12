import { createSignal, For, Show, onCleanup } from 'solid-js'
import { Settings, Archive, RotateCcw, GitBranch, Coins, ListTodo, History, MessageSquare, MonitorPlay } from 'lucide-solid'
import { chatStore } from '../stores/chat'
import { tagsStore, normalizeTagName } from '../stores/tags'
import { clsx } from 'clsx'
import { neocodex } from '../api'
import { NeoPlus, NeoMessage, NeoSearch, NeoChevronRight, NeoTrash, NeoPencil, NeoClose } from './neo-icons'
import { NeoTag } from './NeoTag'
import { TagBar } from './TagBar'
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
}

const GROUP_ORDER = ['今天', '昨天', '前7天', '更早'] as const
type GroupKey = (typeof GROUP_ORDER)[number]

// Segmented Tab 顺序（WAI-ARIA tabs：→ 前进到下一个，← 后退到上一个，边界停在原位）
const VIEW_ORDER = ['chat', 'cowork', 'computer'] as const
type ViewKey = (typeof VIEW_ORDER)[number]

function getGroupKey(date: Date): GroupKey {
  const now = new Date()
  const startOfDay = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime()
  const diffDays = Math.floor((startOfDay(now) - startOfDay(date)) / 86_400_000)
  if (diffDays <= 0) return '今天'
  if (diffDays === 1) return '昨天'
  if (diffDays <= 7) return '前7天'
  return '更早'
}

export function Sidebar(props: SidebarProps) {
  const collapsed = () => props.collapsed ?? false
  const view = () => props.activeView ?? 'chat'
  const viewIdx = () => (view() === 'chat' ? 0 : view() === 'cowork' ? 1 : 2)

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
  let searchInputRef: HTMLInputElement | undefined
  let viewTabsRef: HTMLDivElement | undefined
  // 会话列表容器引用：删除/归档当前会话后焦点回移最近邻条目
  let sessionListRef: HTMLDivElement | undefined
  // 分组模式：时间 / 项目（对标 Claude group-by-project）
  const [groupMode, setGroupMode] = createSignal<'time' | 'project'>('time')

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
    if (groupMode() === 'project') {
      // 项目分组：按 session.project 归组，未知归「其他」
      const map = new Map<string, typeof filtered>()
      for (const s of filtered) {
        const key = s.project || '其他'
        if (!map.has(key)) map.set(key, [])
        map.get(key)!.push(s)
      }
      const sorted = [...map.entries()].sort((a, b) => b[1].length - a[1].length)
      return sorted.map(([key, items]) => ({ key, items }))
    }
    const groups = new Map<GroupKey, typeof filtered>()
    for (const key of GROUP_ORDER) groups.set(key, [])
    for (const session of filtered) {
      const key = getGroupKey(session.updatedAt)
      groups.get(key)?.push(session)
    }
    return GROUP_ORDER.map((key) => ({ key, items: groups.get(key) ?? [] })).filter((g) => g.items.length > 0)
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

  /* ── 打标交互（对标 Obsidian 标签输入） ── */
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
  const tagSuggestions = () => {
    const q = normalizeTagName(tagInput())
    if (!q) return []
    return Object.keys(tagsStore.state.tags)
      .filter((t) => t.includes(q) && t !== q)
      .slice(0, 5)
  }

  const handleToggleTag = (name: string) => {
    props.onToggleTag?.(name)
  }

  return (
    <aside class={clsx(
      'flex-shrink-0 glass-side overflow-hidden transition-all duration-300 flex flex-col relative',
      collapsed() ? 'w-16' : 'w-[250px]'
    )}>
      {/* Header: 红绿灯（自绘 TrafficLights 组件，fixed 定位）+ 折叠按钮（设计 v2） */}
      <div class="flex items-center justify-between h-14 px-4 pl-[64px]" data-tauri-drag-region>
        <div class="flex-1 min-w-0" data-tauri-drag-region />
        <button
          class="p-2 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors flex-shrink-0 focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
          onClick={props.onToggleCollapse}
          aria-label={collapsed() ? '展开侧边栏' : '折叠侧边栏'}
          title={collapsed() ? '展开侧边栏' : '折叠侧边栏'}
        >
          <NeoChevronRight class={clsx('w-4 h-4 transition-transform', !collapsed() && 'rotate-180')} />
        </button>
      </div>

      {!collapsed() && (
        <>
          {/* Segmented Tabs：意识模式（设计 v2）—— E8 六芒星 / 星群 */}
          <div class="px-3 pb-3">
            <div class="seg" role="tablist" aria-label="意识模式视图切换">
              <button
                class={clsx('segb', viewIdx() === 0 && 'on')}
                onClick={() => switchView('chat')}
                role="tab"
                aria-selected={viewIdx() === 0}
                tabIndex={viewIdx() === 0 ? 0 : -1}
                onKeyDown={(e) => {
                  if (e.key === 'ArrowRight') { e.preventDefault(); moveTab(1) }
                  else if (e.key === 'ArrowLeft') { e.preventDefault(); moveTab(-1) }
                  else if (e.key === 'Home') { e.preventDefault(); activateView('chat') }
                  else if (e.key === 'End') { e.preventDefault(); activateView('computer') }
                }}
                aria-label="对话"
                title="对话"
              >
                <svg viewBox="0 0 16 16" fill="none"><circle cx="8" cy="8" r="1.3" stroke="currentColor" stroke-width="1.1" /><line x1="8" y1="2.5" x2="8" y2="0.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /><line x1="8" y1="13.5" x2="8" y2="15.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /><line x1="2.5" y1="8" x2="0.5" y2="8" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /><line x1="13.5" y1="8" x2="15.5" y2="8" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /></svg>
                <span class="segb-t">对话</span>
              </button>
              <button
                class={clsx('segb', viewIdx() === 1 && 'on')}
                onClick={() => switchView('cowork')}
                role="tab"
                aria-selected={viewIdx() === 1}
                tabIndex={viewIdx() === 1 ? 0 : -1}
                onKeyDown={(e) => {
                  if (e.key === 'ArrowRight') { e.preventDefault(); moveTab(1) }
                  else if (e.key === 'ArrowLeft') { e.preventDefault(); moveTab(-1) }
                  else if (e.key === 'Home') { e.preventDefault(); activateView('chat') }
                  else if (e.key === 'End') { e.preventDefault(); activateView('computer') }
                }}
                aria-label="协同"
                title="协同"
              >
                <svg viewBox="0 0 16 16" fill="none"><circle cx="5.5" cy="5.5" r="1.2" stroke="currentColor" stroke-width="1.1" /><line x1="5.5" y1="1.5" x2="5.5" y2="0.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /><line x1="5.5" y1="9.5" x2="5.5" y2="10.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /><line x1="1.5" y1="5.5" x2="0.5" y2="5.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /><line x1="9.5" y1="5.5" x2="10.5" y2="5.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /><circle cx="11" cy="11" r="1.2" stroke="currentColor" stroke-width="1.1" opacity="0.55" /><line x1="11" y1="7" x2="11" y2="6" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.55" /><line x1="11" y1="13" x2="11" y2="14" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.55" /><line x1="7" y1="10" x2="6" y2="10" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.55" /><line x1="13" y1="10" x2="14" y2="10" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.55" /></svg>
                <span class="segb-t">协同</span>
              </button>
              <button
                class={clsx('segb', viewIdx() === 2 && 'on')}
                onClick={() => switchView('computer')}
                role="tab"
                aria-selected={viewIdx() === 2}
                tabIndex={viewIdx() === 2 ? 0 : -1}
                onKeyDown={(e) => {
                  if (e.key === 'ArrowRight') { e.preventDefault(); moveTab(1) }
                  else if (e.key === 'ArrowLeft') { e.preventDefault(); moveTab(-1) }
                  else if (e.key === 'Home') { e.preventDefault(); activateView('chat') }
                  else if (e.key === 'End') { e.preventDefault(); activateView('computer') }
                }}
                aria-label="电脑"
                title="电脑"
              >
                <svg viewBox="0 0 16 16" fill="none"><rect x="1.5" y="2.5" width="13" height="9" rx="1.5" stroke="currentColor" stroke-width="1.1" /><line x1="5.5" y1="14.5" x2="10.5" y2="14.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /><line x1="8" y1="11.5" x2="8" y2="14.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /><line x1="1.5" y1="7.5" x2="14.5" y2="7.5" stroke="currentColor" stroke-width="1" opacity="0.45" /></svg>
                <span class="segb-t">电脑</span>
              </button>
            </div>
          </div>

          {/* 功能面板入口（对标 Claude Code 侧栏：git/成本/任务/检查点/侧聊，点击打开，Esc 关闭）。
              仅 chat 视图渲染面板（Chat.tsx 面板区有 activeView==='chat' 门禁），
              非 chat 视图禁能防"点了没反应"的死按钮。 */}
          <div class="px-3 pb-2 flex items-center gap-1" role="group" aria-label="功能面板">
            <For each={[
              { id: 'git', label: 'Git', icon: GitBranch, active: props.activePanel === 'git' },
              { id: 'cost', label: '成本', icon: Coins, active: props.activePanel === 'cost' },
              { id: 'tasks', label: '任务', icon: ListTodo, active: props.activePanel === 'tasks' },
              { id: 'timeline', label: '检查点', icon: History, active: props.activePanel === 'timeline' },
              { id: 'sidechat', label: '侧聊', icon: MessageSquare, active: props.activePanel === 'sidechat' },
              { id: 'preview', label: '预览', icon: MonitorPlay, active: props.activePanel === 'preview' },
            ]}>
              {(p) => {
                const inChat = view() === 'chat'
                return (
                  <button
                    class={clsx(
                      'flex-1 flex items-center justify-center gap-1 px-1 py-1.5 rounded-lg text-[11px] transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none',
                      p.active
                        ? 'bg-nt-io-500/15 text-nt-io-600'
                        : 'text-text-muted hover:text-text-primary hover:bg-white/60',
                      !inChat && 'opacity-40 cursor-not-allowed hover:bg-transparent hover:text-text-muted'
                    )}
                    onClick={() => inChat && props.onTogglePanel?.(p.id)}
                    aria-label={p.label}
                    aria-pressed={p.active}
                    aria-disabled={!inChat}
                    title={inChat ? p.label : '仅在对话视图可用'}
                  >
                    <p.icon class="w-3.5 h-3.5 flex-shrink-0" />
                    <span class="truncate">{p.label}</span>
                  </button>
                )
              }}
            </For>
          </div>

          {/* 搜索 + 新建 */}
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

          {/* 分组模式：时间 / 项目（对标 Claude group-by-project） */}
          <div class="px-3 pb-2 flex items-center gap-1" role="group" aria-label="会话分组方式">
            <button
              class={clsx(
                'flex-1 px-2 py-1 rounded-md text-11px transition-colors',
                groupMode() === 'time'
                  ? 'bg-white/70 text-text-primary font-medium shadow-[inset_0_1px_0_rgba(255,255,255,0.8)]'
                  : 'text-text-muted hover:text-text-primary hover:bg-white/40'
              )}
              onClick={() => setGroupMode('time')}
              aria-pressed={groupMode() === 'time'}
            >
              按时间
            </button>
            <button
              class={clsx(
                'flex-1 px-2 py-1 rounded-md text-11px transition-colors',
                groupMode() === 'project'
                  ? 'bg-white/70 text-text-primary font-medium shadow-[inset_0_1px_0_rgba(255,255,255,0.8)]'
                  : 'text-text-muted hover:text-text-primary hover:bg-white/40'
              )}
              onClick={() => setGroupMode('project')}
              aria-pressed={groupMode() === 'project'}
            >
              按项目
            </button>
          </div>

          {/* 标签区（对标 Obsidian Tag Pane：层级树 + 多选过滤） */}
          <TagBar
            activeTags={props.activeTags ?? []}
            onToggleTag={handleToggleTag}
            onClearTags={props.onClearTags ?? (() => {})}
          />

          {/* 会话列表（按时间/项目分组）；showArchived 时切换为归档箱视图 */}
          <div ref={sessionListRef} class="flex-1 overflow-y-auto px-3 pb-4">
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
              <For each={groupedSessions()}>
                {(group) => (
                  <div class="mb-4 last:mb-0">
                    <div class="re-h px-2 pb-2 pt-2 text-10px uppercase tracking-widest text-text-muted/60 font-medium">
                      {group.key}
                    </div>
                    <ul class="space-y-1" role="list" aria-label={`${group.key}会话`}>
                      <For each={group.items}>
                        {(session: { id: string; title: string; updatedAt: Date }) => {
                          const active = currentSessionId() === session.id
                          const sessionTags = () => chatStore.tagsForSession(session.id)
                          const isTagging = () => taggingSessionId() === session.id
                          return (
                            <li class="group relative">
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
                                            color={tagsStore.state.tags[tag] ?? '#909098'}
                                            size="sm"
                                            active={(props.activeTags ?? []).includes(tag)}
                                            onClick={() => handleToggleTag(tag)}
                                            onRemove={(t) => handleRemoveTag(session.id, t)}
                                            showHierarchy
                                          />
                                        )}
                                      </For>
                                    </div>
                                  </div>
                                </Show>

                                {/* 打标输入（inline 展开，含已有标签建议） */}
                                <Show when={isTagging()}>
                                  <div class="mx-3 mb-2 px-2 py-1.5 rounded-lg bg-white/70 border border-nt-io-500/30 backdrop-blur-sm">
                                    <div class="flex items-center gap-1.5">
                                      <span class="text-nt-io-600 flex-shrink-0 text-[11px] font-mono">#</span>
                                      <input
                                        ref={tagInputRef}
                                        class="flex-1 min-w-0 bg-transparent border-none outline-none text-[12px] text-text-primary placeholder-text-muted/60"
                                        placeholder="输入标签，Enter 添加（支持 父/子 层级）"
                                        value={tagInput()}
                                        onInput={(e) => setTagInput(e.currentTarget.value)}
                                        onKeyDown={(e) => handleTagInputKey(e, session.id)}
                                      />
                                    </div>
                                    <Show when={tagSuggestions().length > 0}>
                                      <div class="flex flex-wrap gap-1 pt-1">
                                        <For each={tagSuggestions()}>
                                          {(sugg) => (
                                            <button
                                              class="px-1.5 py-0.5 rounded text-[10px] text-text-muted hover:text-nt-io-600 hover:bg-nt-io-500/10 transition-colors"
                                              onClick={() => { chatStore.tagSession(session.id, sugg); setTagInput('') }}
                                            >
                                              # {sugg}
                                            </button>
                                          )}
                                        </For>
                                      </div>
                                    </Show>
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

      {collapsed() && (
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

      {collapsed() && (
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
