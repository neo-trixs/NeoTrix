import { createSignal, onMount, createEffect, Show, For } from 'solid-js'
import { CalendarClock, X, RefreshCw, Loader2, Play, Plus, Trash2, Pause, CirclePlay, History } from 'lucide-solid'
import { invoke } from '@tauri-apps/api/core'
import { clsx } from 'clsx'

interface TaskRun {
  timestamp: number
  summary: string
}

interface BackgroundTask {
  id: string
  name: string
  prompt: string
  schedule: string
  last_run: number | null
  next_run: number | null
  status: string
  runs: TaskRun[]
}

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
  const [showCreate, setShowCreate] = createSignal(false)
  const [name, setName] = createSignal('')
  const [prompt, setPrompt] = createSignal('')
  const [schedule, setSchedule] = createSignal(SCHEDULE_PRESETS[0].value)
  let firstBtnRef: HTMLButtonElement | undefined

  // 面板打开时聚焦首个按钮（对标 Codex 面板聚焦规范）
  createEffect(() => {
    if (props.open && firstBtnRef) firstBtnRef.focus()
  })

  const load = async () => {
    setLoading(true)
    setError(null)
    try {
      const list = await invoke<BackgroundTask[]>('list_background_tasks')
      setTasks(list)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(load)

  const create = async () => {
    if (!name().trim() || !prompt().trim()) {
      setError('任务名和提示词不能为空')
      return
    }
    setBusy('create')
    setError(null)
    try {
      await invoke('create_background_task', { name: name().trim(), prompt: prompt().trim(), schedule: schedule() })
      setName('')
      setPrompt('')
      setShowCreate(false)
      await load()
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(null)
    }
  }

  const action = async (kind: 'pause' | 'resume' | 'delete' | 'run', id: string) => {
    setBusy(`${kind}:${id}`)
    setError(null)
    try {
      const cmd = kind === 'run' ? 'run_background_task_now' : `${kind}_background_task`
      await invoke(cmd, { id })
      await load()
    } catch (e) {
      setError(String(e))
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
      <div class="panel w-[28rem]">
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
            class="p-1.5 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors"
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
            class="w-full flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl bg-nt-repair-500/20 text-nt-repair-300 hover:bg-nt-repair-500/30 transition-colors text-sm font-medium"
            onClick={() => setShowCreate(!showCreate())}
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
                  class="mt-1 w-full bg-bg-primary border border-border-primary rounded-lg px-2.5 py-1.5 text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-nt-repair-400/50"
                />
              </div>
              <div>
                <label class="text-[10px] text-text-muted uppercase tracking-wider">提示词</label>
                <textarea
                  value={prompt()}
                  onInput={(e) => setPrompt(e.currentTarget.value)}
                  placeholder="任务执行内容..."
                  rows={3}
                  class="mt-1 w-full resize-none bg-bg-primary border border-border-primary rounded-lg px-2.5 py-1.5 text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-nt-repair-400/50"
                />
              </div>
              <div>
                <label class="text-[10px] text-text-muted uppercase tracking-wider">调度（RRULE）</label>
                <div class="mt-1 flex gap-1.5 flex-wrap">
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
                  class="mt-1.5 w-full bg-bg-primary border border-border-primary rounded-lg px-2.5 py-1.5 text-xs font-mono text-text-primary focus:outline-none focus:ring-1 focus:ring-nt-repair-400/50"
                />
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
                  <div class="mt-2 flex items-center gap-1.5">
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
                  <div class="mt-2 pt-2 border-t border-border-primary/50 flex items-center justify-between text-[10px] text-text-muted">
                    <span>上次: {formatTime(task.last_run)}</span>
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
    </Show>
  )
}
