/* ════════════════════════════════════════════
   components/settings/GeneralSection.tsx — 通用（对标 OpenWebUI General）
   默认模型 + 界面语言 + API 密钥 + MCP + 输入与启动 + 数据清理。
   模型池完整列表已迁至 ModelsSection；本页聚焦「默认模型」概览与通用偏好。
   ════════════════════════════════════════════ */
import { Show, For, createSignal, onMount } from 'solid-js'
import { clsx } from 'clsx'
import type { ProviderConfig, ProviderMeta } from '../../api/types'
import { ProviderIcon, CategoryBadge, FreeBadge } from '../ProviderIcon'
import { McpSection } from './McpSection'
import { ExpandIcon, InfoIcon, GlobeIcon } from './settingsIcons'

const UI_LANG_KEY = 'neotrix.ui.lang'
type UiLang = 'zh-CN' | 'en'
const UI_LANGS: { value: UiLang; label: string; sub: string }[] = [
  { value: 'zh-CN', label: '简体中文', sub: 'Chinese (Simplified)' },
  { value: 'en', label: 'English', sub: 'English (US)' },
]

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
  enterBehavior: () => 'send' | 'newline'
  setEnterBehavior: (v: 'send' | 'newline') => void
  restoreLastSession: () => boolean
  setRestoreLastSession: (v: boolean) => void
  autostartEnabled: () => boolean
  setAutostartEnabled: (v: boolean) => void
  onClearDemoData: () => void
}

export function GeneralSection(props: Props) {
  const [uiLang, setUiLang] = createSignal<UiLang>('zh-CN')

  onMount(() => {
    const saved = (localStorage.getItem(UI_LANG_KEY) as UiLang | null) || 'zh-CN'
    setUiLang(saved)
    document.documentElement.lang = saved
  })

  const changeLang = (lang: UiLang) => {
    setUiLang(lang)
    localStorage.setItem(UI_LANG_KEY, lang)
    document.documentElement.lang = lang
    props.showNotice(lang === 'zh-CN' ? '界面语言已设为简体中文' : 'Interface language set to English')
  }

  return (
    <div class="space-y-4">
      {/* 默认模型（对标 OpenWebUI：General → Default Model） */}
      <div class="ss-card">
        <div class="ss-card-header">
          <ExpandIcon />
          默认模型
        </div>
        <div class="ss-card-body space-y-3">
          <Show when={props.config()} fallback={
            <div class="flex items-center gap-3">
              <span class="w-8 h-8 rounded-lg bg-zinc-100 flex items-center justify-center text-[14px] font-semibold text-zinc-400 flex-shrink-0">?</span>
              <div class="text-[12px] text-text-muted leading-relaxed">
                后端未连接，无法读取默认模型。请在桌面端打开 NeoTrix，或前往「模型 → 自定义配置」接入本地网关。
              </div>
            </div>
          }>
            {(cfg) => (
              <div class="flex items-center justify-between gap-3">
                <div class="flex items-center gap-3 min-w-0">
                  <Show when={props.activeProvider()} fallback={<span class="w-8 h-8 rounded-lg bg-nt-io-500/12 text-nt-io-600 flex items-center justify-center text-[14px] font-semibold flex-shrink-0">?</span>}>
                    {(ap) => {
                      const matched = cfg().providers.some((p) => p.model === cfg().active_model)
                      return (
                        <>
                           <ProviderIcon name={ap().name} category={ap().category} />
                          <div class="min-w-0">
                            <div class="flex items-center gap-1.5">
                              <span class="text-[13px] font-medium text-text-primary truncate">{ap().display_name}</span>
                              <CategoryBadge category={ap().category} />
                              <Show when={ap().is_free}><FreeBadge free /></Show>
                            </div>
                            <div class={clsx('text-[11px] font-mono truncate mt-0.5', matched ? 'text-text-muted' : 'text-nt-shield-600')}>
                              {matched ? cfg().active_model : '模型未匹配，请前往「模型」标签选择'}
                            </div>
                          </div>
                        </>
                      )
                    }}
                  </Show>
                </div>
                <span class={clsx('text-10px px-2 py-1 rounded-full font-medium flex-shrink-0', cfg().resolvable ? 'bg-nt-core-500/10 text-nt-core-700' : 'bg-nt-shield-500/10 text-nt-shield-600')}>
                  {cfg().resolvable ? 'API 可达' : 'API 不可达'}
                </span>
              </div>
            )}
          </Show>
          <p class="text-[11px] text-text-muted leading-relaxed">
            当前默认模型即对话框左下角「模型切换」所选中项。点击打开 <span class="font-medium text-text-secondary">「模型」</span> 标签可浏览完整模型池并切换。
          </p>
        </div>
      </div>

      {/* 界面语言（对标 OpenWebUI：General → Language） */}
      <div class="ss-card">
        <div class="ss-card-header">
          <GlobeIcon />
          界面语言
        </div>
        <div class="ss-card-body">
          <div class="ss-row">
            <div>
              <div class="ss-row-label">显示语言</div>
              <div class="ss-row-desc">设置后应用于整个界面（重启后保留）</div>
            </div>
            <div class="flex items-center gap-1 p-0.5 rounded-lg bg-zinc-100 border border-black/5">
              <For each={UI_LANGS}>
                {(lang) => (
                  <button
                    class={clsx('px-2.5 py-1.5 rounded-md text-[11px] font-medium transition-all', uiLang() === lang.value ? 'bg-white shadow-sm text-zinc-900' : 'text-zinc-500 hover:text-zinc-700')}
                    onClick={() => changeLang(lang.value)}
                    aria-pressed={uiLang() === lang.value}
                  >{lang.label}</button>
                )}
              </For>
            </div>
          </div>
        </div>
      </div>

      {/* API 密钥管理 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <InfoIcon />
          API 密钥
        </div>
        <div class="ss-card-body space-y-3">
          <p class="text-[11px] text-text-muted leading-relaxed -mt-1">
            密钥保存在本地系统 keyring（<span class="font-mono text-text-secondary">neotrix</span> 服务）。
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

      {/* 输入与启动行为（对标 OpenWebUI：Interface） */}
      <div class="ss-card">
        <div class="ss-card-header">
          <InfoIcon />
          输入与启动
        </div>
        <div class="ss-card-body space-y-4">
          <div class="ss-row">
            <div>
              <div class="ss-row-label">回车键行为</div>
              <div class="ss-row-desc">Enter 发送 / Shift+Enter 换行，或反之</div>
            </div>
            <div class="flex items-center gap-1 p-0.5 rounded-lg bg-zinc-100 border border-black/5">
              <button
                class={clsx('px-2.5 py-1.5 rounded-md text-[11px] font-medium transition-all', props.enterBehavior() === 'send' ? 'bg-white shadow-sm text-zinc-900' : 'text-zinc-500 hover:text-zinc-700')}
                onClick={() => props.setEnterBehavior('send')}
              >Enter 发送</button>
              <button
                class={clsx('px-2.5 py-1.5 rounded-md text-[11px] font-medium transition-all', props.enterBehavior() === 'newline' ? 'bg-white shadow-sm text-zinc-900' : 'text-zinc-500 hover:text-zinc-700')}
                onClick={() => props.setEnterBehavior('newline')}
              >Enter 换行</button>
            </div>
          </div>
          <div class="ss-row">
            <div>
              <div class="ss-row-label">恢复上次会话</div>
              <div class="ss-row-desc">启动时自动打开上次的对话</div>
            </div>
            <button
              class={clsx('relative w-11 h-6 rounded-full transition-colors flex-shrink-0', props.restoreLastSession() ? 'bg-nt-io-500' : 'bg-zinc-300')}
              onClick={() => props.setRestoreLastSession(!props.restoreLastSession())}
              role="switch"
              aria-checked={props.restoreLastSession()}
              aria-label="恢复上次会话"
            >
              <span class={clsx('absolute top-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform', props.restoreLastSession() ? 'left-0.5 translate-x-5' : 'left-0.5')} />
            </button>
          </div>
        </div>
      </div>

      {/* 数据清理 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <InfoIcon />
          数据清理
        </div>
        <div class="ss-card-body">
          <button
            class="w-full flex items-center justify-between px-3 py-3 rounded-xl border border-border-primary/50 bg-white/40 hover:bg-white/70 transition-colors"
            onClick={props.onClearDemoData}
          >
            <span class="text-[12.5px] text-text-primary">清空演示数据</span>
            <span class="text-10px text-text-muted">重置首启动样例</span>
          </button>
        </div>
      </div>
    </div>
  )
}