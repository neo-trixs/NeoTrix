/**
 * useAgentActivity — Agent 活动追踪 (phase/domain/toolCount/log)
 */
import { createSignal } from 'solid-js'
import type { ActivityStep } from '../../../../components/AgentActivityLog'
import type { AgentPhase } from '../../../../components/AgentActivityBar'

export function useAgentActivity() {
  const [agentPhase, setAgentPhase] = createSignal<AgentPhase>('idle')
  const [agentDomain, setAgentDomain] = createSignal<string | null>(null)
  const [agentToolCount, setAgentToolCount] = createSignal(0)
  const [agentLastActivity, setAgentLastActivity] = createSignal<string | null>(null)
  const [agentLog, setAgentLog] = createSignal<ActivityStep[]>([])
  const [logOpen, setLogOpen] = createSignal(false)

  const pushLog = (step: Omit<ActivityStep, 'ts'>) => {
    setAgentLog((prev) => {
      const next = [...prev, { ...step, ts: Date.now() }]
      return next.length > 24 ? next.slice(next.length - 24) : next
    })
  }

  return {
    agentPhase, setAgentPhase,
    agentDomain, setAgentDomain,
    agentToolCount, setAgentToolCount,
    agentLastActivity, setAgentLastActivity,
    agentLog, pushLog,
    logOpen, setLogOpen,
  }
}
