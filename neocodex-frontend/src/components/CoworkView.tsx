import { createSignal, createEffect, For, Show } from 'solid-js'
import { invoke } from '@tauri-apps/api/core'
import { clsx } from 'clsx'

/* ════════════════════════════════════════════
   CoworkView — 协同会话管理（设计 v2，已接线后端）
   左栏：会话列表（cw-slist）← cowork_list
   右栏：任务看板（cw-tlist）← cowork_actions
         + 交付物（cw-deliv）← cowork_list_deliverables
   ════════════════════════════════════════════ */

interface CoworkSession {
  id: string
  name: string
  workspace_path: string
  status: string
  files_read: number
  files_created: number
  files_modified: number
  started_at: number
  last_active_at: number
  deliverables: string[]
  description: string
  tags: string[]
}

interface CoworkAction {
  id: string
  session_id: string
  action_type: string
  target_path: string
  status: string
  started_at: number
  completed_at: number | null
  details: string | null
  result_summary: string | null
}

interface CoworkDeliverable {
  id: string
  session_id: string
  name: string
  path: string
  kind: string
  created_at: number
  size_bytes: number
  description: string
  quality_score: number | null
}

interface CoworkStats {
  total_sessions: number
  total_deliverables: number
  files_processed: number
  active_sessions: number
  avg_files_per_session: number
  top_category: string
  top_template: string
}

/* 状态 → 语义徽章类（与 badge-success/warn/error 体系一致） */
function statusBadge(status: string): string {
  if (status === 'completed' || status === 'done') return 'badge-success'
  if (status === 'failed' || status === 'error' || status === 'stopped') return 'badge-error'
  return 'badge-warn' // active / paused / running
}

function statusLabel(status: string): string {
  const map: Record<string, string> = {
    active: '进行中', paused: '已暂停', completed: '已完成',
    stopped: '已停止', failed: '失败', running: '进行中', done: '已完成',
  }
  return map[status] ?? status
}

export function CoworkView() {
  const [sessions, setSessions] = createSignal<CoworkSession[]>([])
  const [actions, setActions] = createSignal<CoworkAction[]>([])
  const [deliverables, setDeliverables] = createSignal<CoworkDeliverable[]>([])
  const [stats, setStats] = createSignal<CoworkStats | null>(null)
  const [activeId, setActiveId] = createSignal<string | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [showNew, setShowNew] = createSignal(false)
  const [newPath, setNewPath] = createSignal('')
  const [newDesc, setNewDesc] = createSignal('')

  const active = () => sessions().find((s) => s.id === activeId()) ?? sessions()[0]

  const loadSessions = async () => {
    try {
      const list = await invoke<CoworkSession[]>('cowork_list')
      setSessions(list)
      if (list.length > 0 && !list.some((s) => s.id === activeId())) {
        setActiveId(list[0].id)
      }
    } catch (e) {
      setError(String(e))
    }
  }

  const loadStats = async () => {
    try {
      setStats(await invoke<CoworkStats>('cowork_stats'))
    } catch { /* stats 非关键 */ }
  }

  const loadDetail = async (id: string) => {
    try {
      const [acts, dels] = await Promise.all([
        invoke<CoworkAction[]>('cowork_actions', { sessionId: id }),
        invoke<CoworkDeliverable[]>('cowork_list_deliverables', { sessionId: id }),
      ])
      setActions(acts)
      setDeliverables(dels)
    } catch (e) {
      setError(String(e))
    }
  }

  createEffect(() => {
    const id = activeId()
    if (id) loadDetail(id)
  })

  // 初始加载
  createEffect(() => {
    loadSessions()
    loadStats()
  })

  const addSession = async () => {
    const path = newPath().trim() || '.'
    setLoading(true)
    setError(null)
    try {
      const id = await invoke<string>('cowork_start', {
        workspacePath: path,
        description: newDesc().trim(),
        name: null,
        tags: null,
      })
      setNewPath('')
      setNewDesc('')
      setShowNew(false)
      await loadSessions()
      setActiveId(id)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  const controlSession = async (action: 'pause' | 'resume' | 'stop') => {
    const id = active()?.id
    if (!id) return
    const cmdMap = {
      pause: 'cowork_pause',
      resume: 'cowork_resume',
      stop: 'cowork_stop',
    } as const
    try {
      await invoke(cmdMap[action], { sessionId: id })
      await loadSessions()
    } catch (e) {
      setError(String(e))
    }
  }

  const refresh = async () => {
    setError(null)
    await loadSessions()
    await loadStats()
    const id = active()?.id
    if (id) await loadDetail(id)
  }

  const a = active()

  return (
    <div class="vw-cowork">
      <div class="cw-layout">
        {/* 左栏：会话列表 */}
        <div class="cw-sidebar">
          <div class="cw-shead">
            <span>会话</span>
            <div class="cw-shead-actions">
              <button class="cw-add" onClick={() => setShowNew(!showNew())} title="新建会话" aria-label="新建会话">
                <svg viewBox="0 0 14 14">
                  <line x1="7" y1="2" x2="7" y2="12" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                  <line x1="2" y1="7" x2="12" y2="7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                </svg>
              </button>
              <button class="cw-refresh-btn" onClick={refresh} title="刷新" aria-label="刷新">
                <svg viewBox="0 0 14 14">
                  <path d="M12 7a5 5 0 11-1.5-3.5M12 2v3h-3" stroke="currentColor" stroke-width="1.4" fill="none" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
              </button>
            </div>
          </div>

          <Show when={showNew()}>
            <div class="cw-new">
              <input
                class="cw-new-input"
                placeholder="工作区路径（如 /Users/me/proj）"
                value={newPath()}
                onInput={(e) => setNewPath(e.currentTarget.value)}
              />
              <input
                class="cw-new-input"
                placeholder="描述（可选）"
                value={newDesc()}
                onInput={(e) => setNewDesc(e.currentTarget.value)}
              />
              <button class="cw-new-go" disabled={loading()} onClick={addSession}>
                {loading() ? '创建中…' : '创建'}
              </button>
            </div>
          </Show>

          <Show when={error()}>
            <div class="cw-error">{error()}</div>
          </Show>

          <div class="cw-slist">
            <For each={sessions()}>
              {(s) => (
                <div
                  class={clsx('cw-sitem', s.id === activeId() && 'active')}
                  onClick={() => setActiveId(s.id)}
                >
                  <div class="cw-sitem-name">{s.name}</div>
                  <span class="s">
                    {statusLabel(s.status)} · 读 {s.files_read} · 建 {s.files_created} · 改 {s.files_modified}
                  </span>
                </div>
              )}
            </For>
            <Show when={sessions().length === 0 && !loading()}>
              <div class="cw-empty">暂无会话，点击 + 新建</div>
            </Show>
          </div>
        </div>

        {/* 右栏：任务详情 + 交付物 */}
        <div class="cw-main">
          <Show when={a} fallback={<div class="cw-empty-main">选择或新建一个协同会话</div>}>
            {(s) => (
              <div class="cw-content">
                <div class="cw-header">
                  <div>
                    <div class="cw-title">{s().name}</div>
                    <div class="cw-sub">
                      {s().workspace_path} · 读 {s().files_read} · 建 {s().files_created} · 改 {s().files_modified}
                    </div>
                  </div>
                  <div class="cw-header-right">
                    <div class={clsx('badge', statusBadge(s().status))}>{statusLabel(s().status)}</div>
                    <Show when={s().status === 'active'}>
                      <button class="cw-ctl" onClick={() => controlSession('pause')} title="暂停" aria-label="暂停">⏸</button>
                    </Show>
                    <Show when={s().status === 'paused'}>
                      <button class="cw-ctl" onClick={() => controlSession('resume')} title="恢复" aria-label="恢复">▶</button>
                    </Show>
                    <Show when={s().status !== 'stopped' && s().status !== 'completed'}>
                      <button class="cw-ctl" onClick={() => controlSession('stop')} title="停止" aria-label="停止">⏹</button>
                    </Show>
                  </div>
                </div>

                <Show when={s().description}>
                  <div class="cw-desc">{s().description}</div>
                </Show>

                <div class="cw-section-title">
                  <svg viewBox="0 0 12 12">
                    <line x1="2" y1="6" x2="10" y2="6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                    <circle cx="6" cy="6" r="1.5" fill="none" stroke="currentColor" stroke-width="1" />
                  </svg>
                  行动
                </div>
                <div class="cw-tlist">
                  <For each={actions()} fallback={<div class="cw-empty">暂无行动</div>}>
                    {(act) => (
                      <div class="cw-task">
                        <span class={clsx('dot', act.status === 'completed' && 'done', (act.status === 'failed' || act.status === 'error') && 'fail')} />
                        <span class="tname">{act.action_type}</span>
                        <span class="tpath">{act.target_path}</span>
                        <span class="tstat">{statusLabel(act.status)}</span>
                      </div>
                    )}
                  </For>
                </div>

                <div class="cw-section-title">
                  <svg viewBox="0 0 12 12">
                    <path d="M2 3h8v6H2z" stroke="currentColor" stroke-width="1" fill="none" />
                    <path d="M4 5h4" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
                  </svg>
                  交付物
                </div>
                <div class="cw-deliverables">
                  <For each={deliverables()} fallback={<div class="cw-empty">暂无交付物</div>}>
                    {(d) => (
                      <div class="cw-deliverable">
                        <span class="dname">{d.name}</span>
                        <span class="dkind">{d.kind}</span>
                        <span class="dpath">{d.path}</span>
                        <Show when={d.quality_score != null}>
                          <span class="dscore">{d.quality_score}/100</span>
                        </Show>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            )}
          </Show>
        </div>
      </div>
    </div>
  )
}