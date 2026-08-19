import { createSignal, onMount, createEffect, Show, For } from 'solid-js'
import { CalendarClock, X, RefreshCw, Loader2, Play, Plus, Trash2, Pause, CirclePlay, History } from 'lucide-solid'
import { tasks as tasksApi, errText } from '../api'
import type { BackgroundTask } from '../api/types'
import { clsx } from 'clsx'
import { ConfirmModal, type ModalReq } from './ConfirmModal'

// 预置调度模板（RRULE 风格速选）
const SCHEDULE_PRESETS: { label: string; value: string }[] = [
  { label: '每天', value: 'FREQ=DAILY;INTERVAL=1' },
  { label: '每周', value: 'FREQ=WEEKLY;INTERVAL=1' },
  { label: '每 12 小时', value: 'FREQ=HOURLY;INTERVAL=12' },
  { label: '每小时', value: 'FREQ=HOURLY;INTERVAL=1' },
]

interface Props {
  open: boolean
  onClose: () => void
}

export function ScheduledTasks(props: Props) {
  const [tasks, setTasks] = createSignal<BackgroundTask[]>([])
  const [loading, setLoading] = createSignal(false)
  const [busy, setBusy] = createSignal<string | null>(null)
  const [error, setError] = createSignal<string | null>(null)
  // 统一确认模态（替换原生 confirm）
  const [modalReq, setModalReq] = createSignal<ModalReq | null>(null)
  const [pendingDeleteId, setPendingDeleteId] = createSignal<string | null>(null)
  const [showCreate, setShowCreate] = createSignal(false)
  const [name, setName] = createSignal('')
  const [prompt, setPrompt] = createSignal('')
  const [schedule, setSchedule] = createSignal(SCHEDULE_PRESETS[0].value)
  // RRULE 自由输入校验：非法规则给友好错误，不裸 throw
  const [scheduleError, setScheduleError] = createSignal<string | null>(null)
  let firstBtnRef: HTMLButtonElement | undefined
  let panelRef: HTMLDivElement | undefined
  // 打开面板前的触发元素，关闭后还原焦点
  let lastFocusedEl: HTMLElement | null = null

  // 轻量 RRULE 校验（仅防呆，不做完整 RFC 5545 解析）
  const validateRRule = (rrule: string): string | null => {
    const s = rrule.trim().toUpperCase()
    if (!s) return '调度规则不能为空'
    const freq = /(?:^|;)FREQ=([A-Z0-9]+)/.exec(s)?.[1]
    if (!freq) return '调度规则需包含 FREQ=（如 FREQ=DAILY;INTERVAL=1）'
    const allowed = ['SECONDLY', 'MINUTELY', 'HOURLY', 'DAILY', 'WEEKLY', 'MONTHLY', 'YEARLY']
    if (!allowed.includes(freq)) return `不支持的 FREQ=${freq}（支持 ${allowed.join('/')}）`
    return null
  }

  // 自由输入实时校验
  createEffect(() => {
    setScheduleError(validateRRule(schedule()))
  })

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

  const load = async () => {
    setLoading(true)
    setError(null)
    try {
      const list = await tasksApi.listBackgroundTasks()
      setTasks(list)
    } catch (e) {
      setError(errText(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(load)

  // 任务状态实时刷新：面板打开期间每 10s 轮询。
  // 守卫：in-flight 去重 + 页面不可见时不轮询。
  let polling = false
  createEffect(() => {
    if (!props.open) return
    const timer = setInterval(async () => {
      if (polling || document.visibilityState === 'hidden') return
      polling = true
      try {
        await load()
      } finally {
        polling = false
      }
    }, 10000)
    return () => clearInterval(timer)
  })

  const create = async () => {
    if (!name().trim() || !prompt().trim()) {
      setError('任务名和提示词不能为空')
      return
    }
    const rruleErr = validateRRule(schedule())
    if (rruleErr) {
      setError(rruleErr)
      return
    }
    setBusy('create')
    setError(null)
    try {
      await tasksApi.createBackgroundTask(name().trim(), prompt().trim(), schedule())
      setName('')
      setPrompt('')
      setShowCreate(false)
      await load()
    } catch (e) {
      setError(errText(e))
    } finally {
      setBusy(null)
    }
  }

  const action = async (kind: 'pause' | 'resume' | 'delete' | 'run', id: string) => {
    // 破坏性操作确认（对标 Codex）— 统一模态
    if (kind === 'delete') {
      setPendingDeleteId(id)
      setModalReq({
        title: '删除定时任务',
        message: '确定删除该定时任务？',
        danger: true,
        confirmLabel: '删除',
      })
      return
    }
    setBusy(`${kind}:${id}`)
    setError(null)
    try {
      if (kind === 'run') await tasksApi.runBackgroundTaskNow(id)
      else if (kind === 'pause') await tasksApi.pauseBackgroundTask(id)
      else if (kind === 'resume') await tasksApi.resumeBackgroundTask(id)
      await load()
    } catch (e) {
      setError(errText(e))
    } finally {
      setBusy(null)
    }
  }

  const doDelete = async (id: string) => {
    // 确认后真正删除：关闭模态一次 → 调用 API → 失败时提示错误（不再重开确认框）
    setPendingDeleteId(null)
    setModalReq(null)
    setBusy(`delete:${id}`)
    setError(null)
    try {
      await tasksApi.deleteBackgroundTask(id)
      await load()
    } catch (e) {
      setError(errText(e))
    } finally {
      setBusy(null)
    }
  }

  const formatTime = (ts: number | null) => {
    if (ts === null) return '—'
    const d = new Date(ts * 1000)
    return d.toLocaleString([], { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
  }

  const statusClass = (s: string) => {
    switch (s) {
      case 'running': return 'bg-nt-core-400 animate-pulse'
      case 'paused': return 'bg-amber-500'
      case 'error': return 'bg-red-500'
      default: return 'bg-emerald-500'
    }
  }

  const statusLabel = (s: string) => {
    switch (s) {
      case 'running': return '运行中'
      case 'paused': return '已暂停'
      case 'error': return '错误'
      default: return '空闲'
    }
  }

  /* 状态 → 语义徽章类（与 badge-success/warn/error 体系一致） */
  const statusBadge = (s: string) => {
    switch (s) {
      case 'error': return 'badge-error'
      case 'paused': return 'badge-warn'
      case 'running': return 'badge-warn' // 运行中 = 进行中
      default: return 'badge-success' // 空闲
    }
  }

  return (
    <Show when={props.open}>
      <div
        ref={panelRef}
        class="panel w-[28rem]"
        role="dialog"
        aria-modal="true"
        aria-label="定时任务"
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
          <CalendarClock class="panel-head-icon text-nt-repair-300" />
          <span class="panel-title">定时任务</span>
          <span class="panel-sub">{tasks().length} 个任务</span>
          <button
            ref={firstBtnRef}
            class="panel-close"
            onClick={load}
            aria-label="刷新"
          >
            <RefreshCw class={clsx('w-4 h-4', loading() && 'animate-spin')} />
          </button>
          <button
            class="p-2 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
            onClick={props.onClose}
            aria-label="关闭"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div class="flex-1 overflow-y-auto p-4 space-y-4">
          <Show when={error()}>
            <div class="p-3 text-xs text-red-500 bg-red-500/10 rounded-lg">{error()}</div>
          </Show>

          {/* Create button */}
          <button
            class="w-full flex items-center justify-center gap-2 px-3 py-3 rounded-xl bg-nt-repair-500/20 text-nt-repair-300 hover:bg-nt-repair-500/30 transition-colors text-sm font-medium"
            onClick={() => setShowCreate(!showCreate())}
            aria-expanded={showCreate()}
          >
            <Plus class="w-4 h-4" />
            {showCreate() ? '收起表单' : '新建定时任务'}
          </button>

          {/* Create form */}
          <Show when={showCreate()}>
            <div class="rounded-xl border border-border-primary bg-bg-primary/40 p-3 space-y-3">
              <div>
                <label class="text-[10px] text-text-muted uppercase tracking-wider">任务名</label>
                <input
                  value={name()}
                  onInput={(e) => setName(e.currentTarget.value)}
                  onKeyDown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); create() } }}
                  placeholder="例如：每日代码库巡检"
                  class="mt-1 w-full bg-bg-primary border border-border-primary rounded-lg px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-nt-repair-400/50"
                />
              </div>
              <div>
                <label class="text-[10px] text-text-muted uppercase tracking-wider">提示词</label>
                <textarea
                  value={prompt()}
                  onInput={(e) => setPrompt(e.currentTarget.value)}
                  placeholder="任务执行内容..."
                  rows={3}
                  class="mt-1 w-full resize-none bg-bg-primary border border-border-primary rounded-lg px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-nt-repair-400/50"
                />
              </div>
              <div>
                <label class="text-[10px] text-text-muted uppercase tracking-wider">调度（RRULE）</label>
                <div class="mt-1 flex gap-2 flex-wrap">
                  <For each={SCHEDULE_PRESETS}>
                    {(p) => (
                      <button
                        class={clsx(
                          'px-2 py-1 rounded-lg text-xs border transition-colors',
                          schedule() === p.value
                            ? 'border-nt-repair-400/50 bg-nt-repair-500/15 text-nt-repair-300'
                            : 'border-border-primary text-text-muted hover:text-text-primary'
                        )}
                        onClick={() => setSchedule(p.value)}
                      >
                        {p.label}
                      </button>
                    )}
                  </For>
                </div>
                <input
                  value={schedule()}
                  onInput={(e) => setSchedule(e.currentTarget.value)}
                  class="mt-2 w-full bg-bg-primary border border-border-primary rounded-lg px-3 py-2 text-xs font-mono text-text-primary focus:outline-none focus:ring-1 focus:ring-nt-repair-400/50"
                />
                <Show when={scheduleError()}>
                  <div class="mt-1 text-[11px] text-amber-600">{scheduleError()}</div>
                </Show>
              </div>
              <button
                class="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-nt-repair-500/30 text-nt-repair-200 hover:bg-nt-repair-500/40 transition-colors text-sm font-medium"
                onClick={create}
                disabled={busy() !== null}
              >
                {busy() === 'create' ? <Loader2 class="w-4 h-4 animate-spin" /> : <Plus class="w-4 h-4" />}
                创建任务
              </button>
            </div>
          </Show>

          {/* Task list */}
          <Show when={loading() && tasks().length === 0}>
            <div class="flex items-center justify-center gap-2 py-8 text-text-muted text-sm">
              <Loader2 class="w-4 h-4 animate-spin" />
              加载任务...
            </div>
          </Show>
          <Show when={!loading() && tasks().length === 0 && !showCreate()}>
            <div class="py-8 text-center text-xs text-text-muted">暂无定时任务</div>
          </Show>

          <div class="space-y-2">
            <For each={tasks()}>
              {(task) => (
                <div class="rounded-xl border border-border-primary bg-bg-primary/40 p-3">
                  <div class="flex items-center gap-2">
                    <span class={clsx('w-2 h-2 rounded-full flex-shrink-0', statusClass(task.status))} />
                    <span class="text-sm font-medium text-text-primary truncate flex-1">{task.name}</span>
                    <span class={clsx('badge', statusBadge(task.status))}>{statusLabel(task.status)}</span>
                  </div>
                  <div class="mt-1 text-xs text-text-secondary line-clamp-2">{task.prompt}</div>
                  <div class="mt-1 text-[10px] font-mono text-text-muted truncate">{task.schedule}</div>
                  <div class="mt-2 flex items-center gap-2">
                    <button
                      class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs text-nt-core-300 hover:bg-nt-core-500/10 border border-nt-core-500/30 transition-colors"
                      onClick={() => action('run', task.id)}
                      disabled={busy() !== null}
                    >
                      {busy() === `run:${task.id}` ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <CirclePlay class="w-3.5 h-3.5" />}
                      立即执行
                    </button>
                    <Show when={task.status === 'paused'}>
                      <button
                        class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs text-emerald-600 hover:bg-emerald-500/10 border border-emerald-500/30 transition-colors"
                        onClick={() => action('resume', task.id)}
                        disabled={busy() !== null}
                      >
                        <Play class="w-3.5 h-3.5" />
                        恢复
                      </button>
                    </Show>
                    <Show when={task.status !== 'paused'}>
                      <button
                        class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs text-amber-600 hover:bg-amber-500/10 border border-amber-500/30 transition-colors"
                        onClick={() => action('pause', task.id)}
                        disabled={busy() !== null}
                      >
                        <Pause class="w-3.5 h-3.5" />
                        暂停
                      </button>
                    </Show>
                    <button
                      class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs text-red-500 hover:bg-red-500/10 border border-red-500/30 transition-colors"
                      onClick={() => action('delete', task.id)}
                      disabled={busy() !== null}
                    >
                      <Trash2 class="w-3.5 h-3.5" />
                      删除
                    </button>
                  </div>
                  <div class="mt-2 pt-2 border-t border-border-primary/50 flex items-center justify-between gap-2 text-[10px] text-text-muted">
                    <span>上次: {formatTime(task.last_run)}</span>
                    <span>下次: {formatTime(task.next_run)}</span>
                    <span class="flex items-center gap-1">
                      <History class="w-3.5 h-3.5" />
                      {task.runs.length} 次执行
                    </span>
                  </div>
                </div>
              )}
              </For>
            </div>
          </div>
        </div>

      <ConfirmModal
        req={modalReq()}
        onConfirm={() => pendingDeleteId() && doDelete(pendingDeleteId()!)}
        onClose={() => {
          setPendingDeleteId(null)
          setModalReq(null)
        }}
      />
    </Show>
  )
}
