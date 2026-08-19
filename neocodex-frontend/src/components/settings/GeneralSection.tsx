/* ════════════════════════════════════════════
   components/settings/GeneralSection.tsx — 通用：当前提供商 + API 密钥 + MCP
   提供商列表/模型池已迁至 ModelsSection（模型标签，只展示可用模型）。
   ════════════════════════════════════════════ */
import { Show } from 'solid-js'
import { clsx } from 'clsx'
import type { ProviderConfig, ProviderMeta } from '../../api/types'
import { ProviderIcon, CategoryBadge, FreeBadge } from '../ProviderIcon'
import { McpSection } from './McpSection'
import { ExpandIcon, InfoIcon } from './settingsIcons'

interface Props {
  config: () => ProviderConfig | null
  activeProvider: () => ProviderMeta | null
  apiKey: () => string
  setApiKey: (v: string) => void
  hasKey: () => boolean | null
  keyBusy: () => boolean
  onSaveApiKey: () => void
  onRequestDeleteKey: () => void
  showNotice: (msg: string) => void
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
                {(ap) => {
                  const matched = props.config()?.providers.some((p) => p.model === props.config()?.active_model)
                  return (
                    <>
                      <ProviderIcon name={ap().name} />
                      <div class="min-w-0">
                        <div class="flex items-center gap-1.5">
                          <span class="text-[13px] font-medium text-text-primary truncate">{ap().display_name}</span>
                          <CategoryBadge category={ap().category} />
                          <Show when={ap().is_free}><FreeBadge free /></Show>
                        </div>
                        <div class={clsx('text-[11px] font-mono truncate mt-0.5', matched ? 'text-text-muted' : 'text-nt-shield-600')}>
                          {matched ? props.config()?.active_model : '模型未匹配，请前往「模型」标签选择'}
                        </div>
                      </div>
                    </>
                  )
                }}
              </Show>
            </div>
            <span class={clsx('text-[10px] px-2 py-1 rounded-full font-medium flex-shrink-0', props.config()?.resolvable ? 'bg-nt-core-500/10 text-nt-core-700' : 'bg-nt-shield-500/10 text-nt-shield-600')}>
              {props.config()?.resolvable ? 'API 可达' : 'API 不可达'}
            </span>
          </div>
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