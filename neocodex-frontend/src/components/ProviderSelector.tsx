import { createSignal, onMount, createEffect, For } from 'solid-js'
import { invoke } from '@tauri-apps/api/core'
import { ChevronDown, Loader2, Check, AlertCircle, Zap, Brain, Globe } from 'lucide-solid'
import { clsx } from 'clsx'

export interface ProviderEntry {
  name: string
  model: string
  resolvable: boolean
}

export interface ProviderConfig {
  provider_count: number
  resolvable: boolean
  active_model: string
  providers: ProviderEntry[]
}

export function ProviderSelector(props: { iconOnly?: boolean }) {
  const [config, setConfig] = createSignal<ProviderConfig | null>(null)
  const [isOpen, setIsOpen] = createSignal(false)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  // Load provider config on mount
  onMount(async () => {
    await loadConfig()
  })

  const loadConfig = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<ProviderConfig>('neocodex_provider_config')
      setConfig(result)
    } catch (err) {
      const msg = err instanceof Error ? err.message : '获取提供商配置失败'
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
      await invoke('neocodex_set_provider', { name: providerName })
      await loadConfig()
      setIsOpen(false)
    } catch (err) {
      const msg = err instanceof Error ? err.message : '切换提供商失败'
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

  const getProviderIcon = (name: string) => {
    const lower = name.toLowerCase()
    if (lower.includes('openai') || lower.includes('gpt')) return <Zap class="w-4 h-4" />
    if (lower.includes('anthropic') || lower.includes('claude')) return <Brain class="w-4 h-4" />
    if (lower.includes('google') || lower.includes('gemini')) return <Globe class="w-4 h-4" />
    return <Zap class="w-4 h-4" />
  }

  const formatProviderName = (name: string) => {
    return name.charAt(0).toUpperCase() + name.slice(1).toLowerCase()
  }

  return (
    <div class="relative">
      {/* Trigger Button — iconOnly 极简圆形图标 / 完整模式 */}
      <button
        class={clsx(
          props.iconOnly
            ? 'w-8 h-8 rounded-full flex items-center justify-center text-text-muted hover:text-nt-io-600 hover:bg-nt-io-500/10 transition-all'
            : 'flex items-center gap-2 px-3 py-1.5 rounded-lg border border-border-primary bg-bg-secondary text-text-primary hover:bg-bg-tertiary transition-colors min-w-[180px] max-w-[280px]',
          loading() && 'opacity-50 cursor-wait'
        )}
        onClick={() => !loading() && setIsOpen(!isOpen())}
        disabled={loading()}
        aria-label="选择模型提供商"
        aria-expanded={isOpen()}
        aria-haspopup="listbox"
        title={props.iconOnly ? (currentProvider() ? formatProviderName(currentProvider()!.name) : '选择提供商') : undefined}
      >
        {props.iconOnly ? (
          loading() ? (
            <Loader2 class="w-4 h-4 animate-spin text-nt-io-500" />
          ) : currentProvider() ? (
            getProviderIcon(currentProvider()!.name)
          ) : (
            <Zap class="w-4 h-4" />
          )
        ) : (
          <>
            <span class="flex items-center gap-1.5 text-sm truncate">
              {loading() ? (
                <Loader2 class="w-4 h-4 animate-spin text-nt-io-500" />
              ) : currentProvider() ? (
                <>
                  {getProviderIcon(currentProvider()!.name)}
                  <span class="font-medium truncate">{formatProviderName(currentProvider()!.name)}</span>
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
        <div class="absolute top-full left-0 right-0 mt-1 p-2 bg-red-500/20 border border-red-500/30 rounded-lg text-xs text-red-300 flex items-center gap-1.5 animate-in z-10">
          <AlertCircle class="w-3.5 h-3.5 flex-shrink-0" />
          {error()}
          <button onClick={() => setError(null)} class="ml-auto p-0.5 hover:bg-red-500/30 rounded">×</button>
        </div>
      )}

      {/* Dropdown Panel */}
      <Show when={isOpen()}>
        <div class="absolute top-full left-0 right-0 mt-1.5 bg-bg-secondary border border-border-primary rounded-xl shadow-xl overflow-hidden z-50 animate-in min-w-[220px] max-w-[320px]">
          {/* Header */}
          <div class="px-3 py-2 border-b border-border-primary flex items-center justify-between">
            <span class="text-sm font-medium text-text-primary">模型提供商</span>
            <span class="text-xs text-text-muted">{config()?.provider_count || 0} 个可用</span>
          </div>

          {/* Provider List */}
          <div class="max-h-64 overflow-y-auto">
            <For each={config()?.providers || []}>
              {(provider: ProviderEntry) => {
                const isActive = provider.model === config()?.active_model
                return (
                  <button
                    class={clsx(
                      'w-full flex items-center gap-3 px-3 py-2.5 text-left transition-colors',
                      'hover:bg-bg-tertiary',
                      isActive && 'bg-nt-io-500/10 text-nt-io-600'
                    )}
                    onClick={() => handleSelectProvider(provider.name)}
                    disabled={loading() || isActive}
                    role="option"
                    aria-selected={isActive}
                  >
                    {getProviderIcon(provider.name)}
                    <div class="flex-1 min-w-0 flex flex-col gap-0.5">
                      <span class="font-medium truncate">{formatProviderName(provider.name)}</span>
                      <span class="text-xs text-text-muted truncate">{provider.model}</span>
                    </div>
                    {isActive && <Check class="w-4 h-4 text-nt-io-500 flex-shrink-0" />}
                    {!provider.resolvable && (
                      <span class="text-xs text-amber-600 px-1.5 py-0.5 rounded bg-amber-500/10">不可用</span>
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
          <div class="px-3 py-2 border-t border-border-primary text-xs text-text-muted">
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

// Helper component for conditional rendering
function Show(props: { when: boolean; fallback?: any; children: any }) {
  return props.when ? props.children : (props.fallback || null)
}