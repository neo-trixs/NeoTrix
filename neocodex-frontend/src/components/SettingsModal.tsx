import { createSignal, createEffect, For, Show } from 'solid-js'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { clsx } from 'clsx'

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

type SectionId = 'general' | 'appearance' | 'data' | 'about'

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
  { id: 'data', label: '数据', icon: DataIcon },
  { id: 'about', label: '关于', icon: InfoIcon },
]

export function SettingsModal(props: { open: boolean; onClose: () => void }) {
  const [section, setSection] = createSignal<SectionId>('general')
  const [config, setConfig] = createSignal<ProviderConfig | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [switching, setSwitching] = createSignal(false)
  const [notice, setNotice] = createSignal<string | null>(null)
  const [motionPref, setMotionPref] = createSignal<'full' | 'reduced'>('full')
  const [densityPref, setDensityPref] = createSignal<'comfortable' | 'compact'>('comfortable')
  const [memStats, setMemStats] = createSignal<{ total_entries: number; total_categories: number; avg_confidence: number; memory_usage_bytes: number } | null>(null)
  const [dataBusy, setDataBusy] = createSignal(false)
  const [appVersion, setAppVersion] = createSignal<string | null>(null)

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

  createEffect(() => {
    if (props.open) {
      setSection('general')
      setNotice(null)
      loadConfig()
      loadMemStats()
      loadAppVersion()
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

  return (
    <Show when={props.open}>
      <div
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/25 backdrop-blur-[2px] animate-fade-in"
        onClick={props.onClose}
      >
        <div
          class="w-[560px] max-w-[92vw] h-[420px] max-h-[80vh] rounded-2xl bg-bg-primary shadow-2xl border border-border-primary/50 overflow-hidden flex animate-slide-in"
          onClick={(e) => e.stopPropagation()}
          role="dialog"
          aria-label="设置"
        >
          {/* ── 左侧分类导航 ── */}
          <nav class="w-[148px] flex-shrink-0 border-r border-border-primary/40 bg-bg-secondary/60 py-4 px-2 flex flex-col gap-1">
            <div class="px-3 pb-3 text-[10px] uppercase tracking-[0.14em] text-text-muted/70 font-medium">设置</div>
            <For each={SECTIONS}>
              {(s) => (
                <button
                  class={clsx(
                    'flex items-center gap-2.5 px-3 py-2 rounded-lg text-[12.5px] transition-colors',
                    section() === s.id
                      ? 'bg-white/70 text-text-primary font-medium shadow-[inset_0_1px_0_rgba(255,255,255,0.8)]'
                      : 'text-text-secondary hover:text-text-primary hover:bg-white/40'
                  )}
                  onClick={() => setSection(s.id)}
                  aria-current={section() === s.id ? 'true' : 'false'}
                >
                  <span class={clsx('w-4 h-4 flex-shrink-0', section() === s.id ? 'text-nt-io-600' : 'text-text-muted')}>
                    <s.icon />
                  </span>
                  {s.label}
                </button>
              )}
            </For>
          </nav>

          {/* ── 右侧内容 ── */}
          <div class="flex-1 flex flex-col min-w-0">
            <header class="flex items-center justify-between px-5 py-3.5 border-b border-border-primary/40 flex-shrink-0">
              <div>
                <div class="text-[15px] font-semibold text-text-primary">
                  {SECTIONS.find((s) => s.id === section())?.label}
                </div>
                <div class="text-[11px] text-text-muted">
                  {section() === 'general' && '模型提供商与运行参数'}
                  {section() === 'appearance' && '界面视觉与动效'}
                  {section() === 'data' && '记忆与数据管理'}
                  {section() === 'about' && '版本与诊断信息'}
                </div>
              </div>
              <button
                class="p-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors"
                onClick={props.onClose}
                aria-label="关闭设置"
                title="关闭设置"
              >
                <XIcon />
              </button>
            </header>

            <div class="flex-1 overflow-y-auto px-5 py-4">
              {/* ── 通用：模型提供商 ── */}
              <Show when={section() === 'general'}>
                <Show when={loading() && !config()}>
                  <div class="text-xs text-text-muted py-6 text-center">加载配置…</div>
                </Show>
                <Show when={config()}>
                  {(cfg) => (
                    <div class="space-y-5">
                      {/* 当前激活提供商 */}
                      <div>
                        <div class="text-[11px] font-medium text-text-secondary mb-2">当前提供商</div>
                        <div class="flex items-center justify-between p-3 rounded-xl bg-white/60 border border-border-primary/50">
                          <div class="flex items-center gap-3">
                            <span class="w-7 h-7 rounded-lg bg-nt-io-500/12 text-nt-io-600 flex items-center justify-center text-[13px] font-semibold flex-shrink-0">
                              {activeProvider()?.name.charAt(0).toUpperCase() ?? '?'}
                            </span>
                            <div>
                              <div class="text-[13px] font-medium text-text-primary">{activeProvider()?.name ?? '—'}</div>
                              <div class="text-[11px] text-text-muted font-mono">{cfg().active_model}</div>
                            </div>
                          </div>
                          <span class={clsx('text-[10px] px-2 py-0.5 rounded-full font-medium', cfg().resolvable ? 'bg-nt-core-500/10 text-nt-core-700' : 'bg-nt-shield-500/10 text-nt-shield-600')}>
                            {cfg().resolvable ? 'API 可达' : 'API 不可达'}
                          </span>
                        </div>
                      </div>

                      {/* 提供商列表 */}
                      <div>
                        <div class="text-[11px] font-medium text-text-secondary mb-2">
                          {cfg().provider_count} 个可用提供商
                        </div>
                        <div class="space-y-1.5">
                          <For each={cfg().providers}>
                            {(p) => {
                              const isActive = p.model === cfg().active_model
                              return (
                                <button
                                  class={clsx(
                                    'w-full flex items-center justify-between px-3 py-2.5 rounded-xl border transition-colors',
                                    isActive
                                      ? 'border-nt-io-500/40 bg-nt-io-500/6'
                                      : 'border-border-primary/50 bg-white/40 hover:bg-white/70'
                                  )}
                                  onClick={() => !isActive && switchProvider(p.name)}
                                  disabled={switching()}
                                >
                                  <div class="flex items-center gap-2.5">
                                    <span class="text-[12.5px] text-text-primary font-medium">{p.name}</span>
                                    <span class="text-[10.5px] text-text-muted font-mono">{p.model}</span>
                                  </div>
                                  <div class="flex items-center gap-2">
                                    <Show when={p.resolvable}>
                                      <span class="text-[9px] text-nt-core-700 bg-nt-core-500/10 px-1.5 py-0.5 rounded-full">可用</span>
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
                  )}
                </Show>
              </Show>

              {/* ── 外观：动效 ── */}
              <Show when={section() === 'appearance'}>
                <div class="space-y-5">
                  <div>
                    <div class="text-[11px] font-medium text-text-secondary mb-2">动效强度</div>
                    <div class="space-y-1.5">
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-2.5 rounded-xl border transition-colors', motionPref() === 'full' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setMotionPref('full')}
                      >
                        <div class="text-[12.5px] text-text-primary">完整动效</div>
                        <Show when={motionPref() === 'full'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-2.5 rounded-xl border transition-colors', motionPref() === 'reduced' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setMotionPref('reduced')}
                      >
                        <div class="text-[12.5px] text-text-primary">减弱动效</div>
                        <Show when={motionPref() === 'reduced'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                    </div>
                    <p class="text-[10.5px] text-text-muted mt-1.5">减弱后移除无限循环动画，减少视觉干扰</p>
                  </div>

                  <div>
                    <div class="text-[11px] font-medium text-text-secondary mb-2">界面密度</div>
                    <div class="space-y-1.5">
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-2.5 rounded-xl border transition-colors', densityPref() === 'comfortable' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setDensityPref('comfortable')}
                      >
                        <div class="text-[12.5px] text-text-primary">舒适</div>
                        <Show when={densityPref() === 'comfortable'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-2.5 rounded-xl border transition-colors', densityPref() === 'compact' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setDensityPref('compact')}
                      >
                        <div class="text-[12.5px] text-text-primary">紧凑</div>
                        <Show when={densityPref() === 'compact'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                    </div>
                    <p class="text-[10.5px] text-text-muted mt-1.5">紧凑模式缩小消息间距与面板内边距，单屏承载更多信息</p>
                  </div>
                </div>
              </Show>

              {/* ── 数据：记忆统计 + 导出/清空 ── */}
              <Show when={section() === 'data'}>
                <div class="space-y-5">
                  <div>
                    <div class="text-[11px] font-medium text-text-secondary mb-2">记忆统计</div>
                    <Show when={memStats()} fallback={<div class="text-xs text-text-muted py-2">加载记忆统计…</div>}>
                      {(ms) => (
                        <div class="grid grid-cols-2 gap-2">
                          <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                            <div class="text-[10px] text-text-muted mb-0.5">记忆条目</div>
                            <div class="text-[13px] text-text-primary font-medium">{ms().total_entries}</div>
                          </div>
                          <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                            <div class="text-[10px] text-text-muted mb-0.5">分类</div>
                            <div class="text-[13px] text-text-primary font-medium">{ms().total_categories}</div>
                          </div>
                          <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                            <div class="text-[10px] text-text-muted mb-0.5">平均置信度</div>
                            <div class="text-[13px] text-text-primary font-medium">{(ms().avg_confidence * 100).toFixed(0)}%</div>
                          </div>
                          <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                            <div class="text-[10px] text-text-muted mb-0.5">占用空间</div>
                            <div class="text-[13px] text-text-primary font-medium">{(ms().memory_usage_bytes / 1024).toFixed(1)} KB</div>
                          </div>
                        </div>
                      )}
                    </Show>
                  </div>

                  <div>
                    <div class="text-[11px] font-medium text-text-secondary mb-2">数据操作</div>
                    <div class="space-y-1.5">
                      <button
                        class="w-full flex items-center justify-between px-3 py-2.5 rounded-xl border border-border-primary/50 bg-white/40 hover:bg-white/70 transition-colors"
                        onClick={exportMemory}
                        disabled={dataBusy()}
                      >
                        <span class="text-[12.5px] text-text-primary">导出记忆（JSON）</span>
                        <span class="text-[10px] text-text-muted">→ 文件</span>
                      </button>
                      <button
                        class="w-full flex items-center justify-between px-3 py-2.5 rounded-xl border border-red-500/30 bg-red-500/5 hover:bg-red-500/10 transition-colors"
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
                  <div class="flex items-center gap-4 p-4 rounded-xl bg-white/60 border border-border-primary/50">
                    <span class="w-12 h-12 rounded-xl bg-nt-io-500/12 text-nt-io-600 flex items-center justify-center flex-shrink-0">
                      <ExpandIcon />
                    </span>
                    <div>
                      <div class="text-[14px] font-semibold text-text-primary">NeoTrix Desktop</div>
                      <div class="text-[11px] text-text-muted font-mono">v{appVersion() ?? '0.18.0'} · ai.neotrix.desktop</div>
                    </div>
                  </div>
                  <div class="grid grid-cols-2 gap-2">
                    <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                      <div class="text-[10px] text-text-muted mb-0.5">提供商</div>
                      <div class="text-[12.5px] text-text-primary font-medium">{config()?.provider_count ?? '—'} 个</div>
                    </div>
                    <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                      <div class="text-[10px] text-text-muted mb-0.5">API 状态</div>
                      <div class={clsx('text-[12.5px] font-medium', config()?.resolvable ? 'text-nt-core-700' : 'text-nt-shield-600')}>
                        {config()?.resolvable ? '可用' : '不可达'}
                      </div>
                    </div>
                    <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                      <div class="text-[10px] text-text-muted mb-0.5">当前模型</div>
                      <div class="text-[12.5px] text-text-primary font-mono truncate">{config()?.active_model ?? '—'}</div>
                    </div>
                    <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                      <div class="text-[10px] text-text-muted mb-0.5">平台</div>
                      <div class="text-[12.5px] text-text-primary">macOS</div>
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