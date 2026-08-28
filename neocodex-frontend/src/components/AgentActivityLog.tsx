import { For, Show, type JSX } from 'solid-js'
import { clsx } from 'clsx'

/** 活动步骤（OS 正在做什么的审计层，独立于对话线程） */
export interface ActivityStep {
  kind: 'phase' | 'reasoning' | 'tool' | 'error' | 'done'
  label: string
  detail?: string
  domain?: string
  ts: number
}

const KIND_CLASS: Record<ActivityStep['kind'], string> = {
  phase: 'agent-log__dot--phase',
  reasoning: 'agent-log__dot--reasoning',
  tool: 'agent-log__dot--tool',
  error: 'agent-log__dot--error',
  done: 'agent-log__dot--done',
}

function fmtTime(ts: number): string {
  const d = new Date(ts)
  const p = (n: number) => String(n).padStart(2, '0')
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`
}

/** AgentActivityLog — 滚动活动时间线（审计层），对标 2026 agent UI「独立活动面板」 */
export function AgentActivityLog(props: { steps: () => ActivityStep[] }): JSX.Element {
  return (
    <div class="agent-log">
      <Show
        when={props.steps().length > 0}
        fallback={<div class="agent-log__empty">暂无活动</div>}
      >
        <For each={props.steps()}>
          {(s) => (
            <div class="agent-log__row">
              <span class={clsx('agent-log__dot', KIND_CLASS[s.kind])} />
              <span class="agent-log__label">{s.label}</span>
              <Show when={s.domain}>
                <span class="agent-log__domain">{s.domain}</span>
              </Show>
              <Show when={s.detail}>
                <span class="agent-log__detail">{s.detail}</span>
              </Show>
              <span class="agent-log__time">{fmtTime(s.ts)}</span>
            </div>
          )}
        </For>
      </Show>
    </div>
  )
}
