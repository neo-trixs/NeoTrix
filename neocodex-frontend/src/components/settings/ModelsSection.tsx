/* ════════════════════════════════════════════
   components/settings/ModelsSection.tsx — 模型标签：代理池形式展示可用模型
   对标 iPolloWork ai-view / ollama-config：分组 provider 池 + 模型计数 + 行内状态。
   「只展示可以使用」：过滤 resolvable=false 的 provider（name 不可映射到真实类型）。
   数据源：neocodex.providerConfig()（静态目录，零后端改动）。
   交互：点击模型行 = 切到该 provider（setProvider → active_model 更新）。
   ════════════════════════════════════════════ */
 import { createSignal, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import type { ProviderConfig, ProviderMeta, CustomProviderReq } from '../../api/types'
import { ProviderIcon, CategoryBadge, FreeBadge } from '../ProviderIcon'
import { ModelIcon, CheckIcon, ActiveDotIcon, TestTubeIcon, AlertCircleIcon, PlusIcon } from './settingsIcons'

/** 提供商分类分组（对齐 ProviderIcon 分类徽章：本地绿/代理琥珀/云端蓝） */
const CATEGORY_ORDER = ['local', 'proxy', 'cloud', 'unknown'] as const
const CATEGORY_TITLE: Record<string, string> = {
  local: '本地推理',
  proxy: '自定义代理',
  cloud: '云端 API',
  unknown: '其他',
}
const CATEGORY_DESC: Record<string, string> = {
  local: '数据不出设备',
  proxy: 'OpenAI 兼容中转',
  cloud: '需 API 密钥',
  unknown: '',
}

interface Props {
  config: () => ProviderConfig | null
  loading: () => boolean
  switching: () => boolean
  onSwitchProvider: (name: string) => void
  onTestConnection?: (name: string) => void
  testState?: () => Record<string, 'testing' | 'ok' | 'fail'>
  /** 已接入的外部自定义提供商（乐观本地态，跨标签持久） */
  customProviders: () => ProviderMeta[]
  /** 新增外部第三方模型 API（智能配置） */
  onAddCustomProvider: (req: CustomProviderReq) => Promise<void>
}

/** 只保留"可用"提供商（resolvable=true：name 映射到真实 LlmProviderType） */
const usableProviders = (cfg: ProviderConfig) => cfg.providers.filter((p) => p.resolvable)

/** 按 category 分组的可用提供商池 */
const providerPoolGroups = (cfg: ProviderConfig) => {
  const groups: { category: string; title: string; desc: string; providers: ProviderMeta[] }[] = []
  for (const cat of CATEGORY_ORDER) {
    const list = usableProviders(cfg).filter((p) => (p.category ?? 'unknown') === cat)
    if (list.length > 0) {
      groups.push({ category: cat, title: CATEGORY_TITLE[cat] ?? cat, desc: CATEGORY_DESC[cat] ?? '', providers: list })
    }
  }
  const rest = usableProviders(cfg).filter((p) => !CATEGORY_ORDER.includes((p.category ?? 'unknown') as (typeof CATEGORY_ORDER)[number]))
  if (rest.length > 0) groups.push({ category: 'unknown', title: '其他', desc: '', providers: rest })
  return groups
}

export function ModelsSection(props: Props) {
  const handleProviderHeadClick = (p: ProviderMeta) => {
    if (props.switching()) return
    if (p.model === props.config()?.active_model) return
    props.onSwitchProvider(p.name)
  }
  // ── 连通性测试（B3, 吸收 LM Studio endpoint health 模式）──
  // P3-M3 已接线: invoke('provider_test', { base_url }) 真实探测。
  const [testState, setTestState] = createSignal<Record<string, { testing: boolean; latencyMs?: number; failed?: boolean }>>({})
  async function runConnectivityTest(name: string) {
    setTestState((m) => ({ ...m, [name]: { testing: true } }))
    try {
      const cfgp = props.config()?.providers.find((x) => x.name === name)
      if (!cfgp?.base_url || !cfgp.base_url.startsWith('http')) throw new Error('no endpoint')
      const r = await providerTest(cfgp.base_url)
      setTestState((m) => ({ ...m, [name]: { testing: false, latencyMs: r.latency_ms } }))
    } catch {
      setTestState((m) => ({ ...m, [name]: { testing: false, failed: true } }))
    }
  }

  // 双栏：自定义配置 vs 代理池（参考同类产品模型广场）
  const [subTab, setSubTab] = createSignal<'custom' | 'pool'>('pool')
  // 连接测试状态来自父组件（SettingsModal 统一管理异步测试）
  const testState: () => Record<string, 'testing' | 'ok' | 'fail'> = props.testState ?? (() => ({}))

  const handleTest = (p: ProviderMeta) => {
    if (props.switching() || testState()[p.name] === 'testing') return
    props.onTestConnection?.(p.name)
  }

  /* ── 外部第三方模型 API 智能配置 ── */
  const [showAddForm, setShowAddForm] = createSignal(false)
  const [addName, setAddName] = createSignal('')
  const [addBaseUrl, setAddBaseUrl] = createSignal('')
  const [addApiKey, setAddApiKey] = createSignal('')
  const [addModel, setAddModel] = createSignal('')
  const [detectedModels, setDetectedModels] = createSignal<string[]>([])
  const [detecting, setDetecting] = createSignal(false)
  const [detectError, setDetectError] = createSignal<string | null>(null)
  const [saving, setSaving] = createSignal(false)
  const [saveError, setSaveError] = createSignal<string | null>(null)

  const smartDetect = async () => {
    const base = addBaseUrl().trim()
    if (!base) { setDetectError('请先填写 API 基地址'); return }
    setDetecting(true); setDetectError(null); setDetectedModels([])
    try {
      const models = await import('../../api/neocodex').then((m) => m.fetchProviderModels(base, addApiKey().trim()))
      setDetectedModels(models)
      if (models.length > 0 && !addModel().trim()) setAddModel(models[0])
    } catch (e) {
      setDetectError((e as Error).message || '智能检测失败')
    } finally {
      setDetecting(false)
    }
  }

  const handleAddCustom = async () => {
    const name = addName().trim()
    const base = addBaseUrl().trim()
    const model = addModel().trim()
    if (!name || !base || !model) { setSaveError('名称、API 基地址、模型均为必填'); return }
    setSaving(true); setSaveError(null)
    try {
      const req: CustomProviderReq = {
        name: `custom-${name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')}`,
        display_name: name,
        base_url: base,
        api_key: addApiKey().trim(),
        models: detectedModels().length > 0 ? detectedModels() : [model],
        model,
      }
      await props.onAddCustomProvider(req)
      setShowAddForm(false)
      setAddName(''); setAddBaseUrl(''); setAddApiKey(''); setAddModel(''); setDetectedModels([])
    } catch (e) {
      setSaveError((e as Error).message || '保存失败')
    } finally {
      setSaving(false)
    }
  }

  // 自定义提供商（含后端目录中的 proxy/local + 本地乐观接入的外部模型）
  const customProvidersList = () => [
    ...(props.config() ? usableProviders(props.config()!).filter((p) => p.category === 'proxy' || p.category === 'local') : []),
    ...props.customProviders(),
  ]
  const activeModel = () => props.config()?.active_model ?? ''

  return (
    <div class="space-y-4">
      {/* 顶部切换：自定义配置 / 代理池 — Mac 圆角药丸 */}
      <div class="flex items-center gap-2 p-1 rounded-full bg-zinc-100 border border-black/5 w-fit">
        <button
          class={clsx('px-3.5 py-1.5 rounded-full text-[12px] font-medium transition-all', subTab() === 'custom' ? 'bg-white shadow-sm text-zinc-900 border border-black/5' : 'text-zinc-500 hover:text-zinc-700')}
          onClick={() => setSubTab('custom')}
        >自定义配置</button>
        <button
          class={clsx('px-3.5 py-1.5 rounded-full text-[12px] font-medium transition-all', subTab() === 'pool' ? 'bg-white shadow-sm text-zinc-900 border border-black/5' : 'text-zinc-500 hover:text-zinc-700')}
          onClick={() => setSubTab('pool')}
        >代理池 · {(props.config() ? usableProviders(props.config()!).length : 0)} 提供商</button>
      </div>

      {/* 自定义配置：外部第三方模型 API 智能配置（不依赖后端 config，浏览器预览可用） */}
      <Show when={subTab() === 'custom'}>
        <div class="ss-card rounded-2xl border-black/5 shadow-sm">
          <div class="ss-card-header bg-zinc-50/60 border-b border-black/5 flex items-center">
            外部第三方模型 API
            <button
              class="ml-auto inline-flex items-center gap-1 px-2.5 py-1 rounded-lg bg-nt-io-500/10 text-nt-io-700 text-[11px] font-medium hover:bg-nt-io-500/20 transition-colors"
              onClick={() => setShowAddForm(!showAddForm())}
            >
              <PlusIcon class="w-3.5 h-3.5" />
              {showAddForm() ? '收起' : '添加'}
            </button>
          </div>
          <div class="ss-card-body bg-white space-y-3">
            <Show when={showAddForm()}>
              <div class="space-y-2.5">
                <div class="grid grid-cols-2 gap-2">
                  <label class="block">
                    <span class="text-[11px] text-text-muted">显示名称</span>
                    <input
                      class="mt-1 w-full px-2.5 py-1.5 rounded-lg bg-white border border-border-primary text-[12px] text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
                      placeholder="如 My OpenAI 代理"
                      value={addName()}
                      onInput={(e) => setAddName(e.currentTarget.value)}
                    />
                  </label>
                  <label class="block">
                    <span class="text-[11px] text-text-muted">API 基地址</span>
                    <input
                      class="mt-1 w-full px-2.5 py-1.5 rounded-lg bg-white border border-border-primary text-[12px] text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
                      placeholder="https://api.openai.com/v1"
                      value={addBaseUrl()}
                      onInput={(e) => setAddBaseUrl(e.currentTarget.value)}
                    />
                  </label>
                </div>
                <label class="block">
                  <span class="text-[11px] text-text-muted">API 密钥</span>
                  <input
                    type="password"
                    class="mt-1 w-full px-2.5 py-1.5 rounded-lg bg-white border border-border-primary text-[12px] text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
                    placeholder="sk-...（部分网关可留空）"
                    value={addApiKey()}
                    onInput={(e) => setAddApiKey(e.currentTarget.value)}
                  />
                </label>
                <label class="block">
                  <span class="text-[11px] text-text-muted">模型</span>
                  <div class="mt-1 flex items-center gap-2">
                    <Show when={detectedModels().length > 0} fallback={
                      <input
                        class="flex-1 min-w-0 px-2.5 py-1.5 rounded-lg bg-white border border-border-primary text-[12px] text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
                        placeholder="gpt-4o / 自定义模型 id"
                        value={addModel()}
                        onInput={(e) => setAddModel(e.currentTarget.value)}
                      />
                    }>
                      <select
                        class="flex-1 min-w-0 px-2.5 py-1.5 rounded-lg bg-white border border-border-primary text-[12px] text-text-primary focus:outline-none focus:ring-1 focus:ring-nt-io-500"
                        value={addModel()}
                        onChange={(e) => setAddModel(e.currentTarget.value)}
                      >
                        <For each={detectedModels()}>{(m) => <option value={m}>{m}</option>}</For>
                      </select>
                    </Show>
                    <button
                      class="px-2.5 py-1.5 rounded-lg border border-border-primary/60 text-[11px] text-text-secondary hover:bg-white/70 transition-colors disabled:opacity-50 flex-shrink-0"
                      onClick={smartDetect}
                      disabled={detecting()}
                    >
                      {detecting() ? '检测中…' : '智能检测'}
                    </button>
                  </div>
                </label>
                <Show when={detectError()}><div class="text-[11px] text-red-500">{detectError()}</div></Show>
                <Show when={saveError()}><div class="text-[11px] text-red-500">{saveError()}</div></Show>
                <div class="flex items-center justify-end gap-2 pt-1">
                  <button
                    class="px-3 py-1.5 rounded-lg text-[11px] text-text-muted hover:bg-bg-tertiary transition-colors"
                    onClick={() => setShowAddForm(false)}
                  >取消</button>
                  <button
                    class="px-3 py-1.5 rounded-lg bg-nt-io-500 text-text-primary text-[11px] font-medium hover:bg-nt-io-600 disabled:opacity-50 transition-colors"
                    onClick={handleAddCustom}
                    disabled={saving()}
                  >{saving() ? '保存中…' : '保存'}</button>
                </div>
              </div>
            </Show>
          </div>
        </div>

        {/* 已接入的自定义提供商 */}
        <div class="ss-card rounded-2xl border-black/5 shadow-sm">
          <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">已接入的自定义提供商</div>
          <div class="ss-card-body bg-white">
            <Show when={customProvidersList().length > 0} fallback={<div class="text-[11px] text-zinc-500 text-center py-6 border border-dashed border-zinc-200 rounded-xl">暂无自定义配置，点击上方「添加」接入 OpenAI 兼容 / 自定义网关</div>}>
              <div class="grid grid-cols-1 gap-2">
                <For each={customProvidersList()}>
                  {(p) => (
                    <button class={clsx('w-full flex items-center gap-3 px-3 py-3 rounded-xl border text-left transition-colors', p.model === activeModel() ? 'bg-orange-50 border-orange-200' : 'bg-white hover:bg-zinc-50 border-zinc-200')} onClick={() => handleProviderHeadClick(p)}>
                      <ProviderIcon name={p.name} size="sm" />
                      <span class="text-[12.5px] font-medium truncate">{p.display_name}</span>
                      <CategoryBadge category={p.category} className="ml-auto" />
                    </button>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </div>
      </Show>

      {/* 代理池：依赖后端 config */}
      <Show when={subTab() === 'pool'}>
        <Show
          when={props.config()}
          fallback={
            <div class="ss-card">
              <div class="ss-card-body text-[11px] text-text-muted text-center py-4">
                {props.loading() ? '加载模型目录…' : '暂无模型配置（桌面后端未连接，浏览器预览仅支持「自定义配置」）'}
              </div>
            </div>
          }
        >
          {(cfg) => {
            const usable = usableProviders(cfg())
            const total = usable.reduce((s, p) => s + p.models.length, 0)
            const groups = providerPoolGroups(cfg())
            const active = cfg().providers.find((p) => p.model === cfg().active_model) ?? null
            return (
              <>
                {/* 池概览 */}
                <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
                  <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
                    <ModelIcon />
                    模型广场
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
                        <div>
                          <div class="text-[18px] font-semibold text-zinc-900 leading-none">{usable.length}</div>
                          <div class="text-10px text-zinc-500 mt-1">可用提供商</div>
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

                {/* 代理池：按分类展开可用模型 */}
                <Show
                  when={groups.length > 0}
                  fallback={
                    <div class="ss-card rounded-2xl border-black/5 shadow-sm">
                      <div class="ss-card-body bg-white">
                        <div class="text-[11px] text-zinc-500 text-center py-6 border border-dashed border-zinc-200 rounded-xl">
                          当前没有可用的模型。请在「自定义配置」中接入提供商
                        </div>
                      </div>
                    </div>
                  }
                >
                  <For each={groups}>
                    {(group) => (
                      <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
                        <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
                          {group.title}
                          <span class="ml-auto text-10px text-zinc-500 font-mono flex items-center gap-1">
                            {group.providers.reduce((s, p) => s + p.models.length, 0)} 模型 · {group.providers.length} 提供商
                          </span>
                        </div>
                        <div class="ss-card-body bg-white space-y-3">
                        <Show when={group.desc}>
                          <p class="text-10px text-text-muted/80 -mt-1">{group.desc}</p>
                        </Show>
                        <For each={group.providers}>
                          {(p) => {
                            const isActiveProvider = p.model === cfg().active_model
                            return (
                              <div class={clsx('rounded-xl border transition-colors', isActiveProvider ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}>
                                <button
                                  class={clsx('w-full flex items-center justify-between gap-3 px-3 py-2.5 text-left rounded-t-xl transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:ring-inset',
                                    isActiveProvider ? 'bg-nt-io-500/8' : 'hover:bg-white/60'
                                  )}
                                  onClick={() => handleProviderHeadClick(p)}
                                  disabled={props.switching() || isActiveProvider}
                                  aria-label={`切换到 ${p.display_name}`}
                                  title={isActiveProvider ? '当前提供商' : `点击切换到 ${p.display_name}`}
                                >
                                  <div class="flex items-center gap-2.5 min-w-0">
                                    <ProviderIcon name={p.name} size="sm" />
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
                                        aria-label={`测试 ${p.display_name} 连接`}
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
                                            isActive
                                              ? 'bg-nt-io-500/12 text-nt-io-700'
                                              : 'hover:bg-white/70 hover:shadow-sm',
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
              </Show>
              </>
            )
          }}
        </Show>
      </Show>
    </div>
  )
}