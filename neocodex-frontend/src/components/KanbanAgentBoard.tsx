import { createSignal, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import { LayoutGrid, Pause, Play, Square, ChevronDown, Brain, GitBranch, Clock, Activity } from 'lucide-solid'
import type { AgentInfo, AgentStatus, SurfaceMode } from './NeoTrixCore'

/* ════════════════════════════════════════════════════════════
   KanbanAgentBoard — Devin 风格多列 Agent 看板
   
   融合产品模式：
   - Devin Desktop: Agent Command Center (Kanban 视图)
   - Cursor 3: Agents Window (并行 agent 列表)
   - Linear: 状态分列看板
   
   NeoTrix 特有：
   - E8 φ 意识状态与 agent 卡片绑定
   - Constellation 成熟度标记
   - SEAL 情感颜色编码
   ════════════════════════════════════════════════════════════ */

export type KanbanColumn = 'queued' | 'running' | 'review' | 'done' | 'failed'

interface KanbanColumnDef {
  id: KanbanColumn
  label: string
  color: string
  bgColor: string
  borderColor: string
}

const COLUMNS: KanbanColumnDef[] = [
  { id: 'queued',  label: '等待中', color: 'text-zinc-500',    bgColor: 'bg-zinc-50/50',   borderColor: 'border-zinc-200/50' },
  { id: 'running', label: '运行中', color: 'text-emerald-600', bgColor: 'bg-emerald-50/50', borderColor: 'border-emerald-200/50' },
  { id: 'review',  label: '审查中', color: 'text-amber-600',   bgColor: 'bg-amber-50/50',   borderColor: 'border-amber-200/50' },
  { id: 'done',    label: '已完成', color: 'text-blue-600',    bgColor: 'bg-blue-50/50',    borderColor: 'border-blue-200/50' },
  { id: 'failed',  label: '出错',   color: 'text-red-600',     bgColor: 'bg-red-50/50',     borderColor: 'border-red-200/50' },
]

const STATUS_TO_COLUMN: Record<AgentStatus, KanbanColumn> = {
  idle:      'queued',
  running:   'running',
  paused:    'review',
  completed: 'done',
  error:     'failed',
}

const STATUS_COLORS: Record<AgentStatus, string> = {
  running:   'bg-emerald-400',
  paused:    'bg-amber-400',
  completed: 'bg-blue-400',
  error:     'bg-red-400',
  idle:      'bg-zinc-300',
}

const MODE_COLORS: Record<SurfaceMode, string> = {
  chat: 'bg-orange-100 text-orange-700',
  work: 'bg-blue-100 text-blue-700',
  code: 'bg-emerald-100 text-emerald-700',
}

export interface KanbanAgentBoardProps {
  agents: () => AgentInfo[]
  onSelectAgent: (id: string) => void
  onPauseAgent: (id: string) => void
  onStopAgent: (id: string) => void
  onMoveAgent: (id: string, target: KanbanColumn) => void
}

export function KanbanAgentBoard(props: KanbanAgentBoardProps) {
  const [collapsedCols, setCollapsedCols] = createSignal<Set<KanbanColumn>>(new Set())
  const [draggedAgent, setDraggedAgent] = createSignal<string | null>(null)

  const toggleColumn = (col: KanbanColumn) => {
    setCollapsedCols(prev => {
      const next = new Set(prev)
      if (next.has(col)) next.delete(col); else next.add(col)
      return next
    })
  }

  const agentsByColumn = (col: KanbanColumn) =>
    props.agents().filter(a => STATUS_TO_COLUMN[a.status] === col)

  const handleDragStart = (e: DragEvent, agentId: string) => {
    setDraggedAgent(agentId)
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move'
      try { e.dataTransfer.setData('text/plain', agentId) } catch { /* noop */ }
    }
  }

  const handleDragOver = (e: DragEvent) => {
    if (draggedAgent()) { e.preventDefault(); if (e.dataTransfer) e.dataTransfer.dropEffect = 'move' }
  }

  const handleDrop = (e: DragEvent, targetCol: KanbanColumn) => {
    e.preventDefault()
    const agentId = draggedAgent()
    if (agentId) {
      props.onMoveAgent(agentId, targetCol)
      setDraggedAgent(null)
    }
  }

  return (
    <div class="flex flex-col h-full">
      {/* Header */}
      <div class="flex items-center justify-between px-4 py-3 border-b border-border-primary/40">
        <div class="flex items-center gap-2">
          <LayoutGrid class="w-4 h-4 text-text-muted" />
          <span class="text-[13px] font-medium text-text-primary">Agent 看板</span>
          <span class="text-[11px] text-text-muted bg-white/60 px-1.5 py-0.5 rounded-full">
            {props.agents().length} 个 Agent
          </span>
        </div>
      </div>

      {/* Kanban columns */}
      <div class="flex-1 flex overflow-x-auto gap-3 p-3">
        <For each={COLUMNS}>
          {(col) => {
            const colAgents = () => agentsByColumn(col.id)
            const collapsed = () => collapsedCols().has(col.id)
            return (
              <div
                class={clsx(
                  'flex flex-col min-w-[220px] max-w-[280px] flex-1 rounded-xl border transition-all',
                  col.bgColor, col.borderColor
                )}
                onDragOver={handleDragOver}
                onDrop={(e) => handleDrop(e, col.id)}
              >
                {/* Column header */}
                <button
                  class="flex items-center gap-2 px-3 py-2 w-full text-left focus-visible:outline-none"
                  onClick={() => toggleColumn(col.id)}
                >
                  <ChevronDown class={clsx(
                    'w-3 h-3 text-text-muted transition-transform',
                    collapsed() && '-rotate-90'
                  )} />
                  <span class={clsx('text-[11px] font-semibold uppercase tracking-wide', col.color)}>
                    {col.label}
                  </span>
                  <span class="text-[10px] text-text-muted bg-white/60 px-1.5 py-0.5 rounded-full ml-auto">
                    {colAgents().length}
                  </span>
                </button>

                {/* Column cards */}
                <Show when={!collapsed()}>
                  <div class="flex-1 overflow-y-auto px-2 pb-2 space-y-2 min-h-[60px]">
                    <Show
                      when={colAgents().length > 0}
                      fallback={
                        <div class="text-[10px] text-text-muted/50 text-center py-4">
                          拖拽 Agent 到此处
                        </div>
                      }
                    >
                      <For each={colAgents()}>
                        {(agent) => (
                          <AgentKanbanCard
                            agent={agent}
                            onSelect={() => props.onSelectAgent(agent.id)}
                            onPause={() => props.onPauseAgent(agent.id)}
                            onStop={() => props.onStopAgent(agent.id)}
                            onDragStart={(e) => handleDragStart(e, agent.id)}
                            isDragging={draggedAgent() === agent.id}
                          />
                        )}
                      </For>
                    </Show>
                  </div>
                </Show>
              </div>
            )
          }}
        </For>
      </div>
    </div>
  )
}

/* ── Agent Kanban Card ── */

interface AgentKanbanCardProps {
  agent: AgentInfo
  onSelect: () => void
  onPause: () => void
  onStop: () => void
  onDragStart: (e: DragEvent) => void
  isDragging: boolean
}

function AgentKanbanCard(props: AgentKanbanCardProps) {
  const [expanded, setExpanded] = createSignal(false)

  return (
    <div
      class={clsx(
        'p-2.5 rounded-lg border transition-all cursor-pointer select-none',
        'bg-white/70 border-black/5 hover:shadow-sm',
        props.isDragging && 'opacity-50 scale-95'
      )}
      draggable={true}
      onDragStart={props.onDragStart}
      onClick={props.onSelect}
      role="button"
      tabindex={0}
    >
      {/* Title row */}
      <div class="flex items-center gap-2 mb-1.5">
        <div class={clsx('w-2 h-2 rounded-full flex-shrink-0', STATUS_COLORS[props.agent.status])} />
        <span class="text-[12px] font-medium text-text-primary flex-1 min-w-0 truncate">
          {props.agent.name}
        </span>
        <span class={clsx(
          'text-[9px] px-1.5 py-0.5 rounded-full font-medium',
          MODE_COLORS[props.agent.mode]
        )}>
          {props.agent.mode}
        </span>
      </div>

      {/* Branch */}
      <Show when={props.agent.branch}>
        <div class="flex items-center gap-1 text-[10px] text-text-muted mb-1.5 font-mono">
          <GitBranch class="w-3 h-3 flex-shrink-0" />
          <span class="truncate">{props.agent.branch}</span>
        </div>
      </Show>

      {/* Progress bar */}
      <Show when={props.agent.progress !== undefined && props.agent.status === 'running'}>
        <div class="w-full h-1 bg-black/5 rounded-full mb-1.5 overflow-hidden">
          <div
            class="h-full bg-emerald-400 rounded-full transition-all duration-300"
            style={{ width: `${props.agent.progress}%` }}
          />
        </div>
      </Show>

      {/* Stats row */}
      <div class="flex items-center gap-2 text-[9px] text-text-muted">
        <span class="flex items-center gap-0.5">
          <Activity class="w-2.5 h-2.5" />
          {props.agent.filesChanged} 文件
        </span>
        <span>{props.agent.toolCalls} 调用</span>
        <Show when={props.agent.phi !== undefined}>
          <span class="flex items-center gap-0.5">
            <Brain class="w-2.5 h-2.5" />
            φ {(props.agent.phi! * 100).toFixed(0)}%
          </span>
        </Show>
        <span class="ml-auto flex items-center gap-0.5">
          <Clock class="w-2.5 h-2.5" />
          {formatDuration(props.agent.startedAt)}
        </span>
      </div>

      {/* Expanded actions */}
      <Show when={expanded()}>
        <div class="flex gap-1.5 mt-2 pt-2 border-t border-black/5">
          <Show when={props.agent.status === 'running' || props.agent.status === 'paused'}>
            <button
              class="flex-1 flex items-center justify-center gap-1 px-2 py-1 rounded text-[10px] bg-white/60 border border-black/5 hover:bg-white/80 transition-colors"
              onClick={(e) => { e.stopPropagation(); props.onPause() }}
            >
              {props.agent.status === 'running' ? <Pause class="w-3 h-3" /> : <Play class="w-3 h-3" />}
              {props.agent.status === 'running' ? '暂停' : '恢复'}
            </button>
            <button
              class="flex items-center justify-center gap-1 px-2 py-1 rounded text-[10px] text-red-500 bg-red-50 border border-red-200/50 hover:bg-red-100 transition-colors"
              onClick={(e) => { e.stopPropagation(); props.onStop() }}
            >
              <Square class="w-3 h-3" />
              停止
            </button>
          </Show>
        </div>
      </Show>

      {/* Expand toggle */}
      <button
        class="w-full flex justify-center mt-1"
        onClick={(e) => { e.stopPropagation(); setExpanded(!expanded()) }}
      >
        <ChevronDown class={clsx(
          'w-3 h-3 text-text-muted/40 transition-transform',
          expanded() && 'rotate-180'
        )} />
      </button>
    </div>
  )
}

/* ── Helpers ── */

function formatDuration(startedAt: Date): string {
  const diff = Date.now() - startedAt.getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return '<1m'
  if (minutes < 60) return `${minutes}m`
  return `${Math.floor(minutes / 60)}h${minutes % 60 > 0 ? `${minutes % 60}m` : ''}`
}
