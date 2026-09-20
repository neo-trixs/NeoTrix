/**
 * useContextState — 上下文用量轮询 + 压缩提示
 */
import { createSignal, createEffect } from 'solid-js'
import { chatStore } from '../../../../stores/chat'
import { neocodex, errText } from '../../../../api'
import { usePolling } from '../../../../lib/usePolling'
import { query } from '../../../../api/query'
import type { AgentStatus } from '../../../../api/types'

export function useContextState(deps: {
  isGenerating: () => boolean
  showInfo: (msg: string, ms?: number) => void
  setStreamError: (msg: string | null) => void
}) {
  const [contextPct, setContextPct] = createSignal<number | null>(null)
  const [compactHintDismissed, setCompactHintDismissed] = createSignal(false)
  const [compacting, setCompacting] = createSignal(false)

  const compactHintVisible = () => {
    const p = contextPct()
    if (p === null || p < 80) return false
    return !compactHintDismissed()
  }

  const runCompact = async () => {
    if (compacting() || deps.isGenerating()) return
    const sessionId = chatStore.currentSession?.id ?? ''
    if (!sessionId) {
      deps.setStreamError('当前没有激活会话，无法压缩')
      setTimeout(() => deps.setStreamError(null), 3000)
      return
    }
    setCompacting(true)
    try {
      await neocodex.compactSession(sessionId, 8)
      await chatStore.loadSessionMessages(sessionId)
      setCompactHintDismissed(true)
      deps.showInfo('上下文已压缩，更早的对话被截断', 3000)
    } catch (error) {
      console.error('[Chat] Compact session failed:', error)
      deps.setStreamError(errText(error) || '压缩会话失败，请重试')
      setTimeout(() => deps.setStreamError(null), 3000)
    } finally {
      setCompacting(false)
    }
  }

  // 轮询 agent status 获取 context_usage
  usePolling({
    intervalMs: 15000,
    immediate: true,
    run: async () => {
      try {
        const s = await query<AgentStatus>('agent_status', () => neocodex.agentStatus(), { ttlMs: 3000 })
        if (s && typeof s.context_usage === 'number') setContextPct(s.context_usage * 100)
      } catch { /* 只读轮询，失败静默 */ }
    },
  })

  createEffect(() => {
    const p = contextPct()
    if (p !== null && p < 80) setCompactHintDismissed(false)
  })

  return {
    contextPct, compactHintDismissed, setCompactHintDismissed,
    compactHintVisible, compacting, runCompact,
  }
}
