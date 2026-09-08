import { createSignal, createEffect, onMount, onCleanup, For, Show } from 'solid-js'
import { ChevronDown, Loader2, Check, AlertCircle } from 'lucide-solid'
import { clsx } from 'clsx'
import { ProviderIcon, CategoryBadge, FreeBadge } from './ProviderIcon'
import { neocodex, errText } from '../api'
import { getModelPoolStatus } from '../api/model-pool'
import type { ProviderConfig, ProviderMeta } from '../api/types'
import type { ModelPoolEntry } from '../api/model-pool'

/* ════════════════════════════════════════════
   ModelSwitcher — 动态模型切换
   从 config.toml + provider_pool.toml 自动加载真实模型列表。
   ════════════════════════════════════════════ */

export function ModelSwitcher(props: {
  disabled?: boolean
}) {
  const [config, setConfig] = createSignal<ProviderConfig | null>(null)
  const [poolModels, setPoolModels] = createSignal<ModelPoolEntry[]>([])
  const [isOpen, setIsOpen] = createSignal(false)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  // 合并后的完整模型列表：config.toml 活跃模型 + provider_pool.toml 池子模型
  const allModels = (): ProviderMeta[] => {
    const cfg = config()
    const pool = poolModels()
    const providers: ProviderMeta[] = []
    const seen = new Set<string>()

    // 1. config.toml 中的活跃模型（优先）
    if (cfg) {
      for (const p of cfg.providers) {
        if (!seen.has(p.model)) {
          seen.add(p.model)
          providers.push(p)
        }
      }
    }

    // 2. provider_pool.toml 中的池子模型
    for (const entry of pool) {
      if (!seen.has(entry.model)) {
        seen.add(entry.model)
        providers.push({
          id: entry.provider,
          name: entry.label,
          display_name: entry.label,
          category: entry.base_url?.includes('127.0.0.1') ? 'local' : 'cloud',
          is_free: entry.tags.includes('free') || entry.api_key_masked === 'no-key',
          base_url: entry.base_url || '',
          model: entry.model,
          models: [entry.model],
          resolvable: true,
          api_key: entry.api_key_masked,
        })
      }
    }

    return providers
  }

  onMount(async () => {
    await loadData()
    window.addEventListener('neotrix:provider-changed', handleProviderChanged)
    window.addEventListener('keydown', handleEsc)
  })
  onCleanup(() => {
    window.removeEventListener('neotrix:provider-changed', handleProviderChanged)
    window.removeEventListener('keydown', handleEsc)
  })

  const handleEsc = (e: KeyboardEvent) => { if (e.key === 'Escape') setIsOpen(false) }
  const handleProviderChanged = () => loadData()

  const loadData = async () => {
    setLoading(true)
    setError(null)
    try {
      // 并行加载 config + pool
      const [cfgResult, poolResult] = await Promise.allSettled([
        neocodex.providerConfig(),
        getModelPoolStatus(),
      ])

      if (cfgResult.status === 'fulfilled') {
        setConfig(cfgResult.value)
      } else {
        console.warn('[ModelSwitcher] providerConfig failed:', cfgResult.reason)
      }

      if (poolResult.status === 'fulfilled') {
        setPoolModels(poolResult.value.providers || [])
      } else {
        console.warn('[ModelSwitcher] model_pool_status failed:', poolResult.reason)
      }
    } catch (err) {
      setError(errText(err) || '加载模型列表失败')
    } finally {
      setLoading(false)
    }
  }

  const handleSelectModel = async (model: string) => {
    setLoading(true)
    setError(null)
    try {
      await neocodex.setProvider(model)
      await loadData()
      setIsOpen(false)
      window.dispatchEvent(new CustomEvent('neotrix:provider-changed', { detail: { model } }))
    } catch (err) {
      setError(errText(err) || '切换模型失败')
    } finally {
      setLoading(false)
    }
  }

  const currentModel = (): ProviderMeta | null => {
    const cfg = config()
    const models = allModels()
    if (!cfg) return models[0] || null
    return models.find((p) => p.model === cfg.active_model) || models[0] || null
  }

  const pillLabel = () => {
    const p = currentModel()
    if (!p) return '选择模型'
    const name = p.display_name || p.model
    return name.length > 18 ? name.slice(0, 18) + '…' : name
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
        title={pillLabel()}
      >
        {loading() ? (
          <Loader2 class="w-3.5 h-3.5 animate-spin text-orange-500" />
        ) : currentModel() ? (
          <ProviderIcon name={currentModel()!.id} size="sm" category={currentModel()!.category} />
        ) : (
          <span class="w-5 h-5 rounded-full bg-nt-core-500/15 text-nt-core-600 flex items-center justify-center text-[9px] font-bold">N</span>
        )}
        <span class="max-w-[120px] truncate">{pillLabel()}</span>
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
        <div ref={panelRef} class="absolute bottom-full left-0 mb-2 glass-pop border border-black/8 rounded-2xl shadow-2xl overflow-hidden z-50 animate-in w-[min(400px,92vw)]">
          <div class="px-3 py-2 border-b border-white/40 text-[11px] font-medium text-text-muted flex items-center justify-between">
            <span>可用模型 ({allModels().length})</span>
            <button onClick={loadData} class="text-nt-io-500 hover:text-nt-io-600 text-[10px]">刷新</button>
          </div>
          <div class="max-h-64 overflow-y-auto" role="listbox" aria-label="模型列表">
            <For each={allModels()}>
              {(model: ProviderMeta, i) => {
                const isActive = () => model.model === config()?.active_model
                return (
                  <button
                    class={clsx(
                      'w-full flex items-center gap-3 px-3 py-2.5 text-left transition-colors hover:bg-bg-tertiary focus-visible:bg-bg-tertiary focus-visible:outline-none',
                      isActive() && 'bg-nt-io-500/10 text-nt-io-600',
                    )}
                    onClick={() => handleSelectModel(model.model)}
                    disabled={loading() || isActive()}
                    role="option"
                    aria-selected={isActive()}
                    onKeyDown={(e) => {
                      if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return
                      e.preventDefault()
                      const list = allModels()
                      if (list.length === 0) return
                      requestAnimationFrame(() => {
                        const opts = Array.from(panelRef?.querySelectorAll<HTMLElement>('[role="option"]') ?? [])
                        opts[(i() + (e.key === 'ArrowDown' ? 1 : -1) + list.length) % list.length]?.focus?.()
                      })
                    }}
                  >
                    <ProviderIcon name={model.id} size="sm" category={model.category} />
                    <div class="flex-1 min-w-0 flex flex-col gap-0.5">
                      <div class="flex items-center gap-1.5 min-w-0">
                        <span class="font-medium text-[12px] truncate">{model.display_name}</span>
                        {model.is_free && <FreeBadge free />}
                      </div>
                      <div class="flex items-center gap-1.5 min-w-0">
                        <span class="text-[10px] text-text-muted truncate font-mono">{model.model}</span>
                        <CategoryBadge category={model.category} className="hidden sm:inline-flex" />
                      </div>
                    </div>
                    {isActive() && <Check class="w-4 h-4 text-nt-io-500 flex-shrink-0" />}
                  </button>
                )
              }}
            </For>
            {allModels().length === 0 && !loading() && (
              <div class="px-3 py-6 text-center text-text-muted text-sm">暂无可用模型</div>
            )}
          </div>

          <div class="px-3 py-2 border-t border-white/40 text-[10px] text-text-muted">
            活跃: <span class="font-mono text-text-secondary">{config()?.active_model || '未配置'}</span>
            <span class="mx-1">·</span>
            池子: {poolModels().length} 个
          </div>
        </div>
      </Show>

      <div class={isOpen() ? 'fixed inset-0 z-40' : 'hidden'} onClick={() => setIsOpen(false)} aria-hidden="true" />
    </div>
  )
}
