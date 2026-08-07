import { createSignal, For, Show } from 'solid-js'
import { clsx } from 'clsx'

/* ════════════════════════════════════════════
   CoworkView — 协同会话管理（设计 v2）
   左栏：会话列表（cw-slist）
   右栏：任务看板（cw-tlist）+ 智能体网格（cw-agents）
   ════════════════════════════════════════════ */

interface CwAgent {
  n: string
  on: boolean
}

interface CwSession {
  name: string
  status: string
  tasks: number
  done: number
  fail: number
  agents: CwAgent[]
}

const CW_DATA: CwSession[] = [
  {
    name: '架构讨论', status: '进行中', tasks: 3, done: 1, fail: 0,
    agents: [{ n: '分析员', on: true }, { n: '架构师', on: true }, { n: '审查员', on: false }],
  },
  {
    name: '代码审查 Sprint', status: '进行中', tasks: 5, done: 3, fail: 0,
    agents: [{ n: '审查员', on: true }, { n: '检查员', on: true }],
  },
  {
    name: '文档生成', status: '已完成', tasks: 2, done: 2, fail: 0,
    agents: [{ n: '写手', on: false }],
  },
]

/* 状态 → 语义徽章类（与 badge-success/warn/error 体系一致） */
function statusBadge(status: string): string {
  if (status.includes('完成')) return 'badge-success'
  if (status.includes('失败') || status.includes('错误')) return 'badge-error'
  return 'badge-warn' // 进行中/待处理
}

export function CoworkView() {
  const [sessions, setSessions] = createSignal<CwSession[]>(CW_DATA)
  const [activeIdx, setActiveIdx] = createSignal(0)

  const active = () => sessions()[activeIdx()] ?? sessions()[0]

  const addSession = () => {
    const s: CwSession = { name: '新任务', status: '进行中', tasks: 1, done: 0, fail: 0, agents: [{ n: '协调员', on: true }] }
    setSessions([s, ...sessions()])
    setActiveIdx(0)
  }

  return (
    <div class="vw-cowork">
      <div class="cw-layout">
        {/* 左栏：会话列表 */}
        <div class="cw-sidebar">
          <div class="cw-shead">
            <span>会话</span>
            <button class="cw-add" onClick={addSession} title="新建会话" aria-label="新建会话">
              <svg viewBox="0 0 14 14">
                <line x1="7" y1="2" x2="7" y2="12" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                <line x1="2" y1="7" x2="12" y2="7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
              </svg>
            </button>
          </div>
          <div class="cw-slist">
            <For each={sessions()}>
              {(s, i) => {
                const pct = s.tasks > 0 ? Math.round((s.done / s.tasks) * 100) : 0
                return (
                  <div
                    class={clsx('cw-sitem', i() === activeIdx() && 'active')}
                    onClick={() => setActiveIdx(i())}
                  >
                    {s.name}
                    <span class="s">{s.done}/{s.tasks} 任务 · {pct}%</span>
                  </div>
                )
              }}
            </For>
          </div>
        </div>

        {/* 右栏：任务详情 + 智能体 */}
        <div class="cw-main">
          <Show when={active()}>
            {(a) => (
              <div class="cw-content">
                <div class="cw-header">
                  <div>
                    <div class="cw-title">{a().name}</div>
                    <div class="cw-sub">{a().tasks} 任务 · {a().done} 完成 · {a().fail} 失败</div>
                  </div>
                  <div class={clsx('badge', statusBadge(a().status))}>{a().status}</div>
                </div>

                <div class="cw-section-title">
                  <svg viewBox="0 0 12 12">
                    <line x1="2" y1="6" x2="10" y2="6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                    <circle cx="6" cy="6" r="1.5" fill="none" stroke="currentColor" stroke-width="1" />
                  </svg>
                  任务
                </div>
                <div class="cw-tlist">
                  <For each={Array.from({ length: a().tasks }, (_, i) => i)}>
                    {(i) => {
                      const done = i < a().done
                      const fail = !done && i < a().done + a().fail
                      const label = done ? '已完成' : fail ? '失败' : '进行中'
                      return (
                        <div class="cw-task">
                          <span class={clsx('dot', done && 'done', fail && 'fail')} />
                          <span class="tname">任务 #{i + 1}</span>
                          <span class="tstat">{label}</span>
                        </div>
                      )
                    }}
                  </For>
                </div>

                <div class="cw-section-title">
                  <svg viewBox="0 0 12 12">
                    <circle cx="6" cy="4" r="2.5" stroke="currentColor" stroke-width="1" fill="none" />
                    <path d="M2 11a4 4 0 018 0" stroke="currentColor" stroke-width="1" fill="none" stroke-linecap="round" />
                  </svg>
                  智能体
                </div>
                <div class="cw-agents">
                  <For each={a().agents}>
                    {(ag) => (
                      <div class="cw-agent">
                        <span class="adot" style={{ background: ag.on ? '#4caf50' : '#b0b0b8' }} />
                        {ag.n}
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
