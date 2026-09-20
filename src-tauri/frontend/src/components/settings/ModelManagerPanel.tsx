import { createSignal, onMount, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import { llamacpp, type LlamacppModel } from '../../api/domain'
import { ModelIcon } from './settingsIcons'

const formatSize = (bytes: number) => {
  const gb = bytes / (1024 * 1024 * 1024)
  return `${gb.toFixed(2)} GB`
}

export function ModelManagerPanel() {
  const [models, setModels] = createSignal<LlamacppModel[]>([])
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [selectedModel, setSelectedModel] = createSignal<string | null>(null)

  const fetchModels = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await llamacpp.models()
      setModels(result || [])
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(fetchModels)

  return (
    <div class="space-y-4">
      {/* 模型列表 */}
      <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
        <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
          <ModelIcon />
          本地模型管理
          <button
            class={clsx(
              'ml-auto px-2.5 py-1 rounded-lg text-[11px] font-medium border transition-colors',
              loading()
                ? 'text-zinc-400 border-zinc-200 cursor-not-allowed'
                : 'text-nt-io-600 border-nt-io-200 hover:bg-nt-io-50'
            )}
            onClick={fetchModels}
            disabled={loading()}
          >
            {loading() ? '刷新中...' : '刷新'}
          </button>
        </div>

        <div class="ss-card-body bg-white">
          <Show when={error()}>
            <div class="mb-2 px-2.5 py-1.5 rounded-lg bg-red-50 border border-red-200 text-[11px] text-red-600">
              {error()}
            </div>
          </Show>

          <Show when={!loading()} fallback={
            <div class="text-[11px] text-zinc-400 text-center py-3">加载模型列表…</div>
          }>
            <Show when={models().length > 0} fallback={
              <div class="text-[11px] text-zinc-400 text-center py-2">暂无本地模型</div>
            }>
              <div class="grid grid-cols-1 gap-1.5">
                <For each={models()}>
                  {(model) => (
                    <div
                      class={clsx(
                        'flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-[11px] cursor-pointer transition-colors',
                        selectedModel() === model.path
                          ? 'bg-nt-io-500/10 border border-nt-io-300/50'
                          : 'bg-zinc-50/60 hover:bg-zinc-100/60 border border-transparent'
                      )}
                      onClick={() => setSelectedModel(model.path)}
                    >
                      <span class={clsx(
                        'w-2 h-2 rounded-full flex-shrink-0',
                        selectedModel() === model.path ? 'bg-nt-io-500' : 'bg-zinc-300'
                      )} />
                      <span class="font-medium text-text-primary truncate min-w-0 flex-1 font-mono">{model.name}</span>
                      <span class="text-zinc-400 font-mono w-16 text-right">{formatSize(model.size)}</span>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </Show>
        </div>
      </div>

      {/* 模型操作 */}
      <Show when={selectedModel()}>
        <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
          <div class="ss-card-header bg-zinc-50/60 border-b border-black/5">
            模型操作
          </div>
          <div class="ss-card-body bg-white">
            <div class="text-[11px] text-zinc-500 mb-2">
              已选择：<span class="font-mono text-text-primary">{selectedModel()?.split('/').pop()}</span>
            </div>
            <div class="flex items-center gap-2">
              <button class="px-3 py-1.5 rounded-lg text-[11px] font-medium border text-nt-io-600 border-nt-io-200 hover:bg-nt-io-50 transition-colors">
                切换到此模型
              </button>
              <button class="px-3 py-1.5 rounded-lg text-[11px] font-medium border text-red-600 border-red-200 hover:bg-red-50 transition-colors">
                删除模型
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  )
}
