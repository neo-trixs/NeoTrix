/* ════════════════════════════════════════════
   components/settings/ModelsSection.tsx — 模型标签：代理池形式展示可用模型
   对标 iPolloWork ai-view / ollama-config：分组 provider 池 + 模型计数 + 行内状态。
   「只展示可以使用」：过滤 resolvable=false 的 provider（name 不可映射到真实类型）。
   数据源：neocodex.providerConfig()（静态目录，零后端改动）。
   交互：点击模型行 = 切到该 provider（setProvider → active_model 更新）。
   ════════════════════════════════════════════ */
import { For, Show } from 'solid-js'
import { clsx } from 'clsx'
import type { ProviderConfig, ProviderMeta } from '../../api/types'
import { ProviderIcon, CategoryBadge, FreeBadge } from '../ProviderIcon'
import { ModelIcon, CheckIcon, ActiveDotIcon } from './settingsIcons'

/** 提供商分类分组（对齐 GeneralSection CATEGORY_ORDER，随迁独立） */
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
  showNotice: (msg: string) => void
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
  return (
    <div class="space-y-4">
      <Show
        when={props.config()}
        fallback={
          <div class="ss-card">
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
              {/* 池概览：可用模型数 + 提供商数 + 当前激活 */}
              <div class="ss-card">
                <div class="ss-card-header">
                  <ModelIcon />
                  模型代理池
                </div>
                <div class="ss-card-body">
                  <div class="flex items-center justify-between gap-3">
                    <div class="flex items-center gap-4">
                      <div>
                        <div class="text-[18px] font-semibold text-text-primary leading-none">{total}</div>
                        <div class="text-[10px] text-text-muted mt-1">可用模型</div>
                      </div>
                      <div class="w-px h-8 bg-border-primary/60" />
                      <div>
                        <div class="text-[18px] font-semibold text-text-primary leading-none">{usable.length}</div>
                        <div class="text-[10px] text-text-muted mt-1">可用提供商</div>
                      </div>
                      <div class="w-px h-8 bg-border-primary/60" />
                      <div class="min-w-0">
                        <div class="text-[13px] font-medium text-text-primary truncate">{cfg().active_model || '未激活'}</div>
                        <div class="text-[10px] text-text-muted mt-0.5 flex items-center gap-1">
                          <ActiveDotIcon class="w-2 h-2 text-nt-core-600" />
                          当前模型
                        </div>
                      </div>
                    </div>
                    <span class={clsx('text-[10px] px-2 py-1 rounded-full font-medium flex-shrink-0', cfg().resolvable ? 'bg-nt-core-500/10 text-nt-core-700' : 'bg-nt-shield-500/10 text-nt-shield-600')}>
                      {cfg().resolvable ? '网关就绪' : '网关不可达'}
                    </span>
                  </div>
                </div>
              </div>

              {/* 分组代理池：按分类展开可用模型 */}
              <Show
                when={groups.length > 0}
                fallback={
                  <div class="ss-card">
                    <div class="ss-card-body">
                      <div class="text-[11px] text-text-muted text-center py-3 border border-dashed border-border-primary/60 rounded-lg">
                        当前没有可用的模型。请在「通用」配置 API 密钥后重试。
                      </div>
                    </div>
                  </div>
                }
              >
                <For each={groups}>
                  {(group) => (
                    <div class="ss-card">
                      <div class="ss-card-header">
                        {group.title}
                        <span class="ml-auto text-[10px] text-text-muted font-mono flex items-center gap-1">
                          {group.providers.reduce((s, p) => s + p.models.length, 0)} 模型 · {group.providers.length} 提供商
                        </span>
                      </div>
                      <div class="ss-card-body space-y-3">
                        <Show when={group.desc}>
                          <p class="text-[10px] text-text-muted/80 -mt-1">{group.desc}</p>
                        </Show>
                        <For each={group.providers}>
                          {(p) => {
                            const isActiveProvider = p.model === cfg().active_model
                            return (
                              <div class={clsx('rounded-xl border transition-colors', isActiveProvider ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}>
                                {/* 提供商头：图标 + 名 + 徽章 */}
                                <div class="flex items-center justify-between gap-3 px-3 py-2.5">
                                  <div class="flex items-center gap-2.5 min-w-0">
                                    <ProviderIcon name={p.name} size="sm" />
                                    <div class="flex items-center gap-1.5 min-w-0">
                                      <span class="text-[12.5px] font-medium text-text-primary truncate">{p.display_name}</span>
                                      <Show when={p.is_free}><FreeBadge free /></Show>
                                    </div>
                                  </div>
                                  <div class="flex items-center gap-2 flex-shrink-0">
                                    <CategoryBadge category={p.category} />
                                    <span class="text-[10px] text-text-muted font-mono">{p.models.length} 个</span>
                                  </div>
                                </div>
                                {/* 模型列表：代理池行 */}
                                <div class="px-2 pb-2 flex flex-col gap-1">
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
                                              <span class="inline-flex items-center gap-1 text-[10px] text-nt-io-600 font-medium">
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
    </div>
  )
}