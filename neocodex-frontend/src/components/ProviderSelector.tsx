import { createSignal, createEffect, onMount, onCleanup, For, Show } from 'solid-js'
import { ChevronDown, Loader2, Check, AlertCircle } from 'lucide-solid'
import { clsx } from 'clsx'
import { ProviderIcon, CategoryBadge, FreeBadge } from './ProviderIcon'
import { neocodex, errText } from '../api'
import type { ProviderConfig } from '../api/types'

/* ════════════════════════════════════════════
   ProviderSelector — 模型提供商选择器（统一 v2）
   与 SettingsModal 共用 ProviderIcon/CategoryBadge 视觉语言
   - 触发按钮：品牌 monogram + 显示名 + 分类徽章
   - 下拉：分类徽章 + 免费徽章 + 可达性标识
   ════════════════════════════════════════════ */

export function ProviderSelector(props: { iconOnly?: boolean }) {
  const [config, setConfig] = createSignal<ProviderConfig | null>(null)
  const [isOpen, setIsOpen] = createSignal(false)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  // Load provider config on mount
  onMount(async () => {
    await loadConfig()
    // 设置面板切换提供商后同步刷新
    window.addEventListener('neotrix:provider-changed', handleProviderChanged)
    // Esc 关闭下拉（对标 Codex 下拉规范）
    window.addEventListener('keydown', handleEsc)
  })

  onCleanup(() => {
    window.removeEventListener('neotrix:provider-changed', handleProviderChanged)
    window.removeEventListener('keydown', handleEsc)
  })

  const handleEsc = (e: KeyboardEvent) => {
    if (e.key === 'Escape') setIsOpen(false)
  }

  const handleProviderChanged = () => {
    loadConfig()
  }

  const loadConfig = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await neocodex.providerConfig()
      setConfig(result)
    } catch (err) {
      const msg = errText(err) || '获取提供商配置失败'
      setError(msg)
      console.error('[ProviderSelector] Failed to load config:', err)
    } finally {
      setLoading(false)
    }
  }

  const handleSelectProvider = async (providerName: string) => {
    setLoading(true)
    setError(null)
    try {
      await neocodex.setProvider(providerName)
      await loadConfig()
      setIsOpen(false)
      // 广播提供商变更，Chat 状态栏 / 其他监听方即时刷新
      window.dispatchEvent(new CustomEvent('neotrix:provider-changed', { detail: { name: providerName } }))
    } catch (err) {
      const msg = errText(err) || '切换提供商失败'
      setError(msg)
      console.error('[ProviderSelector] Failed to switch provider:', err)
    } finally {
      setLoading(false)
    }
  }

  const currentProvider = () => {
    const cfg = config()
    if (!cfg) return null
    return cfg.providers.find(p => p.model === cfg.active_model) || cfg.providers[0] || null
  }

  // 下拉面板 ref：焦点管理 / 方向键导航均限定在本面板内（避免多 listbox 全局串扰）
  let panelRef: HTMLDivElement | undefined

  // 打开后焦点移入列表并高亮当前项（对标 Codex 下拉规范）：优先聚焦当前激活项，
  // 激活项 disabled（不可再选）时回退到首个可用项
  createEffect(() => {
    if (!isOpen() || loading()) return
    requestAnimationFrame(() => {
      const opts = Array.from(panelRef?.querySelectorAll<HTMLElement>('[role="option"]') ?? [])
      if (opts.length === 0) return
      const active = opts.find(o => o.getAttribute('aria-selected') === 'true')
      const target = active && !active.hasAttribute('disabled')
        ? active
        : opts.find(o => !o.hasAttribute('disabled'))
      target?.focus()
    })
  })

  // Claude Code 风格：iconOnly 时展示药丸标签（图标+短模型名+chevron），而非纯圆点
  const pillLabel = () => {
    const p = currentProvider()
    if (!p) return '选择模型'
    // 显示短模型名（取最后一个 / 后的段）
    const short = p.model.split('/').pop() || p.display_name
    return short.length > 14 ? short.slice(0, 14) + '…' : short
  }

  return (
    <div class="relative">
      {/* Trigger Button — iconOnly Claude Code 药丸 / 完整模式 */}
      <button
        class={clsx(
          props.iconOnly
            ? 'inline-flex items-center gap-1.5 pl-1.5 pr-2.5 py-1.5 rounded-full bg-white border border-black/8 shadow-sm text-[12px] font-medium text-zinc-700 hover:border-orange-200 hover:bg-orange-50/60 hover:text-orange-700 transition-all'
            : 'flex items-center gap-2 px-3 py-2 rounded-xl border border-black/8 bg-white text-zinc-700 hover:bg-orange-50/40 hover:border-orange-200 transition-colors min-w-[180px] max-w-[280px] shadow-sm focus-visible:ring-2 focus-visible:ring-orange-400 focus-visible:outline-none',
          loading() && 'opacity-50 cursor-wait'
        )}
        onClick={() => !loading() && setIsOpen(!isOpen())}
        disabled={loading()}
        aria-label="选择模型提供商"
        aria-expanded={isOpen()}
        aria-haspopup="listbox"
        title={props.iconOnly ? (currentProvider() ? `${currentProvider()!.display_name} · ${currentProvider()!.model}` : '选择模型') : undefined}
      >
        {props.iconOnly ? (
          loading() ? (
            <Loader2 class="w-3.5 h-3.5 animate-spin text-orange-500" />
          ) : currentProvider() ? (
            <>
              <ProviderIcon name={currentProvider()!.name} size="sm" />
              <span class="max-w-[90px] truncate">{pillLabel()}</span>
              <ChevronDown class={clsx('w-3 h-3 text-zinc-400 flex-shrink-0 transition-transform', isOpen() && 'rotate-180')} />
            </>
          ) : (
            <><span class="w-5 h-5 rounded-full bg-orange-500/15 text-orange-600 flex items-center justify-center text-[10px] font-bold">?</span><span>选择模型</span></>
          )
        ) : (
          <>
            <span class="flex items-center gap-2 text-sm truncate">
              {loading() ? (
                <Loader2 class="w-4 h-4 animate-spin text-nt-io-500" />
              ) : currentProvider() ? (
                <>
                  <ProviderIcon name={currentProvider()!.name} />
                  <span class="font-medium truncate">{currentProvider()!.display_name}</span>
                  <CategoryBadge category={currentProvider()!.category} className="hidden md:inline-flex" />
                </>
              ) : (
                <span class="text-text-muted">选择提供商</span>
              )}
            </span>
            <ChevronDown class={clsx('w-4 h-4 text-text-muted flex-shrink-0 transition-transform', isOpen() && 'rotate-180')} />
          </>
        )}
      </button>

      {/* Error Toast */}
      {error() && (
        <div class="absolute top-full left-0 right-0 mt-1 p-2 bg-red-500/20 border border-red-500/30 rounded-lg text-xs text-red-300 flex items-center gap-2 animate-in z-10">
          <AlertCircle class="w-3.5 h-3.5 flex-shrink-0" />
          {error()}
          <button onClick={() => setError(null)} class="ml-auto p-1 hover:bg-red-500/30 rounded">×</button>
        </div>
      )}

      {/* Dropdown Panel — 对标 Claude Code 居中命令面板式模型选择 */}
      <Show when={isOpen()}>
        <div ref={panelRef} class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 glass-pop border border-black/8 rounded-2xl shadow-2xl overflow-hidden z-50 animate-in min-w-[340px] max-w-[420px] w-[min(420px,90vw)]">
          {/* Header */}
          <div class="px-3 py-2 border-b border-white/40 flex items-center justify-between">
            <span class="text-sm font-medium text-text-primary">模型提供商</span>
            <span class="text-xs text-text-muted">{config()?.provider_count || 0} 个可用</span>
          </div>

          {/* Provider List */}
          <div class="max-h-64 overflow-y-auto" role="listbox" aria-label="模型提供商列表">
            <For each={config()?.providers || []}>
              {(provider: import('../api/types').ProviderMeta, i) => {
                // 响应式 getter：const 捕获在 For 不重跑时会失效（同 SettingsModal/ProjectView 反模式）
                const isActive = () => provider.model === config()?.active_model
                return (
                  <button
                    class={clsx(
                      'w-full flex items-center gap-3 px-3 py-3 text-left transition-colors',
                      'hover:bg-bg-tertiary focus-visible:bg-bg-tertiary focus-visible:outline-none',
                      isActive() && 'bg-nt-io-500/10 text-nt-io-600'
                    )}
                    onClick={() => handleSelectProvider(provider.name)}
                    disabled={loading() || isActive()}
                    role="option"
                    aria-selected={isActive()}
                    onKeyDown={(e) => {
                      if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return
                      e.preventDefault()
                      const dir = e.key === 'ArrowDown' ? 1 : -1
                      const list = config()?.providers || []
                      if (list.length === 0) return
                      // roving tabindex：聚焦下一选项（限定本面板内）
                      requestAnimationFrame(() => {
                        const opts = Array.from(panelRef?.querySelectorAll<HTMLElement>('[role="option"]') ?? [])
                        opts[(i() + dir + list.length) % list.length]?.focus?.()
                      })
                    }}
                  >
                    <ProviderIcon name={provider.name} size="sm" />
                    <div class="flex-1 min-w-0 flex flex-col gap-1">
                      <div class="flex items-center gap-1.5 min-w-0">
                        <span class="font-medium truncate">{provider.display_name}</span>
                        {provider.is_free && <FreeBadge free />}
                      </div>
                      <div class="flex items-center gap-1.5 min-w-0">
                        <span class="text-xs text-text-muted truncate font-mono">{provider.model}</span>
                        <CategoryBadge category={provider.category} className="hidden sm:inline-flex" />
                      </div>
                    </div>
                    {isActive() && <Check class="w-4 h-4 text-nt-io-500 flex-shrink-0" />}
                    {!provider.resolvable && (
                      <span class="text-xs text-amber-600 px-2 py-1 rounded bg-amber-500/10 flex-shrink-0">不可用</span>
                    )}
                  </button>
                )
              }}
            </For>
            {(config()?.providers?.length || 0) === 0 && !loading() && (
              <div class="px-3 py-6 text-center text-text-muted text-sm">
                暂无可用提供商
              </div>
            )}
          </div>

          {/* Footer */}
          <div class="px-3 py-2 border-t border-white/40 text-xs text-text-muted">
            当前模型: <span class="font-mono text-text-secondary">{config()?.active_model || '—'}</span>
          </div>
        </div>
      </Show>

      {/* Click outside to close */}
      <div
        class={isOpen() ? 'fixed inset-0 z-40' : 'hidden'}
        onClick={() => setIsOpen(false)}
        aria-hidden="true"
      />
    </div>
  )
}
