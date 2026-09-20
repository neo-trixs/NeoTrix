/**
 * SettingsHealthPanel — 设置页 API 连通性测试面板
 * 
 * 显示每个设置标签页的后端 API 连接状态和响应时间
 */
import { createSignal, onMount, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import { checkAllApis, type HealthCheckResult, type ApiHealthReport } from '../../lib/apiHealthCheck'
import { InfoIcon, CheckIcon, AlertCircleIcon } from './settingsIcons'
import { ProviderHealthPanel } from './ProviderHealthPanel'

interface Props {
  onClose?: () => void
}

/** 标签页 → API 映射 */
const TAB_API_MAP: Record<string, string[]> = {
  general: ['domain.list', 'model_pool.status'],
  models: ['model_pool.status', 'model_pool.list'],
  network: ['proxy_pool.status'],
  im: ['im.status', 'im.channels'],
  appearance: [],
  market: ['market.status'],
  data: [],
  tags: [],
  capabilities: ['domain.list'],
  about: ['domain.list'],
}

const STATUS_CONFIG = {
  ok: { color: 'bg-emerald-500', text: 'text-emerald-600', label: '正常' },
  error: { color: 'bg-red-500', text: 'text-red-600', label: '异常' },
  timeout: { color: 'bg-amber-500', text: 'text-amber-600', label: '超时' },
}

export function SettingsHealthPanel(props: Props) {
  const [report, setReport] = createSignal<ApiHealthReport | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [selectedTab, setSelectedTab] = createSignal<string | null>(null)

  const runCheck = async () => {
    setLoading(true)
    try {
      const result = await checkAllApis()
      setReport(result)
    } catch (e) {
      console.error('Health check failed:', e)
    } finally {
      setLoading(false)
    }
  }

  onMount(runCheck)

  const getTabStatus = (tab: string): 'ok' | 'error' | 'warning' => {
    if (!report()) return 'warning'
    const apis = TAB_API_MAP[tab] ?? []
    if (apis.length === 0) return 'ok'
    
    const tabResults = report()!.results.filter(r => 
      apis.some(api => r.name.startsWith(api.split('.')[0]))
    )
    
    if (tabResults.length === 0) return 'warning'
    if (tabResults.every(r => r.status === 'ok')) return 'ok'
    return 'error'
  }

  const getTabLatency = (tab: string): number => {
    if (!report()) return 0
    const apis = TAB_API_MAP[tab] ?? []
    const tabResults = report()!.results.filter(r => 
      apis.some(api => r.name.startsWith(api.split('.')[0]))
    )
    return Math.round(tabResults.reduce((sum, r) => sum + r.latencyMs, 0) / Math.max(tabResults.length, 1))
  }

  return (
    <div class="space-y-4">
      {/* 概览卡片 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <InfoIcon />
          API 连通性测试
        </div>
        <div class="ss-card-body">
          <div class="flex items-center justify-between mb-4">
            <div class="text-sm text-text-secondary">
              测试后端 API 连接状态和响应时间
            </div>
            <button
              class={clsx(
                'px-3 py-1.5 rounded-lg text-xs font-medium transition-colors',
                loading()
                  ? 'bg-zinc-100 text-zinc-400 cursor-not-allowed'
                  : 'bg-nt-io-500/10 text-nt-io-600 hover:bg-nt-io-500/20'
              )}
              onClick={runCheck}
              disabled={loading()}
            >
              {loading() ? '测试中...' : '重新测试'}
            </button>
          </div>

          <Show when={report()}>
            {(r) => (
              <div class="grid grid-cols-4 gap-3 mb-4">
                <div class="p-3 rounded-xl bg-bg-primary/40 text-center">
                  <div class="text-2xl font-bold text-text-primary">{r().summary.total}</div>
                  <div class="text-[10px] text-text-muted">总 API 数</div>
                </div>
                <div class="p-3 rounded-xl bg-emerald-50/50 text-center">
                  <div class="text-2xl font-bold text-emerald-600">{r().summary.ok}</div>
                  <div class="text-[10px] text-emerald-600">正常</div>
                </div>
                <div class="p-3 rounded-xl bg-red-50/50 text-center">
                  <div class="text-2xl font-bold text-red-600">{r().summary.error}</div>
                  <div class="text-[10px] text-red-600">异常</div>
                </div>
                <div class="p-3 rounded-xl bg-bg-primary/40 text-center">
                  <div class="text-2xl font-bold text-text-primary">{r().summary.avgLatencyMs}ms</div>
                  <div class="text-[10px] text-text-muted">平均延迟</div>
                </div>
              </div>
            )}
          </Show>
        </div>
      </div>

      {/* 各标签页状态 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <CheckIcon />
          设置标签页状态
        </div>
        <div class="ss-card-body">
          <div class="space-y-2">
            <For each={['general', 'models', 'network', 'im', 'appearance', 'market', 'data', 'tags', 'capabilities', 'about']}>
              {(tab) => {
                const status = getTabStatus(tab)
                const latency = getTabLatency(tab)
                const statusConfig = status === 'ok' 
                  ? STATUS_CONFIG.ok 
                  : status === 'error' 
                    ? STATUS_CONFIG.error 
                    : STATUS_CONFIG.timeout
                
                return (
                  <div
                    class={clsx(
                      'flex items-center gap-3 p-2.5 rounded-lg border transition-colors cursor-pointer',
                      selectedTab() === tab 
                        ? 'border-nt-io-300 bg-nt-io-50/50' 
                        : 'border-border-primary/40 hover:bg-bg-primary/40'
                    )}
                    onClick={() => setSelectedTab(selectedTab() === tab ? null : tab)}
                  >
                    <span class={clsx('w-2.5 h-2.5 rounded-full', statusConfig.color)} />
                    <span class="text-sm font-medium text-text-primary capitalize">{tab}</span>
                    <span class={clsx('text-[10px] ml-auto', statusConfig.text)}>
                      {statusConfig.label}
                    </span>
                    {latency > 0 && (
                      <span class="text-[10px] text-text-muted">{latency}ms</span>
                    )}
                  </div>
                )
              }}
            </For>
          </div>
        </div>
      </div>

      {/* 详细 API 结果 */}
      <Show when={report()}>
        {(r) => (
          <div class="ss-card">
            <div class="ss-card-header">
              <AlertCircleIcon />
              API 详细结果
            </div>
            <div class="ss-card-body">
              <div class="space-y-1.5 max-h-64 overflow-y-auto">
                <For each={r().results}>
                  {(result) => (
                    <div class="flex items-center gap-2 p-2 rounded-lg bg-bg-primary/40 text-xs">
                      <span class={clsx(
                        'w-2 h-2 rounded-full flex-shrink-0',
                        result.status === 'ok' ? 'bg-emerald-500' : 'bg-red-500'
                      )} />
                      <span class="font-mono text-text-primary truncate flex-1">{result.name}</span>
                      <span class="text-text-muted">{result.latencyMs}ms</span>
                      {result.error && (
                        <span class="text-red-500 truncate max-w-[200px]" title={result.error}>
                          {result.error}
                        </span>
                      )}
                    </div>
                  )}
                </For>
              </div>
            </div>
          </div>
        )}
      </Show>

      {/* Provider 健康状态 */}
      <ProviderHealthPanel />
    </div>
  )
}
