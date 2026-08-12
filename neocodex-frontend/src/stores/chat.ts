
import { createStore, produce } from 'solid-js/store'
import { tagsStore, normalizeTagName } from './tags'
import { neocodex } from '../api'
import type {
  NeoCodexAttachmentDto,
  NeoCodexMessageItem,
  NeoCodexSessionInfo,
  ToolCallRecord,
} from '../api/types'

// 兼容再导出：旧导入点（routes/Chat, ToolCallCard, FilePreview）逐步迁移到 api/types
export type { NeoCodexAttachmentDto, NeoCodexMessageItem, NeoCodexSessionInfo, ToolCallRecord } from '../api/types'

export interface Message {
  id: string
  role: 'user' | 'assistant' | 'system' | 'tool'
  content: string
  timestamp: Date
  isStreaming?: boolean
  toolCalls?: ToolCallRecord[]
  attachments?: NeoCodexAttachmentDto[]
  metadata?: {
    model?: string
    tokens?: number
    duration?: number
  }
}

export interface Session {
  id: string
  title: string
  messages: Message[]
  createdAt: Date
  updatedAt: Date
  checkpointId?: string
  /** 项目名（从会话 wire_path 提取，对标 Claude 项目分组） */
  project?: string
  /** 会话标签（本地持久化，对标 Obsidian tag） */
  tags: string[]
}

export interface ChatState {
  sessions: Session[]
  currentSessionId: string | null
  isGenerating: boolean
  abortController: AbortController | null
  isLoadingSessions: boolean
  isLoadingMessages: boolean
}

function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 11)}`
}

/** 新会话默认标题（首条用户消息前占位；自动命名后替换，对标 Claude） */
const DEFAULT_SESSION_TITLE = '新对话'

/** 从会话 wire_path 提取项目名（取路径中项目目录段；含 neotrix 仓库关键词时取上一级） */
function projectFromPath(wirePath: string): string | undefined {
  if (!wirePath) return undefined
  const segs = wirePath.split('/').filter(Boolean)
  if (segs.length < 2) return segs[segs.length - 1]
  const last = segs[segs.length - 1]
  if (last === 'neotrix' || last === 'NeoTrix') return segs[segs.length - 2]
  return last
}

export function convertBackendMessage(item: NeoCodexMessageItem): Message {
  const msg: Message = {
    id: `msg-${item.id}`,
    role: item.role as Message['role'],
    content: item.content,
    timestamp: new Date(item.timestamp * 1000),
    isStreaming: false,
    attachments: item.attachments,
  }
  // Restored tool calls become structured cards (parity with live streaming).
  if (item.role === 'tool' && item.tool_call) {
    const tc = item.tool_call
    msg.toolCalls = [{
      id: `msg-${item.id}-tool`,
      name: tc.name,
      args: tc.args,
      result: tc.result,
      duration_ms: tc.duration_ms,
      success: tc.success,
    }]
    msg.metadata = { model: undefined }
  }
  return msg
}

function createChatStore() {
  const [state, setState] = createStore<ChatState>({
    sessions: [],
    currentSessionId: null,
    isGenerating: false,
    abortController: null,
    isLoadingSessions: false,
    isLoadingMessages: false,
  })

  const currentSession = () => 
    state.sessions.find(s => s.id === state.currentSessionId) || null

  const currentMessages = () => 
    currentSession()?.messages || []

  // Load sessions from backend
  const loadSessions = async (projectPath?: string): Promise<void> => {
    setState('isLoadingSessions', true)
    try {
      const backendSessions = await neocodex.listSessions(projectPath)
      
      const sessions: Session[] = backendSessions.map(s => ({
        id: s.id,
        title: s.name,
        messages: [],
        createdAt: new Date(s.updated_at * 1000),
        updatedAt: new Date(s.updated_at * 1000),
        project: projectFromPath(s.wire_path),
        tags: s.tags ?? tagsStore.tagsForSession(s.id),
      }))

      // 后端标签回填 tags store（本地已有则不覆盖），随后同步到 session.tags
      for (const s of backendSessions) {
        tagsStore.importSessionTags(s.id, s.tags ?? [])
      }
      
      setState('sessions', sessions)
      hydrateSessionTags()
      
      // If no current session, select the first one
      if (!state.currentSessionId && sessions.length > 0) {
        await switchSession(sessions[0].id)
      }
    } catch (error) {
      console.error('[chatStore] Failed to load sessions:', error)
      // Fallback: create a new session if none exist
      if (state.sessions.length === 0) {
        addSession()
      }
    } finally {
      setState('isLoadingSessions', false)
    }
  }

  // Load messages for a specific session
  const loadSessionMessages = async (sessionId: string): Promise<void> => {
    setState('isLoadingMessages', true)
    try {
      const backendMessages = await neocodex.getSessionMessages(sessionId)
      
      const messages = backendMessages.map(convertBackendMessage)
      
      setState('sessions', produce(s => {
        const sess = s.find(sess => sess.id === sessionId)
        if (sess) {
          sess.messages = messages
        }
      }))
    } catch (error) {
      console.error('[chatStore] Failed to load session messages:', error)
    } finally {
      setState('isLoadingMessages', false)
    }
  }

  const addSession = async (title = DEFAULT_SESSION_TITLE): Promise<string> => {
    try {
      const backendSession = await neocodex.createSession(title)
      
      const session: Session = {
        id: backendSession.id,
        title: backendSession.name,
        messages: [],
        createdAt: new Date(backendSession.updated_at * 1000),
        updatedAt: new Date(backendSession.updated_at * 1000),
        tags: [],
      }
      
      setState('sessions', produce(s => s.unshift(session)))
      await switchSession(session.id)
      return session.id
    } catch (error) {
      console.error('[chatStore] Failed to create session:', error)
      // Fallback to local-only session
      const session: Session = {
        id: generateId(),
        title,
        messages: [],
        createdAt: new Date(),
        updatedAt: new Date(),
        tags: [],
      }
      setState('sessions', produce(s => s.unshift(session)))
      setState('currentSessionId', session.id)
      return session.id
    }
  }

  const deleteSession = async (id: string): Promise<void> => {
    try {
      await neocodex.deleteSession(id)
    } catch (error) {
      console.error('[chatStore] Failed to delete session:', error)
    }
    
    setState('sessions', produce(s => {
      const idx = s.findIndex(sess => sess.id === id)
      if (idx !== -1) s.splice(idx, 1)
    }))
    
    if (state.currentSessionId === id) {
      const nextId = state.sessions[0]?.id || null
      setState('currentSessionId', nextId)
      // 回退会话需要加载消息，否则消息区为空
      if (nextId) {
        await loadSessionMessages(nextId)
      }
    }

    // 清理标签映射（localStorage）
    tagsStore.clearSessionTags(id)
  }

  /** 归档会话（对标 Claude Code Archive）：后端移入 archived/ 并从活跃列表移除 */
  const archiveSession = async (id: string): Promise<void> => {
    try {
      await neocodex.archiveSession(id)
    } catch (error) {
      console.error('[chatStore] Failed to archive session:', error)
      return
    }
    setState('sessions', produce(s => {
      const idx = s.findIndex(sess => sess.id === id)
      if (idx !== -1) s.splice(idx, 1)
    }))
    // 当前会话被归档时切到相邻会话（回退需加载消息，否则消息区为空）
    if (state.currentSessionId === id) {
      const nextId = state.sessions[0]?.id || null
      setState('currentSessionId', nextId)
      if (nextId) {
        await loadSessionMessages(nextId)
      }
    }
  }

  /** 恢复归档会话：后端移回活跃列表，重新拉取列表并切入该会话 */
  const restoreSession = async (id: string): Promise<void> => {
    try {
      await neocodex.restoreSession(id)
    } catch (error) {
      console.error('[chatStore] Failed to restore session:', error)
      return
    }
    // 重新拉取活跃列表（含恢复的会话）
    await loadSessions()
    if (state.sessions.some(sess => sess.id === id)) {
      await switchSession(id)
    }
  }

  /** 列出归档会话（只读查询；渲染层拉取，失败返回空列表） */
  const listArchived = async (): Promise<NeoCodexSessionInfo[]> => {
    try {
      return await neocodex.listArchived()
    } catch (error) {
      console.error('[chatStore] Failed to list archived sessions:', error)
      return []
    }
  }

  const switchSession = async (id: string): Promise<void> => {
    if (state.currentSessionId === id) return

    // 若仍在流式生成，先中止后端流，避免旧会话的 token 事件污染新会话
    if (state.isGenerating) {
      try {
        await neocodex.stopStream()
      } catch (error) {
        console.error('[chatStore] Failed to stop stream on switch:', error)
      }
      setState('isGenerating', false)
    }
    
    setState('currentSessionId', id)
    
    // Load messages for the new session
    await loadSessionMessages(id)
    
    // Notify backend to switch context
    try {
      await neocodex.switchSession(id)
    } catch (error) {
      console.error('[chatStore] Failed to switch session on backend:', error)
    }
  }

  const updateSessionTitle = async (id: string, title: string): Promise<void> => {
    try {
      await neocodex.renameSession(id, title)
    } catch (error) {
      console.error('[chatStore] Failed to rename session:', error)
    }
    
    setState('sessions', produce(s => {
      const sess = s.find(sess => sess.id === id)
      if (sess) sess.title = title
    }))
  }

  const addMessage = (message: Omit<Message, 'id' | 'timestamp'>): string => {
    const msg: Message = {
      ...message,
      id: generateId(),
      timestamp: new Date(),
    }
    let autoTitle: string | null = null
    setState('sessions', produce(s => {
      const sess = s.find(sess => sess.id === state.currentSessionId)
      if (sess) {
        // 自动命名（对标 Claude）：标题仍为默认占位（「新对话」/空）且收到首条
        // 用户消息时，用首条消息前 24 字命名（超出加 …）。未发消息前保持空态占位。
        const isDefaultTitle = sess.title === DEFAULT_SESSION_TITLE || sess.title.trim() === ''
        const hasUserMsg = sess.messages.some(m => m.role === 'user')
        if (message.role === 'user' && isDefaultTitle && !hasUserMsg) {
          const trimmed = message.content.trim()
          if (trimmed) {
            autoTitle = trimmed.length > 24 ? `${trimmed.slice(0, 24)}…` : trimmed
            sess.title = autoTitle
          }
        }
        sess.messages.push(msg)
        sess.updatedAt = new Date()
      }
    }))
    // 自动命名异步持久化到后端（失败不阻断发送）
    if (autoTitle !== null && state.currentSessionId) {
      neocodex.renameSession(state.currentSessionId, autoTitle).catch((error) => {
        console.error('[chatStore] Failed to persist auto-title:', error)
      })
    }
    return msg.id
  }

  const updateMessage = (id: string, content: string, isStreaming = false): void => {
    setState('sessions', produce(s => {
      const sess = s.find(sess => sess.id === state.currentSessionId)
      if (sess) {
        const msg = sess.messages.find(m => m.id === id)
        if (msg) {
          msg.content = content
          msg.isStreaming = isStreaming
        }
      }
    }))
  }

  /** 结束流式消息：保留内容，仅清除 streaming 标记（停止/取消兜底路径，供 Chat 本地复位） */
  const finishMessage = (id: string): void => {
    setState('sessions', produce(s => {
      const sess = s.find(sess => sess.id === state.currentSessionId)
      if (sess) {
        const msg = sess.messages.find(m => m.id === id)
        if (msg) msg.isStreaming = false
      }
    }))
  }

  const messageContent = (id: string): string | null => {
    const sess = currentSession()
    if (!sess) return null
    return sess.messages.find(m => m.id === id)?.content ?? null
  }

  const appendMessageContent = (id: string, delta: string): void => {
    setState('sessions', produce(s => {
      const sess = s.find(sess => sess.id === state.currentSessionId)
      if (sess) {
        const msg = sess.messages.find(m => m.id === id)
        if (msg) {
          msg.content += delta
        }
      }
    }))
  }

  const appendToolCall = (id: string, toolCall: ToolCallRecord): void => {
    setState('sessions', produce(s => {
      const sess = s.find(sess => sess.id === state.currentSessionId)
      if (sess) {
        const msg = sess.messages.find(m => m.id === id)
        if (msg) {
          if (!msg.toolCalls) msg.toolCalls = []
          msg.toolCalls.push(toolCall)
        }
      }
    }))
  }

  const deleteMessage = (id: string): void => {
    setState('sessions', produce(s => {
      const sess = s.find(sess => sess.id === state.currentSessionId)
      if (sess) {
        const idx = sess.messages.findIndex(m => m.id === id)
        if (idx !== -1) sess.messages.splice(idx, 1)
      }
    }))
  }

  const clearMessages = (): void => {
    setState('sessions', produce(s => {
      const sess = s.find(sess => sess.id === state.currentSessionId)
      if (sess) {
        sess.messages = []
        sess.updatedAt = new Date()
      }
    }))
  }

  const setGenerating = (generating: boolean, controller?: AbortController): void => {
    setState('isGenerating', generating)
    if (controller) {
      setState('abortController', controller)
    } else if (!generating) {
      setState('abortController', null)
    }
  }

  const abortGeneration = (): void => {
    state.abortController?.abort()
    setState('isGenerating', false)
    setState('abortController', null)
  }

  /** 按目标消息定位重生成轮：截断被点 assistant 消息及之后全部消息，返回其所在轮的 user 内容 */
  const regenerateFrom = (messageId: string): string => {
    const messages = currentMessages()
    const idx = messages.findIndex(m => m.id === messageId)
    if (idx === -1 || messages[idx]?.role !== 'assistant') return ''
    // 定位被点消息所在轮：其之前最近的一条用户消息
    let userIdx = -1
    for (let i = idx - 1; i >= 0; i--) {
      if (messages[i].role === 'user') { userIdx = i; break }
    }
    if (userIdx === -1) return ''
    // 截断该条助手消息及之后全部消息（该轮重生成，后续消息一并移除）
    setState('sessions', produce(s => {
      const sess = s.find(sess => sess.id === state.currentSessionId)
      if (sess) sess.messages = sess.messages.slice(0, idx)
    }))
    return messages[userIdx].content
  }

  const editAndResend = (messageId: string, newContent: string): boolean => {
    const messages = currentMessages()
    const msgIndex = messages.findIndex(m => m.id === messageId)
    
    if (msgIndex !== -1) {
      setState('sessions', produce(s => {
        const sess = s.find(sess => sess.id === state.currentSessionId)
        if (sess) {
          sess.messages = sess.messages.slice(0, msgIndex)
        }
      }))
      
      addMessage({ role: 'user', content: newContent })
      return true
    }
    return false
  }

  const createCheckpoint = (): string | null => {
    const session = currentSession()
    if (session) {
      const checkpointId = `checkpoint-${generateId()}`
      setState('sessions', produce(s => {
        const sess = s.find(sess => sess.id === state.currentSessionId)
        if (sess) {
          sess.checkpointId = checkpointId
        }
      }))
      return checkpointId
    }
    return null
  }

  const rewindToCheckpoint = (checkpointId: string): void => {
    console.log('Rewind to checkpoint:', checkpointId)
  }

  /** 从 tags store 同步某会话的标签到响应式 session（渲染层拉取） */
  const tagsForSession = (sessionId: string): string[] => {
    return tagsStore.tagsForSession(sessionId)
  }

  /** 给会话打标（自动注册标签；本地优先，异步持久化到后端） */
  const tagSession = async (sessionId: string, rawName: string): Promise<string[]> => {
    tagsStore.addSessionTag(sessionId, rawName)
    // 同步响应式 session.tags（驱动会话行标签展示）
    setState('sessions', produce(s => {
      const sess = s.find(x => x.id === sessionId)
      if (sess) sess.tags = tagsStore.tagsForSession(sessionId)
    }))
    // 异步持久化到 JSONL SessionMeta（失败不阻断本地体验）
    try {
      const info = await neocodex.tagSession(sessionId, normalizeTagName(rawName))
      setState('sessions', produce(s => {
        const sess = s.find(x => x.id === sessionId)
        if (sess) sess.tags = info.tags ?? []
      }))
      return info.tags ?? []
    } catch (error) {
      console.error('[chatStore] Failed to persist tag:', error)
      return tagsStore.tagsForSession(sessionId)
    }
  }

  /** 取消会话标签 */
  const untagSession = async (sessionId: string, name: string): Promise<string[]> => {
    tagsStore.removeSessionTag(sessionId, name)
    setState('sessions', produce(s => {
      const sess = s.find(x => x.id === sessionId)
      if (sess) sess.tags = tagsStore.tagsForSession(sessionId)
    }))
    // 异步持久化到后端
    try {
      const info = await neocodex.untagSession(sessionId, name)
      setState('sessions', produce(s => {
        const sess = s.find(x => x.id === sessionId)
        if (sess) sess.tags = info.tags ?? []
      }))
      return info.tags ?? []
    } catch (error) {
      console.error('[chatStore] Failed to persist untag:', error)
      return tagsStore.tagsForSession(sessionId)
    }
  }

  /** 会话加载后补挂标签（loadSessions 后调用，渲染入口统一） */
  const hydrateSessionTags = (): void => {
    setState('sessions', produce(s => {
      for (const sess of s) {
        sess.tags = tagsStore.tagsForSession(sess.id)
      }
    }))
  }

  return {
    get state() { return state },
    get currentSession() { return currentSession() },
    get currentMessages() { return currentMessages() },
    get isGenerating() { return state.isGenerating },
    get isLoadingSessions() { return state.isLoadingSessions },
    get isLoadingMessages() { return state.isLoadingMessages },
    loadSessions,
    loadSessionMessages,
    addSession,
    deleteSession,
    archiveSession,
    restoreSession,
    listArchived,
    switchSession,
    updateSessionTitle,
    addMessage,
    updateMessage,
    finishMessage,
    appendMessageContent,
    messageContent,
    appendToolCall,
    deleteMessage,
    clearMessages,
    setGenerating,
    abortGeneration,
    regenerateFrom,
    editAndResend,
    createCheckpoint,
    rewindToCheckpoint,
    tagsForSession,
    tagSession,
    untagSession,
    hydrateSessionTags,
  }
}

export const chatStore = createChatStore()