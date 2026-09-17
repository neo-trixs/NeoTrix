// ══════════════════════════════════════════════════════════════════════════
//  ContextDashboard — 上下文分析仪表板
//  对标：Cursor Context Usage Report + Claude Context Window + Windsurf Context
//  Token 用量分布 / 上下文效率 / 压缩建议 / 历史趋势
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, For, Show, createMemo } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { ArrowLeft, BarChart3, TrendingUp, Layers, Zap, AlertTriangle, CheckCircle } from 'lucide-solid'
import { clsx } from 'clsx'
import { chatStore } from '../stores/chat'

interface ContextSegment {
  label: string
  tokens: number
  color: string
  icon: string
}

interface ContextMetric {
  label: string
  value: string
  change?: number
  icon: string
}

interface CompressionSuggestion {
  id: string
  title: string
  description: string
  impact: 'high' | 'medium' | 'low'
  savings: number
}

/** Mock 后端数据 — 接线后替换 */
function useContextMetrics() {
  const maxTokens = 200000

  const segments = createMemo<ContextSegment[]>(() => {
    const sessions = chatStore.state.sessions
    let systemPrompt = 0
    let conversation = 0
    let toolResults = 0
    let codeBlocks = 0
    let attachments = 0

    for (const s of sessions.slice(0, 10)) {
      for (const m of s.messages) {
        if (m.role === 'system') systemPrompt += m.content.length / 4
        else if (m.role === 'user' || m.role === 'assistant') conversation += m.content.length / 4
        if (m.toolCalls) toolResults += m.toolCalls.length * 500
        if (m.content.includes('```')) codeBlocks += 2000
      }
    }

    attachments = sessions.reduce((sum, s) => sum + s.messages.filter(m => m.attachments?.length).length * 1500, 0)

    return [
      { label: '系统提示', tokens: Math.round(systemPrompt), color: '#6366f1', icon: '⚙️' },
      { label: '对话历史', tokens: Math.round(conversation), color: '#f0913a', icon: '💬' },
      { label: '工具结果', tokens: Math.round(toolResults), color: '#14b8a6', icon: '🔧' },
      { label: '代码块', tokens: Math.round(codeBlocks), color: '#8b5cf6', icon: '💻' },
      { label: '附件', tokens: Math.round(attachments), color: '#dc2626', icon: '📎' },
    ].filter(s => s.tokens > 0)
  })

  const totalUsed = createMemo(() => segments().reduce((sum, s) => sum + s.tokens, 0))
  const usagePct = createMemo(() => Math.round((totalUsed() / maxTokens) * 100))

  const metrics = createMemo<ContextMetric[]>(() => {
    const sessions = chatStore.state.sessions
    const totalSessions = sessions.length
    const avgMessagesPerSession = totalSessions > 0
      ? Math.round(sessions.reduce((sum, s) => sum + s.messages.length, 0) / totalSessions)
      : 0
    const avgTokensPerMessage = totalUsed() > 0 && avgMessagesPerSession > 0
      ? Math.round(totalUsed() / avgMessagesPerSession)
      : 0

    return [
      { label: '总 Token', value: totalUsed().toLocaleString(), icon: '📊' },
      { label: '使用率', value: `${usagePct()}%`, change: usagePct() > 80 ? -5 : 2, icon: '📈' },
      { label: '活跃会话', value: totalSessions.toString(), icon: '💬' },
      { label: '平均消息/会话', value: avgMessagesPerSession.toString(), icon: '📝' },
      { label: '平均 Token/消息', value: avgTokensPerMessage.toLocaleString(), icon: '🔤' },
    ]
  })

  const suggestions = createMemo<CompressionSuggestion[]>(() => {
    const items: CompressionSuggestion[] = []
    const pct = usagePct()

    if (pct > 70) {
      items.push({
        id: 'compress-old',
        title: '压缩旧对话',
        description: '超过 30 分钟的对话历史可摘要压缩，释放上下文空间',
        impact: 'high',
        savings: Math.round(totalUsed() * 0.3),
      })
    }

    const toolResults = segments().find(s => s.label === '工具结果')
    if (toolResults && toolResults.tokens > totalUsed() * 0.2) {
      items.push({
        id: 'trim-tools',
        title: '裁剪工具输出',
        description: '大型工具结果（如文件列表、搜索结果）可截断到关键部分',
        impact: 'medium',
        savings: Math.round(toolResults.tokens * 0.5),
      })
    }

    const codeBlocks = segments().find(s => s.label === '代码块')
    if (codeBlocks && codeBlocks.tokens > 5000) {
      items.push({
        id: 'dedup-code',
        title: '去重代码片段',
        description: '重复出现的代码块可合并，减少冗余上下文占用',
        impact: 'low',
        savings: Math.round(codeBlocks.tokens * 0.2),
      })
    }

    if (pct < 40) {
      items.push({
        id: 'expand-context',
        title: '上下文充足',
        description: '当前上下文使用率较低，可以加载更多相关文件或历史',
        impact: 'low',
        savings: 0,
      })
    }

    return items
  })

  return { segments, totalUsed, usagePct, metrics, suggestions, maxTokens }
}

export function ContextDashboard() {
  const navigate = useNavigate()
  const { segments, totalUsed, usagePct, metrics, suggestions, maxTokens } = useContextMetrics()
  const [tab, setTab] = createSignal<'overview' | 'segments' | 'suggestions'>('overview')

  const usageColor = createMemo(() => {
    const pct = usagePct()
    if (pct > 80) return 'text-red-500'
    if (pct > 60) return 'text-amber-500'
    return 'text-emerald-500'
  })

  const usageBarColor = createMemo(() => {
    const pct = usagePct()
    if (pct > 80) return 'bg-red-500'
    if (pct > 60) return 'bg-amber-500'
    return 'bg-emerald-500'
  })

  const impactColors: Record<string, string> = {
    high: 'bg-red-500/15 text-red-600 border-red-500/30',
    medium: 'bg-amber-500/15 text-amber-600 border-amber-500/30',
    low: 'bg-emerald-500/15 text-emerald-600 border-emerald-500/30',
  }

  return (
    <div class="h-screen flex flex-col bg-bg-primary">
      {/* Header */}
      <div class="flex items-center gap-3 px-5 h-12 border-b border-border-primary/40 shrink-0">
        <button
          class="flex items-center gap-1.5 text-13px text-text-muted hover:text-text-primary transition-colors"
          onClick={() => navigate('/chat')}
          aria-label="返回对话"
        >
          <ArrowLeft class="w-4 h-4" />
          对话
        </button>
        <h1 class="text-14px font-semibold flex items-center gap-1.5">
          <BarChart3 class="w-4 h-4 text-nt-io-600" />
          上下文仪表板
        </h1>
        <span class={clsx('text-11px font-medium', usageColor())}>{usagePct()}% 已用</span>
        <div class="flex-1" />
      </div>

      {/* Tabs */}
      <div class="flex items-center gap-1 px-5 py-2 border-b border-border-primary/20">
        {[
          { id: 'overview' as const, label: '概览', icon: '📊' },
          { id: 'segments' as const, label: '分布', icon: '🧩' },
          { id: 'suggestions' as const, label: '建议', icon: '💡' },
        ].map(t => (
          <button
            class={clsx(
              'px-3 py-1.5 rounded-lg text-xs font-medium transition-colors',
              tab() === t.id
                ? 'bg-nt-io-500/10 text-nt-io-600'
                : 'text-text-muted hover:text-text-primary hover:bg-white/60'
            )}
            onClick={() => setTab(t.id)}
          >
            {t.icon} {t.label}
          </button>
        ))}
      </div>

      <div class="flex-1 overflow-auto p-5">
        <Show when={tab() === 'overview'}>
          <div class="max-w-3xl mx-auto space-y-6">
            {/* 主用量环 */}
            <div class="flex items-center gap-6 p-5 rounded-xl bg-white/40 border border-border-primary/30">
              <div class="relative w-28 h-28">
                <svg class="w-full h-full -rotate-90" viewBox="0 0 120 120">
                  <circle cx="60" cy="60" r="52" fill="none" stroke="#e5e7eb" stroke-width="8" />
                  <circle
                    cx="60" cy="60" r="52"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="8"
                    stroke-linecap="round"
                    stroke-dasharray={`${usagePct() * 3.267} 326.7`}
                    class={usageColor()}
                  />
                </svg>
                <div class="absolute inset-0 flex flex-col items-center justify-center">
                  <span class={clsx('text-2xl font-bold', usageColor())}>{usagePct()}%</span>
                  <span class="text-10px text-text-muted">已用</span>
                </div>
              </div>
              <div class="flex-1 space-y-3">
                <div class="text-13px text-text-muted">
                  已使用 <span class="text-text-primary font-semibold">{totalUsed().toLocaleString()}</span> / {maxTokens.toLocaleString()} tokens
                </div>
                <div class="w-full h-2 rounded-full bg-gray-200 overflow-hidden">
                  <div class={clsx('h-full rounded-full transition-all', usageBarColor())} style={{ width: `${usagePct()}%` }} />
                </div>
                <div class="flex gap-4 text-11px text-text-muted">
                  <span>剩余 {(maxTokens - totalUsed()).toLocaleString()} tokens</span>
                  <span>约 {Math.round((maxTokens - totalUsed()) / 4)} 字符</span>
                </div>
              </div>
            </div>

            {/* 指标网格 */}
            <div class="grid grid-cols-5 gap-3">
              <For each={metrics()}>
                {(m) => (
                  <div class="p-3 rounded-xl bg-white/40 border border-border-primary/30 text-center">
                    <div class="text-lg mb-1">{m.icon}</div>
                    <div class="text-14px font-semibold text-text-primary">{m.value}</div>
                    <div class="text-10px text-text-muted mt-0.5">{m.label}</div>
                    <Show when={m.change !== undefined}>
                      <div class={clsx('text-10px mt-1', m.change! > 0 ? 'text-emerald-500' : 'text-red-500')}>
                        {m.change! > 0 ? '↑' : '↓'} {Math.abs(m.change!)}%
                      </div>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </div>
        </Show>

        <Show when={tab() === 'segments'}>
          <div class="max-w-3xl mx-auto space-y-4">
            <div class="p-4 rounded-xl bg-white/40 border border-border-primary/30">
              <div class="text-12px font-semibold text-text-muted mb-3">Token 分布</div>
              <div class="flex h-8 rounded-lg overflow-hidden">
                <For each={segments()}>
                  {(s) => (
                    <div
                      class="h-full transition-all relative group"
                      style={{
                        width: `${totalUsed() > 0 ? (s.tokens / totalUsed()) * 100 : 0}%`,
                        background: s.color,
                        'min-width': s.tokens > 0 ? '4px' : '0',
                      }}
                    >
                      <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 rounded bg-black/80 text-white text-10px whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                        {s.label}: {s.tokens.toLocaleString()} tokens ({totalUsed() > 0 ? Math.round((s.tokens / totalUsed()) * 100) : 0}%)
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </div>

            <div class="space-y-2">
              <For each={segments()}>
                {(s) => (
                  <div class="flex items-center gap-3 p-3 rounded-xl bg-white/40 border border-border-primary/30">
                    <span class="text-lg">{s.icon}</span>
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <span class="text-12px font-medium text-text-primary">{s.label}</span>
                        <span class="text-11px text-text-muted">{s.tokens.toLocaleString()} tokens</span>
                      </div>
                      <div class="w-full h-1.5 rounded-full bg-gray-200 mt-1.5 overflow-hidden">
                        <div
                          class="h-full rounded-full transition-all"
                          style={{ width: `${totalUsed() > 0 ? (s.tokens / totalUsed()) * 100 : 0}%`, background: s.color }}
                        />
                      </div>
                    </div>
                    <span class="text-12px font-semibold text-text-primary">
                      {totalUsed() > 0 ? Math.round((s.tokens / totalUsed()) * 100) : 0}%
                    </span>
                  </div>
                )}
              </For>
            </div>
          </div>
        </Show>

        <Show when={tab() === 'suggestions'}>
          <div class="max-w-3xl mx-auto space-y-3">
            <Show
              when={suggestions().length > 0}
              fallback={
                <div class="flex flex-col items-center justify-center py-16 text-text-muted">
                  <CheckCircle class="w-12 h-12 mb-3 text-emerald-400" />
                  <span class="text-13px">上下文使用健康，无需优化</span>
                </div>
              }
            >
              <For each={suggestions()}>
                {(s) => (
                  <div class="p-4 rounded-xl bg-white/40 border border-border-primary/30">
                    <div class="flex items-start gap-3">
                      <span class="text-lg mt-0.5">
                        {s.impact === 'high' ? '🔴' : s.impact === 'medium' ? '🟡' : '🟢'}
                      </span>
                      <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-2">
                          <span class="text-13px font-semibold text-text-primary">{s.title}</span>
                          <span class={clsx('text-10px px-1.5 py-0.5 rounded border', impactColors[s.impact])}>
                            {s.impact === 'high' ? '高影响' : s.impact === 'medium' ? '中影响' : '低影响'}
                          </span>
                        </div>
                        <p class="text-12px text-text-muted mt-1">{s.description}</p>
                        <Show when={s.savings > 0}>
                          <div class="text-11px text-emerald-600 mt-2">
                            预计节省 {s.savings.toLocaleString()} tokens
                          </div>
                        </Show>
                      </div>
                    </div>
                  </div>
                )}
              </For>
            </Show>
          </div>
        </Show>
      </div>
    </div>
  )
}
