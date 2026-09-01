/* ════════════════════════════════════════════
   components/settings/NetworkSection.tsx — 网络代理健康度
   内部代理（StealthNet）+ 外部网络（LLM Provider）交互可视化
   ════════════════════════════════════════════ */
import { createSignal, For, Show, onMount, onCleanup } from 'solid-js'
import { clsx } from 'clsx'
import type { ProviderHealthStatus, ProbeResult } from '../../api/neocodex'

/* ── 类型 ── */
interface ProxyNode {
  url: string
  tag: string
  latency_ms: number | null
  success_count: number
  fail_count: number
  geo_tag: string | null
  from_subscription: boolean
}

interface HeartbeatRecord {
  tick: number
  timestamp: string
  proxy_url: string
  proxy_geo: string | null
  proxy_latency_ms: number
  fingerprint_id: number
  success: boolean
}

interface SystemProxyStatus {
  enabled: boolean
  os: string
  http_proxy: string | null
  https_proxy: string | null
  socks_proxy: string | null
  mode: string
}

interface SubscriptionSource {
  url: string
  last_fetch: string | null
  node_count: number
  status: 'active' | 'stale' | 'error'
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
const statusColor = (s: string) => {
  switch (s) {
    case 'active': return 'bg-emerald-500'
    case 'stale': return 'bg-amber-500'
    case 'error': return 'bg-red-500'
    default: return 'bg-zinc-400'
  }
}
const statusBadge = (s: string) => {
  switch (s) {
    case 'active': return 'bg-emerald-50 text-emerald-700 border-emerald-200'
    case 'stale': return 'bg-amber-50 text-amber-700 border-amber-200'
    case 'error': return 'bg-red-50 text-red-600 border-red-200'
    default: return 'bg-zinc-50 text-zinc-500 border-zinc-200'
  }
}
const formatNum = (n: number) => n.toLocaleString()
const formatMs = (ms: number) => ms < 1000 ? `${Math.round(ms)}ms` : `${(ms / 1000).toFixed(1)}s`

/* ── 小组件 ── */
function FlowArrow() {
  return (
    <div class="flex items-center justify-center py-1">
      <svg width="24" height="16" viewBox="0 0 24 16" class="text-zinc-300">
        <path d="M4 8 L18 8 M14 4 L18 8 L14 12" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </div>
  )
}

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
  const [proxyNodes, setProxyNodes] = createSignal<ProxyNode[]>([])
  const [heartbeat, setHeartbeat] = createSignal<HeartbeatRecord[]>([])
  const [systemProxy, setSystemProxy] = createSignal<SystemProxyStatus | null>(null)
  const [subscriptions, setSubscriptions] = createSignal<SubscriptionSource[]>([])
  const [diagnostics, setDiagnostics] = createSignal<NetworkDiagnostic[]>([])
  const [providerHealth, setProviderHealth] = createSignal<ProviderHealthStatus[]>([])
  const [loading, setLoading] = createSignal(true)
  const [activeTab, setActiveTab] = createSignal<'proxy' | 'system' | 'flow'>('proxy')
  const [error, setError] = createSignal<string | null>(null)

  /* ── 数据获取 ── */
  const fetchAll = async () => {
    setLoading(true)
    setError(null)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const [nodes, hb, sys, subs, diag, health] = await Promise.all([
        invoke<ProxyNode[]>('stealth_proxy_pool_status').catch(() => []),
        invoke<HeartbeatRecord[]>('stealth_heartbeat_history').catch(() => []),
        invoke<SystemProxyStatus>('stealth_system_proxy_status').catch(() => null),
        invoke<SubscriptionSource[]>('stealth_subscription_list').catch(() => []),
        invoke<NetworkDiagnostic[]>('stealth_network_diagnostics').catch(() => []),
        invoke<ProviderHealthStatus[]>('provider_status').catch(() => []),
      ])
      setProxyNodes(nodes)
      setHeartbeat(hb)
      setSystemProxy(sys)
      setSubscriptions(subs)
      setDiagnostics(diag)
      setProviderHealth(health)
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
  const activeNodes = () => proxyNodes().filter((n) => !n.from_subscription || n.success_count > 0)
  const healthyNodes = () => activeNodes().filter((n) => (n.fail_count ?? 0) < 3)
  const totalSuccess = () => proxyNodes().reduce((s, n) => s + (n.success_count ?? 0), 0)
  const totalFails = () => proxyNodes().reduce((s, n) => s + (n.fail_count ?? 0), 0)
  const avgLatency = () => {
    const withLatency = proxyNodes().filter((n) => n.latency_ms != null && n.latency_ms > 0)
    if (withLatency.length === 0) return 0
    return withLatency.reduce((s, n) => s + (n.latency_ms ?? 0), 0) / withLatency.length
  }

  return (
    <div class="space-y-4">
      {/* ── 顶部概览卡片 ── */}
      <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
        <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
          <svg class="w-4 h-4 text-nt-io-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/>
          </svg>
          网络代理健康度
          <Show when={!loading()}>
            <button
              class="ml-auto text-10px text-zinc-400 hover:text-zinc-600 transition-colors"
              onClick={fetchAll}
            >刷新</button>
          </Show>
        </div>
        <div class="ss-card-body bg-white">
          <Show when={!loading()} fallback={
            <div class="text-[11px] text-zinc-400 text-center py-3">加载中…</div>
          }>
            <Show when={error()}>
              <div class="text-[11px] text-red-500 text-center py-2 mb-2">{error()}</div>
            </Show>
            <div class="flex items-center justify-around">
              <MetricCard
                label="代理节点"
                value={activeNodes().length}
                sub={`${healthyNodes().length} 健康`}
                color="text-emerald-600"
              />
              <div class="w-px h-8 bg-black/5" />
              <MetricCard
                label="总成功率"
                value={`${totalSuccess() + totalFails() > 0 ? Math.round(totalSuccess() / (totalSuccess() + totalFails()) * 100) : 0}%`}
                sub={`${formatNum(totalSuccess())} 成功`}
                color="text-zinc-900"
              />
              <div class="w-px h-8 bg-black/5" />
              <MetricCard
                label="平均延迟"
                value={avgLatency() > 0 ? formatMs(avgLatency()) : '—'}
                color="text-zinc-900"
              />
              <div class="w-px h-8 bg-black/5" />
              <MetricCard
                label="心跳轮次"
                value={heartbeat().length}
                sub={heartbeat().length > 0 ? `${heartbeat()[heartbeat().length - 1].tick}` : ''}
                color="text-zinc-900"
              />
            </div>
          </Show>
        </div>
      </div>

      {/* ── Tab 导航 ── */}
      <div class="flex gap-1 border-b border-black/5">
        {(['proxy', 'system', 'flow'] as const).map((tab) => (
          <button
            class={clsx(
              'px-3 py-1.5 text-[11px] font-medium rounded-t-lg transition-colors -mb-px',
              activeTab() === tab
                ? 'text-nt-io-600 border-b-2 border-nt-io-500 bg-white'
                : 'text-zinc-500 hover:text-zinc-700'
            )}
            onClick={() => setActiveTab(tab)}
          >
            {tab === 'proxy' ? '代理池' : tab === 'system' ? '系统代理' : '流量拓扑'}
          </button>
        ))}
      </div>

      {/* ════════════════════════════════════════
         代理池 Tab
         ════════════════════════════════════════ */}
      <Show when={activeTab() === 'proxy'}>
        {/* 订阅源 */}
        <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
          <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
            订阅源
            <span class="ml-auto text-10px text-zinc-500 font-mono">{subscriptions().length} 源</span>
          </div>
          <div class="ss-card-body bg-white">
            <Show when={subscriptions().length > 0} fallback={
              <div class="text-[11px] text-zinc-400 text-center py-2">暂无订阅源</div>
            }>
              <div class="grid grid-cols-1 gap-1.5">
                <For each={subscriptions()}>
                  {(sub) => (
                    <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                      <span class={clsx('w-2 h-2 rounded-full flex-shrink-0', statusColor(sub.status))} />
                      <span class="font-medium text-text-primary truncate min-w-0 flex-1 font-mono text-10px">{sub.url}</span>
                      <span class={clsx('px-1.5 py-0.5 rounded text-10px font-medium border', statusBadge(sub.status))}>
                        {sub.status}
                      </span>
                      <span class="text-zinc-400 font-mono">{sub.node_count} 节点</span>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </div>

        {/* 代理节点列表 */}
        <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
          <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
            代理节点
            <span class="ml-auto text-10px text-zinc-500 font-mono">{activeNodes().length} 可用</span>
          </div>
          <div class="ss-card-body bg-white">
            <Show when={activeNodes().length > 0} fallback={
              <div class="text-[11px] text-zinc-400 text-center py-2">暂无代理节点</div>
            }>
              <div class="grid grid-cols-1 gap-1.5 max-h-48 overflow-y-auto">
                <For each={activeNodes().slice(0, 30)}>
                  {(node) => {
                    const failRate = node.success_count + node.fail_count > 0
                      ? node.fail_count / (node.success_count + node.fail_count)
                      : 0
                    const healthColor = failRate < 0.1 ? 'bg-emerald-500' : failRate < 0.3 ? 'bg-amber-500' : 'bg-red-500'
                    return (
                      <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                        <span class={clsx('w-2 h-2 rounded-full flex-shrink-0', healthColor)} />
                        <span class="font-medium text-text-primary truncate min-w-0 flex-1">{node.tag}</span>
                        <Show when={node.geo_tag}>
                          <span class="text-10px text-zinc-400">{node.geo_tag}</span>
                        </Show>
                        <span class="text-zinc-400 font-mono w-12 text-right">
                          {node.latency_ms != null ? `${Math.round(node.latency_ms)}ms` : '—'}
                        </span>
                        <span class="text-zinc-400 font-mono w-16 text-right">
                          {formatNum(node.success_count)}/{formatNum(node.fail_count)}
                        </span>
                      </div>
                    )
                  }}
                </For>
              </div>
            </Show>
          </div>
        </div>

        {/* 心跳记录 */}
        <Show when={heartbeat().length > 0}>
          <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
            <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
              心跳轮转
              <span class="ml-auto text-10px text-zinc-500 font-mono">最近 {Math.min(heartbeat().length, 10)} 轮</span>
            </div>
            <div class="ss-card-body bg-white">
              <div class="grid grid-cols-1 gap-1.5 max-h-32 overflow-y-auto">
                <For each={heartbeat().slice(-10).reverse()}>
                  {(hb) => (
                    <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                      <span class={clsx('w-2 h-2 rounded-full flex-shrink-0', hb.success ? 'bg-emerald-500' : 'bg-red-500')} />
                      <span class="text-zinc-400 font-mono">#{hb.tick}</span>
                      <span class="font-medium text-text-primary truncate min-w-0 flex-1">{hb.proxy_url}</span>
                      <Show when={hb.proxy_geo}>
                        <span class="text-10px text-zinc-400">{hb.proxy_geo}</span>
                      </Show>
                      <span class="text-zinc-400 font-mono">{Math.round(hb.proxy_latency_ms)}ms</span>
                      <span class="text-10px text-zinc-400">FP:{hb.fingerprint_id}</span>
                    </div>
                  )}
                </For>
              </div>
            </div>
          </div>
        </Show>
      </Show>

      {/* ════════════════════════════════════════
         系统代理 Tab
         ════════════════════════════════════════ */}
      <Show when={activeTab() === 'system'}>
        <Show when={systemProxy()}>
          {(sp) => (
            <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
              <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
                系统代理配置
              </div>
              <div class="ss-card-body bg-white space-y-3">
                <div class="flex items-center gap-3">
                  <span class={clsx(
                    'w-3 h-3 rounded-full flex-shrink-0',
                    sp().enabled ? 'bg-emerald-500' : 'bg-zinc-300'
                  )} />
                  <span class="text-[12px] font-medium text-text-primary">
                    {sp().enabled ? '已启用' : '未启用'}
                  </span>
                  <span class="text-10px text-zinc-500 ml-auto">{sp().os}</span>
                </div>

                <div class="grid grid-cols-1 gap-2">
                  <Show when={sp().http_proxy}>
                    <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                      <span class="text-zinc-500 font-mono w-16">HTTP</span>
                      <span class="font-mono text-text-primary truncate flex-1">{sp().http_proxy}</span>
                    </div>
                  </Show>
                  <Show when={sp().https_proxy}>
                    <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                      <span class="text-zinc-500 font-mono w-16">HTTPS</span>
                      <span class="font-mono text-text-primary truncate flex-1">{sp().https_proxy}</span>
                    </div>
                  </Show>
                  <Show when={sp().socks_proxy}>
                    <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                      <span class="text-zinc-500 font-mono w-16">SOCKS</span>
                      <span class="font-mono text-text-primary truncate flex-1">{sp().socks_proxy}</span>
                    </div>
                  </Show>
                </div>

                <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                  <span class="text-zinc-500">模式</span>
                  <span class={clsx(
                    'px-1.5 py-0.5 rounded text-10px font-medium border',
                    sp().mode === 'stealth' ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                      : sp().mode === 'tor' ? 'bg-purple-50 text-purple-700 border-purple-200'
                      : sp().mode === 'geo' ? 'bg-blue-50 text-blue-700 border-blue-200'
                      : 'bg-zinc-50 text-zinc-500 border-zinc-200'
                  )}>
                    {sp().mode}
                  </span>
                </div>
              </div>
            </div>
          )}
        </Show>

        {/* 网络诊断 */}
        <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
          <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
            外部网络探测
            <span class="ml-auto text-10px text-zinc-500 font-mono">{diagnostics().length} 端点</span>
          </div>
          <div class="ss-card-body bg-white">
            <Show when={diagnostics().length > 0} fallback={
              <div class="text-[11px] text-zinc-400 text-center py-2">暂无诊断数据</div>
            }>
              <div class="grid grid-cols-1 gap-1.5 max-h-48 overflow-y-auto">
                <For each={diagnostics()}>
                  {(d) => (
                    <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                      <span class={clsx('w-2 h-2 rounded-full flex-shrink-0', d.reachable ? 'bg-emerald-500' : 'bg-red-500')} />
                      <span class="font-medium text-text-primary truncate min-w-0 flex-1 font-mono text-10px">{d.endpoint}</span>
                      <Show when={d.region}>
                        <span class="text-10px text-zinc-400">{d.region}</span>
                      </Show>
                      <span class="text-zinc-400 font-mono w-12 text-right">{formatMs(d.latency_ms)}</span>
                      <span class={clsx(
                        'px-1.5 py-0.5 rounded text-10px font-medium border',
                        d.reachable ? 'bg-emerald-50 text-emerald-700 border-emerald-200' : 'bg-red-50 text-red-600 border-red-200'
                      )}>
                        {d.reachable ? `HTTP ${d.status_code}` : '不可达'}
                      </span>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </div>
      </Show>

      {/* ════════════════════════════════════════
         流量拓扑 Tab
         ════════════════════════════════════════ */}
      <Show when={activeTab() === 'flow'}>
        <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
          <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
            流量拓扑：内部代理 ↔ 系统代理 ↔ 外部网络
          </div>
          <div class="ss-card-body bg-white">
            {/* 流程图 */}
            <div class="space-y-1">
              {/* 应用层 */}
              <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-blue-50/60 border border-blue-100 text-[11px]">
                <span class="w-3 h-3 rounded-full bg-blue-500 flex-shrink-0" />
                <span class="font-medium text-blue-800">应用层</span>
                <span class="text-blue-600 ml-auto">Browser / API / CLI / Tauri</span>
              </div>
              <FlowArrow />
              {/* 内部代理 */}
              <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-emerald-50/60 border border-emerald-100 text-[11px]">
                <span class="w-3 h-3 rounded-full bg-emerald-500 flex-shrink-0" />
                <span class="font-medium text-emerald-800">内部代理</span>
                <span class="text-emerald-600 ml-auto">
                  LocalProxy :11080 · 规则引擎 · Geo 分流
                </span>
              </div>
              <FlowArrow />
              {/* 代理池 */}
              <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-violet-50/60 border border-violet-100 text-[11px]">
                <span class="w-3 h-3 rounded-full bg-violet-500 flex-shrink-0" />
                <span class="font-medium text-violet-800">代理池</span>
                <span class="text-violet-600 ml-auto">
                  {activeNodes().length} 节点 · 心跳轮转 · 指纹轮换
                </span>
              </div>
              <FlowArrow />
              {/* 系统代理 */}
              <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-amber-50/60 border border-amber-100 text-[11px]">
                <span class="w-3 h-3 rounded-full bg-amber-500 flex-shrink-0" />
                <span class="font-medium text-amber-800">系统代理</span>
                <span class="text-amber-600 ml-auto">
                  {systemProxy()?.enabled ? '已启用' : '未启用'} · {systemProxy()?.mode ?? 'off'}
                </span>
              </div>
              <FlowArrow />
              {/* 外部网络 */}
              <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-rose-50/60 border border-rose-100 text-[11px]">
                <span class="w-3 h-3 rounded-full bg-rose-500 flex-shrink-0" />
                <span class="font-medium text-rose-800">外部网络</span>
                <span class="text-rose-600 ml-auto">
                  LLM Provider · 订阅源 · 目标站点
                </span>
              </div>
            </div>

            {/* LLM Provider 健康 */}
            <Show when={providerHealth().length > 0}>
              <div class="mt-4 pt-3 border-t border-border-primary/30">
                <div class="text-10px text-zinc-500 mb-2">LLM Provider 连接状态</div>
                <div class="grid grid-cols-1 gap-1.5 max-h-32 overflow-y-auto">
                  <For each={providerHealth()}>
                    {(h) => {
                      const rate = parseFloat(h.success_rate) || 0
                      return (
                        <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                          <span class={clsx(
                            'w-2 h-2 rounded-full flex-shrink-0',
                            h.circuit_state === 'Closed' ? 'bg-emerald-500'
                              : h.circuit_state === 'Open' ? 'bg-red-500'
                              : 'bg-amber-500'
                          )} />
                          <span class="font-medium text-text-primary truncate min-w-0 flex-1">{h.name}</span>
                          <span class="text-10px text-zinc-400">{h.category}</span>
                          <div class="flex items-center gap-1 w-16">
                            <div class="flex-1 h-1.5 bg-zinc-200 rounded-full overflow-hidden">
                              <div
                                class={clsx('h-full rounded-full', rate >= 0.95 ? 'bg-emerald-500' : rate >= 0.8 ? 'bg-amber-500' : 'bg-red-500')}
                                style={{ width: `${Math.min(rate * 100, 100)}%` }}
                              />
                            </div>
                            <span class="text-10px text-zinc-500 font-mono w-8 text-right">{(rate * 100).toFixed(0)}%</span>
                          </div>
                          <span class="text-zinc-400 font-mono w-12 text-right">{h.latency_p95_ms}ms</span>
                        </div>
                      )
                    }}
                  </For>
                </div>
              </div>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  )
}
