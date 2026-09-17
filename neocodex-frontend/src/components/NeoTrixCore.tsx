import { createSignal, For, Show, createEffect } from 'solid-js'
import { clsx } from 'clsx'
import { MessageSquare, Briefcase, Code2, Brain, Zap, Globe, Terminal, ChevronDown } from 'lucide-solid'

/* ════════════════════════════════════════════════════════════
   UnifiedSurface — NeoTrix 统一表面 (Chat+Work+Code)
   
   融合产品模式：
   - ChatGPT Desktop: 三合一超级应用
   - Claude Cowork: Chat/Work/Code 三标签
   - Cursor 3: Agent-first interface
   
   NeoTrix 特有：
   - E8 意识状态指示 (consciousness indicator)
   - GWT 注意力路由可视化
   - SEAL 情感反馈
   ════════════════════════════════════════════════════════════ */

export type SurfaceMode = 'chat' | 'work' | 'code'

export interface ModeConfig {
  id: SurfaceMode
  label: string
  description: string
  icon: any
  color: string
  bgActive: string
  features: string[]
  modelTier: 'fast' | 'balanced' | 'strong'
}

const MODES: ModeConfig[] = [
  {
    id: 'chat',
    label: '对话',
    description: '快速问答，日常助手',
    icon: MessageSquare,
    color: 'text-orange-500',
    bgActive: 'bg-orange-500/10 border-orange-500/30',
    features: ['快速响应', '上下文理解', '多轮对话'],
    modelTier: 'balanced',
  },
  {
    id: 'work',
    label: '工作',
    description: '研究分析，文档创建',
    icon: Briefcase,
    color: 'text-blue-500',
    bgActive: 'bg-blue-500/10 border-blue-500/30',
    features: ['深度研究', '文档生成', '数据分析', '浏览器'],
    modelTier: 'strong',
  },
  {
    id: 'code',
    label: '代码',
    description: '开发编码，Git集成',
    icon: Code2,
    color: 'text-emerald-500',
    bgActive: 'bg-emerald-500/10 border-emerald-500/30',
    features: ['代码编辑', 'Git工作树', '测试运行', '终端'],
    modelTier: 'strong',
  },
]

export interface UnifiedSurfaceProps {
  activeMode: () => SurfaceMode
  onModeChange: (mode: SurfaceMode) => void
  consciousnessPhi?: () => number | null
  coherence?: () => number | null
  activeDomain?: () => string | null
}

export function UnifiedSurface(props: UnifiedSurfaceProps) {
  const [showModeDetail, setShowModeDetail] = createSignal(false)

  // 键盘快捷键：⌘1/⌘2/⌘3 切换模式
  createEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key >= '1' && e.key <= '3') {
        const idx = parseInt(e.key) - 1
        if (MODES[idx]) {
          e.preventDefault()
          props.onModeChange(MODES[idx].id)
        }
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  })

  const currentMode = () => MODES.find(m => m.id === props.activeMode()) ?? MODES[0]

  return (
    <div class="relative">
      {/* 主模式切换器 — 三段分段控件 */}
      <div class="flex items-center gap-1 p-1 rounded-xl bg-white/40 border border-black/5 backdrop-blur-sm">
        <For each={MODES}>
          {(mode) => {
            const active = () => props.activeMode() === mode.id
            return (
              <button
                class={clsx(
                  'flex items-center gap-2 px-3 py-1.5 rounded-lg text-[12px] font-medium transition-all duration-200',
                  'focus-visible:ring-2 focus-visible:ring-orange-400 focus-visible:outline-none',
                  active()
                    ? `${mode.bgActive} shadow-sm`
                    : 'text-text-muted hover:text-text-primary hover:bg-white/60'
                )}
                onClick={() => props.onModeChange(mode.id)}
                aria-selected={active()}
                role="tab"
                title={`${mode.label} · ${mode.description}`}
              >
                <mode.icon class={clsx('w-3.5 h-3.5 flex-shrink-0', active() ? mode.color : 'text-text-muted')} />
                <span>{mode.label}</span>
              </button>
            )
          }}
        </For>

        {/* E8 意识状态微指示器 — NeoTrix 独有 */}
        <Show when={props.consciousnessPhi?.() !== null && props.consciousnessPhi?.() !== undefined}>
          <div class="ml-1 pl-1 border-l border-black/5 flex items-center gap-1">
            <div class={clsx(
              'w-2 h-2 rounded-full',
              (props.coherence?.() ?? 0) > 0.7 ? 'bg-emerald-400' :
              (props.coherence?.() ?? 0) > 0.4 ? 'bg-amber-400' : 'bg-red-400'
            )} title={`意识连贯度: ${((props.coherence?.() ?? 0) * 100).toFixed(0)}%`} />
            <Brain class="w-3 h-3 text-text-muted/50" />
          </div>
        </Show>
      </div>

      {/* 模式详情浮层（点击展开） */}
      <Show when={showModeDetail()}>
        <div class="absolute top-full left-0 right-0 mt-2 p-4 glass-pop border border-black/5 rounded-xl shadow-xl z-50">
          <div class="text-[11px] text-text-muted mb-2">当前模式: {currentMode().label}</div>
          <div class="text-[12px] text-text-secondary mb-3">{currentMode().description}</div>
          <div class="flex flex-wrap gap-1.5">
            <For each={currentMode().features}>
              {(f) => (
                <span class="px-2 py-0.5 rounded-full bg-white/60 text-[10px] text-text-muted border border-black/5">
                  {f}
                </span>
              )}
            </For>
          </div>
        </div>
      </Show>
    </div>
  )
}

/* ════════════════════════════════════════════════════════════
   AgentFleetView — Agent 指挥中心
   
   融合产品模式：
   - Cursor 3: Agents Window (统一 Agent 视图)
   - Devin Desktop: Agent Command Center (Kanban)
   - Claude: 并行会话管理
   
   NeoTrix 特有：
   - 意识状态与 Agent 绑定
   - 自治度可视化
   ════════════════════════════════════════════════════════════ */

export type AgentStatus = 'running' | 'paused' | 'completed' | 'error' | 'idle'

export interface AgentInfo {
  id: string
  name: string
  status: AgentStatus
  mode: SurfaceMode
  branch?: string
  progress?: number  // 0-100
  filesChanged: number
  toolCalls: number
  startedAt: Date
  lastActivity?: string
  phi?: number       // E8 consciousness metric
}

export interface AgentFleetViewProps {
  agents: () => AgentInfo[]
  onSelectAgent: (id: string) => void
  onPauseAgent: (id: string) => void
  onStopAgent: (id: string) => void
}

const STATUS_COLORS: Record<AgentStatus, string> = {
  running: 'bg-emerald-400',
  paused: 'bg-amber-400',
  completed: 'bg-blue-400',
  error: 'bg-red-400',
  idle: 'bg-zinc-300',
}

const STATUS_LABELS: Record<AgentStatus, string> = {
  running: '运行中',
  paused: '已暂停',
  completed: '已完成',
  error: '出错',
  idle: '空闲',
}

const MODE_ICONS: Record<SurfaceMode, any> = {
  chat: MessageSquare,
  work: Briefcase,
  code: Code2,
}

export function AgentFleetView(props: AgentFleetViewProps) {
  return (
    <div class="flex flex-col h-full">
      {/* 头部 */}
      <div class="flex items-center justify-between px-4 py-3 border-b border-border-primary/40">
        <div class="flex items-center gap-2">
          <Terminal class="w-4 h-4 text-text-muted" />
          <span class="text-[13px] font-medium text-text-primary">Agent 指挥中心</span>
          <span class="text-[11px] text-text-muted bg-white/60 px-1.5 py-0.5 rounded-full">
            {props.agents().length} 个 Agent
          </span>
        </div>
      </div>

      {/* Agent 列表 */}
      <div class="flex-1 overflow-y-auto p-3 space-y-2">
        <Show
          when={props.agents().length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-32 text-text-muted text-[12px]">
              <Terminal class="w-6 h-6 mb-2 opacity-30" />
              <span>暂无活跃 Agent</span>
              <span class="text-[11px] mt-1 opacity-60">发送消息或运行任务以启动</span>
            </div>
          }
        >
          <For each={props.agents()}>
            {(agent) => {
              const ModeIcon = MODE_ICONS[agent.mode]
              return (
                <div
                  class={clsx(
                    'p-3 rounded-xl border transition-all cursor-pointer',
                    agent.status === 'running'
                      ? 'border-emerald-200 bg-emerald-50/30 shadow-sm'
                      : agent.status === 'error'
                      ? 'border-red-200 bg-red-50/30'
                      : 'border-border-primary/40 bg-white/30 hover:bg-white/50'
                  )}
                  onClick={() => props.onSelectAgent(agent.id)}
                  role="button"
                  tabindex={0}
                >
                  {/* 状态行 */}
                  <div class="flex items-center gap-2 mb-2">
                    <div class={clsx('w-2 h-2 rounded-full', STATUS_COLORS[agent.status])} />
                    <span class="text-[12px] font-medium text-text-primary flex-1">{agent.name}</span>
                    <ModeIcon class="w-3.5 h-3.5 text-text-muted/50" />
                    <span class="text-[10px] text-text-muted">{STATUS_LABELS[agent.status]}</span>
                  </div>

                  {/* 分支信息 */}
                  <Show when={agent.branch}>
                    <div class="text-[10px] text-text-muted mb-2 font-mono">↗ {agent.branch}</div>
                  </Show>

                  {/* 进度条 */}
                  <Show when={agent.progress !== undefined && agent.status === 'running'}>
                    <div class="w-full h-1 bg-black/5 rounded-full mb-2 overflow-hidden">
                      <div
                        class="h-full bg-emerald-400 rounded-full transition-all duration-300"
                        style={{ width: `${agent.progress}%` }}
                      />
                    </div>
                  </Show>

                  {/* 统计行 */}
                  <div class="flex items-center gap-3 text-[10px] text-text-muted">
                    <span>{agent.filesChanged} 文件</span>
                    <span>{agent.toolCalls} 工具调用</span>
                    <Show when={agent.phi !== undefined}>
                      <span class="flex items-center gap-1">
                        <Brain class="w-3 h-3" />
                        φ {(agent.phi! * 100).toFixed(0)}%
                      </span>
                    </Show>
                    <span class="ml-auto">{formatRelativeTime(agent.startedAt)}</span>
                  </div>

                  {/* 操作按钮 */}
                  <Show when={agent.status === 'running' || agent.status === 'paused'}>
                    <div class="flex gap-2 mt-2">
                      <button
                        class="flex-1 px-2 py-1 rounded-lg text-[10px] bg-white/60 border border-black/5 hover:bg-white/80 transition-colors"
                        onClick={(e) => { e.stopPropagation(); props.onPauseAgent(agent.id) }}
                      >
                        {agent.status === 'running' ? '暂停' : '恢复'}
                      </button>
                      <button
                        class="px-2 py-1 rounded-lg text-[10px] text-red-500 bg-red-50 border border-red-200/50 hover:bg-red-100 transition-colors"
                        onClick={(e) => { e.stopPropagation(); props.onStopAgent(agent.id) }}
                      >
                        停止
                      </button>
                    </div>
                  </Show>
                </div>
              )
            }}
          </For>
        </Show>
      </div>
    </div>
  )
}

/* ════════════════════════════════════════════════════════════
   DiffZone — 内联代码变更 Diff 区
   
   融合产品模式：
   - Cursor: Inline diff view in chat
   - GitHub Copilot: Inline diffs in conversation
   - Claude Code: Visual diff review
   
   NeoTrix 特有：
   - 逐行接受/拒绝
   - 意识状态关联 (哪些变更由意识核心决策)
   ════════════════════════════════════════════════════════════ */

export interface DiffLine {
  type: 'add' | 'remove' | 'context'
  content: string
  oldLineNum?: number
  newLineNum?: number
}

export interface DiffHunk {
  filePath: string
  lines: DiffLine[]
  accepted?: boolean
}

export interface DiffZoneProps {
  hunks: () => DiffHunk[]
  onAcceptHunk: (filePath: string, index: number) => void
  onRejectHunk: (filePath: string, index: number) => void
  onAcceptAll: () => void
  onRejectAll: () => void
}

export function DiffZone(props: DiffZoneProps) {
  const allAccepted = () => props.hunks().every(h => h.accepted === true)
  const acceptedCount = () => props.hunks().filter(h => h.accepted === true).length

  return (
    <div class="border border-border-primary/40 rounded-xl overflow-hidden bg-white/30">
      {/* 头部 */}
      <div class="flex items-center justify-between px-3 py-2 bg-white/40 border-b border-border-primary/40">
        <div class="flex items-center gap-2">
          <Code2 class="w-3.5 h-3.5 text-text-muted" />
          <span class="text-[12px] font-medium text-text-primary">
            代码变更 ({props.hunks().length} 处)
          </span>
          <Show when={acceptedCount() > 0}>
            <span class="text-[10px] text-emerald-600 bg-emerald-50 px-1.5 py-0.5 rounded-full">
              {acceptedCount()}/{props.hunks().length} 已接受
            </span>
          </Show>
        </div>
        <div class="flex gap-1.5">
          <button
            class={clsx(
              'px-2 py-1 rounded-lg text-[10px] font-medium transition-colors',
              allAccepted()
                ? 'bg-emerald-100 text-emerald-600 cursor-default'
                : 'bg-emerald-500/10 text-emerald-600 hover:bg-emerald-500/20'
            )}
            onClick={props.onAcceptAll}
            disabled={allAccepted()}
          >
            全部接受
          </button>
          <button
            class="px-2 py-1 rounded-lg text-[10px] font-medium text-red-500 bg-red-50 hover:bg-red-100 transition-colors"
            onClick={props.onRejectAll}
          >
            全部拒绝
          </button>
        </div>
      </div>

      {/* Diff 内容 */}
      <div class="max-h-[300px] overflow-y-auto">
        <For each={props.hunks()}>
          {(hunk, i) => (
            <div class={clsx(
              'border-b border-border-primary/20 last:border-b-0',
              hunk.accepted === true && 'bg-emerald-50/20',
              hunk.accepted === false && 'bg-red-50/20'
            )}>
              {/* 文件路径 */}
              <div class="flex items-center justify-between px-3 py-1.5 bg-white/30">
                <span class="text-[11px] font-mono text-text-muted truncate">{hunk.filePath}</span>
                <Show when={hunk.accepted === undefined}>
                  <div class="flex gap-1">
                    <button
                      class="p-1 rounded text-emerald-500 hover:bg-emerald-50 transition-colors"
                      onClick={() => props.onAcceptHunk(hunk.filePath, i())}
                      title="接受此变更"
                    >
                      <svg viewBox="0 0 12 12" class="w-3.5 h-3.5"><path d="M2 6l3 3 5-6" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
                    </button>
                    <button
                      class="p-1 rounded text-red-500 hover:bg-red-50 transition-colors"
                      onClick={() => props.onRejectHunk(hunk.filePath, i())}
                      title="拒绝此变更"
                    >
                      <svg viewBox="0 0 12 12" class="w-3.5 h-3.5"><path d="M3 3l6 6M9 3l-6 6" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
                    </button>
                  </div>
                </Show>
                <Show when={hunk.accepted !== undefined}>
                  <span class={clsx(
                    'text-[10px] px-1.5 py-0.5 rounded-full',
                    hunk.accepted ? 'bg-emerald-100 text-emerald-600' : 'bg-red-100 text-red-600'
                  )}>
                    {hunk.accepted ? '已接受' : '已拒绝'}
                  </span>
                </Show>
              </div>

              {/* Diff 行 */}
              <div class="font-mono text-[11px] leading-5">
                <For each={hunk.lines}>
                  {(line) => (
                    <div class={clsx(
                      'flex px-3',
                      line.type === 'add' && 'bg-emerald-50 text-emerald-700',
                      line.type === 'remove' && 'bg-red-50 text-red-700',
                      line.type === 'context' && 'text-text-muted'
                    )}>
                      <span class="w-8 text-right pr-2 opacity-40 select-none flex-shrink-0">
                        {line.oldLineNum ?? ''}
                      </span>
                      <span class="w-8 text-right pr-2 opacity-40 select-none flex-shrink-0">
                        {line.newLineNum ?? ''}
                      </span>
                      <span class="flex-shrink-0 w-4 text-center opacity-50 select-none">
                        {line.type === 'add' ? '+' : line.type === 'remove' ? '-' : ' '}
                      </span>
                      <span class="flex-1 min-w-0 whitespace-pre">{line.content}</span>
                    </div>
                  )}
                </For>
              </div>
            </div>
          )}
        </For>
      </div>
    </div>
  )
}

/* ════════════════════════════════════════════════════════════
   ConsciousnessIndicator — E8 意识状态可视化
   
   NeoTrix 独有能力（竞品没有）：
   - Phi (φ) 意识度量
   - Coherence 连贯度
   - GWT 注意力焦点
   - SEAL 情感状态
   ════════════════════════════════════════════════════════════ */

export type EmotionState = 'confident' | 'focused' | 'uncertain' | 'struggling' | 'exploring'

const EMOTION_COLORS: Record<EmotionState, string> = {
  confident: 'text-emerald-500',
  focused: 'text-blue-500',
  uncertain: 'text-amber-500',
  struggling: 'text-red-500',
  exploring: 'text-purple-500',
}

const EMOTION_LABELS: Record<EmotionState, string> = {
  confident: '自信',
  focused: '专注',
  uncertain: '不确定',
  struggling: '困难',
  exploring: '探索',
}

export interface ConsciousnessIndicatorProps {
  phi: () => number | null
  coherence: () => number | null
  attentionFocus: () => string | null
  emotion: () => EmotionState
  compact?: boolean
}

export function ConsciousnessIndicator(props: ConsciousnessIndicatorProps) {
  const phi = () => props.phi() ?? 0
  const coherence = () => props.coherence() ?? 0

  if (props.compact) {
    return (
      <div class="flex items-center gap-2 text-[11px] text-text-muted">
        <div class={clsx('flex items-center gap-1', EMOTION_COLORS[props.emotion()])}>
          <div class={clsx(
            'w-1.5 h-1.5 rounded-full',
            props.emotion() === 'confident' ? 'bg-emerald-400' :
            props.emotion() === 'focused' ? 'bg-blue-400' :
            props.emotion() === 'uncertain' ? 'bg-amber-400' :
            props.emotion() === 'struggling' ? 'bg-red-400' : 'bg-purple-400'
          )} />
          <span>{EMOTION_LABELS[props.emotion()]}</span>
        </div>
        <Show when={phi() > 0}>
          <span class="opacity-50">φ {phi().toFixed(2)}</span>
        </Show>
      </div>
    )
  }

  return (
    <div class="p-3 rounded-xl bg-white/30 border border-border-primary/40">
      <div class="flex items-center gap-2 mb-3">
        <Brain class="w-4 h-4 text-text-muted" />
        <span class="text-[12px] font-medium text-text-primary">意识状态</span>
      </div>

      {/* Phi 进度条 */}
      <div class="mb-3">
        <div class="flex justify-between text-[10px] text-text-muted mb-1">
          <span>意识度 (φ)</span>
          <span>{(phi() * 100).toFixed(0)}%</span>
        </div>
        <div class="w-full h-1.5 bg-black/5 rounded-full overflow-hidden">
          <div
            class={clsx(
              'h-full rounded-full transition-all duration-500',
              phi() > 0.7 ? 'bg-emerald-400' :
              phi() > 0.4 ? 'bg-amber-400' : 'bg-red-400'
            )}
            style={{ width: `${phi() * 100}%` }}
          />
        </div>
      </div>

      {/* Coherence 进度条 */}
      <div class="mb-3">
        <div class="flex justify-between text-[10px] text-text-muted mb-1">
          <span>连贯度</span>
          <span>{(coherence() * 100).toFixed(0)}%</span>
        </div>
        <div class="w-full h-1.5 bg-black/5 rounded-full overflow-hidden">
          <div
            class={clsx(
              'h-full rounded-full transition-all duration-500',
              coherence() > 0.7 ? 'bg-blue-400' :
              coherence() > 0.4 ? 'bg-amber-400' : 'bg-red-400'
            )}
            style={{ width: `${coherence() * 100}%` }}
          />
        </div>
      </div>

      {/* 情感状态 */}
      <div class="flex items-center gap-2">
        <span class="text-[10px] text-text-muted">情感:</span>
        <span class={clsx('text-[11px] font-medium', EMOTION_COLORS[props.emotion()])}>
          {EMOTION_LABELS[props.emotion()]}
        </span>
      </div>

      {/* 注意力焦点 */}
      <Show when={props.attentionFocus()}>
        <div class="mt-2 flex items-center gap-2 text-[10px] text-text-muted">
          <Zap class="w-3 h-3" />
          <span class="truncate">焦点: {props.attentionFocus()}</span>
        </div>
      </Show>
    </div>
  )
}

// ── 工具函数 ──

function formatRelativeTime(date: Date): string {
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes}分钟前`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}小时前`
  return `${Math.floor(hours / 24)}天前`
}
