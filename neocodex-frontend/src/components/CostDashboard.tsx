import { createSignal, onMount, createEffect, Show } from 'solid-js'
import { Coins, X, RefreshCw, Loader2, Cpu, Activity, Wallet, Repeat } from 'lucide-solid'
import { neocodex } from '../api'
import type { AgentStatus } from '../api/types'
import { clsx } from 'clsx'

interface Props {
  open: boolean
  onClose: () => void
}

export function CostDashboard(props: Props) {
  const [status, setStatus] = createSignal<AgentStatus | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  // 轮询连续失败提示（静默吞错修复：失败达阈值时给轻量提示）
  const [pollError, setPollError] = createSignal<string | null>(null)
  let pollFailures = 0
  let firstBtnRef: HTMLButtonElement | undefined
  let panelRef: HTMLDivElement | undefined
  // 打开面板前的触发元素，关闭后还原焦点
  let lastFocusedEl: HTMLElement | null = null

  // 面板打开时聚焦首个按钮（对标 Codex 面板聚焦规范），并记录触发元素；
  // 关闭时（Esc/关闭按钮/遮罩点击触发卸载）经 effect 清理还原焦点
  createEffect(() => {
    if (!props.open) return
    lastFocusedEl = document.activeElement as HTMLElement | null
    const raf = requestAnimationFrame(() => {
      if (firstBtnRef) firstBtnRef.focus()
      else panelRef?.focus()
    })
    return () => {
      cancelAnimationFrame(raf)
      if (lastFocusedEl?.isConnected) lastFocusedEl.focus()
    }
  })

  // Esc 关闭（参照 SettingsModal 浮层关闭模式：window keydown + 打开时挂载；
  // 焦点在面板内时由容器 onKeyDown 兜底处理）
  createEffect(() => {
    if (!props.open) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return
      e.preventDefault()
      props.onClose()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  })

  const load = async () => {
    setLoading(true)
    setError(null)
    try {
      const s = await neocodex.agentStatus()
      setStatus(s)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(load)

  // 成本看板自动刷新：面板打开期间每 5s 轮询（对标 Claude usage 实时面板）。
  // 连续失败不再静默：达 2 次时显示轻量提示；单次失败不影响已有数据。
  createEffect(() => {
    if (!props.open) return
    const timer = setInterval(() => {
      neocodex
        .agentStatus()
        .then((s) => {
          pollFailures = 0
          setPollError(null)
          setStatus(s)
        })
        .catch(() => {
          pollFailures += 1
          if (pollFailures >= 2) setPollError('自动刷新失败，数据可能已过期')
        })
    }, 5000)
    return () => clearInterval(timer)
  })

  const formatTokens = (n: number) => {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`
    return String(n)
  }

  const formatCost = (n: number) => {
    if (n === 0) return '$0'
    // 极小金额固定 4 位小数会退化为 $0.0000：改用科学计数保留有效信息
    if (n < 0.0001) return `$${n.toExponential(2)}`
    return `$${n.toFixed(4)}`
  }

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
      {sub && <div class="text-[10px] text-text-muted mt-1">{sub}</div>}
    </div>
  )

  return (
    <Show when={props.open}>
      <div
        ref={panelRef}
        class="panel w-[26rem]"
        role="dialog"
        aria-label="成本 / Token 看板"
        aria-modal="true"
        tabIndex={-1}
        onKeyDown={(e) => {
          if (e.key === 'Escape') {
            e.preventDefault()
            props.onClose()
            return
          }
          if (e.key === 'Tab' && panelRef) {
            const focusables = panelRef.querySelectorAll<HTMLElement>(
              'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
            )
            if (focusables.length === 0) return
            const first = focusables[0]
            const last = focusables[focusables.length - 1]
            const active = document.activeElement
            if (e.shiftKey && (active === first || active === panelRef)) {
              e.preventDefault()
              last.focus()
            } else if (!e.shiftKey && active === last) {
              e.preventDefault()
              first.focus()
            }
          }
        }}
      >
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
            class="p-2 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
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
          <Show when={pollError()}>
            <div class="p-2 text-xs text-amber-600 bg-amber-500/10 rounded-lg" role="status">{pollError()}</div>
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