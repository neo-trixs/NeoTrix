import { Show, type JSX } from 'solid-js'
import { clsx } from 'clsx'

/* ════════════════════════════════════════════════════════════
   AgentActivityBar — 「对话即操作系统」实时活动透明度层
   对标 2026 Agent UX 头号原则：agent 操作必须可观测（anti black-box）。
   聚合既有 neocodex_stream_* 事件为可见阶段 + 活跃域 + 工具计数，
   让 OS「思考/调用工具」对用户实时可见。后端无需改动（纯前端聚合）。
   ════════════════════════════════════════════════════════════ */

export type AgentPhase = 'idle' | 'thinking' | 'generating' | 'tooling' | 'error' | 'done'

const PHASE_LABEL: Record<AgentPhase, string> = {
  idle: '空闲',
  thinking: '思考中',
  generating: '生成中',
  tooling: '调用工具',
  error: '出错',
  done: '完成',
}

export interface AgentActivityBarProps {
  phase: () => AgentPhase
  activeDomain: () => string | null
  toolCount: () => number
  lastActivity: () => string | null
}

export function AgentActivityBar(props: AgentActivityBarProps): JSX.Element {
  return (
    <Show when={props.phase() !== 'idle'}>
      <div
        class={clsx('agent-activity', `agent-activity--${props.phase()}`)}
        title={props.lastActivity() ?? PHASE_LABEL[props.phase()]}
        aria-live="polite"
      >
        <span class="agent-activity__dot" aria-hidden="true" />
        <span class="agent-activity__label">{PHASE_LABEL[props.phase()]}</span>
        <Show when={props.activeDomain()}>
          <span class="agent-activity__domain">{props.activeDomain()}</span>
        </Show>
        <Show when={props.toolCount() > 0}>
          <span class="agent-activity__tools">⚙ {props.toolCount()}</span>
        </Show>
      </div>
    </Show>
  )
}
