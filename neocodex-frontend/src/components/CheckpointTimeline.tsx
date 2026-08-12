import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js'
import { History, RotateCcw, Loader2, X, Clock, RefreshCw } from 'lucide-solid'
import { neocodex } from '../api'
import type { Checkpoint, NeoCodexMessageItem } from '../api/types'
import { clsx } from 'clsx'
import { ConfirmModal, type ModalReq } from './ConfirmModal'

interface Props {
  open: boolean
  sessionId: string | null
  onClose: () => void
  /** Called after a successful restore with the restored messages so the app can reload/rehydrate. */
  onRestored?: (msgs: NeoCodexMessageItem[]) => void
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
  // 统一确认模态（替换原生 confirm）
  const [modalReq, setModalReq] = createSignal<ModalReq | null>(null)
  const [pendingCp, setPendingCp] = createSignal<Checkpoint | null>(null)
  // 恢复成功就地反馈：顶部成功提示条 + 列表内短暂高亮
  const [restoreMsg, setRestoreMsg] = createSignal<string | null>(null)
  const [highlightedId, setHighlightedId] = createSignal<string | null>(null)
  let restoreMsgTimer: ReturnType<typeof setTimeout> | undefined
  onCleanup(() => clearTimeout(restoreMsgTimer))
  // 打开确认框前的触发元素（恢复按钮），关闭后还原焦点
  let lastTriggerEl: HTMLElement | null = null
  let firstBtnRef: HTMLButtonElement | undefined
  let panelRef: HTMLDivElement | undefined
  // 打开面板前的触发元素，关闭后还原焦点
  let lastFocusedEl: HTMLElement | null = null

  // 面板打开时聚焦首个按钮（对标 Codex 面板聚焦规范），并记录触发元素；
  // 关闭时（Esc/关闭按钮/遮罩点击触发卸载）经 effect 清理还原焦点
  createEffect(() => {
    if (!props.open) return
    lastFocusedEl = document.activeElement as HTMLElement | null
    const raf = requestAnimationFrame(() => {
      if (firstBtnRef) firstBtnRef.focus()
      else panelRef?.focus()
    })
    return () => {
      cancelAnimationFrame(raf)
      if (lastFocusedEl?.isConnected) lastFocusedEl.focus()
    }
  })

  // ConfirmModal 仅在有输入框时处理 Esc；纯确认模式在此补全局 Esc + 焦点迁入 dialog
  createEffect(() => {
    if (!modalReq()) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return
      e.preventDefault()
      e.stopPropagation()
      closeModal()
    }
    document.addEventListener('keydown', onKey, true)
    // 焦点迁入 dialog（无 input 时 ConfirmModal 无 autofocus）
    const raf = requestAnimationFrame(() => {
      document.querySelector<HTMLElement>('.glass-modal button:last-child')?.focus()
    })
    return () => {
      document.removeEventListener('keydown', onKey, true)
      cancelAnimationFrame(raf)
    }
  })

  const load = async () => {
    if (!props.sessionId) return
    setLoading(true)
    setError(null)
    try {
      const list = await neocodex.checkpointList(props.sessionId)
      setCheckpoints(list)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  // 会话切换时重新加载快照时间线（组件随面板打开挂载，首轮即覆盖打开加载，
  // 故不再 onMount(load) 避免同一 sessionId 触发两次请求）
  createEffect(() => {
    if (props.open && props.sessionId) {
      load()
    }
  })

  const closeModal = () => {
    setPendingCp(null)
    setModalReq(null)
    if (lastTriggerEl?.isConnected) lastTriggerEl.focus()
  }

  const restore = async (cp: Checkpoint) => {
    if (!props.sessionId) return
    lastTriggerEl = document.activeElement as HTMLElement | null
    setPendingCp(cp)
    setModalReq({
      title: '恢复快照',
      message: `恢复到 ${formatTs(cp.created_at)} 的快照（${cp.message_count} 条消息）？`,
      danger: true,
      confirmLabel: '恢复',
    })
  }

  const doRestore = async (cp: Checkpoint) => {
    setPendingCp(null)
    setModalReq(null)
    setRestoring(cp.id)
    setError(null)
    try {
      const restored = await neocodex.checkpointRestore(props.sessionId!, cp.id)
      props.onRestored?.(restored)
      await load()
      // 就地反馈：成功提示条 + 恢复的快照短暂高亮
      const msg = `已恢复 ${formatTs(cp.created_at)} 的快照`
      setRestoreMsg(msg)
      setHighlightedId(cp.id)
      clearTimeout(restoreMsgTimer)
      restoreMsgTimer = setTimeout(() => {
        setRestoreMsg(null)
        setHighlightedId(null)
      }, 3000)
    } catch (e) {
      setError(String(e))
    } finally {
      setRestoring(null)
    }
  }

  return (
    <Show when={props.open}>
      <div
        ref={panelRef}
        class="panel w-80"
        role="dialog"
        aria-modal="true"
        aria-label="检查点"
        tabIndex={-1}
        onKeyDown={(e) => {
          if (e.key === 'Escape') {
            e.preventDefault()
            props.onClose()
            return
          }
          if (e.key === 'Tab' && panelRef) {
            const focusables = panelRef.querySelectorAll<HTMLElement>(
              'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
            )
            if (focusables.length === 0) return
            const first = focusables[0]
            const last = focusables[focusables.length - 1]
            const active = document.activeElement
            if (e.shiftKey && (active === first || active === panelRef)) {
              e.preventDefault()
              last.focus()
            } else if (!e.shiftKey && active === last) {
              e.preventDefault()
              first.focus()
            }
          }
        }}
      >
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
          <Show when={restoreMsg()}>
            <div class="p-3 text-xs text-emerald-600 bg-emerald-500/10 rounded-lg mb-2" role="status">
              {restoreMsg()}
            </div>
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
                  <div
                    class={clsx(
                      'relative pb-4',
                      highlightedId() === cp.id && 'rounded-lg bg-nt-repair-500/10 ring-1 ring-nt-repair-500/50'
                    )}
                  >
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

      <ConfirmModal
        req={modalReq()}
        onConfirm={() => pendingCp() && doRestore(pendingCp()!)}
        onClose={closeModal}
      />
    </Show>
  )
}
