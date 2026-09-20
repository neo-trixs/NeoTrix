/**
 * widgets/chat-panel/model/useStreamHandlers.ts — 流式事件处理
 *
 * 从 Chat.tsx 的 onMount 流式事件订阅提取
 */
import { onMount, onCleanup } from 'solid-js'
import { chatStore, type ToolCallRecord } from '../../../stores/chat'
import { subscribeStream, subscribeMenuEvents, type StreamToolPayload } from '../../../api/events'
import { neocodex } from '../../../api'
import { rootCause } from '../../../lib/errorRootCause'
import type { ChatStateReturn } from './useChatState'

export function useStreamHandlers(state: ChatStateReturn) {
  const setupStreamListeners = async () => {
    const unlistenStream = await subscribeStream({
      onStart: () => {
        state.activeGen.value = ++state.generation.value
        state.setAgentPhase('thinking')
        state.setAgentToolCount(() => 0)
        state.setAgentLastActivity(null)
        state.pushLog({ kind: 'phase', label: '开始思考' })
      },
      onToken: (delta: string) => {
        if (state.activeGen.value !== state.generation.value) return
        const msgId = state.currentAssistantMsgId()
        if (msgId) chatStore.appendMessageContent(msgId, delta)
        if (state.agentPhase() === 'thinking') {
          state.setAgentPhase('generating')
          state.pushLog({ kind: 'phase', label: '生成回复' })
        }
      },
      onEnd: (content: string) => {
        if (state.activeGen.value !== state.generation.value) return
        const msgId = state.currentAssistantMsgId()
        if (msgId) chatStore.updateMessage(msgId, content, false)
      },
      onDone: (payload: { content: string; cancelled?: boolean }) => {
        if (state.activeGen.value !== state.generation.value) return
        const msgId = state.currentAssistantMsgId()
        const wasCancelled = payload.cancelled
        if (msgId) {
          chatStore.updateMessage(msgId, payload.content, false)
          if (!wasCancelled && state.permissionMode() === 'plan') {
            state.setPlanPending({ msgId })
          }
        }
        chatStore.setGenerating(false)
        state.setCurrentAssistantMsgId(null)
        state.setAgentPhase('done')
        state.pushLog({ kind: wasCancelled ? 'error' : 'done', label: wasCancelled ? '生成已停止' : '完成' })
        setTimeout(() => state.setAgentPhase('idle'), 1500)
        if (state.streamWatchdogTimer.value) { clearTimeout(state.streamWatchdogTimer.value); state.streamWatchdogTimer.value = undefined }
      },
      onTool: (payload: StreamToolPayload) => {
        if (state.activeGen.value !== state.generation.value) return
        const msgId = state.currentAssistantMsgId()
        const domain = state.harnessRoute()?.domain
        if (msgId) {
          const toolCall: ToolCallRecord = {
            id: `tool-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
            name: payload.name,
            args: payload.args,
            result: payload.result,
            duration_ms: payload.duration_ms,
            success: payload.success,
            domain,
          }
          chatStore.appendToolCall(msgId, toolCall)
        }
        state.setAgentPhase('tooling')
        state.setAgentToolCount((c: number) => c + 1)
        state.setAgentLastActivity(`工具调用：${payload.name}${payload.success ? '' : '（失败）'}`)
        state.pushLog({ kind: 'tool', label: payload.name, detail: payload.success ? '成功' : '失败', domain })
      },
      onError: (payload: { message?: string; partial?: string; what?: string; why?: string; next?: string }) => {
        if (state.activeGen.value !== state.generation.value) return
        const msgId = state.currentAssistantMsgId()
        if (msgId) {
          const text = payload.partial || `⚠️ ${payload.message}`
          chatStore.updateMessage(msgId, text, false)
        }
        chatStore.setGenerating(false)
        state.setCurrentAssistantMsgId(null)
        state.setStreamError(payload.message || '生成失败')
        setTimeout(() => state.setStreamError(null), 5000)
        const rc = rootCause(payload.message || payload.what || '生成失败')
        state.setStreamErrorDetail({
          what: payload.what ?? payload.message ?? '生成失败',
          why: payload.why ?? rc.why,
          next: payload.next ?? rc.next,
        })
        setTimeout(() => state.setStreamErrorDetail(null), 6000)
        state.setAgentPhase('error')
        state.setAgentLastActivity(payload.message || '生成失败')
        state.pushLog({ kind: 'error', label: payload.message || '生成失败' })
        state.generation.value++
        if (state.streamWatchdogTimer.value) { clearTimeout(state.streamWatchdogTimer.value); state.streamWatchdogTimer.value = undefined }
      },
      onReasoning: (payload: { text: string }) => {
        if (state.activeGen.value !== state.generation.value) return
        state.pushLog({ kind: 'reasoning', label: payload.text.slice(0, 80) })
      },
      onSubscribeError: (event: string) => {
        state.setStreamError(`流式事件 ${event} 订阅失败，回复可能不完整`)
        setTimeout(() => state.setStreamError(null), 5000)
      },
    })
    state.setUnlistenStream(() => unlistenStream)

    // 读取当前激活模型
    try { const cfg = await neocodex.providerConfig(); state.setActiveModel(cfg.active_model || null) } catch { /* 静默 */ }
    // 读取应用版本
    try { state.setAppVersion(await neocodex.appVersion()) } catch { /* 静默 */ }

    // 监听提供商切换事件
    const onProviderChanged = () => {
      neocodex.providerConfig().then((cfg) => state.setActiveModel(cfg.active_model || null)).catch(() => {})
    }
    window.addEventListener('neotrix:provider-changed', onProviderChanged)

    // 全局键盘事件
    const onGlobalKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault(); state.setPaletteOpen((open: boolean) => !open); return
      }
      if ((e.metaKey || e.ctrlKey) && e.key === '/') {
        e.preventDefault(); state.setShortcutHelpOpen((open: boolean) => !open); return
      }
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'f') {
        e.preventDefault(); state.setMsgSearchOpen(true); state.setMsgSearch('')
        requestAnimationFrame(() => state.msgSearchInput()?.focus()); return
      }
      if (e.key !== 'Escape') return
      if (state.shortcutHelpOpen()) { state.setShortcutHelpOpen(false); return }
      if (state.paletteOpen()) { state.setPaletteOpen(false); return }
      if (state.activePanel()) { state.setActivePanel(null); return }
      if (state.settingsOpen()) { state.setSettingsOpen(false); return }
    }
    window.addEventListener('keydown', onGlobalKeyDown)

    // 桌面菜单事件桥
    const unlistenMenu = await subscribeMenuEvents({
      onNewSession: () => chatStore.addSession(),
      onOpenSettings: () => state.setSettingsOpen(true),
      onOpenPalette: () => state.setPaletteOpen(true),
      onCheckUpdates: () => state.setSettingsOpen(true),
    })
    state.setUnlistenMenu(() => unlistenMenu)

    // Cleanup
    onCleanup(() => {
      state.unlistenStream()?.()
      state.unlistenMenu()?.()
      window.removeEventListener('neotrix:provider-changed', onProviderChanged)
      window.removeEventListener('keydown', onGlobalKeyDown)
      if (state.isGenerating()) {
        chatStore.abortGeneration()
        const msgId = state.currentAssistantMsgId()
        if (msgId) chatStore.finishMessage(msgId)
      }
    })
  }

  onMount(() => {
    chatStore.loadSessions()
    void setupStreamListeners()
  })
}
