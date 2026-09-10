/* ════════════════════════════════════════════
   components/settings/NetworkSection.tsx — 网络代理健康度
   代理 IP 池 + 系统代理 + 网络诊断
   ════════════════════════════════════════════ */
import { createSignal, For, Show, onMount, onCleanup } from 'solid-js'
import { clsx } from 'clsx'
import type { ProviderHealthStatus } from '../../api/neocodex'

/* ── 类型 ── */
interface ProxyNode {
  url: string
  tag: string
  latency_ms: number | null
  success_count: number
  fail_count: number
  geo_tag: string | null
  from_subscription: boolean
  speed_tier: string
}

interface ProxyPoolStatus {
  total: number
  healthy: number
  unhealthy: number
  strategy: string
  nodes: ProxyNode[]
  subscriptions: string[]
}

interface SystemProxyStatus {
  enabled: boolean
  os: string
  http_proxy: string | null
  https_proxy: string | null
  socks_proxy: string | null
  mode: string
}

interface NetworkDiagnostic {
  endpoint: string
  reachable: boolean
  latency_ms: number
  status_code: number
  region: string | null
  last_check: string
}

/* ── 工具函数 ── */
const statusBadge = (healthy: boolean) => {
  return healthy
    ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
    : 'bg-red-50 text-red-600 border-red-200'
}
const formatNum = (n: number) => n.toLocaleString()
const formatMs = (ms: number) => ms < 1000 ? `${Math.round(ms)}ms` : `${(ms / 1000).toFixed(1)}s`

/* ── 小组件 ── */
function MetricCard(props: { label: string; value: string | number; sub?: string; color?: string }) {
  return (
    <div class="text-center px-3">
      <div class={clsx('text-[16px] font-semibold leading-none', props.color ?? 'text-zinc-900')}>
        {props.value}
      </div>
      <div class="text-10px text-zinc-500 mt-1">{props.label}</div>
      <Show when={props.sub}>
        <div class="text-10px text-zinc-400">{props.sub}</div>
      </Show>
    </div>
  )
}

/* ════════════════════════════════════════
   主组件
   ════════════════════════════════════════ */
export function NetworkSection() {
  /* ── 状态 ── */
  const [proxyStatus, setProxyStatus] = createSignal<ProxyPoolStatus | null>(null)
  const [systemProxy, setSystemProxy] = createSignal<SystemProxyStatus | null>(null)
  const [providerHealth, setProviderHealth] = createSignal<ProviderHealthStatus[]>([])
  const [loading, setLoading] = createSignal(true)
  const [activeTab, setActiveTab] = createSignal<'proxy' | 'system' | 'providers'>('proxy')
  const [error, setError] = createSignal<string | null>(null)

  /* ── 数据获取 ── */
  const fetchAll = async () => {
    setLoading(true)
    setError(null)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const [status, health] = await Promise.all([
        invoke<ProxyPoolStatus>('proxy_pool_status').catch(() => null),
        invoke<ProviderHealthStatus[]>('provider_status').catch(() => []),
      ])
      setProxyStatus(status)
      setProviderHealth(health)

      // 系统代理状态（简单检测）
      const sysProxy: SystemProxyStatus = {
        enabled: false,
        os: navigator.platform,
        http_proxy: null,
        https_proxy: null,
        socks_proxy: null,
        mode: 'none',
      }
      setSystemProxy(sysProxy)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  let timer: ReturnType<typeof setInterval> | undefined
  onMount(() => { fetchAll(); timer = setInterval(fetchAll, 20_000) })
  onCleanup(() => { if (timer) clearInterval(timer) })

  /* ── 计算统计 ── */
  const nodes = () => proxyStatus()?.nodes ?? []
  const healthyNodes = () => nodes().filter((n) => n.fail_count < 3)
  const totalSuccess = () => nodes().reduce((s, n) => s + (n.success_count ?? 0), 0)
  const totalFails = () => nodes().reduce((s, n) => s + (n.fail_count ?? 0), 0)
  const avgLatency = () => {
    const withLatency = nodes().filter((n) => n.latency_ms !== null && n.latency_ms > 0)
    if (withLatency.length === 0) return 0
    return withLatency.reduce((s, n) => s + (n.latency_ms ?? 0), 0) / withLatency.length
  }

  return (
    <div class="space-y-4">
      {/* ════════════════════════════════════════
         代理池概览
         ════════════════════════════════════════ */}
      <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
        <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
          <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class="nt-ic">
            <circle cx="8" cy="8" r="1.4" fill="url(#nt-current-grad)" />
            <line x1="8" y1="4" x2="8" y2="0.8" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
            <line x1="8" y1="12" x2="8" y2="15.2" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
            <line x1="4" y1="8" x2="0.8" y2="8" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
            <line x1="12" y1="8" x2="15.2" y2="8" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
          </svg>
          代理 IP 池
          <div class="ml-auto flex items-center gap-1.5">
            <Show when={proxyStatus()}>
              {(status) => (
                <span class={clsx(
                  'text-10px px-2 py-0.5 rounded-full font-medium border',
                  status().healthy > 0 ? 'bg-emerald-50 text-emerald-700 border-emerald-200' : 'bg-amber-50 text-amber-700 border-amber-200'
                )}>
                  {status().healthy} 健康 / {status().total} 总计
                </span>
              )}
            </Show>
          </div>
        </div>

        <div class="ss-card-body bg-white">
          <Show when={!loading()} fallback={
            <div class="text-[11px] text-zinc-400 text-center py-3">加载中…</div>
          }>
            {/* 概览统计 */}
            <Show when={proxyStatus()}>
              {(status) => (
                <div class="flex items-center gap-4 mb-3 pb-3 border-b border-border-primary/30">
                  <MetricCard label="总节点" value={status().total} />
                  <div class="w-px h-8 bg-black/5" />
                  <MetricCard label="健康节点" value={status().healthy} color="text-emerald-600" />
                  <div class="w-px h-8 bg-black/5" />
                  <MetricCard label="异常节点" value={status().unhealthy} color={status().unhealthy > 0 ? 'text-red-600' : 'text-zinc-900'} />
                  <div class="w-px h-8 bg-black/5" />
                  <MetricCard label="平均延迟" value={formatMs(avgLatency())} />
                  <div class="ml-auto">
                    <span class="text-10px px-2 py-0.5 rounded-full font-medium border bg-zinc-50 text-zinc-600 border-zinc-200">
                      策略: {status().strategy}
                    </span>
                  </div>
                </div>
              )}
            </Show>

            {/* Tab 切换 */}
            <div class="flex items-center gap-1.5 mb-3">
              <button
                class={clsx(
                  'text-10px px-2.5 py-1 rounded-md border transition-colors',
                  activeTab() === 'proxy'
                    ? 'bg-nt-io-500/10 text-nt-io-600 border-nt-io-200'
                    : 'text-zinc-500 border-zinc-200 hover:bg-zinc-100'
                )}
                onClick={() => setActiveTab('proxy')}
              >代理节点</button>
              <button
                class={clsx(
                  'text-10px px-2.5 py-1 rounded-md border transition-colors',
                  activeTab() === 'system'
                    ? 'bg-nt-io-500/10 text-nt-io-600 border-nt-io-200'
                    : 'text-zinc-500 border-zinc-200 hover:bg-zinc-100'
                )}
                onClick={() => setActiveTab('system')}
              >系统代理</button>
              <button
                class={clsx(
                  'text-10px px-2.5 py-1 rounded-md border transition-colors',
                  activeTab() === 'providers'
                    ? 'bg-nt-io-500/10 text-nt-io-600 border-nt-io-200'
                    : 'text-zinc-500 border-zinc-200 hover:bg-zinc-100'
                )}
                onClick={() => setActiveTab('providers')}
              >LLM 提供商</button>
            </div>

            {/* 代理节点列表 */}
            <Show when={activeTab() === 'proxy'}>
              <Show when={nodes().length > 0} fallback={
                <div class="text-[11px] text-zinc-400 text-center py-4">
                  暂无代理节点。点击「添加节点」或配置订阅源。
                </div>
              }>
                <div class="grid grid-cols-1 gap-1.5 max-h-64 overflow-y-auto">
                  <For each={nodes()}>
                    {(node) => (
                      <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                        <span class={clsx(
                          'w-2 h-2 rounded-full flex-shrink-0',
                          node.fail_count < 3 ? 'bg-emerald-500' : 'bg-red-500'
                        )} />
                        <span class="font-medium text-text-primary truncate min-w-0 flex-1 font-mono text-[10px]">
                          {node.url.replace(/\/\/.*@/, '//***@')}
                        </span>
                        <span class="text-10px text-zinc-400 px-1">{node.tag}</span>
                        <Show when={node.geo_tag}>
                          <span class="text-10px text-zinc-400 px-1">{node.geo_tag}</span>
                        </Show>
                        <span class={clsx(
                          'px-1.5 py-0.5 rounded text-10px font-medium border',
                          statusBadge(node.fail_count < 3)
                        )}>
                          {node.fail_count < 3 ? '健康' : '异常'}
                        </span>
                        <Show when={node.latency_ms !== null && node.latency_ms > 0}>
                          <span class="text-zinc-400 font-mono w-12 text-right">{formatMs(node.latency_ms ?? 0)}</span>
                        </Show>
                        <span class="text-zinc-400 font-mono w-16 text-right">{node.success_count}/{node.fail_count}</span>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </Show>

            {/* 系统代理 */}
            <Show when={activeTab() === 'system'}>
              <Show when={systemProxy()}>
                {(sys) => (
                  <div class="space-y-2">
                    <div class="flex items-center gap-3 p-2.5 rounded-lg bg-zinc-50/60 text-[11px]">
                      <span class={clsx(
                        'w-2 h-2 rounded-full flex-shrink-0',
                        sys().enabled ? 'bg-emerald-500' : 'bg-zinc-400'
                      )} />
                      <span class="font-medium text-text-primary">系统代理状态</span>
                      <span class={clsx(
                        'px-1.5 py-0.5 rounded text-10px font-medium border ml-auto',
                        sys().enabled
                          ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                          : 'bg-zinc-50 text-zinc-500 border-zinc-200'
                      )}>
                        {sys().enabled ? '已启用' : '未启用'}
                      </span>
                    </div>
                    <div class="p-2.5 rounded-lg bg-zinc-50/60 text-[11px] space-y-1">
                      <div class="flex items-center gap-2">
                        <span class="text-zinc-500 w-20">操作系统:</span>
                        <span class="text-text-primary">{sys().os}</span>
                      </div>
                      <div class="flex items-center gap-2">
                        <span class="text-zinc-500 w-20">代理模式:</span>
                        <span class="text-text-primary">{sys().mode}</span>
                      </div>
                      <Show when={sys().http_proxy}>
                        <div class="flex items-center gap-2">
                          <span class="text-zinc-500 w-20">HTTP:</span>
                          <span class="text-text-primary font-mono">{sys().http_proxy}</span>
                        </div>
                      </Show>
                      <Show when={sys().https_proxy}>
                        <div class="flex items-center gap-2">
                          <span class="text-zinc-500 w-20">HTTPS:</span>
                          <span class="text-text-primary font-mono">{sys().https_proxy}</span>
                        </div>
                      </Show>
                    </div>
                  </div>
                )}
              </Show>
            </Show>

            {/* LLM 提供商 */}
            <Show when={activeTab() === 'providers'}>
              <Show when={providerHealth().length > 0} fallback={
                <div class="text-[11px] text-zinc-400 text-center py-4">暂无提供商数据</div>
              }>
                <div class="grid grid-cols-1 gap-1.5 max-h-64 overflow-y-auto">
                  <For each={providerHealth()}>
                    {(h) => (
                      <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                        <span class={clsx(
                          'w-2 h-2 rounded-full flex-shrink-0',
                          h.available ? 'bg-emerald-500' : 'bg-red-500'
                        )} />
                        <span class="font-medium text-text-primary truncate min-w-0 flex-1">{h.name}</span>
                        <span class="text-10px text-zinc-400 font-mono w-12 text-right">{h.category}</span>
                        <span class={clsx(
                          'px-1.5 py-0.5 rounded text-10px font-medium border',
                          h.available
                            ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                            : 'bg-red-50 text-red-600 border-red-200'
                        )}>
                          {h.available ? '可用' : '不可用'}
                        </span>
                        <span class="text-zinc-400 font-mono w-12 text-right">{(parseFloat(h.success_rate) * 100).toFixed(0)}%</span>
                        <span class="text-zinc-400 font-mono w-16 text-right">{formatNum(h.total_calls)}/{formatNum(h.total_errors)}</span>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </Show>
          </Show>
        </div>
      </div>

      {/* ════════════════════════════════════════
         订阅源
         ════════════════════════════════════════ */}
      <Show when={proxyStatus()?.subscriptions && (proxyStatus()?.subscriptions?.length ?? 0) > 0}>
        <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
          <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
            订阅源
            <span class="ml-auto text-10px text-zinc-500 font-mono">
              {proxyStatus()?.subscriptions?.length ?? 0} 个
            </span>
          </div>
          <div class="ss-card-body bg-white">
            <div class="space-y-1.5">
              <For each={proxyStatus()?.subscriptions ?? []}>
                {(sub) => (
                  <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                    <span class="w-2 h-2 rounded-full bg-emerald-500 flex-shrink-0" />
                    <span class="font-mono text-text-primary truncate min-w-0 flex-1">{sub}</span>
                    <span class="text-10px text-zinc-400">活跃</span>
                  </div>
                )}
              </For>
            </div>
          </div>
        </div>
      </Show>
    </div>
  )
}
