/* ════════════════════════════════════════════
   components/settings/ModelsSection.tsx — Free LLM 池子 + 模型池
   双面板：Free LLM 池子模型列表 + 模型池选择
   ════════════════════════════════════════════ */
import { createSignal, For, Show, onMount, onCleanup } from 'solid-js'
import { clsx } from 'clsx'
import type { ProviderConfig, ProviderMeta } from '../../api/types'
import type { ProviderHealthStatus, PoolSufficiencyReport } from '../../api/neocodex'
import { llamacpp, type LlamacppModel } from '../../api/domain'
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

const formatNum = (n: number) => n.toLocaleString()

export function ModelsSection(props: Props) {
  const [health, setHealth] = createSignal<ProviderHealthStatus[]>([])
  const [poolReport, setPoolReport] = createSignal<PoolSufficiencyReport | null>(null)
  const [healthLoading, setHealthLoading] = createSignal(true)
  const [discoverLoading, setDiscoverLoading] = createSignal(false)
  const [discoverResult, setDiscoverResult] = createSignal<{ discovered_count: number; registered_total: number; models: { provider: string; model_id: string; base_url: string; is_free: boolean; tier: string }[] } | null>(null)

  // llamacpp local model state
  const [llamacppModels, setLlamacppModels] = createSignal<LlamacppModel[]>([])
  const [llamacppHealth, setLlamacppHealth] = createSignal<{ status: string; pid?: number; uptime_secs?: number } | null>(null)
  const [llamacppLoading, setLlamacppLoading] = createSignal(true)
  const [selectedModel, setSelectedModel] = createSignal('')
  const [swapping, setSwapping] = createSignal(false)
  const [llamacppError, setLlamacppError] = createSignal<string | null>(null)

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

  const handleDiscover = async () => {
    setDiscoverLoading(true)
    try {
      const result = await import('../../api/neocodex').then((m) => m.discoverModels())
      setDiscoverResult(result)
      await fetchHealth()
    } catch { /* silent */ } finally {
      setDiscoverLoading(false)
    }
  }

  const fetchLlamacpp = async () => {
    try {
      const [models, status] = await Promise.all([
        llamacpp.models(),
        llamacpp.health(),
      ])
      setLlamacppModels(models)
      setLlamacppHealth(status)
      if (models.length > 0 && !selectedModel()) {
        setSelectedModel(models[0].path)
      }
    } catch {
      setLlamacppError('无法连接 llamacpp 服务')
    } finally {
      setLlamacppLoading(false)
    }
  }

  const handleSwapModel = async () => {
    const model = selectedModel()
    if (!model || swapping()) return
    setSwapping(true)
    setLlamacppError(null)
    try {
      await llamacpp.swap(model)
      await fetchLlamacpp()
    } catch (e) {
      setLlamacppError(e instanceof Error ? e.message : '切换失败')
    } finally {
      setSwapping(false)
    }
  }

  let timer: ReturnType<typeof setInterval> | undefined
  onMount(() => {
    fetchHealth()
    fetchLlamacpp()
    timer = setInterval(fetchHealth, 15_000)
  })
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

  // 过滤免费模型
  const freeModels = () => health().filter(h => h.is_free)

  return (
    <div class="space-y-4">
      {/* ════════════════════════════════════════
         Free LLM 池子面板
         ════════════════════════════════════════ */}
      <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
        <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
          <ModelIcon />
          Free LLM 池子
          <div class="ml-auto flex items-center gap-1.5">
            <Show when={poolReport()}>
              {(rpt) => (
                <span class={clsx(
                  'text-10px px-2 py-0.5 rounded-full font-medium border',
                  rpt().sufficient ? 'bg-emerald-50 text-emerald-700 border-emerald-200' : 'bg-amber-50 text-amber-700 border-amber-200'
                )}>
                  {rpt().free_available} 可用 / {rpt().free_total} 总计
                </span>
              )}
            </Show>
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

            {/* 免费模型列表 */}
            <Show when={freeModels().length > 0} fallback={
              <div class="text-[11px] text-zinc-400 text-center py-2">暂无免费模型</div>
            }>
              <div class="grid grid-cols-1 gap-1.5">
                <For each={freeModels()}>
                  {(h) => (
                    <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                      {/* 状态灯 */}
                      <span class={clsx(
                        'w-2 h-2 rounded-full flex-shrink-0',
                        h.available ? 'bg-emerald-500' : 'bg-red-500'
                      )} />
                      {/* 名称 */}
                      <span class="font-medium text-text-primary truncate min-w-0 flex-1">{h.name}</span>
                      {/* 分类 */}
                      <span class="text-10px text-zinc-400 font-mono w-12 text-right">{h.category}</span>
                      {/* 可用状态 */}
                      <span class={clsx(
                        'px-1.5 py-0.5 rounded text-10px font-medium border',
                        h.available
                          ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                          : 'bg-red-50 text-red-600 border-red-200'
                      )}>
                        {h.available ? '可用' : '不可用'}
                      </span>
                      {/* 成功率 */}
                      <span class="text-zinc-400 font-mono w-12 text-right">{(parseFloat(h.success_rate) * 100).toFixed(0)}%</span>
                      {/* 调用统计 */}
                      <span class="text-zinc-400 font-mono w-16 text-right">{formatNum(h.total_calls)}/{formatNum(h.total_errors)}</span>
                    </div>
                  )}
                </For>
              </div>
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
              刷新免费模型目录（12+ 源：OpenRouter / Groq / Cloudflare / GitHub 等），自动注册到池子。
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
         Llamacpp 本地模型
         ════════════════════════════════════════ */}
      <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
        <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
          <ModelIcon />
          Llamacpp 本地推理
          <Show when={llamacppHealth()}>
            <span class={clsx(
              'ml-auto text-10px px-2 py-0.5 rounded-full font-medium border',
              llamacppHealth()!.status === 'running'
                ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                : 'bg-zinc-100 text-zinc-500 border-zinc-200'
            )}>
              {llamacppHealth()!.status === 'running'
                ? `运行中 · PID ${llamacppHealth()!.pid}`
                : '未运行'}
            </span>
          </Show>
        </div>
        <div class="ss-card-body bg-white">
          <Show when={!llamacppLoading()} fallback={
            <div class="text-[11px] text-zinc-400 text-center py-3">加载模型列表…</div>
          }>
            <Show when={llamacppModels().length > 0} fallback={
              <div class="text-[11px] text-zinc-400 text-center py-2">暂无本地模型</div>
            }>
              {/* 模型选择 + 切换 */}
              <div class="flex items-center gap-2 mb-3">
                <select
                  class="flex-1 text-[12px] font-mono px-2.5 py-1.5 rounded-lg border border-border-primary/50 bg-white text-text-primary focus:outline-none focus:ring-2 focus:ring-nt-io-500/40"
                  value={selectedModel()}
                  onChange={(e) => setSelectedModel(e.currentTarget.value)}
                >
                  <For each={llamacppModels()}>
                    {(m) => <option value={m.path}>{m.name} ({(m.size / 1073741824).toFixed(1)} GB)</option>}
                  </For>
                </select>
                <button
                  class={clsx(
                    'px-3 py-1.5 rounded-lg text-[11px] font-medium border transition-colors flex-shrink-0',
                    swapping() || !selectedModel()
                      ? 'text-zinc-400 border-zinc-200 cursor-not-allowed'
                      : 'text-nt-io-600 border-nt-io-200 hover:bg-nt-io-50'
                  )}
                  onClick={handleSwapModel}
                  disabled={swapping() || !selectedModel()}
                >
                  {swapping() ? '切换中…' : '切换模型'}
                </button>
              </div>

              {/* 错误提示 */}
              <Show when={llamacppError()}>
                <div class="mb-2 px-2.5 py-1.5 rounded-lg bg-red-50 border border-red-200 text-[11px] text-red-600">
                  {llamacppError()}
                </div>
              </Show>

              {/* 模型列表 */}
              <div class="grid grid-cols-1 gap-1.5">
                <For each={llamacppModels()}>
                  {(m) => (
                    <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                      <span class="w-2 h-2 rounded-full flex-shrink-0 bg-zinc-300" />
                      <span class="font-medium text-text-primary truncate min-w-0 flex-1 font-mono">{m.name}</span>
                      <span class="text-zinc-400 font-mono w-16 text-right">{(m.size / 1073741824).toFixed(1)} GB</span>
                    </div>
                  )}
                </For>
              </div>
            </Show>
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
