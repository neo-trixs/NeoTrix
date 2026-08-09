import { createSignal, onMount, createEffect, Show } from 'solid-js'
import { Coins, X, RefreshCw, Loader2, Cpu, Activity, Wallet, Repeat } from 'lucide-solid'
import { invoke } from '@tauri-apps/api/core'
import { clsx } from 'clsx'

interface AgentStatus {
  running: boolean
  current_task: string | null
  uptime_secs: number
  turn_count: number
  tokens_used: number
  context_usage: number
  provider_model: string
  evolution_iterations: number
  cost_spent: number
  cost_budget: number
}

interface Props {
  open: boolean
  onClose: () => void
}

export function CostDashboard(props: Props) {
  const [status, setStatus] = createSignal<AgentStatus | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  let firstBtnRef: HTMLButtonElement | undefined

  // 面板打开时聚焦首个按钮（对标 Codex 面板聚焦规范）
  createEffect(() => {
    if (props.open && firstBtnRef) firstBtnRef.focus()
  })

  const load = async () => {
    setLoading(true)
    setError(null)
    try {
      const s = await invoke<AgentStatus>('neocodex_agent_status')
      setStatus(s)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(load)

  const formatTokens = (n: number) => {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`
    return String(n)
  }

  const formatCost = (n: number) => `$${n.toFixed(4)}`

  const budgetPct = () => {
    const s = status()
    if (!s || s.cost_budget <= 0) return 0
    return Math.min(100, (s.cost_spent / s.cost_budget) * 100)
  }

  const contextPct = () => {
    const s = status()
    if (!s) return 0
    return Math.min(100, s.context_usage * 100)
  }

  const formatUptime = (secs: number) => {
    const h = Math.floor(secs / 3600)
    const m = Math.floor((secs % 3600) / 60)
    const s = secs % 60
    if (h > 0) return `${h}h ${m}m`
    if (m > 0) return `${m}m ${s}s`
    return `${s}s`
  }

  const renderStat = (icon: any, label: string, value: string, color: string, sub?: string) => (
    <div class="rounded-xl border border-border-primary bg-bg-primary/40 p-3">
      <div class="flex items-center gap-2 text-text-muted text-xs mb-1">
        {icon({ class: `w-3.5 h-3.5 ${color}` })}
        {label}
      </div>
      <div class="text-lg font-semibold text-text-primary">{value}</div>
      {sub && <div class="text-[10px] text-text-muted mt-0.5">{sub}</div>}
    </div>
  )

  return (
    <Show when={props.open}>
      <div class="panel w-[26rem]">
        {/* Header */}
        <div class="panel-head">
          <Coins class="panel-head-icon text-nt-memory-600" />
          <span class="panel-title">成本 / Token 看板</span>
          <Show when={status()}>
            <span class="panel-sub font-mono">{status()!.provider_model}</span>
          </Show>
          <button
            ref={firstBtnRef}
            class="panel-close"
            onClick={load}
            aria-label="刷新"
          >
            <RefreshCw class={clsx('w-4 h-4', loading() && 'animate-spin')} />
          </button>
          <button
            class="p-1.5 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
            onClick={props.onClose}
            aria-label="关闭"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div class="flex-1 overflow-y-auto p-4 space-y-4">
          <Show when={loading() && !status()}>
            <div class="flex items-center justify-center gap-2 py-8 text-text-muted text-sm">
              <Loader2 class="w-4 h-4 animate-spin" />
              加载...
            </div>
          </Show>
          <Show when={error()}>
            <div class="p-3 text-xs text-red-500 bg-red-500/10 rounded-lg">{error()}</div>
          </Show>
          <Show when={!loading() && !status() && !error()}>
            <div class="py-10 text-center text-xs text-text-muted">暂无成本数据</div>
          </Show>
          <Show when={status()}>
            <div class="grid grid-cols-2 gap-3">
              {renderStat(Wallet, '已花费', formatCost(status()!.cost_spent), 'text-nt-memory-600', `预算 ${formatCost(status()!.cost_budget)}`)}
              {renderStat(Cpu, 'Token 用量', formatTokens(status()!.tokens_used), 'text-nt-core-700', `${status()!.turn_count} 轮`)}
              {renderStat(Activity, '上下文占用', `${contextPct().toFixed(0)}%`, 'text-nt-mind-600', 'context_usage')}
              {renderStat(Repeat, '进化迭代', String(status()!.evolution_iterations), 'text-nt-repair-700', `运行 ${formatUptime(status()!.uptime_secs)}`)}
            </div>

            {/* Budget bar */}
            <div class="rounded-xl border border-border-primary bg-bg-primary/40 p-3">
              <div class="flex items-center justify-between text-xs mb-2">
                <span class="text-text-muted">预算使用</span>
                <span class="text-text-primary font-mono">{budgetPct().toFixed(1)}%</span>
              </div>
              <div class="h-2 rounded-full bg-bg-tertiary overflow-hidden">
                <div
                  class={clsx('h-full rounded-full transition-all', budgetPct() > 80 ? 'bg-red-500' : budgetPct() > 50 ? 'bg-amber-500' : 'bg-emerald-500')}
                  style={{ width: `${budgetPct()}%` }}
                />
              </div>
            </div>

            {/* Context bar */}
            <div class="rounded-xl border border-border-primary bg-bg-primary/40 p-3">
              <div class="flex items-center justify-between text-xs mb-2">
                <span class="text-text-muted">上下文窗口占用</span>
                <span class="text-text-primary font-mono">{contextPct().toFixed(1)}%</span>
              </div>
              <div class="h-2 rounded-full bg-bg-tertiary overflow-hidden">
                <div
                  class={clsx('h-full rounded-full transition-all', contextPct() > 80 ? 'bg-red-500' : contextPct() > 50 ? 'bg-amber-500' : 'bg-nt-core-400')}
                  style={{ width: `${contextPct()}%` }}
                />
              </div>
            </div>

            {/* Status */}
            <div class="rounded-xl border border-border-primary bg-bg-primary/40 p-3">
              <div class="flex items-center gap-2 text-sm">
                <span class={clsx('badge', status()!.running ? 'badge-warn' : 'badge-success')}>
                  {status()!.running ? '运行中' : '空闲'}
                </span>
                {status()!.current_task && <span class="text-xs text-text-muted truncate">{status()!.current_task}</span>}
              </div>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  )
}