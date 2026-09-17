// ══════════════════════════════════════════════════════════════════════════
//  Activity — 统一活动视图（对标 ChatGPT Activity View + Devin Command Center）
//  汇聚：Agent 进度 / 定时任务 / 最近对话 / 工具调用 / 画板更新
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, For, Show, createMemo } from 'solid-js'
import { clsx } from 'clsx'
import { chatStore } from '../stores/chat'
import { canvasStore } from '../stores/canvas'

type ActivityTab = 'all' | 'agents' | 'tasks' | 'recent' | 'canvas'

interface ActivityItem {
  id: string
  type: 'agent' | 'task' | 'message' | 'canvas' | 'tool'
  title: string
  detail?: string
  status: 'running' | 'completed' | 'failed' | 'pending'
  timestamp: number
  source?: string
}

function formatTime(ts: number): string {
  const d = new Date(ts)
  const now = Date.now()
  const diff = now - ts
  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`
  return d.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}

const STATUS_COLORS: Record<string, string> = {
  running: 'bg-amber-400',
  completed: 'bg-emerald-400',
  failed: 'bg-red-400',
  pending: 'bg-slate-300',
}

const TYPE_ICONS: Record<string, string> = {
  agent: '🤖',
  task: '⏰',
  message: '💬',
  canvas: '🎨',
  tool: '🔧',
}

export function Activity() {
  const [tab, setTab] = createSignal<ActivityTab>('all')

  // 从 chatStore 提取活动
  const chatActivity = (): ActivityItem[] => {
    const sessions = chatStore.state.sessions
    const items: ActivityItem[] = []
    for (const s of sessions.slice(0, 20)) {
      const lastMsg = s.messages[s.messages.length - 1]
      if (lastMsg) {
        items.push({
          id: `chat:${s.id}`,
          type: 'message',
          title: s.title || '新对话',
          detail: lastMsg.content.slice(0, 80) + (lastMsg.content.length > 80 ? '…' : ''),
          status: lastMsg.role === 'assistant' ? 'completed' : 'pending',
          timestamp: lastMsg.timestamp.getTime(),
          source: s.project,
        })
      }
      // 工具调用活动
      for (const m of s.messages) {
        if (m.toolCalls) {
          for (const tc of m.toolCalls) {
            items.push({
              id: `tool:${tc.id}`,
              type: 'tool',
              title: tc.name,
              detail: tc.success ? '✓ 完成' : '✗ 失败',
              status: tc.success ? 'completed' : 'failed',
              timestamp: m.timestamp.getTime(),
              source: tc.domain,
            })
          }
        }
      }
    }
    return items
  }

  // 从 canvasStore 提取活动
  const canvasActivity = (): ActivityItem[] => {
    return canvasStore.nodes.slice(0, 30).map(n => ({
      id: `canvas:${n.id}`,
      type: 'canvas' as const,
      title: n.title ?? n.kind,
      detail: n.source,
      status: 'completed' as const,
      timestamp: n.createdAt ?? Date.now(),
      source: n.source,
    }))
  }

  // 合并并排序
  const allActivity = createMemo(() => {
    const items = [...chatActivity(), ...canvasActivity()]
    return items.sort((a, b) => b.timestamp - a.timestamp)
  })

  const filtered = createMemo(() => {
    const t = tab()
    if (t === 'all') return allActivity()
    if (t === 'agents') return allActivity().filter(i => i.type === 'agent')
    if (t === 'tasks') return allActivity().filter(i => i.type === 'task')
    if (t === 'recent') return allActivity().filter(i => i.type === 'message')
    if (t === 'canvas') return allActivity().filter(i => i.type === 'canvas')
    return allActivity()
  })

  const TABS: { id: ActivityTab; label: string; icon: string }[] = [
    { id: 'all', label: '全部', icon: '📋' },
    { id: 'agents', label: 'Agent', icon: '🤖' },
    { id: 'tasks', label: '定时', icon: '⏰' },
    { id: 'recent', label: '对话', icon: '💬' },
    { id: 'canvas', label: '画板', icon: '🎨' },
  ]

  return (
    <div class="h-screen flex flex-col bg-bg-primary">
      {/* Header */}
      <div class="flex items-center gap-3 px-6 py-4 border-b border-border-primary/20">
        <h1 class="text-lg font-semibold text-text-primary">活动</h1>
        <span class="text-xs text-text-muted">{allActivity().length} 项</span>
      </div>

      {/* Tabs */}
      <div class="flex items-center gap-1 px-6 py-2 border-b border-border-primary/10">
        <For each={TABS}>
          {(t) => (
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
          )}
        </For>
      </div>

      {/* Activity List */}
      <div class="flex-1 overflow-auto px-6 py-3">
        <Show
          when={filtered().length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-64 text-text-muted">
              <span class="text-4xl mb-3">📭</span>
              <span class="text-sm">暂无活动</span>
            </div>
          }
        >
          <div class="space-y-2">
            <For each={filtered()}>
              {(item) => (
                <div class="flex items-start gap-3 p-3 rounded-lg hover:bg-white/60 transition-colors cursor-pointer">
                  <span class="text-lg mt-0.5">{TYPE_ICONS[item.type]}</span>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="text-sm font-medium text-text-primary truncate">{item.title}</span>
                      <span class={clsx('w-2 h-2 rounded-full flex-shrink-0', STATUS_COLORS[item.status])} />
                      <Show when={item.source}>
                        <span class="text-[10px] px-1.5 py-0.5 rounded bg-nt-io-500/10 text-nt-io-600">{item.source}</span>
                      </Show>
                    </div>
                    <Show when={item.detail}>
                      <p class="text-xs text-text-muted mt-0.5 truncate">{item.detail}</p>
                    </Show>
                  </div>
                  <span class="text-[10px] text-text-muted whitespace-nowrap">{formatTime(item.timestamp)}</span>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  )
}
