import { createSignal, For, Show, createMemo } from 'solid-js'
import { clsx } from 'clsx'
import { BarChart3, ChevronDown, ChevronRight, Zap, Brain, FileText, Wrench, Clock, AlertTriangle } from 'lucide-solid'

/* ════════════════════════════════════════════════════════════
   ContextUsageReport — Token 分解可视化
   
   融合产品模式：
   - Cursor 3: Context Usage Report (交互式 token 分解)
   - Claude: Context window 管理
   - OpenHands: Token budget 分配
   
   NeoTrix 特有：
   - E8 意识度量与 token 使用关联
   - Constellation 成熟度影响分配策略
   - GWT 注意力路由 token 分配
   ════════════════════════════════════════════════════════════ */

export type ContextCategory =
  | 'system_prompt'
  | 'rules'
  | 'skills'
  | 'tools'
  | 'conversation'
  | 'file_content'
  | 'knowledge_base'
  | 'other'

export interface ContextUsageItem {
  category: ContextCategory
  label: string
  tokens: number
  percentage: number
  breakdown?: { label: string; tokens: number }[]
  warning?: boolean  // approaching limit
}

export interface ContextUsageReportProps {
  items: () => ContextUsageItem[]
  totalTokens: () => number
  maxTokens: () => number
  model?: () => string
  phi?: () => number | null
}

const CATEGORY_CONFIG: Record<ContextCategory, { icon: any; color: string; bgColor: string }> = {
  system_prompt:  { icon: Brain,      color: 'text-purple-600', bgColor: 'bg-purple-500' },
  rules:          { icon: FileText,   color: 'text-blue-600',   bgColor: 'bg-blue-500' },
  skills:         { icon: Zap,        color: 'text-amber-600',  bgColor: 'bg-amber-500' },
  tools:          { icon: Wrench,     color: 'text-emerald-600', bgColor: 'bg-emerald-500' },
  conversation:   { icon: Clock,      color: 'text-orange-600', bgColor: 'bg-orange-500' },
  file_content:   { icon: FileText,   color: 'text-cyan-600',   bgColor: 'bg-cyan-500' },
  knowledge_base: { icon: BarChart3,  color: 'text-pink-600',   bgColor: 'bg-pink-500' },
  other:          { icon: AlertTriangle, color: 'text-zinc-500', bgColor: 'bg-zinc-400' },
}

export function ContextUsageReport(props: ContextUsageReportProps) {
  const [expandedItem, setExpandedItem] = createSignal<string | null>(null)
  const [showDetails, setShowDetails] = createSignal(false)

  const usagePercentage = () =>
    props.maxTokens() > 0 ? (props.totalTokens() / props.maxTokens()) * 100 : 0

  const usageColor = () => {
    const pct = usagePercentage()
    if (pct > 90) return 'text-red-500'
    if (pct > 70) return 'text-amber-500'
    return 'text-emerald-500'
  }

  const sortedItems = () =>
    [...props.items()].sort((a, b) => b.tokens - a.tokens)

  const toggleExpand = (category: string) => {
    setExpandedItem(prev => prev === category ? null : category)
  }

  return (
    <div class="flex flex-col h-full">
      {/* Header */}
      <div class="flex items-center justify-between px-4 py-3 border-b border-border-primary/40">
        <div class="flex items-center gap-2">
          <BarChart3 class="w-4 h-4 text-text-muted" />
          <span class="text-[13px] font-medium text-text-primary">上下文使用量</span>
        </div>
        <Show when={props.model?.()}>
          <span class="text-[10px] text-text-muted bg-white/60 px-1.5 py-0.5 rounded-full font-mono">
            {props.model?.() ?? ''}
          </span>
        </Show>
      </div>

      {/* Total usage summary */}
      <div class="px-4 py-3 border-b border-border-primary/20">
        <div class="flex items-center justify-between mb-2">
          <span class="text-[11px] text-text-muted">已使用</span>
          <span class={clsx('text-[13px] font-semibold', usageColor())}>
            {formatTokens(props.totalTokens())}
            <span class="text-[10px] font-normal text-text-muted"> / {formatTokens(props.maxTokens())}</span>
          </span>
        </div>

        {/* Usage bar */}
        <div class="w-full h-2 bg-black/5 rounded-full overflow-hidden">
          <div
            class={clsx(
              'h-full rounded-full transition-all duration-500',
              usagePercentage() > 90 ? 'bg-red-400' :
              usagePercentage() > 70 ? 'bg-amber-400' : 'bg-emerald-400'
            )}
            style={{ width: `${Math.min(usagePercentage(), 100)}%` }}
          />
        </div>

        <div class="flex justify-between mt-1">
          <span class={clsx('text-[10px]', usageColor())}>{usagePercentage().toFixed(1)}%</span>
          <span class="text-[10px] text-text-muted">
            剩余 {formatTokens(props.maxTokens() - props.totalTokens())}
          </span>
        </div>

        {/* E8 φ indicator */}
        <Show when={props.phi && props.phi() !== null && props.phi() !== undefined}>
          <div class="flex items-center gap-2 mt-2 pt-2 border-t border-black/5">
            <Brain class="w-3 h-3 text-purple-500" />
            <span class="text-[10px] text-text-muted">意识度 (φ):</span>
            <div class="flex-1 h-1 bg-black/5 rounded-full overflow-hidden">
              <div
                class="h-full bg-purple-400 rounded-full"
                style={{ width: `${((props.phi?.() ?? 0) * 100)}%` }}
              />
            </div>
            <span class="text-[10px] text-purple-600">{((props.phi?.() ?? 0) * 100).toFixed(0)}%</span>
          </div>
        </Show>
      </div>

      {/* Stacked bar visualization */}
      <div class="px-4 py-3 border-b border-border-primary/20">
        <div class="text-[10px] text-text-muted mb-2">分配概览</div>
        <div class="flex h-3 rounded-full overflow-hidden gap-0.5">
          <For each={sortedItems()}>
            {(item) => (
              <div
                class={clsx(
                  'h-full transition-all duration-300 first:rounded-l-full last:rounded-r-full',
                  CATEGORY_CONFIG[item.category].bgColor,
                  item.warning && 'animate-pulse'
                )}
                style={{ width: `${item.percentage}%` }}
                title={`${item.label}: ${formatTokens(item.tokens)} (${item.percentage.toFixed(1)}%)`}
              />
            )}
          </For>
        </div>
        {/* Legend */}
        <div class="flex flex-wrap gap-2 mt-2">
          <For each={sortedItems()}>
            {(item) => {
              const config = CATEGORY_CONFIG[item.category]
              return (
                <div class="flex items-center gap-1">
                  <div class={clsx('w-2 h-2 rounded-full', config.bgColor)} />
                  <span class="text-[9px] text-text-muted">{item.label}</span>
                </div>
              )
            }}
          </For>
        </div>
      </div>

      {/* Detailed breakdown */}
      <div class="flex-1 overflow-y-auto">
        <div class="px-4 py-2">
          <button
            class="flex items-center gap-1 text-[10px] text-text-muted hover:text-text-primary transition-colors"
            onClick={() => setShowDetails(!showDetails())}
          >
            {showDetails() ? <ChevronDown class="w-3 h-3" /> : <ChevronRight class="w-3 h-3" />}
            详细分解
          </button>
        </div>

        <Show when={showDetails()}>
          <div class="px-4 pb-3 space-y-1">
            <For each={sortedItems()}>
              {(item) => {
                const config = CATEGORY_CONFIG[item.category]
                const ConfigIcon = config.icon
                const expanded = () => expandedItem() === item.category
                return (
                  <div class={clsx(
                    'rounded-lg border transition-all',
                    item.warning ? 'border-amber-200/50 bg-amber-50/30' : 'border-black/5 bg-white/30'
                  )}>
                    <button
                      class="w-full flex items-center gap-2 px-3 py-2 text-left focus-visible:outline-none"
                      onClick={() => toggleExpand(item.category)}
                    >
                      <ConfigIcon class={clsx('w-3.5 h-3.5', config.color)} />
                      <span class="text-[11px] text-text-primary flex-1">{item.label}</span>
                      <span class="text-[10px] text-text-muted">{formatTokens(item.tokens)}</span>
                      <span class="text-[10px] text-text-muted w-12 text-right">{item.percentage.toFixed(1)}%</span>
                      <Show when={item.breakdown && item.breakdown.length > 0}>
                        <ChevronDown class={clsx(
                          'w-3 h-3 text-text-muted transition-transform',
                          expanded() && 'rotate-180'
                        )} />
                      </Show>
                    </button>

                    {/* Sub-breakdown */}
                    <Show when={expanded() && item.breakdown && item.breakdown.length > 0}>
                      <div class="px-3 pb-2 space-y-1">
                        <For each={item.breakdown!}>
                          {(sub) => (
                            <div class="flex items-center gap-2 pl-6 text-[10px] text-text-muted">
                              <span class="flex-1 truncate">{sub.label}</span>
                              <span>{formatTokens(sub.tokens)}</span>
                            </div>
                          )}
                        </For>
                      </div>
                    </Show>
                  </div>
                )
              }}
            </For>
          </div>
        </Show>
      </div>
    </div>
  )
}

/* ── Helpers ── */

function formatTokens(tokens: number): string {
  if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toFixed(1)}M`
  if (tokens >= 1_000) return `${(tokens / 1_000).toFixed(1)}K`
  return tokens.toString()
}
