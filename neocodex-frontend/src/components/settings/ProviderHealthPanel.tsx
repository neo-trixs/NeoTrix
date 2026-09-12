/**
 * ProviderHealthPanel — Provider 健康状态面板
 *
 * 展示各 LLM Provider 的电路状态、成功率、延迟、得分等健康指标。
 * 数据来源：provider_status Tauri command → ProviderHealthStatus[]
 */
import { createSignal, onMount, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import { providerStatus, type ProviderHealthStatus } from '../../api/neocodex'
import { InfoIcon } from './settingsIcons'

export function ProviderHealthPanel() {
  const [providers, setProviders] = createSignal<ProviderHealthStatus[]>([])
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  const fetchHealth = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await providerStatus()
      setProviders(result)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(fetchHealth)

  const parseScore = (s: string): number => {
    const n = parseFloat(s)
    return isNaN(n) ? 0 : n
  }

  const circuitColor = (state: string) => {
    switch (state) {
      case 'closed': return 'bg-emerald-500'
      case 'open': return 'bg-red-500'
      case 'half_open': return 'bg-amber-500'
      default: return 'bg-zinc-400'
    }
  }

  const circuitLabel = (state: string) => {
    switch (state) {
      case 'closed': return '正常'
      case 'open': return '熔断'
      case 'half_open': return '半开'
      default: return state
    }
  }

  return (
    <div class="ss-card">
      <div class="ss-card-header">
        <InfoIcon />
        Provider 健康状态
      </div>
      <div class="ss-card-body">
        <div class="flex items-center justify-between mb-4">
          <div class="text-sm text-text-secondary">
            电路状态 / 成功率 / 延迟 / 综合得分
          </div>
          <button
            class={clsx(
              'px-3 py-1.5 rounded-lg text-xs font-medium transition-colors',
              loading()
                ? 'bg-zinc-100 text-zinc-400 cursor-not-allowed'
                : 'bg-nt-io-500/10 text-nt-io-600 hover:bg-nt-io-500/20'
            )}
            onClick={fetchHealth}
            disabled={loading()}
          >
            {loading() ? '刷新中...' : '刷新'}
          </button>
        </div>

        <Show when={error()}>
          <div class="p-3 rounded-lg bg-red-50/50 text-red-600 text-xs mb-3">
            {error()}
          </div>
        </Show>

        <Show when={providers().length === 0 && !loading() && !error()}>
          <div class="text-center text-text-muted text-xs py-6">
            暂无 Provider 数据
          </div>
        </Show>

        <For each={providers()}>
          {(p) => {
            const score = parseScore(p.composite_score)
            const successRate = parseScore(p.success_rate)
            return (
              <div class="flex items-center gap-3 p-2.5 rounded-lg border border-border-primary/40 hover:bg-bg-primary/40 transition-colors mb-1.5">
                <span class={clsx('w-2.5 h-2.5 rounded-full flex-shrink-0', circuitColor(p.circuit_state))} />
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-medium text-text-primary truncate">{p.name}</span>
                    <Show when={p.is_free}>
                      <span class="px-1.5 py-0.5 rounded text-[10px] font-medium bg-emerald-50/80 text-emerald-600 border border-emerald-200/60">
                        Free
                      </span>
                    </Show>
                    <span class={clsx(
                      'px-1.5 py-0.5 rounded text-[10px] font-medium',
                      p.available
                        ? 'bg-emerald-50/80 text-emerald-600 border border-emerald-200/60'
                        : 'bg-red-50/80 text-red-600 border border-red-200/60'
                    )}>
                      {p.available ? '可用' : '不可用'}
                    </span>
                  </div>
                  <div class="flex items-center gap-3 mt-1 text-[11px] text-text-muted">
                    <span>电路: {circuitLabel(p.circuit_state)}</span>
                    <span>成功率: {(successRate * 100).toFixed(1)}%</span>
                    <span>调用: {p.total_calls}</span>
                    <span>错误: {p.total_errors}</span>
                    <span>P95: {p.latency_p95_ms}ms</span>
                  </div>
                </div>
                <div class="text-right flex-shrink-0">
                  <div class="text-lg font-bold text-text-primary">{(score * 100).toFixed(1)}</div>
                  <div class="text-[10px] text-text-muted">综合得分</div>
                </div>
              </div>
            )
          }}
        </For>
      </div>
    </div>
  )
}
