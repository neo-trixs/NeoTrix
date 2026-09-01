/* ════════════════════════════════════════════
   components/settings/ModelsSection.tsx — 代理池健康度 + 模型池
   双面板：健康度（内部通信 + 外部网络）+ 模型池选择
   ════════════════════════════════════════════ */
import { createSignal, For, Show, onMount, onCleanup, batch } from 'solid-js'
import { clsx } from 'clsx'
import type { ProviderConfig, ProviderMeta } from '../../api/types'
import type { ProviderHealthStatus, PoolSufficiencyReport, ProbeResult, DiscoveryResult } from '../../api/neocodex'
import { ProviderIcon, CategoryBadge, FreeBadge } from '../ProviderIcon'
import { ModelIcon, CheckIcon, ActiveDotIcon, TestTubeIcon, AlertCircleIcon } from './settingsIcons'

const CATEGORY_ORDER = ['local', 'proxy', 'cloud', 'unknown'] as const
const CATEGORY_TITLE: Record<string, string> = {
  local: '本地推理',
  proxy: '自定义代理',
  cloud: '云端 API',
  unknown: '其他',
}

interface Props {
  config: () => ProviderConfig | null
  loading: () => boolean
  switching: () => boolean
  onSwitchProvider: (name: string) => void
  onTestConnection?: (name: string) => void
  testState?: () => Record<string, 'testing' | 'ok' | 'fail'>
}

const usableProviders = (cfg: ProviderConfig) => cfg.providers.filter((p) => p.resolvable)

const providerPoolGroups = (cfg: ProviderConfig) => {
  const groups: { category: string; title: string; providers: ProviderMeta[] }[] = []
  for (const cat of CATEGORY_ORDER) {
    const list = usableProviders(cfg).filter((p) => (p.category ?? 'unknown') === cat)
    if (list.length > 0) {
      groups.push({ category: cat, title: CATEGORY_TITLE[cat] ?? cat, providers: list })
    }
  }
  return groups
}

/* ── 小型可视化组件 ── */
const circuitColor = (state: string) => {
  switch (state) {
    case 'Closed': return 'text-emerald-600 bg-emerald-50 border-emerald-200'
    case 'Open': return 'text-red-600 bg-red-50 border-red-200'
    case 'HalfOpen': return 'text-amber-600 bg-amber-50 border-amber-200'
    default: return 'text-zinc-500 bg-zinc-50 border-zinc-200'
  }
}
const circuitLabel = (state: string) => {
  switch (state) {
    case 'Closed': return '正常'
    case 'Open': return '熔断'
    case 'HalfOpen': return '半开'
    default: return state
  }
}
const circuitDot = (state: string) => {
  switch (state) {
    case 'Closed': return 'bg-emerald-500'
    case 'Open': return 'bg-red-500'
    case 'HalfOpen': return 'bg-amber-500'
    default: return 'bg-zinc-400'
  }
}
const healthBar = (rate: number) => {
  if (rate >= 0.95) return 'bg-emerald-500'
  if (rate >= 0.8) return 'bg-amber-500'
  return 'bg-red-500'
}
const formatNum = (n: number) => n.toLocaleString()
const parseRate = (s: string) => parseFloat(s) || 0

export function ModelsSection(props: Props) {
  const [health, setHealth] = createSignal<ProviderHealthStatus[]>([])
  const [poolReport, setPoolReport] = createSignal<PoolSufficiencyReport | null>(null)
  const [probes, setProbes] = createSignal<ProbeResult[]>([])
  const [healthLoading, setHealthLoading] = createSignal(true)
  const [probeLoading, setProbeLoading] = createSignal(false)
  const [discoverLoading, setDiscoverLoading] = createSignal(false)
  const [discoverResult, setDiscoverResult] = createSignal<DiscoveryResult | null>(null)
  const [healthTab, setHealthTab] = createSignal<'internal' | 'external'>('internal')

  const fetchHealth = async () => {
    try {
      const [status, report] = await Promise.all([
        import('../../api/neocodex').then((m) => m.providerStatus()),
        import('../../api/neocodex').then((m) => m.poolSufficiency()),
      ])
      setHealth(status)
      setPoolReport(report)
    } catch { /* silent */ } finally {
      setHealthLoading(false)
    }
  }

  const fetchProbes = async () => {
    setProbeLoading(true)
    try {
      const result = await import('../../api/neocodex').then((m) => m.probeAllProviders())
      setProbes(result)
    } catch { /* silent */ } finally {
      setProbeLoading(false)
    }
  }

  const handleDiscover = async () => {
    setDiscoverLoading(true)
    try {
      const result = await import('../../api/neocodex').then((m) => m.discoverModels())
      setDiscoverResult(result)
      // 刷新健康数据
      await fetchHealth()
    } catch { /* silent */ } finally {
      setDiscoverLoading(false)
    }
  }

  let timer: ReturnType<typeof setInterval> | undefined
  onMount(() => { fetchHealth(); timer = setInterval(fetchHealth, 15_000) })
  onCleanup(() => { if (timer) clearInterval(timer) })

  const handleProviderClick = (p: ProviderMeta) => {
    if (props.switching()) return
    if (p.model === props.config()?.active_model) return
    props.onSwitchProvider(p.name)
  }

  const handleTest = (p: ProviderMeta) => {
    if (props.switching() || (props.testState?.()[p.name] === 'testing')) return
    props.onTestConnection?.(p.name)
  }

  const testState: () => Record<string, 'testing' | 'ok' | 'fail'> = props.testState ?? (() => ({}))

  return (
    <div class="space-y-4">
      {/* ════════════════════════════════════════
         代理池健康度面板
         ════════════════════════════════════════ */}
      <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
        <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
          <ModelIcon />
          代理池健康度
          <div class="ml-auto flex items-center gap-1.5">
            <button
              class={clsx(
                'text-10px px-2 py-0.5 rounded-md border transition-colors',
                healthTab() === 'internal'
                  ? 'bg-nt-io-500/10 text-nt-io-600 border-nt-io-200'
                  : 'text-zinc-500 border-zinc-200 hover:bg-zinc-100'
              )}
              onClick={() => setHealthTab('internal')}
            >内部通信</button>
            <button
              class={clsx(
                'text-10px px-2 py-0.5 rounded-md border transition-colors',
                healthTab() === 'external'
                  ? 'bg-nt-io-500/10 text-nt-io-600 border-nt-io-200'
                  : 'text-zinc-500 border-zinc-200 hover:bg-zinc-100'
              )}
              onClick={() => { setHealthTab('external'); if (probes().length === 0) fetchProbes() }}
            >外部网络</button>
          </div>
        </div>

        <div class="ss-card-body bg-white">
          <Show when={!healthLoading()} fallback={
            <div class="text-[11px] text-zinc-400 text-center py-3">加载中…</div>
          }>
            {/* 池概览 */}
            <Show when={poolReport()}>
              {(rpt) => (
                <div class="flex items-center gap-4 mb-3 pb-3 border-b border-border-primary/30">
                  <div class="text-center">
                    <div class="text-[16px] font-semibold text-zinc-900">{rpt().total_providers}</div>
                    <div class="text-10px text-zinc-500">总提供商</div>
                  </div>
                  <div class="w-px h-6 bg-black/5" />
                  <div class="text-center">
                    <div class="text-[16px] font-semibold text-emerald-600">{rpt().free_available}</div>
                    <div class="text-10px text-zinc-500">免费可用</div>
                  </div>
                  <div class="w-px h-6 bg-black/5" />
                  <div class="text-center">
                    <div class="text-[16px] font-semibold text-zinc-900">{rpt().locked_models}</div>
                    <div class="text-10px text-zinc-500">锁定模型</div>
                  </div>
                  <div class="ml-auto">
                    <span class={clsx(
                      'text-10px px-2 py-0.5 rounded-full font-medium border',
                      rpt().sufficient ? 'bg-emerald-50 text-emerald-700 border-emerald-200' : 'bg-amber-50 text-amber-700 border-amber-200'
                    )}>
                      {rpt().sufficient ? '池子充足' : '池子不足'}
                    </span>
                  </div>
                </div>
              )}
            </Show>

            {/* ── 内部通信 Tab ── */}
            <Show when={healthTab() === 'internal'}>
              <Show when={health().length > 0} fallback={
                <div class="text-[11px] text-zinc-400 text-center py-2">暂无运行时数据</div>
              }>
                <div class="grid grid-cols-1 gap-1.5">
                  <For each={health()}>
                    {(h) => {
                      const rate = parseRate(h.success_rate)
                      const locked = h.model_locked_count
                      return (
                        <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                          {/* 状态灯 */}
                          <span class={clsx('w-2 h-2 rounded-full flex-shrink-0', circuitDot(h.circuit_state))} />
                          {/* 名称 + 分类 */}
                          <span class="font-medium text-text-primary truncate min-w-0 flex-1">{h.name}</span>
                          <span class="text-10px text-zinc-400 font-mono w-12 text-right">{h.category}</span>
                          {/* 电路状态 */}
                          <span class={clsx('px-1.5 py-0.5 rounded text-10px font-medium border', circuitColor(h.circuit_state))}>
                            {circuitLabel(h.circuit_state)}
                          </span>
                          {/* 成功率条 */}
                          <div class="flex items-center gap-1 w-16">
                            <div class="flex-1 h-1.5 bg-zinc-200 rounded-full overflow-hidden">
                              <div class={clsx('h-full rounded-full', healthBar(rate))} style={{ width: `${Math.min(rate * 100, 100)}%` }} />
                            </div>
                            <span class="text-10px text-zinc-500 font-mono w-8 text-right">{(rate * 100).toFixed(0)}%</span>
                          </div>
                          {/* 延迟 */}
                          <span class="text-zinc-400 font-mono w-12 text-right">{h.latency_p95_ms}ms</span>
                          {/* 调用统计 */}
                          <span class="text-zinc-400 font-mono w-16 text-right">{formatNum(h.total_calls)}/{formatNum(h.total_errors)}</span>
                          {/* 锁定数 */}
                          <Show when={locked > 0}>
                            <span class="text-amber-500 font-mono text-10px">{locked}锁</span>
                          </Show>
                        </div>
                      )
                    }}
                  </For>
                </div>
              </Show>
            </Show>

            {/* ── 外部网络 Tab ── */}
            <Show when={healthTab() === 'external'}>
              <div class="flex items-center justify-between mb-2">
                <span class="text-10px text-zinc-500">端点可达性探测（HEAD 5s 超时）</span>
                <button
                  class={clsx(
                    'text-10px px-2 py-0.5 rounded-md border transition-colors',
                    probeLoading()
                      ? 'text-zinc-400 border-zinc-200 cursor-not-allowed'
                      : 'text-nt-io-600 border-nt-io-200 hover:bg-nt-io-50'
                  )}
                  onClick={fetchProbes}
                  disabled={probeLoading()}
                >
                  {probeLoading() ? '探测中…' : '重新探测'}
                </button>
              </div>

              <Show when={probes().length > 0} fallback={
                <div class="text-[11px] text-zinc-400 text-center py-2">
                  {probeLoading() ? '正在探测所有端点…' : '点击「重新探测」检测网络可达性'}
                </div>
              }>
                <div class="grid grid-cols-1 gap-1.5">
                  <For each={probes()}>
                    {(p) => (
                      <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                        <span class={clsx(
                          'w-2 h-2 rounded-full flex-shrink-0',
                          p.reachable ? 'bg-emerald-500' : 'bg-red-500'
                        )} />
                        <span class="font-medium text-text-primary truncate min-w-0 flex-1">{p.name}</span>
                        <span class={clsx(
                          'px-1.5 py-0.5 rounded text-10px font-medium border',
                          p.reachable
                            ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                            : 'bg-red-50 text-red-600 border-red-200'
                        )}>
                          {p.reachable ? `HTTP ${p.status_code}` : '不可达'}
                        </span>
                        <span class="text-zinc-400 font-mono w-12 text-right">{p.latency_ms}ms</span>
                        <Show when={p.error}>
                          <span class="text-red-400 text-10px truncate max-w-24">{p.error}</span>
                        </Show>
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
         自动发现模型
         ════════════════════════════════════════ */}
      <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
        <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
          自动发现模型
        </div>
        <div class="ss-card-body bg-white">
          <div class="flex items-center gap-3">
            <p class="text-[11px] text-zinc-500 flex-1">
              刷新免费模型目录（12+ 源：OpenRouter / Groq / Cloudflare / GitHub 等），自动注册到代理池。
            </p>
            <button
              class={clsx(
                'px-3 py-1.5 rounded-lg text-[11px] font-medium border transition-colors',
                discoverLoading()
                  ? 'text-zinc-400 border-zinc-200 cursor-not-allowed'
                  : 'text-nt-io-600 border-nt-io-200 hover:bg-nt-io-50'
              )}
              onClick={handleDiscover}
              disabled={discoverLoading()}
            >
              {discoverLoading() ? '发现中…' : '立即发现'}
            </button>
          </div>
          <Show when={discoverResult()}>
            {(r) => (
              <div class="mt-2 p-2 rounded-lg bg-emerald-50/60 border border-emerald-100 text-[11px]">
                <span class="text-emerald-700 font-medium">✓ 发现 {r().discovered_count} 个模型</span>
                <span class="text-zinc-500">，已注册 {r().registered_total} 个提供商</span>
                <Show when={r().models.length > 0}>
                  <div class="mt-1.5 max-h-24 overflow-y-auto text-10px text-zinc-600 space-y-0.5">
                    <For each={r().models.slice(0, 20)}>
                      {(m) => (
                        <div class="flex items-center gap-1">
                          <span class="font-mono text-zinc-500">{m.provider}</span>
                          <span>/</span>
                          <span class="font-mono text-zinc-700">{m.model_id}</span>
                          <span class="ml-auto text-zinc-400">{m.tier}</span>
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
              </div>
            )}
          </Show>
        </div>
      </div>

      {/* ════════════════════════════════════════
         模型池
         ════════════════════════════════════════ */}
      <Show
        when={props.config()}
        fallback={
          <div class="ss-card rounded-2xl border-black/5 shadow-sm">
            <div class="ss-card-body text-[11px] text-text-muted text-center py-4">
              {props.loading() ? '加载模型目录…' : '暂无模型配置'}
            </div>
          </div>
        }
      >
        {(cfg) => {
          const usable = usableProviders(cfg())
          const total = usable.reduce((s, p) => s + p.models.length, 0)
          const groups = providerPoolGroups(cfg())
          return (
            <>
              {/* 池概览 */}
              <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
                <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
                  模型池
                  <span class="ml-auto text-[11px] font-mono text-zinc-500">{total} 模型 · {usable.length} 提供商</span>
                </div>
                <div class="ss-card-body bg-white">
                  <div class="flex items-center justify-between gap-3">
                    <div class="flex items-center gap-4">
                      <div>
                        <div class="text-[18px] font-semibold text-zinc-900 leading-none">{total}</div>
                        <div class="text-10px text-zinc-500 mt-1">可用模型</div>
                      </div>
                      <div class="w-px h-8 bg-black/5" />
                      <div class="min-w-0">
                        <div class="text-[13px] font-medium text-zinc-900 truncate">{cfg().active_model || '未激活'}</div>
                        <div class="text-10px text-zinc-500 mt-0.5 flex items-center gap-1">
                          <ActiveDotIcon class="w-2 h-2 text-emerald-500" />
                          当前模型
                        </div>
                      </div>
                    </div>
                    <span class={clsx('text-10px px-2.5 py-1 rounded-full font-medium border flex-shrink-0', cfg().resolvable ? 'bg-emerald-50 text-emerald-700 border-emerald-200' : 'bg-red-50 text-red-600 border-red-200')}>
                      {cfg().resolvable ? '● 可用' : '○ 不可用'}
                    </span>
                  </div>
                </div>
              </div>

              {/* 按分类展示提供商 */}
              <For each={groups}>
                {(group) => (
                  <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
                    <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
                      {group.title}
                      <span class="ml-auto text-10px text-zinc-500 font-mono">
                        {group.providers.reduce((s, p) => s + p.models.length, 0)} 模型
                      </span>
                    </div>
                    <div class="ss-card-body bg-white space-y-3">
                      <For each={group.providers}>
                        {(p) => {
                          const isActiveProvider = p.model === cfg().active_model
                          return (
                            <div class={clsx('rounded-xl border transition-colors', isActiveProvider ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}>
                              <button
                                class={clsx('w-full flex items-center justify-between gap-3 px-3 py-2.5 text-left rounded-t-xl transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:ring-inset',
                                  isActiveProvider ? 'bg-nt-io-500/8' : 'hover:bg-white/60'
                                )}
                                onClick={() => handleProviderClick(p)}
                                disabled={props.switching() || isActiveProvider}
                                aria-label={`切换到 ${p.display_name}`}
                              >
                                <div class="flex items-center gap-2.5 min-w-0">
                                  <ProviderIcon name={p.name} size="sm" category={p.category} />
                                  <div class="flex items-center gap-1.5 min-w-0">
                                    <span class="text-[12.5px] font-medium text-text-primary truncate">{p.display_name}</span>
                                    <Show when={p.is_free}><FreeBadge free /></Show>
                                  </div>
                                </div>
                                <div class="flex items-center gap-2 flex-shrink-0">
                                  <Show when={props.onTestConnection}>
                                    <button
                                      class="p-1.5 rounded-lg border border-border-primary/50 bg-white/40 hover:bg-white/70 transition-colors flex-shrink-0"
                                      onClick={(e) => { e.stopPropagation(); handleTest(p) }}
                                      disabled={props.switching() || testState()[p.name] === 'testing'}
                                      title="测试连接"
                                    >
                                      <Show when={testState()[p.name] === 'testing'} fallback={
                                        <Show when={testState()[p.name] === 'ok'} fallback={
                                          <Show when={testState()[p.name] === 'fail'} fallback={<TestTubeIcon class="w-3.5 h-3.5 text-text-muted" />}>
                                            <AlertCircleIcon class="w-3.5 h-3.5 text-red-500" />
                                          </Show>
                                        }>
                                          <CheckIcon class="w-3.5 h-3.5 text-emerald-500" />
                                        </Show>
                                      }>
                                        <span class="w-3.5 h-3.5 flex items-center justify-center">
                                          <span class="w-3 h-3 border-2 border-nt-io-500/30 border-t-nt-io-500 rounded-full animate-spin" />
                                        </span>
                                      </Show>
                                    </button>
                                  </Show>
                                  <CategoryBadge category={p.category} />
                                  <span class="text-10px text-text-muted font-mono">{p.models.length} 个</span>
                                </div>
                              </button>
                              <div class="px-2 pb-2 flex flex-col gap-1" role="radiogroup" aria-label={`${p.display_name} 模型池`}>
                                <For each={p.models}>
                                  {(modelId) => {
                                    const isActive = modelId === cfg().active_model
                                    return (
                                      <button
                                        class={clsx(
                                          'w-full flex items-center justify-between gap-3 px-3 py-2 rounded-lg text-left transition-colors group/model',
                                          isActive ? 'bg-nt-io-500/12 text-nt-io-700' : 'hover:bg-white/70 hover:shadow-sm',
                                        )}
                                        onClick={() => !isActive && !props.switching() && props.onSwitchProvider(p.name)}
                                        disabled={props.switching()}
                                        role="radio"
                                        aria-checked={isActive}
                                      >
                                        <span class={clsx('text-[12px] font-mono truncate', isActive ? 'font-semibold text-nt-io-700' : 'text-text-secondary group-hover/model:text-text-primary')}>
                                          {modelId}
                                        </span>
                                        <span class="flex-shrink-0">
                                          <Show when={isActive} fallback={<span class="w-2 h-2 rounded-full border border-border-primary/70 group-hover/model:border-nt-io-500/50" />}>
                                            <span class="inline-flex items-center gap-1 text-10px text-nt-io-600 font-medium">
                                              <CheckIcon class="w-3 h-3" /> 当前
                                            </span>
                                          </Show>
                                        </span>
                                      </button>
                                    )
                                  }}
                                </For>
                              </div>
                            </div>
                          )
                        }}
                      </For>
                    </div>
                  </div>
                )}
              </For>
            </>
          )
        }}
      </Show>
    </div>
  )
}
