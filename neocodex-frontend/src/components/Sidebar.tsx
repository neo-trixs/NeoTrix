import { createSignal, For, Show } from 'solid-js'
import { MessageSquare, Plus, Trash2, Pencil, ChevronLeft, Settings, Search, X } from 'lucide-solid'
import { chatStore } from '../stores/chat'
import { SettingsModal } from './SettingsModal'
import { clsx } from 'clsx'

interface SidebarProps {
  collapsed?: boolean
  onToggleCollapse?: () => void
  activeView?: 'chat' | 'cowork'
  onSwitchView?: (view: 'chat' | 'cowork') => void
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
  const viewIdx = () => (view() === 'chat' ? 0 : 1)

  // 会话搜索（前端过滤）
  const [searchOpen, setSearchOpen] = createSignal(false)
  const [searchQuery, setSearchQuery] = createSignal('')

  const toggleSearch = () => {
    const next = !searchOpen()
    setSearchOpen(next)
    if (!next) setSearchQuery('')
  }

  // 设置弹窗（统一 SettingsModal）
  const [settingsOpen, setSettingsOpen] = createSignal(false)

  const openSettings = () => {
    setSettingsOpen(true)
  }

  const switchView = (v: 'chat' | 'cowork') => {
    props.onSwitchView?.(v)
  }

  const sessions = chatStore.state.sessions
  const currentSessionId = chatStore.state.currentSessionId

  const groupedSessions = () => {
    const groups = new Map<GroupKey, typeof sessions>()
    for (const key of GROUP_ORDER) groups.set(key, [])
    const q = searchQuery().trim().toLowerCase()
    for (const session of sessions) {
      if (q && !session.title.toLowerCase().includes(q)) continue
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

  return (
    <aside class={clsx(
      'flex-shrink-0 h-screen bg-bg-primary border-r border-border-primary/60 transition-all duration-300 flex flex-col relative',
      collapsed() ? 'w-16' : 'w-[250px]'
    )}>
      {/* Header: 红绿灯（自绘 TrafficLights 组件，fixed 定位）+ 折叠按钮（设计 v2） */}
      <div class="flex items-center justify-between h-14 px-4 pl-[64px]" data-tauri-drag-region>
        <div class="flex-1 min-w-0" data-tauri-drag-region />
        <button
          class="p-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors flex-shrink-0"
          onClick={props.onToggleCollapse}
          aria-label={collapsed() ? '展开侧边栏' : '折叠侧边栏'}
          title={collapsed() ? '展开侧边栏' : '折叠侧边栏'}
        >
          <ChevronLeft class={clsx('w-4 h-4 transition-transform', collapsed() && 'rotate-180')} />
        </button>
      </div>

      {!collapsed() && (
        <>
          {/* Segmented Tabs：意识模式（设计 v2）—— E8 六芒星 / 星群 */}
          <div class="px-3 pb-3">
            <div class="seg">
              <button
                class={clsx('segb', viewIdx() === 0 && 'on')}
                onClick={() => switchView('chat')}
                aria-label="对话"
                title="对话"
              >
                <svg viewBox="0 0 16 16"><path d="M8 2l1.8 4.2L14 8l-4.2 1.8L8 14l-1.8-4.2L2 8l4.2-1.8z" stroke="currentColor" stroke-width="1" fill="none" stroke-linejoin="round" /><circle cx="8" cy="8" r="1.5" fill="currentColor" stroke="none" /></svg>
                <span class="segb-t">对话</span>
              </button>
              <button
                class={clsx('segb', viewIdx() === 1 && 'on')}
                onClick={() => switchView('cowork')}
                aria-label="协同"
                title="协同"
              >
                <svg viewBox="0 0 16 16"><path d="M6 3l1.2 2.8L10 7l-2.8 1.2L6 11l-1.2-2.8L2 7l2.8-1.2z" stroke="currentColor" stroke-width="1" fill="none" stroke-linejoin="round" /><path d="M10 7.5l1.2 2.8L14 11l-2.8 1.2L10 15l-1.2-2.8L6 11l2.8-1.2z" stroke="currentColor" stroke-width="0.8" fill="none" stroke-linejoin="round" opacity="0.45" /></svg>
                <span class="segb-t">协同</span>
              </button>
            </div>
          </div>

          {/* 搜索 + 新建 */}
          <div class="px-3 pb-2 flex items-center gap-2">
            <Show
              when={searchOpen()}
              fallback={
                <button
                  class="flex-1 flex items-center gap-2 px-3 py-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors text-left border border-border-primary/40"
                  onClick={toggleSearch}
                  aria-label="搜索会话"
                  title="搜索会话"
                >
                  <Search class="w-3.5 h-3.5" />
                  <span class="text-[12px]">搜索</span>
                </button>
              }
            >
              <div class="flex-1 flex items-center gap-1.5 px-2 py-1 rounded-lg border border-nt-io-500/40 bg-white/60">
                <Search class="w-3.5 h-3.5 text-nt-io-600 flex-shrink-0" />
                <input
                  class="flex-1 min-w-0 bg-transparent border-none outline-none text-[12px] text-text-primary placeholder-text-muted/60"
                  placeholder="搜索会话标题…"
                  value={searchQuery()}
                  onInput={(e) => setSearchQuery(e.currentTarget.value)}
                  autofocus
                />
                <button
                  class="p-0.5 text-text-muted hover:text-text-primary flex-shrink-0"
                  onClick={toggleSearch}
                  aria-label="关闭搜索"
                  title="关闭搜索"
                >
                  <X class="w-3.5 h-3.5" />
                </button>
              </div>
            </Show>
            <button
              class="p-1.5 rounded-lg bg-nt-io-500/10 text-nt-io-600 hover:bg-nt-io-500/20 transition-colors"
              onClick={handleNewChat}
              aria-label="新建对话"
              title="新建对话"
            >
              <Plus class="w-4 h-4" />
            </button>
          </div>

          {/* 会话列表（按时间分组） */}
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
                    <div class="re-h px-2 pb-1.5 pt-2 text-[10px] uppercase tracking-widest text-text-muted/60 font-medium">
                      {group.key}
                    </div>
                    <ul class="space-y-0.5" role="list" aria-label={`${group.key}会话`}>
                      <For each={group.items}>
                        {(session: { id: string; title: string; updatedAt: Date }) => {
                          const active = currentSessionId === session.id
                          return (
                            <li class="group relative">
                              <div class={clsx(
                                'flex items-center rounded-lg transition-colors',
                                active
                                  ? 'bg-nt-io-500/8 shadow-[inset_0_1px_0_rgba(255,255,255,0.7)]'
                                  : 'hover:bg-white/40'
                              )}>
                                <button
                                  class="flex-1 flex items-center gap-3 px-3 py-2 min-w-0 text-left"
                                  onClick={() => handleSwitchSession(session.id)}
                                  aria-current={active ? 'true' : 'false'}
                                  title={session.title}
                                >
                                  <MessageSquare class={clsx(
                                    'w-4 h-4 flex-shrink-0',
                                    active ? 'text-nt-io-600' : 'text-text-muted'
                                  )} />
                                  <span class={clsx(
                                    'flex-1 min-w-0 truncate text-[13px]',
                                    active ? 'text-text-primary font-medium' : 'text-text-secondary'
                                  )}>
                                    {session.title}
                                  </span>
                                  <span class="text-[11px] text-text-muted flex-shrink-0">
                                    {formatRelativeTime(session.updatedAt)}
                                  </span>
                                </button>
                                <div class="flex items-center gap-0.5 pr-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
                                  <button
                                    class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70 transition-colors"
                                    onClick={(e) => handleRenameSession(e, session.id)}
                                    aria-label="重命名会话"
                                    title="重命名"
                                  >
                                    <Pencil class="w-3.5 h-3.5" />
                                  </button>
                                  <button
                                    class="p-1 rounded text-text-muted hover:text-red-600 hover:bg-red-500/10 transition-colors"
                                    onClick={(e) => handleDeleteSession(e, session.id)}
                                    aria-label="删除会话"
                                    title="删除"
                                  >
                                    <Trash2 class="w-3.5 h-3.5" />
                                  </button>
                                </div>
                              </div>
                              {/* 当前会话左侧红色指示条 */}
                              {active && (
                                <div class="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-6 rounded-r-full bg-nt-io-500 shadow-[0_0_8px_rgba(232,84,84,0.6)]" />
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
            <Plus class="w-4 h-4" />
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

      {/* 设置弹窗（统一设计） */}
      <SettingsModal open={settingsOpen()} onClose={() => setSettingsOpen(false)} />
    </aside>
  )
}
