import { createSignal, createEffect, For, Show } from 'solid-js'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { clsx } from 'clsx'
import { PluginMarketplace } from './PluginMarketplace'
import { TrafficLights } from './TrafficLights'

/* ════════════════════════════════════════════
   SettingsModal — 统一设置面板（设计 v2）
   对标主流产品（Claude/Cursor）设置结构：
   左侧分类导航（外扩线条图标） + 右侧内容分区
   图标语言：极简线条 · 外扩（open）而非内敛 —— 开阔心态
   ════════════════════════════════════════════ */

interface ProviderEntry {
  name: string
  model: string
  resolvable: boolean
}

interface ProviderConfig {
  provider_count: number
  resolvable: boolean
  active_model: string
  providers: ProviderEntry[]
}

type SectionId = 'general' | 'appearance' | 'plugins' | 'data' | 'about'

/* ── 外扩线条图标（open/expand 语义，非内敛） ── */
function ExpandIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 中央节点 + 四条外射线：向外打开 */}
      <circle cx="8" cy="8" r="1.2" stroke="currentColor" stroke-width="1.3" />
      <line x1="8" y1="3" x2="8" y2="0.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
      <line x1="8" y1="13" x2="8" y2="15.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
      <line x1="3" y1="8" x2="0.5" y2="8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
      <line x1="13" y1="8" x2="15.5" y2="8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
    </svg>
  )
}

function PaletteIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 色板 + 外扩扇区 */}
      <path d="M8 2.5a5.5 5.5 0 100 11c1.5 0 2-1 1-2-.7-.7-.3-1.5 1-1.5h1.5c1.1 0 2-.9 2-2A5.5 5.5 0 008 2.5z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
      <circle cx="5.5" cy="6.5" r="0.7" fill="currentColor" />
      <circle cx="8" cy="5" r="0.7" fill="currentColor" />
      <circle cx="10.5" cy="6.5" r="0.7" fill="currentColor" />
    </svg>
  )
}

function InfoIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.2" />
      <line x1="8" y1="7.5" x2="8" y2="11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <circle cx="8" cy="4.8" r="0.7" fill="currentColor" />
    </svg>
  )
}

function PluginsIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 拼图块 + 外扩射线（插件扩展语义） */}
      <path d="M5 3h4v2.5a1.5 1.5 0 010 3V11H5V8.5a1.5 1.5 0 010-3V3z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
      <line x1="8" y1="1" x2="8" y2="0.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="8" y1="15" x2="8" y2="15.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="1" y1="8" x2="0.5" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="15" y1="8" x2="15.5" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
    </svg>
  )
}

function DataIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 数据库圆柱 + 外扩箭头（导出语义） */}
      <ellipse cx="8" cy="4" rx="5" ry="2.2" stroke="currentColor" stroke-width="1.2" />
      <path d="M3 4v8c0 1.2 2.2 2.2 5 2.2s5-1 5-2.2V4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="3" y1="8" x2="8" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="13" y1="8" x2="13" y2="3.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="13" y1="3.5" x2="11.5" y2="5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
      <line x1="13" y1="3.5" x2="14.5" y2="5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  )
}

function XIcon() {
  return (
    <svg viewBox="0 0 12 12" fill="none">
      <line x1="3" y1="3" x2="9" y2="9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="9" y1="3" x2="3" y2="9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
    </svg>
  )
}

const SECTIONS: { id: SectionId; label: string; icon: () => any }[] = [
  { id: 'general', label: '通用', icon: ExpandIcon },
  { id: 'appearance', label: '外观', icon: PaletteIcon },
  { id: 'plugins', label: '插件', icon: PluginsIcon },
  { id: 'data', label: '数据', icon: DataIcon },
  { id: 'about', label: '关于', icon: InfoIcon },
]

/* 分组侧栏导航（对标 osaurus ManagementView 分组结构）：
   常规 General / 扩展 Extensions / 数据 Data / 系统 System */
const NAV_GROUPS: { title: string; ids: SectionId[] }[] = [
  { title: '常规', ids: ['general', 'appearance'] },
  { title: '扩展', ids: ['plugins'] },
  { title: '数据', ids: ['data'] },
  { title: '系统', ids: ['about'] },
]

const sectionById = (id: SectionId) => SECTIONS.find((s) => s.id === id)!

export function SettingsModal(props: { open: boolean; onClose: () => void }) {
  const [section, setSection] = createSignal<SectionId>('general')
  const [config, setConfig] = createSignal<ProviderConfig | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [switching, setSwitching] = createSignal(false)
  const [notice, setNotice] = createSignal<string | null>(null)
  const [motionPref, setMotionPref] = createSignal<'full' | 'reduced'>('full')
  const [densityPref, setDensityPref] = createSignal<'comfortable' | 'compact'>('comfortable')
  const [themePref] = createSignal<'light'>('light')
  const [fontSizePref, setFontSizePref] = createSignal<'sm' | 'md' | 'lg'>('md')
  const [memStats, setMemStats] = createSignal<{ total_entries: number; total_categories: number; avg_confidence: number; memory_usage_bytes: number } | null>(null)
  const [dataBusy, setDataBusy] = createSignal(false)
  const [appVersion, setAppVersion] = createSignal<string | null>(null)
  // API 密钥管理（对标 Claude 设置）
  const [apiKey, setApiKey] = createSignal('')
  const [hasKey, setHasKey] = createSignal<boolean | null>(null)
  const [keyBusy, setKeyBusy] = createSignal(false)

  // 偏好持久化：localStorage + 根元素 data-* 属性（CSS 属性选择器响应）
  const applyPrefs = (density: 'comfortable' | 'compact', motion: 'full' | 'reduced', fontSize: 'sm' | 'md' | 'lg') => {
    const root = document.documentElement
    root.dataset.density = density
    root.dataset.motion = motion
    root.dataset.fontSize = fontSize
    root.dataset.theme = 'light'
    try {
      localStorage.setItem('neotrix:prefs', JSON.stringify({ density, motion, theme: 'light', fontSize }))
    } catch { /* 持久化失败静默 */ }
  }

  const setDensity = (d: 'comfortable' | 'compact') => {
    setDensityPref(d)
    applyPrefs(d, motionPref(), fontSizePref())
  }

  const setMotion = (m: 'full' | 'reduced') => {
    setMotionPref(m)
    applyPrefs(densityPref(), m, fontSizePref())
  }

  const setFontSize = (s: 'sm' | 'md' | 'lg') => {
    setFontSizePref(s)
    applyPrefs(densityPref(), motionPref(), s)
  }

  // 启动时恢复偏好
  createEffect(() => {
    if (props.open) {
      try {
        const raw = localStorage.getItem('neotrix:prefs')
        if (raw) {
          const p = JSON.parse(raw)
          if (p.density) setDensityPref(p.density)
          if (p.motion) setMotionPref(p.motion)
          if (p.fontSize) setFontSizePref(p.fontSize)
          applyPrefs(p.density ?? 'comfortable', p.motion ?? 'full', p.fontSize ?? 'md')
        } else {
          applyPrefs('comfortable', 'full', 'md')
        }
      } catch { /* 解析失败用默认 */ }
    }
  })

  const loadConfig = async () => {
    setLoading(true)
    try {
      setConfig(await invoke<ProviderConfig>('neocodex_provider_config'))
    } catch (e) {
      setNotice(String(e))
    } finally {
      setLoading(false)
    }
  }

  const loadMemStats = async () => {
    try {
      setMemStats(await invoke<{ total_entries: number; total_categories: number; avg_confidence: number; memory_usage_bytes: number }>('memory_stats'))
    } catch { /* 记忆统计非关键 */ }
  }

  const loadAppVersion = async () => {
    try {
      setAppVersion(await invoke<string>('neocodex_app_version'))
    } catch { /* 版本非关键 */ }
  }

  const exportMemory = async () => {
    setDataBusy(true)
    setNotice(null)
    try {
      const json = await invoke<string>('memory_export', { format: 'json' })
      const path = await save({
        defaultPath: `neotrix-memory-${new Date().toISOString().slice(0, 10)}.json`,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      })
      if (path) {
        await writeTextFile(path, json)
        setNotice(`已导出记忆到 ${path}`)
      }
    } catch (e) {
      setNotice(String(e))
    } finally {
      setDataBusy(false)
    }
  }

  const clearMemory = async () => {
    setDataBusy(true)
    setNotice(null)
    try {
      const n = await invoke<number>('memory_clear', { kind: null })
      setNotice(`已清空 ${n} 条记忆`)
      await loadMemStats()
    } catch (e) {
      setNotice(String(e))
    } finally {
      setDataBusy(false)
    }
  }

  /* API 密钥：读状态 / 保存 / 删除（对标 Claude 设置中的 API 密钥管理） */
  const loadApiKeyStatus = async () => {
    try {
      setHasKey(await invoke<boolean>('has_api_key'))
    } catch { /* 非关键 */ }
  }

  const saveApiKey = async () => {
    const key = apiKey().trim()
    if (!key) return
    setKeyBusy(true)
    setNotice(null)
    try {
      await invoke('save_api_key', { key })
      setApiKey('')
      await loadApiKeyStatus()
      setNotice('API 密钥已保存')
    } catch (e) {
      setNotice(String(e))
    } finally {
      setKeyBusy(false)
    }
  }

  const deleteApiKey = async () => {
    setKeyBusy(true)
    setNotice(null)
    try {
      await invoke('delete_api_key')
      await loadApiKeyStatus()
      setNotice('API 密钥已删除')
    } catch (e) {
      setNotice(String(e))
    } finally {
      setKeyBusy(false)
    }
  }

  createEffect(() => {
    if (props.open) {
      setSection('general')
      setNotice(null)
      loadConfig()
      loadMemStats()
      loadAppVersion()
      loadApiKeyStatus()
    }
  })

  const switchProvider = async (name: string) => {
    setSwitching(true)
    setNotice(null)
    try {
      await invoke('neocodex_set_provider', { name })
      setNotice(`已切换到 ${name}`)
      await loadConfig()
      // 广播提供商变更，输入区 ProviderSelector 即时刷新
      window.dispatchEvent(new CustomEvent('neotrix:provider-changed', { detail: { name } }))
    } catch (e) {
      setNotice(String(e))
    } finally {
      setSwitching(false)
    }
  }

  const activeProvider = () => {
    const cfg = config()
    if (!cfg) return null
    return cfg.providers.find((p) => p.model === cfg.active_model) ?? cfg.providers[0] ?? null
  }

  const [navRef, setNavRef] = createSignal<HTMLElement | null>(null)

  // 弹窗键盘：Esc 关闭 + 打开聚焦（对标 Claude/Cursor 弹窗规范）
  createEffect(() => {
    if (!props.open) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') props.onClose()
    }
    window.addEventListener('keydown', onKey)
    navRef()?.querySelector<HTMLButtonElement>('button')?.focus()
    return () => window.removeEventListener('keydown', onKey)
  })

  return (
    <Show when={props.open}>
      <div
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/25 backdrop-blur-[2px] animate-fade-in"
        onClick={props.onClose}
      >
        <div
          class="w-[780px] max-w-[94vw] h-[620px] max-h-[88vh] rounded-2xl glass-modal border border-white/40 overflow-hidden flex animate-slide-in"
          onClick={(e) => e.stopPropagation()}
          role="dialog"
          aria-label="设置"
          aria-modal="true"
        >
          {/* ── 左侧分组导航（对标 osaurus ManagementView） ── */}
          <nav ref={setNavRef} class="w-[190px] flex-shrink-0 border-r border-white/30 bg-white/10 py-4 px-2 flex flex-col gap-1 overflow-y-auto" role="tablist" aria-label="设置分类">
            <div class="px-3 pb-3">
                <div class="flex items-center gap-2">
                  <TrafficLights />
                </div>
              </div>
            <For each={NAV_GROUPS}>
              {(group) => (
                <div class="mb-2">
                  <div class="px-3 pb-2 pt-2 text-[10px] uppercase tracking-[0.14em] text-text-muted/70 font-medium">
                    {group.title}
                  </div>
                  <For each={group.ids}>
                    {(id) => {
                      const s = sectionById(id)
                      const isActive = section() === id
                      return (
                        <button
                          class={clsx(
                            'w-full flex items-center gap-3 px-3 py-2 rounded-lg text-[12.5px] transition-colors',
                            isActive
                              ? 'bg-nt-io-500/12 text-nt-io-700 font-medium shadow-[inset_0_1px_0_rgba(255,255,255,0.6)]'
                              : 'text-text-secondary hover:text-text-primary hover:bg-white/40'
                          )}
                          role="tab"
                          aria-selected={isActive}
                          tabIndex={isActive ? 0 : -1}
                          onClick={() => setSection(id)}
                          onKeyDown={(e) => {
                            if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
                              e.preventDefault()
                              const flat = NAV_GROUPS.flatMap((g) => g.ids)
                              const idx = flat.indexOf(id)
                              const dir = e.key === 'ArrowDown' ? 1 : -1
                              setSection(flat[(idx + dir + flat.length) % flat.length])
                            } else if (e.key === 'Home') {
                              e.preventDefault(); setSection(NAV_GROUPS[0].ids[0])
                            } else if (e.key === 'End') {
                              e.preventDefault(); const flat = NAV_GROUPS.flatMap((g) => g.ids); setSection(flat[flat.length - 1])
                            }
                          }}
                        >
                          <span class={clsx('w-4 h-4 flex-shrink-0', isActive ? 'text-nt-io-600' : 'text-text-muted')}>
                            <s.icon />
                          </span>
                          <span class="flex-1 text-left truncate">{s.label}</span>
                          {isActive && <span class="w-2 h-2 rounded-full bg-nt-io-500 flex-shrink-0" />}
                        </button>
                      )
                    }}
                  </For>
                </div>
              )}
            </For>
          </nav>

          {/* ── 右侧内容 ── */}
          <div class="flex-1 flex flex-col min-w-0">
            <header class="flex items-center justify-between px-6 py-4 border-b border-border-primary/40 flex-shrink-0 bg-white/20">
              <div class="flex items-center gap-3">
                <span class="w-8 h-8 rounded-lg bg-nt-io-500/12 text-nt-io-600 flex items-center justify-center flex-shrink-0">
                  {sectionById(section()).icon()}
                </span>
                <div>
                  <div class="text-[15px] font-semibold text-text-primary">
                    {sectionById(section()).label}
                  </div>
                  <div class="text-[11px] text-text-muted">
                    {section() === 'general' && '模型提供商与运行参数'}
                    {section() === 'appearance' && '界面视觉与动效'}
                    {section() === 'plugins' && '技能插件与扩展'}
                    {section() === 'data' && '记忆与数据管理'}
                    {section() === 'about' && '版本与诊断信息'}
                  </div>
                </div>
              </div>
              <button
                class="p-2 rounded-lg text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
                onClick={props.onClose}
                aria-label="关闭设置"
                title="关闭设置"
              >
                <XIcon />
              </button>
            </header>

            <div class="flex-1 overflow-y-auto px-6 py-5">
              {/* ── 通用：模型提供商 ── */}
              <Show when={section() === 'general'}>
                <Show when={loading() && !config()}>
                  <div class="text-xs text-text-muted py-6 text-center">加载配置…</div>
                </Show>
                <Show when={config()}>
                  {(cfg) => (
                    <div class="space-y-4">
                      {/* 当前激活提供商 */}
                      <div class="ss-card">
                        <div class="ss-card-header">
                          <ExpandIcon />
                          当前提供商
                        </div>
                        <div class="ss-card-body">
                          <div class="flex items-center justify-between">
                            <div class="flex items-center gap-3">
                              <span class="w-8 h-8 rounded-lg bg-nt-io-500/12 text-nt-io-600 flex items-center justify-center text-[14px] font-semibold flex-shrink-0">
                                {activeProvider()?.name.charAt(0).toUpperCase() ?? '?'}
                              </span>
                              <div>
                                <div class="text-[13px] font-medium text-text-primary">{activeProvider()?.name ?? '—'}</div>
                                <div class="text-[11px] text-text-muted font-mono">{cfg().active_model}</div>
                              </div>
                            </div>
                            <span class={clsx('text-[10px] px-2 py-1 rounded-full font-medium', cfg().resolvable ? 'bg-nt-core-500/10 text-nt-core-700' : 'bg-nt-shield-500/10 text-nt-shield-600')}>
                              {cfg().resolvable ? 'API 可达' : 'API 不可达'}
                            </span>
                          </div>
                        </div>
                      </div>

                      {/* 提供商列表 */}
                      <div class="ss-card">
                        <div class="ss-card-header">
                          <DataIcon />
                          {cfg().provider_count} 个可用提供商
                        </div>
                        <div class="ss-card-body">
                          <div class="space-y-2">
                            <For each={cfg().providers}>
                              {(p) => {
                                const isActive = p.model === cfg().active_model
                                return (
                                  <button
                                    class={clsx(
                                      'w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors',
                                      isActive
                                        ? 'border-nt-io-500/40 bg-nt-io-500/6'
                                        : 'border-border-primary/50 bg-white/40 hover:bg-white/70'
                                    )}
                                    onClick={() => !isActive && switchProvider(p.name)}
                                    disabled={switching()}
                                    role="radio"
                                    aria-checked={isActive}
                                  >
                                    <div class="flex items-center gap-3">
                                      <span class="text-[12.5px] text-text-primary font-medium">{p.name}</span>
                                      <span class="text-[10.5px] text-text-muted font-mono">{p.model}</span>
                                    </div>
                                    <div class="flex items-center gap-2">
                                      <Show when={p.resolvable}>
                                        <span class="text-[9px] text-nt-core-700 bg-nt-core-500/10 px-2 py-1 rounded-full">可用</span>
                                      </Show>
                                      <Show when={isActive}>
                                        <span class="text-[10px] text-nt-io-600">✓ 当前</span>
                                      </Show>
                                    </div>
                                  </button>
                                )
                              }}
                            </For>
                          </div>
                        </div>
                      </div>

                      {/* API 密钥管理（对标 Claude 设置） */}
                      <div class="ss-card">
                        <div class="ss-card-header">
                          <InfoIcon />
                          API 密钥
                        </div>
                        <div class="ss-card-body space-y-3">
                          <div class="flex items-center gap-2">
                            <input
                              type="password"
                              class="flex-1 min-w-0 px-3 py-2 rounded-lg bg-white/70 border border-border-primary text-[12.5px] text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
                              placeholder={hasKey() === false ? '输入 API 密钥…' : '输入新密钥替换…'}
                              value={apiKey()}
                              onInput={(e) => setApiKey(e.currentTarget.value)}
                              onKeyDown={(e) => { if (e.key === 'Enter') saveApiKey() }}
                              aria-label="API 密钥"
                            />
                            <button
                              class="px-3 py-2 rounded-lg bg-nt-io-500 text-text-primary text-[12px] font-medium hover:bg-nt-io-600 disabled:opacity-50 transition-colors flex-shrink-0"
                              onClick={saveApiKey}
                              disabled={keyBusy() || !apiKey().trim()}
                            >
                              保存
                            </button>
                          </div>
                          <div class="flex items-center justify-between">
                            <span class={clsx('text-[11px]', hasKey() === true ? 'text-nt-core-700' : 'text-text-muted')}>
                              {hasKey() === true ? '✓ 已配置 API 密钥' : hasKey() === false ? '未配置 API 密钥' : '检测中…'}
                            </span>
                            <Show when={hasKey() === true}>
                              <button
                                class="px-3 py-1 rounded-lg border border-red-500/30 bg-red-500/5 text-[11px] text-red-500 hover:bg-red-500/10 disabled:opacity-50 transition-colors"
                                onClick={deleteApiKey}
                                disabled={keyBusy()}
                              >
                                删除
                              </button>
                            </Show>
                          </div>
                        </div>
                      </div>
                    </div>
                  )}
                </Show>
              </Show>

              {/* ── 外观：主题 / 字号 / 动效 / 密度 ── */}
              <Show when={section() === 'appearance'}>
                <div class="space-y-4">
                  <div class="ss-card">
                    <div class="ss-card-header">
                      <PaletteIcon />
                      主题
                    </div>
                    <div class="ss-card-body">
                      <div class="ss-row">
                        <div>
                          <div class="ss-row-label">雪域白 · 浅橙</div>
                          <div class="ss-row-desc">唯一主题 · 极简 Mac 圆角</div>
                        </div>
                        <span class="text-[10px] text-nt-io-600">✓ 当前</span>
                      </div>
                    </div>
                  </div>

                  <div class="ss-card">
                    <div class="ss-card-header">
                      <ExpandIcon />
                      界面字号
                    </div>
                    <div class="ss-card-body space-y-2">
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', fontSizePref() === 'sm' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setFontSize('sm')}
                        role="radio"
                        aria-checked={fontSizePref() === 'sm'}
                      >
                        <div class="text-[12.5px] text-text-primary">小</div>
                        <Show when={fontSizePref() === 'sm'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', fontSizePref() === 'md' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setFontSize('md')}
                        role="radio"
                        aria-checked={fontSizePref() === 'md'}
                      >
                        <div class="text-[12.5px] text-text-primary">中</div>
                        <Show when={fontSizePref() === 'md'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', fontSizePref() === 'lg' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setFontSize('lg')}
                        role="radio"
                        aria-checked={fontSizePref() === 'lg'}
                      >
                        <div class="text-[12.5px] text-text-primary">大</div>
                        <Show when={fontSizePref() === 'lg'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                    </div>
                  </div>

                  <div class="ss-card">
                    <div class="ss-card-header">
                      <DataIcon />
                      动效强度
                    </div>
                    <div class="ss-card-body space-y-2">
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', motionPref() === 'full' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setMotion('full')}
                        role="radio"
                        aria-checked={motionPref() === 'full'}
                      >
                        <div class="text-[12.5px] text-text-primary">完整动效</div>
                        <Show when={motionPref() === 'full'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', motionPref() === 'reduced' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setMotion('reduced')}
                        role="radio"
                        aria-checked={motionPref() === 'reduced'}
                      >
                        <div class="text-[12.5px] text-text-primary">减弱动效</div>
                        <Show when={motionPref() === 'reduced'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                    </div>
                    <div class="px-4 pb-3 -mt-1">
                      <p class="text-[10.5px] text-text-muted">减弱后移除无限循环动画，减少视觉干扰</p>
                    </div>
                  </div>

                  <div class="ss-card">
                    <div class="ss-card-header">
                      <InfoIcon />
                      界面密度
                    </div>
                    <div class="ss-card-body space-y-2">
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', densityPref() === 'comfortable' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setDensity('comfortable')}
                        role="radio"
                        aria-checked={densityPref() === 'comfortable'}
                      >
                        <div class="text-[12.5px] text-text-primary">舒适</div>
                        <Show when={densityPref() === 'comfortable'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', densityPref() === 'compact' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setDensity('compact')}
                        role="radio"
                        aria-checked={densityPref() === 'compact'}
                      >
                        <div class="text-[12.5px] text-text-primary">紧凑</div>
                        <Show when={densityPref() === 'compact'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                    </div>
                    <div class="px-4 pb-3 -mt-1">
                      <p class="text-[10.5px] text-text-muted">紧凑模式缩小消息间距与面板内边距，单屏承载更多信息</p>
                    </div>
                  </div>
                </div>
              </Show>

              {/* ── 插件：技能插件市场（内嵌，试错/扩展入口） ── */}
              <Show when={section() === 'plugins'}>
                <PluginMarketplace embedded open onClose={() => {}} />
              </Show>

              {/* ── 数据：记忆统计 + 导出/清空 ── */}
              <Show when={section() === 'data'}>
                <div class="space-y-4">
                  <div class="ss-card">
                    <div class="ss-card-header">
                      <DataIcon />
                      记忆统计
                    </div>
                    <div class="ss-card-body">
                      <Show when={memStats()} fallback={<div class="text-xs text-text-muted py-2">加载记忆统计…</div>}>
                        {(ms) => (
                          <div class="grid grid-cols-2 gap-2">
                            <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                              <div class="text-[10px] text-text-muted mb-1">记忆条目</div>
                              <div class="text-[13px] text-text-primary font-medium">{ms().total_entries}</div>
                            </div>
                            <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                              <div class="text-[10px] text-text-muted mb-1">分类</div>
                              <div class="text-[13px] text-text-primary font-medium">{ms().total_categories}</div>
                            </div>
                            <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                              <div class="text-[10px] text-text-muted mb-1">平均置信度</div>
                              <div class="text-[13px] text-text-primary font-medium">{(ms().avg_confidence * 100).toFixed(0)}%</div>
                            </div>
                            <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                              <div class="text-[10px] text-text-muted mb-1">占用空间</div>
                              <div class="text-[13px] text-text-primary font-medium">{(ms().memory_usage_bytes / 1024).toFixed(1)} KB</div>
                            </div>
                          </div>
                        )}
                      </Show>
                    </div>
                  </div>

                  <div class="ss-card">
                    <div class="ss-card-header">
                      <ExpandIcon />
                      数据操作
                    </div>
                    <div class="ss-card-body space-y-2">
                      <button
                        class="w-full flex items-center justify-between px-3 py-3 rounded-xl border border-border-primary/50 bg-white/40 hover:bg-white/70 transition-colors"
                        onClick={exportMemory}
                        disabled={dataBusy()}
                      >
                        <span class="text-[12.5px] text-text-primary">导出记忆（JSON）</span>
                        <span class="text-[10px] text-text-muted">→ 文件</span>
                      </button>
                      <button
                        class="w-full flex items-center justify-between px-3 py-3 rounded-xl border border-red-500/30 bg-red-500/5 hover:bg-red-500/10 transition-colors"
                        onClick={clearMemory}
                        disabled={dataBusy()}
                      >
                        <span class="text-[12.5px] text-red-500">清空全部记忆</span>
                        <span class="text-[10px] text-red-400">不可恢复</span>
                      </button>
                    </div>
                  </div>
                </div>
              </Show>

              {/* ── 关于 ── */}
              <Show when={section() === 'about'}>
                <div class="space-y-4">
                  <div class="ss-card">
                    <div class="ss-card-header">
                      <InfoIcon />
                      NeoTrix Desktop
                    </div>
                    <div class="ss-card-body">
                      <div class="flex items-center gap-4">
                        <span class="w-12 h-12 rounded-xl bg-nt-io-500/12 text-nt-io-600 flex items-center justify-center flex-shrink-0">
                          <ExpandIcon />
                        </span>
                        <div>
                          <div class="text-[14px] font-semibold text-text-primary">NeoTrix Desktop</div>
                          <div class="text-[11px] text-text-muted font-mono">v{appVersion() ?? '0.18.0'} · ai.neotrix.desktop</div>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div class="ss-card">
                    <div class="ss-card-header">
                      <DataIcon />
                      诊断信息
                    </div>
                    <div class="ss-card-body">
                      <div class="grid grid-cols-2 gap-2">
                        <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                          <div class="text-[10px] text-text-muted mb-1">提供商</div>
                          <div class="text-[12.5px] text-text-primary font-medium">{config()?.provider_count ?? '—'} 个</div>
                        </div>
                        <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                          <div class="text-[10px] text-text-muted mb-1">API 状态</div>
                          <div class={clsx('text-[12.5px] font-medium', config()?.resolvable ? 'text-nt-core-700' : 'text-nt-shield-600')}>
                            {config()?.resolvable ? '可用' : '不可达'}
                          </div>
                        </div>
                        <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                          <div class="text-[10px] text-text-muted mb-1">当前模型</div>
                          <div class="text-[12.5px] text-text-primary font-mono truncate">{config()?.active_model ?? '—'}</div>
                        </div>
                        <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                          <div class="text-[10px] text-text-muted mb-1">平台</div>
                          <div class="text-[12.5px] text-text-primary">macOS</div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </Show>
            </div>

            {/* 底部通知条 */}
            <Show when={notice()}>
              <div class="px-5 py-2 border-t border-border-primary/40 text-[11px] text-text-secondary flex-shrink-0">
                {notice()}
              </div>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  )
}