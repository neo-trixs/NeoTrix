import { createSignal, For, Show } from 'solid-js'
import { Settings } from 'lucide-solid'
import { chatStore } from '../stores/chat'
import { tagsStore, normalizeTagName } from '../stores/tags'
import { clsx } from 'clsx'
import { NeoPlus, NeoMessage, NeoSearch, NeoChevronRight, NeoTrash, NeoPencil, NeoClose } from './neo-icons'
import { NeoTag } from './NeoTag'
import { TagBar } from './TagBar'

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

  // 会话搜索（前端过滤）
  const [searchOpen, setSearchOpen] = createSignal(false)
  const [searchQuery, setSearchQuery] = createSignal('')
  // 分组模式：时间 / 项目（对标 Claude group-by-project）
  const [groupMode, setGroupMode] = createSignal<'time' | 'project'>('time')

  const toggleSearch = () => {
    const next = !searchOpen()
    setSearchOpen(next)
    if (!next) setSearchQuery('')
  }

  // 设置入口：由 Chat 根级渲染弹窗（避免 aside overflow-hidden 裁剪 fixed 弹窗）
  const openSettings = () => {
    props.onOpenSettings?.()
  }

  const switchView = (v: 'chat' | 'cowork' | 'computer') => {
    props.onSwitchView?.(v)
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

  const handleNewChat = () => {
    chatStore.addSession()
  }

  const handleSwitchSession = (id: string) => {
    chatStore.switchSession(id)
  }

  const handleDeleteSession = (e: Event, id: string) => {
    e.stopPropagation()
    // 破坏性操作确认（对标 Codex）
    if (!window.confirm('确定删除该会话？此操作不可撤销。')) return
    chatStore.deleteSession(id)
  }

  const handleRenameSession = (e: Event, id: string) => {
    e.stopPropagation()
    const newTitle = prompt('重命名会话:')
    if (newTitle?.trim()) {
      chatStore.updateSessionTitle(id, newTitle.trim())
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
                  if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') { e.preventDefault(); switchView(view() === 'chat' ? 'cowork' : 'chat') }
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
                  if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') { e.preventDefault(); switchView(view() === 'chat' ? 'cowork' : 'chat') }
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
                  if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') { e.preventDefault(); switchView(view() === 'cowork' ? 'computer' : 'cowork') }
                }}
                aria-label="电脑"
                title="电脑"
              >
                <svg viewBox="0 0 16 16" fill="none"><rect x="1.5" y="2.5" width="13" height="9" rx="1.5" stroke="currentColor" stroke-width="1.1" /><line x1="5.5" y1="14.5" x2="10.5" y2="14.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /><line x1="8" y1="11.5" x2="8" y2="14.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /><line x1="1.5" y1="7.5" x2="14.5" y2="7.5" stroke="currentColor" stroke-width="1" opacity="0.45" /></svg>
                <span class="segb-t">电脑</span>
              </button>
            </div>
          </div>

          {/* 搜索 + 新建 */}
          <div class="px-3 pb-2 flex items-center gap-2">
            <Show
              when={searchOpen()}
              fallback={
                <button
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
              <div class="flex-1 flex items-center gap-2 px-2 py-1 rounded-lg border border-nt-io-500/40 bg-white/60">
                <NeoSearch class="w-4 h-4 text-nt-io-600 flex-shrink-0" />
                <input
                  class="flex-1 min-w-0 bg-transparent border-none outline-none text-12px text-text-primary placeholder-text-muted/60 focus-visible:ring-0"
                  placeholder="搜索会话标题…"
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

          {/* 会话列表（按时间/项目分组） */}
          <div class="flex-1 overflow-y-auto px-3 pb-4">
            <Show
              when={groupedSessions().length > 0}
              fallback={
                <div class="px-3 py-8 text-center text-text-muted text-sm">暂无对话记录</div>
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
                                    aria-current={active ? 'true' : 'false'}
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
          </div>

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
    </aside>
  )
}
