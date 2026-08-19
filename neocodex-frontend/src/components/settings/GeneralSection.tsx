/* ════════════════════════════════════════════
   components/settings/GeneralSection.tsx — 通用：提供商 + API 密钥 + MCP
   provider 分组逻辑自包含（CATEGORY_ORDER 随迁）；API 密钥操作经父组件回调。
   ════════════════════════════════════════════ */
import { For, Show } from 'solid-js'
import { clsx } from 'clsx'
import type { ProviderConfig, ProviderMeta } from '../../api/types'
import { ProviderIcon, CategoryBadge, FreeBadge } from '../ProviderIcon'
import { McpSection } from './McpSection'
import { ExpandIcon, DataIcon, InfoIcon } from './settingsIcons'

/** 提供商分类分组（对标 Claude Desktop 分类设置） */
const CATEGORY_ORDER = ['local', 'proxy', 'cloud', 'unknown'] as const
const CATEGORY_TITLE: Record<string, string> = {
  local: '本地推理 · 数据不出设备',
  proxy: '自定义代理 · OpenAI 兼容中转',
  cloud: '云端 API · 需密钥',
  unknown: '其他',
}

interface Props {
  config: () => ProviderConfig | null
  loading: () => boolean
  activeProvider: () => ProviderMeta | null
  switching: () => boolean
  apiKey: () => string
  setApiKey: (v: string) => void
  hasKey: () => boolean | null
  keyBusy: () => boolean
  onSwitchProvider: (name: string) => void
  onSaveApiKey: () => void
  onRequestDeleteKey: () => void
  showNotice: (msg: string) => void
}

const providerGroups = (cfg: ProviderConfig) => {
  const groups: { category: string; title: string; providers: ProviderMeta[] }[] = []
  for (const cat of CATEGORY_ORDER) {
    const list = cfg.providers.filter((p) => (p.category ?? 'unknown') === cat)
    if (list.length > 0) {
      groups.push({ category: cat, title: CATEGORY_TITLE[cat] ?? cat, providers: list })
    }
  }
  const rest = cfg.providers.filter((p) => !CATEGORY_ORDER.includes((p.category ?? 'unknown') as (typeof CATEGORY_ORDER)[number]))
  if (rest.length > 0) groups.push({ category: 'unknown', title: '其他', providers: rest })
  return groups
}

export function GeneralSection(props: Props) {
  return (
    <div class="space-y-4">
      {/* 当前激活提供商 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <ExpandIcon />
          当前提供商
        </div>
        <div class="ss-card-body">
          <div class="flex items-center justify-between gap-3">
            <div class="flex items-center gap-3 min-w-0">
              <Show when={props.activeProvider()} fallback={<span class="w-8 h-8 rounded-lg bg-nt-io-500/12 text-nt-io-600 flex items-center justify-center text-[14px] font-semibold flex-shrink-0">?</span>}>
                {(ap) => (
                  <>
                    <ProviderIcon name={ap().name} />
                    <div class="min-w-0">
                      <div class="flex items-center gap-1.5">
                        <span class="text-[13px] font-medium text-text-primary truncate">{ap().display_name}</span>
                        <CategoryBadge category={ap().category} />
                        <Show when={ap().is_free}><FreeBadge free /></Show>
                      </div>
                      <div class="text-[11px] text-text-muted font-mono truncate mt-0.5">{props.config()?.active_model}</div>
                    </div>
                  </>
                )}
              </Show>
            </div>
            <span class={clsx('text-[10px] px-2 py-1 rounded-full font-medium flex-shrink-0', props.config()?.resolvable ? 'bg-nt-core-500/10 text-nt-core-700' : 'bg-nt-shield-500/10 text-nt-shield-600')}>
              {props.config()?.resolvable ? 'API 可达' : 'API 不可达'}
            </span>
          </div>
        </div>
      </div>

      {/* 提供商列表 — 按分类分组 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <DataIcon />
          {props.config()?.provider_count} 个提供商
        </div>
        <div class="ss-card-body space-y-4">
          <For each={props.config() ? providerGroups(props.config()!) : []}>
            {(group) => (
              <div>
                <div class="flex items-center gap-2 mb-2">
                  <span class="text-[10px] uppercase tracking-[0.1em] text-text-muted/80 font-medium">{group.title}</span>
                  <span class="text-[9px] text-text-muted/60 font-mono">{group.providers.length}</span>
                </div>
                <div class="space-y-2">
                  <For each={group.providers}>
                    {(p) => {
                      const isActive = p.model === props.config()?.active_model
                      return (
                        <button
                          class={clsx(
                            'w-full flex items-center justify-between gap-3 px-3 py-3 rounded-xl border transition-colors',
                            isActive
                              ? 'border-nt-io-500/40 bg-nt-io-500/6'
                              : 'border-border-primary/50 bg-white/40 hover:bg-white/70'
                          )}
                          onClick={() => !isActive && props.onSwitchProvider(p.name)}
                          disabled={props.switching()}
                          role="radio"
                          aria-checked={isActive}
                        >
                          <div class="flex items-center gap-3 min-w-0">
                            <ProviderIcon name={p.name} size="sm" />
                            <div class="min-w-0">
                              <div class="flex items-center gap-1.5">
                                <span class="text-[12.5px] text-text-primary font-medium truncate">{p.display_name}</span>
                                <Show when={p.is_free}><FreeBadge free /></Show>
                              </div>
                              <div class="text-[10.5px] text-text-muted font-mono truncate">{p.model}</div>
                            </div>
                          </div>
                          <div class="flex items-center gap-2 flex-shrink-0">
                            <CategoryBadge category={p.category} className="hidden sm:inline-flex" />
                            <Show when={isActive}>
                              <span class="text-[10px] text-nt-io-600 font-medium">✓ 当前</span>
                            </Show>
                          </div>
                        </button>
                      )
                    }}
                  </For>
                </div>
              </div>
            )}
          </For>
        </div>
      </div>

      {/* API 密钥管理（明确作用域：ANTHROPIC_API_KEY） */}
      <div class="ss-card">
        <div class="ss-card-header">
          <InfoIcon />
          API 密钥
        </div>
        <div class="ss-card-body space-y-3">
          <p class="text-[11px] text-text-muted leading-relaxed -mt-1">
            密钥保存在本地 <span class="font-mono text-text-secondary">ANTHROPIC_API_KEY</span>（Claude 网关）。
            各云端提供商分别读取自己的环境变量（如 <span class="font-mono">OPENAI_API_KEY</span> / <span class="font-mono">GOOGLE_API_KEY</span>）。
          </p>
          <div class="flex items-center gap-2">
            <input
              type="password"
              class="flex-1 min-w-0 px-3 py-2 rounded-lg bg-white/70 border border-border-primary text-[12.5px] text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
              placeholder={props.hasKey() === false ? '输入 API 密钥…' : '输入新密钥替换…'}
              value={props.apiKey()}
              onInput={(e) => props.setApiKey(e.currentTarget.value)}
              onKeyDown={(e) => { if (e.key === 'Enter') props.onSaveApiKey() }}
              aria-label="API 密钥"
            />
            <button
              class="px-3 py-2 rounded-lg bg-nt-io-500 text-text-primary text-[12px] font-medium hover:bg-nt-io-600 disabled:opacity-50 transition-colors flex-shrink-0"
              onClick={props.onSaveApiKey}
              disabled={props.keyBusy() || !props.apiKey().trim()}
            >
              保存
            </button>
          </div>
          <div class="flex items-center justify-between">
            <span class={clsx('text-[11px]', props.hasKey() === true ? 'text-nt-core-700' : 'text-text-muted')}>
              {props.hasKey() === true ? '✓ 已配置 API 密钥' : props.hasKey() === false ? '未配置 API 密钥' : '检测中…'}
            </span>
            <Show when={props.hasKey() === true}>
              <button
                class="px-3 py-1 rounded-lg border border-red-500/30 bg-red-500/5 text-[11px] text-red-500 hover:bg-red-500/10 disabled:opacity-50 transition-colors"
                onClick={props.onRequestDeleteKey}
                disabled={props.keyBusy()}
              >
                删除
              </button>
            </Show>
          </div>
        </div>
      </div>

      <McpSection showNotice={props.showNotice} />
    </div>
  )
}