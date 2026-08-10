import { createSignal, onMount, createEffect, Show, For } from 'solid-js'
import { History, RotateCcw, Loader2, X, Clock, RefreshCw } from 'lucide-solid'
import { invoke } from '@tauri-apps/api/core'
import { clsx } from 'clsx'

interface Checkpoint {
  id: string
  created_at: number
  message_count: number
}

interface Props {
  open: boolean
  sessionId: string | null
  onClose: () => void
  /** Called after a successful restore so the app can reload messages. */
  onRestored?: () => void
}

function formatTs(ms: number): string {
  const d = new Date(ms)
  const now = new Date()
  const diff = now.getTime() - d.getTime()
  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`
  return d.toLocaleString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

export function CheckpointTimeline(props: Props) {
  const [checkpoints, setCheckpoints] = createSignal<Checkpoint[]>([])
  const [loading, setLoading] = createSignal(false)
  const [restoring, setRestoring] = createSignal<string | null>(null)
  const [error, setError] = createSignal<string | null>(null)
  let firstBtnRef: HTMLButtonElement | undefined

  // 面板打开时聚焦首个按钮（对标 Codex 面板聚焦规范）
  createEffect(() => {
    if (props.open && firstBtnRef) firstBtnRef.focus()
  })

  const load = async () => {
    if (!props.sessionId) return
    setLoading(true)
    setError(null)
    try {
      const list = await invoke<Checkpoint[]>('neocodex_checkpoint_list', {
        session_id: props.sessionId,
      })
      setCheckpoints(list)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(load)

  const restore = async (cp: Checkpoint) => {
    if (!props.sessionId) return
    if (!window.confirm(`恢复到 ${formatTs(cp.created_at)} 的快照（${cp.message_count} 条消息）？`)) return
    setRestoring(cp.id)
    setError(null)
    try {
      await invoke('neocodex_checkpoint_restore', {
        session_id: props.sessionId,
        checkpoint_id: cp.id,
      })
      props.onRestored?.()
      await load()
    } catch (e) {
      setError(String(e))
    } finally {
      setRestoring(null)
    }
  }

  return (
    <Show when={props.open}>
      <div class="panel w-80">
        {/* Header */}
        <div class="panel-head">
          <History class="panel-head-icon text-nt-repair-600" />
          <span class="panel-title">时间线</span>
          <span class="panel-sub">({checkpoints().length} 个快照)</span>
          <button
            ref={firstBtnRef}
            class="panel-close"
            onClick={load}
            aria-label="刷新"
            title="刷新"
          >
            <RefreshCw class={clsx('w-4 h-4', loading() && 'animate-spin')} />
          </button>
          <button
            class="panel-close"
            onClick={props.onClose}
            aria-label="关闭时间线"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div class="flex-1 overflow-y-auto p-3">
          <Show when={loading}>
            <div class="flex items-center justify-center gap-2 py-8 text-text-muted text-sm">
              <Loader2 class="w-4 h-4 animate-spin" />
              加载快照...
            </div>
          </Show>
          <Show when={error()}>
            <div class="p-3 text-xs text-red-500 bg-red-500/10 rounded-lg mb-2">{error()}</div>
          </Show>
          <Show when={!loading && checkpoints().length === 0 && !error()}>
            <div class="py-10 text-center text-xs text-text-muted">
              暂无快照<br />
              <span class="text-text-muted/70">每轮对话结束会自动生成检查点</span>
            </div>
          </Show>
          <Show when={!loading && checkpoints().length > 0}>
            <div class="relative pl-5 before:absolute before:left-[7px] before:top-1 before:bottom-1 before:w-px before:bg-border-primary">
              <For each={checkpoints()}>
                {(cp, i) => (
                  <div class="relative pb-4">
                    {/* Timeline dot */}
                    <span
                      class={clsx(
                        'absolute -left-[21px] top-2 w-3.5 h-3.5 rounded-full border-2',
                        i() === 0 ? 'bg-nt-repair-400 border-bg-secondary' : 'bg-bg-tertiary border-border-primary'
                      )}
                    />
                    <div class="flex items-center gap-2">
                      <div class="flex-1 min-w-0">
                        <div class="text-sm text-text-primary font-medium">
                          {i() === 0 ? '最新快照' : `快照 #${checkpoints().length - i()}`}
                        </div>
                        <div class="text-xs text-text-muted flex items-center gap-1 mt-1">
                          <Clock class="w-3.5 h-3.5" />
                          {formatTs(cp.created_at)} · {cp.message_count} 条
                        </div>
                      </div>
                      <button
                        class={clsx(
                          'flex items-center gap-1 px-2 py-1 rounded-lg text-xs transition-colors',
                          restoring() === cp.id
                            ? 'text-text-muted'
                            : 'text-nt-repair-700 hover:bg-nt-repair-500/10 border border-nt-repair-500/30'
                        )}
                        onClick={() => restore(cp)}
                        disabled={restoring() !== null}
                        aria-label="恢复到该快照"
                        title="恢复到该快照"
                      >
                        {restoring() === cp.id ? (
                          <Loader2 class="w-3.5 h-3.5 animate-spin" />
                        ) : (
                          <RotateCcw class="w-3.5 h-3.5" />
                        )}
                        恢复
                      </button>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  )
}
