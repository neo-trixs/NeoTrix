/**
 * widgets/chat-panel/model/useChatActions.ts — Chat 操作逻辑
 *
 * 从 Chat.tsx 提取：sendMessage, handleSend, handleRegenerate, handleEditMessage,
 * handleSaveEdit, handleCancelEdit, handleBranch, handleQuote, copyMessage,
 * approvePlan, rejectPlan, cancelPlan, handleStop, handlePickAttachment,
 * handlePasteImage, removeAttachment, exportConversation
 */
import { type ChatStateReturn } from './useChatState'
import { chatStore, type Message } from '../../../stores/chat'
import { tagsStore } from '../../../stores/tags'
import { PERMISSION_MODES } from '../../../components/PermissionModeSelector'
import { neocodex, system, errText } from '../../../api'
import type { NeoCodexAttachmentDto } from '../../../stores/chat'
import { guessMime, estimateTokens } from '../../../lib/text'

export interface ChatActionsReturn {
  sendMessage: (content: string, opts?: { userMessageAdded?: boolean; regenerate?: boolean }) => Promise<void>
  handleSend: () => Promise<void>
  handleStop: () => Promise<void>
  handleRegenerate: (message: Message) => void
  handleEditMessage: (message: Message) => void
  handleSaveEdit: () => void
  handleCancelEdit: () => void
  handleBranch: (message: Message) => Promise<void>
  handleQuote: (message: Message) => void
  copyMessage: (m: Message) => Promise<void>
  handleCopy: (content: string, id: string) => Promise<void>
  approvePlan: () => Promise<void>
  rejectPlan: () => void
  cancelPlan: () => void
  handlePickAttachment: () => Promise<void>
  handlePasteImage: (e: ClipboardEvent) => void
  exportConversation: () => void
  buildExport: (fmt: 'md' | 'json' | 'html') => string
  copyExport: (fmt: 'md' | 'json' | 'html') => Promise<void>
  downloadMd: () => void
}

export function useChatActions(state: ChatStateReturn): ChatActionsReturn {
  const saveDraft = (id: string, text: string) => {
    const DRAFT_KEY = 'nt_session_drafts'
    const d: Record<string, string> = (() => {
      try { return JSON.parse(localStorage.getItem(DRAFT_KEY) || '{}') } catch { return {} }
    })()
    if (text) d[id] = text
    else delete d[id]
    try { localStorage.setItem(DRAFT_KEY, JSON.stringify(d)) } catch { /* 忽略 */ }
  }

  const sendMessage = async (content: string, opts?: { userMessageAdded?: boolean; regenerate?: boolean }) => {
    if (!content || state.isGenerating()) return
    if (!state.currentSession()) await chatStore.addSession()
    state.setPlanPending(null)
    let userMsgId: string | null = null
    if (!opts?.userMessageAdded) {
      userMsgId = chatStore.addMessage({ role: 'user', content })
      const sid = chatStore.state.currentSessionId
      if (sid) tagsStore.autoTagFromText(sid, content)
    }
    const atts = state.pendingAttachments()
    state.setInputValue('')
    state.setAnnotationHint(null)
    state.setPendingAttachments(() => [])
    state.setMentionRefs(() => [])
    state.adjustTextarea()
    state.setStreamError(null)
    const assistantMsgId = chatStore.addMessage({ role: 'assistant', content: '', isStreaming: true })
    state.setCurrentAssistantMsgId(assistantMsgId)
    chatStore.setGenerating(true)
    if (state.streamWatchdogTimer.value) clearTimeout(state.streamWatchdogTimer.value)
    state.streamWatchdogTimer.value = setTimeout(() => {
      if (!state.isGenerating()) return
      console.error('[Chat] Stream watchdog fired: no done event, force resetting')
      state.setStreamError('回复超时，请重试')
      setTimeout(() => state.setStreamError(null), 3000)
      const msgId = state.currentAssistantMsgId()
      if (msgId) chatStore.finishMessage(msgId)
      chatStore.setGenerating(false)
      state.setCurrentAssistantMsgId(null)
      state.generation.value++
    }, 600_000)
    try {
      await neocodex.sendMessageStream({
        content,
        attachments: atts.length > 0 ? atts : undefined,
        regenerate: opts?.regenerate ?? false,
        permission_mode: state.permissionMode(),
        temperature: 0.7,
        max_tokens: 4096,
      })
    } catch (error) {
      console.error('[Chat] Send message failed:', error)
      state.setStreamError(errText(error) || '发送失败，请重试')
      setTimeout(() => state.setStreamError(null), 3000)
      if (userMsgId) chatStore.deleteMessage(userMsgId)
      if (assistantMsgId) chatStore.deleteMessage(assistantMsgId)
      chatStore.setGenerating(false)
      state.generation.value++
      state.setCurrentAssistantMsgId(null)
      if (state.streamWatchdogTimer.value) { clearTimeout(state.streamWatchdogTimer.value); state.streamWatchdogTimer.value = undefined }
    }
  }

  const handleSend = async () => {
    let content = state.inputValue().trim()
    if (!content && !state.annotationHint() && state.pendingAttachments().length === 0) return
    if (state.annotationHint() && content !== state.annotationHint()!) {
      content = `${content}\n\n${state.annotationHint()}`
    }
    state.setHarnessRoute(null)
    void (await import('../../../api')).harness.harnessExecute({ instruction: content })
      .then((r: any) => {
        if (r.capability_tag && r.capability_tag !== 'orchestration') {
          state.setHarnessRoute({ tag: r.capability_tag, domain: r.domain, specialist: r.specialist })
        }
      })
      .catch(() => {})
    await sendMessage(content)
  }

  const handleStop = async () => {
    state.generation.value++
    const msgId = state.currentAssistantMsgId()
    try { await neocodex.stopStream() } catch (e) { console.error('[Chat] Stop stream failed:', e) }
    chatStore.abortGeneration()
    if (msgId) chatStore.finishMessage(msgId)
    state.setCurrentAssistantMsgId(null)
    if (state.streamWatchdogTimer.value) { clearTimeout(state.streamWatchdogTimer.value); state.streamWatchdogTimer.value = undefined }
  }

  const handleRegenerate = (message: Message) => {
    if (state.isGenerating()) return
    const sid = chatStore.state.currentSessionId
    let visibleIdx = -1
    if (sid) {
      const msgs = chatStore.currentMessages
      const idx = msgs.findIndex(m => m.id === message.id)
      if (idx >= 0) {
        visibleIdx = msgs.slice(0, idx + 1).filter(m => m.role === 'user' || m.role === 'assistant').length - 1
      }
    }
    const userContent = chatStore.regenerateFrom(message.id)
    if (userContent) {
      if (sid && visibleIdx >= 0) {
        neocodex.regenerate(sid, visibleIdx).catch((e: Error) => {
          console.error('[Chat] 持久化重新生成失败:', e)
          state.setStreamError(errText(e) || '重新生成失败')
          setTimeout(() => state.setStreamError(null), 3000)
        })
      }
      sendMessage(userContent, { userMessageAdded: true, regenerate: true })
    }
  }

  const handleEditMessage = (message: Message) => {
    state.setEditingMessageId(message.id)
    state.setEditContent(message.content)
  }

  const handleSaveEdit = () => {
    if (state.isGenerating()) return
    const content = state.editContent().trim()
    const msgId = state.editingMessageId()
    if (msgId && content) {
      const sid = chatStore.state.currentSessionId
      let editVisibleIdx = -1
      if (sid) {
        const msgs = chatStore.currentMessages
        const idx = msgs.findIndex(m => m.id === msgId)
        if (idx >= 0) {
          editVisibleIdx = msgs.slice(0, idx).filter(m => m.role === 'user' || m.role === 'assistant').length
        }
      }
      chatStore.editAndResend(msgId, content)
      state.setEditingMessageId(null)
      state.setEditContent('')
      if (sid && editVisibleIdx >= 0) {
        neocodex.regenerate(sid, editVisibleIdx).catch((e: Error) => {
          console.error('[Chat] 持久化编辑失败:', e)
        })
      }
      sendMessage(content, { userMessageAdded: true })
    }
  }

  const handleCancelEdit = () => {
    state.setEditingMessageId(null)
    state.setEditContent('')
  }

  const handleBranch = async (message: Message) => {
    const newId = await chatStore.addSession()
    state.setInputValue(message.content)
    saveDraft(newId, message.content)
    state.setMsgSearch('')
  }

  const handleQuote = (message: Message) => {
    const snippet = message.content.replace(/\n+/g, ' ').trim().slice(0, 160)
    const who = message.role === 'user' ? '用户' : '助手'
    const quote = `> ${who}：${snippet}${message.content.length > 160 ? '…' : ''}\n\n`
    state.setInputValue(quote + state.inputValue())
    requestAnimationFrame(() => state.textareaRef()?.focus())
  }

  const copyMessage = async (m: Message) => {
    try { await navigator.clipboard.writeText(m.content) } catch { /* 忽略 */ }
  }

  const handleCopy = async (content: string, id: string) => {
    try {
      await navigator.clipboard.writeText(content)
      state.setCopiedId(id)
      setTimeout(() => state.setCopiedId(null), 1500)
    } catch { /* ignore */ }
  }

  const approvePlan = async () => {
    const pending = state.planPending()
    if (!pending) return
    const planText = chatStore.messageContent(pending.msgId) ?? ''
    state.setPlanPending(null)
    const targetMode = 'accept_edits' as const
    state.setPermissionMode(targetMode)
    state.showInfo(`计划已批准，切换至「${PERMISSION_MODES.find(m => m.value === targetMode)?.label}」执行`, 3000)
    const body = planText.trim() ? `已批准以下计划，请按计划执行：\n\n${planText}` : '已批准计划，请执行。'
    await sendMessage(body, { userMessageAdded: false })
    state.setApprovalAccepted((n: number) => n + 1)
  }

  const rejectPlan = () => {
    const pending = state.planPending()
    if (!pending) return
    state.setPlanPending(null)
    state.setApprovalRejected((n: number) => n + 1)
    state.showInfo('计划已拒绝，可继续规划或补充需求', 3000)
  }

  const cancelPlan = () => {
    state.setPlanPending(null)
    state.showInfo('计划已取消，保持在规划模式', 3000)
  }

  const handlePickAttachment = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({ multiple: true })
      if (!selected) return
      const paths = Array.isArray(selected) ? selected : [selected]
      const atts: NeoCodexAttachmentDto[] = []
      for (const p of paths) {
        try {
          const content = await system.readFile(p)
          const name = p.split('/').pop() || p
          const mime = guessMime(name)
          atts.push({ name, size: content.length, mime_type: mime, data: content })
        } catch (e) { console.error('[Chat] Read attachment failed:', p, e) }
      }
      if (atts.length > 0) state.setPendingAttachments(prev => [...prev, ...atts])
    } catch (e) { console.error('[Chat] Open dialog failed:', e) }
  }

  const handlePasteImage = (e: ClipboardEvent) => {
    const items = e.clipboardData?.items
    if (!items) return
    for (const item of Array.from(items)) {
      if (item.type.startsWith('image/')) {
        const file = item.getAsFile()
        if (!file) continue
        e.preventDefault()
        const reader = new FileReader()
        reader.onload = () => {
          const data = typeof reader.result === 'string' ? reader.result.split(',')[1] ?? '' : ''
          if (!data) return
          const att: NeoCodexAttachmentDto = {
            name: file.name || `pasted-image-${Date.now()}.png`,
            size: file.size,
            mime_type: item.type,
            data,
          }
          state.setPendingAttachments(prev => [...prev, att])
        }
        reader.readAsDataURL(file)
        break
      }
    }
  }

  const exportConversation = () => state.exportMenuOpen() ? state.setExportMenuOpen(false) : state.setExportMenuOpen(true)
  const buildExport = (fmt: 'md' | 'json' | 'html'): string => {
    const msgs = chatStore.currentMessages.filter((m) => m.role === 'user' || m.role === 'assistant')
    if (fmt === 'json') return JSON.stringify(msgs.map((m) => ({ role: m.role, content: m.content })), null, 2)
    const md = msgs.map((m) => `### ${m.role === 'user' ? '用户' : 'NeoTrix'}\n\n${m.content}`).join('\n\n')
    if (fmt === 'md') return md
    const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    return `<!doctype html><html lang="zh"><head><meta charset="utf-8"><title>NeoTrix 会话导出</title></head><body>${msgs.map((m) => `<h3>${m.role === 'user' ? '用户' : 'NeoTrix'}</h3><pre>${esc(m.content)}</pre>`).join('\n')}</body></html>`
  }
  const copyExport = async (fmt: 'md' | 'json' | 'html') => {
    try { await navigator.clipboard.writeText(buildExport(fmt)); state.showInfo(`会话已复制为 ${fmt.toUpperCase()}`, 2000) }
    catch { state.showInfo('复制失败（剪贴板不可用）', 2000) }
    state.setExportMenuOpen(false)
  }
  const downloadMd = () => {
    const blob = new Blob([buildExport('md')], { type: 'text/markdown' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a'); a.href = url; a.download = `neotrix-session-${chatStore.state.currentSessionId ?? 'export'}.md`; a.click()
    URL.revokeObjectURL(url); state.setExportMenuOpen(false)
  }

  return {
    sendMessage, handleSend, handleStop, handleRegenerate, handleEditMessage,
    handleSaveEdit, handleCancelEdit, handleBranch, handleQuote, copyMessage, handleCopy,
    approvePlan, rejectPlan, cancelPlan,
    handlePickAttachment, handlePasteImage,
    exportConversation, buildExport, copyExport, downloadMd,
  }
}
