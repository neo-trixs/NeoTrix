import { createSignal, createEffect, onMount, onCleanup, For, Show } from 'solid-js'
import { Dynamic } from 'solid-js/web'
import { ChevronDown, Loader2, Check, AlertCircle } from 'lucide-solid'
import { clsx } from 'clsx'
import { ProviderIcon, CategoryBadge, FreeBadge } from './ProviderIcon'
import { neocodex, errText } from '../api'
import type { ProviderConfig, ProviderMeta } from '../api/types'

/* ════════════════════════════════════════════
   ModelSwitcher — 模型切换（仅模型，权限模式已移除）
   对标 OpenWebUI 顶栏模型选择：单药丸展示当前模型，下拉为模型池。
   默认展示「neotrix意识核心模型」（provider 未加载时的品牌占位）。
   ════════════════════════════════════════════ */

const DEFAULT_MODEL_LABEL = 'neotrix意识核心模型'

export function ModelSwitcher(props: {
  disabled?: boolean
}) {
  const [config, setConfig] = createSignal<ProviderConfig | null>(null)
  const [isOpen, setIsOpen] = createSignal(false)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  onMount(async () => {
    await loadConfig()
    window.addEventListener('neotrix:provider-changed', handleProviderChanged)
    window.addEventListener('keydown', handleEsc)
  })
  onCleanup(() => {
    window.removeEventListener('neotrix:provider-changed', handleProviderChanged)
    window.removeEventListener('keydown', handleEsc)
  })

  const handleEsc = (e: KeyboardEvent) => { if (e.key === 'Escape') setIsOpen(false) }
  const handleProviderChanged = () => loadConfig()

  const loadConfig = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await neocodex.providerConfig()
      setConfig(result)
    } catch (err) {
      setError(errText(err) || '获取模型配置失败')
      console.error('[ModelSwitcher] load failed:', err)
    } finally {
      setLoading(false)
    }
  }

  const handleSelectProvider = async (name: string) => {
    setLoading(true)
    setError(null)
    try {
      await neocodex.setProvider(name)
      await loadConfig()
      setIsOpen(false)
      window.dispatchEvent(new CustomEvent('neotrix:provider-changed', { detail: { name } }))
    } catch (err) {
      setError(errText(err) || '切换模型失败')
    } finally {
      setLoading(false)
    }
  }

  const currentProvider = (): ProviderMeta | null => {
    const cfg = config()
    if (!cfg) return null
    return cfg.providers.find((p) => p.model === cfg.active_model) || cfg.providers[0] || null
  }

  const pillModel = () => {
    const p = currentProvider()
    if (!p) return DEFAULT_MODEL_LABEL
    const short = p.model.split('/').pop() || p.display_name
    return short.length > 14 ? short.slice(0, 14) + '…' : short
  }

  let panelRef: HTMLDivElement | undefined
  createEffect(() => {
    if (!isOpen() || loading()) return
    requestAnimationFrame(() => {
      const opts = Array.from(panelRef?.querySelectorAll<HTMLElement>('[role="option"]') ?? [])
      if (opts.length === 0) return
      const active = opts.find((o) => o.getAttribute('aria-selected') === 'true')
      const target = active && !active.hasAttribute('disabled') ? active : opts.find((o) => !o.hasAttribute('disabled'))
      target?.focus()
    })
  })

  return (
    <div class="relative">
      <button
        class={clsx(
          'inline-flex items-center gap-1 pl-1 pr-1.5 py-1 rounded-md text-[12px] font-medium text-text-secondary hover:text-nt-io-700 hover:bg-white/40 transition-all focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none',
          loading() && 'opacity-50 cursor-wait',
          props.disabled && 'opacity-50 cursor-not-allowed',
        )}
        onClick={() => !loading() && !props.disabled && setIsOpen(!isOpen())}
        disabled={loading() || props.disabled}
        aria-label="模型切换"
        aria-expanded={isOpen()}
        aria-haspopup="listbox"
        title={pillModel()}
      >
        {loading() ? (
          <Loader2 class="w-3.5 h-3.5 animate-spin text-orange-500" />
        ) : currentProvider() ? (
          <ProviderIcon name={currentProvider()!.name} size="sm" />
        ) : (
          <span class="w-4 h-4 rounded-full bg-orange-500/15 text-orange-600 flex items-center justify-center text-[9px] font-bold">N</span>
        )}
        <span class="max-w-[110px] truncate">{pillModel()}</span>
        <ChevronDown class={clsx('w-3 h-3 text-zinc-400 flex-shrink-0 transition-transform', isOpen() && 'rotate-180')} />
      </button>

      {error() && (
        <div class="absolute top-full left-0 right-0 mt-1 p-2 bg-red-500/20 border border-red-500/30 rounded-lg text-xs text-red-300 flex items-center gap-2 animate-in z-10">
          <AlertCircle class="w-3.5 h-3.5 flex-shrink-0" />
          {error()}
          <button onClick={() => setError(null)} class="ml-auto p-1 hover:bg-red-500/30 rounded">×</button>
        </div>
      )}

      <Show when={isOpen()}>
        <div ref={panelRef} class="absolute bottom-full left-0 mb-2 glass-pop border border-black/8 rounded-2xl shadow-2xl overflow-hidden z-50 animate-in w-[min(380px,92vw)]">
          <div class="px-3 py-2 border-b border-white/40 text-[11px] font-medium text-text-muted">
            选择模型
          </div>
          <div class="max-h-64 overflow-y-auto" role="listbox" aria-label="模型列表">
            <For each={config()?.providers || []}>
              {(provider: ProviderMeta, i) => {
                const isActive = () => provider.model === config()?.active_model
                return (
                  <button
                    class={clsx(
                      'w-full flex items-center gap-3 px-3 py-3 text-left transition-colors hover:bg-bg-tertiary focus-visible:bg-bg-tertiary focus-visible:outline-none',
                      isActive() && 'bg-nt-io-500/10 text-nt-io-600',
                    )}
                    onClick={() => handleSelectProvider(provider.name)}
                    disabled={loading() || isActive()}
                    role="option"
                    aria-selected={isActive()}
                    onKeyDown={(e) => {
                      if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return
                      e.preventDefault()
                      const list = config()?.providers || []
                      if (list.length === 0) return
                      requestAnimationFrame(() => {
                        const opts = Array.from(panelRef?.querySelectorAll<HTMLElement>('[role="option"]') ?? [])
                        opts[(i() + (e.key === 'ArrowDown' ? 1 : -1) + list.length) % list.length]?.focus?.()
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
                    {!provider.resolvable && <span class="text-xs text-amber-600 px-2 py-1 rounded bg-amber-500/10 flex-shrink-0">不可用</span>}
                  </button>
                )
              }}
            </For>
            {(config()?.providers?.length || 0) === 0 && !loading() && (
              <div class="px-3 py-6 text-center text-text-muted text-sm">暂无可用模型</div>
            )}
          </div>

          <div class="px-3 py-2 border-t border-white/40 text-xs text-text-muted">
            当前模型: <span class="font-mono text-text-secondary">{config()?.active_model || DEFAULT_MODEL_LABEL}</span>
          </div>
        </div>
      </Show>

      <div class={isOpen() ? 'fixed inset-0 z-40' : 'hidden'} onClick={() => setIsOpen(false)} aria-hidden="true" />
    </div>
  )
}
