/* ════════════════════════════════════════════
   routes/Workflows.tsx — 工作流页 (Phase 2 B5, 直连后端)

   吸收 n8n/Dify 列表+运行态模式：
   - 工作流卡列表 (步骤数/标签/更新时间)
   - 运行按钮 → workflow_run → 轮询 run_status 进度条
   ════════════════════════════════════════════ */
import { createSignal, For, Show, onMount, onCleanup } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { ArrowLeft, Workflow as WorkflowIcon, Play, Loader2, RefreshCw, Layers } from 'lucide-solid'
import { clsx } from 'clsx'
import { workflowList, workflowRun, workflowRunStatus, type Workflow, type WorkflowRun } from '../api/workflows'
import { errText } from '../api'

export function Workflows() {
  const navigate = useNavigate()
  const [workflows, setWorkflows] = createSignal<Workflow[]>([])
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [runningId, setRunningId] = createSignal<string | null>(null)
  const [runState, setRunState] = createSignal<WorkflowRun | null>(null)

  let pollTimer: ReturnType<typeof setInterval> | undefined

  onCleanup(() => {
    if (pollTimer) clearInterval(pollTimer)
  })

  async function load() {
    setLoading(true)
    setError(null)
    try {
      setWorkflows(await workflowList())
    } catch (e) {
      setError(errText(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(() => void load())

  async function start(wf: Workflow) {
    setError(null)
    try {
      const runId = await workflowRun(wf.id)
      setRunningId(wf.id)
      pollTimer = setInterval(async () => {
        try {
          const r = await workflowRunStatus(runId)
          setRunState(r)
          if (['completed', 'failed', 'cancelled'].includes(r.status)) {
            if (pollTimer) clearInterval(pollTimer)
            setRunningId(null)
          }
        } catch {
          if (pollTimer) clearInterval(pollTimer)
          setRunningId(null)
        }
      }, 1000)
    } catch (e) {
      setError(errText(e))
    }
  }

  return (
    <div class="min-h-screen mac-safe bg-bg-primary text-text-primary flex flex-col">
      <header class="flex items-center gap-3 px-5 h-12 border-b border-border-primary/40 shrink-0">
        <button
          class="flex items-center gap-1.5 text-13px text-text-muted hover:text-text-primary transition-colors"
          onClick={() => navigate('/chat')}
          aria-label="返回对话"
        >
          <ArrowLeft class="w-4 h-4" />
          对话
        </button>
        <h1 class="text-14px font-semibold flex items-center gap-1.5">
          <WorkflowIcon class="w-4 h-4 text-nt-io-600" />
          工作流
        </h1>
        <span class="text-11px text-text-muted">{workflows().length} 条</span>
        <div class="flex-1" />
        <button
          class="flex items-center gap-1.5 h-8 px-3 rounded-lg text-13px text-text-muted hover:text-text-primary hover:bg-white/40 transition-colors"
          onClick={() => void load()}
          aria-label="刷新工作流"
          title="刷新"
        >
          <RefreshCw class={clsx('w-4 h-4', loading() && 'animate-spin')} />
        </button>
      </header>

      <main class="flex-1 overflow-y-auto p-5">
        <Show when={!loading()} fallback={
          <div class="flex items-center justify-center py-24 text-text-muted" role="status">
            <Loader2 class="w-5 h-5 animate-spin mr-2" /> 加载中…
          </div>
        }>
          <Show when={!error()} fallback={
            <div class="text-center py-24">
              <p class="text-13px text-text-muted mb-3">{error()}</p>
              <button class="text-13px text-nt-io-600 hover:underline" onClick={() => void load()}>重试</button>
            </div>
          }>
            <Show when={workflows().length > 0} fallback={
              <div class="text-center py-24 text-text-muted text-13px">暂无工作流</div>
            }>
              <div class="max-w-4xl mx-auto space-y-3">
                <For each={workflows()}>
                  {(wf) => (
                    <article
                      class="rounded-xl border border-border-primary/50 bg-bg-secondary p-4 flex items-center gap-4"
                      aria-label={`工作流 ${wf.name}`}
                    >
                      <div class="w-9 h-9 rounded-lg bg-nt-io-500/10 flex items-center justify-center shrink-0">
                        <Layers class="w-4.5 h-4.5 text-nt-io-600" />
                      </div>
                      <div class="min-w-0 flex-1">
                        <h2 class="text-13px font-semibold truncate">{wf.name}</h2>
                        <p class="text-12px text-text-muted line-clamp-1">{wf.description || '暂无描述'}</p>
                        <div class="flex gap-2 mt-1 text-11px text-text-muted">
                          <span>{wf.steps.length} 步骤</span>
                          <For each={wf.tags.slice(0, 3)}>
                            {(tag) => <span class="px-1.5 rounded-full bg-black/5">#{tag}</span>}
                          </For>
                        </div>
                      </div>
                      {/* 运行态进度条 */}
                      <Show when={runningId() === wf.id && runState()}>
                        {(r) => (
                          <div class="w-28 shrink-0" aria-label={`运行进度 ${r().progress_pct}`}>
                            <div class="h-1.5 rounded-full bg-black/10 overflow-hidden">
                              <div class="h-full bg-nt-io-500 rounded-full transition-all" style={{ width: `${r().progress_pct}%` }} />
                            </div>
                            <p class="text-[10px] text-text-muted mt-0.5 text-center">{r().status} {Math.round(r().progress_pct)}%</p>
                          </div>
                        )}
                      </Show>
                      <button
                        class="flex items-center gap-1.5 h-8 px-3 rounded-lg text-13px font-medium bg-nt-io-500 text-white hover:bg-nt-io-600 transition-colors disabled:opacity-40 shrink-0"
                        disabled={runningId() !== null}
                        onClick={() => void start(wf)}
                        aria-label={`运行 ${wf.name}`}
                      >
                        <Play class="w-3.5 h-3.5" />
                        运行
                      </button>
                    </article>
                  )}
                </For>
              </div>
            </Show>
          </Show>
        </Show>
      </main>
    </div>
  )
}
