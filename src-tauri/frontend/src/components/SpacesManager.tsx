import { createSignal, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import { FolderOpen, Plus, Trash2, GitBranch, MessageSquare, Code2, Briefcase, ChevronRight, Globe, Lock } from 'lucide-solid'

/* ════════════════════════════════════════════════════════════
   SpacesManager — 工作空间组织器
   
   融合产品模式：
   - Devin Desktop: Spaces (共享上下文工作空间)
   - Claude Desktop: Projects (项目级上下文分组)
   - Linear: Workspace 组织
   
   NeoTrix 特有：
   - Constellation 成熟度标记
   - E8 意识状态聚合
   - 跨空间 agent 共享上下文
   ════════════════════════════════════════════════════════════ */

export type SpaceVisibility = 'private' | 'shared' | 'public'

export interface SpaceInfo {
  id: string
  name: string
  description?: string
  visibility: SpaceVisibility
  sessionCount: number
  agentCount: number
  branch?: string
  lastActivity: Date
  constellationMaturity?: string  // C0-C6
  phi?: number                    // 聚合意识度
  color: string
}

export interface SpacesManagerProps {
  spaces: () => SpaceInfo[]
  activeSpaceId: () => string | null
  onSelectSpace: (id: string) => void
  onCreateSpace: () => void
  onDeleteSpace: (id: string) => void
}

const VISIBILITY_CONFIG: Record<SpaceVisibility, { icon: any; label: string; color: string }> = {
  private: { icon: Lock,     label: '私有', color: 'text-zinc-500' },
  shared:  { icon: Globe,    label: '共享', color: 'text-blue-500' },
  public:  { icon: Globe,    label: '公开', color: 'text-emerald-500' },
}

const SPACE_COLORS = [
  'bg-orange-100 border-orange-200',
  'bg-blue-100 border-blue-200',
  'bg-emerald-100 border-emerald-200',
  'bg-purple-100 border-purple-200',
  'bg-amber-100 border-amber-200',
  'bg-pink-100 border-pink-200',
]

export function SpacesManager(props: SpacesManagerProps) {
  const [searchQuery, setSearchQuery] = createSignal('')

  const filteredSpaces = () => {
    const q = searchQuery().toLowerCase()
    return props.spaces().filter(s =>
      s.name.toLowerCase().includes(q) || s.description?.toLowerCase().includes(q)
    )
  }

  const sortedSpaces = () =>
    [...filteredSpaces()].sort((a, b) => b.lastActivity.getTime() - a.lastActivity.getTime())

  return (
    <div class="flex flex-col h-full">
      {/* Header */}
      <div class="flex items-center justify-between px-4 py-3 border-b border-border-primary/40">
        <div class="flex items-center gap-2">
          <FolderOpen class="w-4 h-4 text-text-muted" />
          <span class="text-[13px] font-medium text-text-primary">工作空间</span>
          <span class="text-[11px] text-text-muted bg-white/60 px-1.5 py-0.5 rounded-full">
            {props.spaces().length} 个空间
          </span>
        </div>
        <button
          class="p-1.5 rounded-lg text-nt-io-600 bg-nt-io-500/10 hover:bg-nt-io-500/20 transition-colors"
          onClick={props.onCreateSpace}
          title="创建新空间"
        >
          <Plus class="w-4 h-4" />
        </button>
      </div>

      {/* Search */}
      <div class="px-4 py-2">
        <input
          class="w-full px-3 py-1.5 rounded-lg bg-white/50 border border-black/5 text-[12px] text-text-primary placeholder-text-muted/50 outline-none focus:border-nt-io-500/40 transition-colors"
          placeholder="搜索空间…"
          value={searchQuery()}
          onInput={(e) => setSearchQuery(e.currentTarget.value)}
        />
      </div>

      {/* Space list */}
      <div class="flex-1 overflow-y-auto px-4 pb-3 space-y-2">
        <Show
          when={sortedSpaces().length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center py-12 text-text-muted">
              <FolderOpen class="w-8 h-8 mb-2 opacity-30" />
              <span class="text-[12px]">暂无工作空间</span>
              <span class="text-[11px] mt-1 opacity-60">点击 + 创建第一个空间</span>
            </div>
          }
        >
          <For each={sortedSpaces()}>
            {(space) => {
              const active = () => props.activeSpaceId() === space.id
              const visConfig = VISIBILITY_CONFIG[space.visibility]
              const VisIcon = visConfig.icon
              return (
                <div
                  class={clsx(
                    'p-3 rounded-xl border transition-all cursor-pointer',
                    active()
                      ? 'border-nt-io-500/40 bg-nt-io-500/5 shadow-sm'
                      : 'border-black/5 bg-white/40 hover:bg-white/60'
                  )}
                  onClick={() => props.onSelectSpace(space.id)}
                  role="button"
                  tabindex={0}
                >
                  {/* Title row */}
                  <div class="flex items-center gap-2 mb-1">
                    <div class={clsx('w-3 h-3 rounded-md border', space.color || SPACE_COLORS[0])} />
                    <span class="text-[12px] font-medium text-text-primary flex-1 min-w-0 truncate">
                      {space.name}
                    </span>
                    <VisIcon class={clsx('w-3 h-3', visConfig.color)} />
                    <button
                      class="p-1 rounded text-text-muted/40 hover:text-red-500 hover:bg-red-50 transition-colors"
                      onClick={(e) => { e.stopPropagation(); props.onDeleteSpace(space.id) }}
                      title="删除空间"
                    >
                      <Trash2 class="w-3 h-3" />
                    </button>
                  </div>

                  {/* Description */}
                  <Show when={space.description}>
                    <div class="text-[10px] text-text-muted mb-1.5 line-clamp-2">{space.description}</div>
                  </Show>

                  {/* Branch */}
                  <Show when={space.branch}>
                    <div class="flex items-center gap-1 text-[10px] text-text-muted mb-1.5 font-mono">
                      <GitBranch class="w-3 h-3" />
                      <span class="truncate">{space.branch}</span>
                    </div>
                  </Show>

                  {/* Stats */}
                  <div class="flex items-center gap-3 text-[10px] text-text-muted">
                    <span class="flex items-center gap-1">
                      <MessageSquare class="w-3 h-3" />
                      {space.sessionCount} 会话
                    </span>
                    <span class="flex items-center gap-1">
                      <Code2 class="w-3 h-3" />
                      {space.agentCount} Agent
                    </span>
                    <Show when={space.constellationMaturity}>
                      <span class="px-1.5 py-0.5 rounded-full bg-purple-100 text-purple-600 text-[9px] font-medium">
                        {space.constellationMaturity}
                      </span>
                    </Show>
                    <Show when={space.phi !== undefined}>
                      <span class="ml-auto">φ {(space.phi! * 100).toFixed(0)}%</span>
                    </Show>
                  </div>
                </div>
              )
            }}
          </For>
        </Show>
      </div>
    </div>
  )
}
