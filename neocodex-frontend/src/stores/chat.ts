
import { createStore, produce } from 'solid-js/store'
import { invoke } from '@tauri-apps/api/core'
import { tagsStore } from './tags'

// Backend types matching neocodex_cmds.rs
export interface NeoCodexSessionInfo {
  id: string
  name: string
  mode: string
  message_count: number
  wire_path: string
  updated_at: number
}

export interface NeoCodexAttachmentDto {
  name: string
  size: number
  mime_type: string
  data?: string
}

export interface NeoCodexMessageItem {
  id: number
  role: string
  content: string
  timestamp: number
  attachments?: NeoCodexAttachmentDto[]
  tool_call?: {
    name: string
    args: string
    result: string
    duration_ms: number
    success: boolean
  }
}

export interface ToolCallRecord {
  id: string
  name: string
  args: string
  result: string
  duration_ms: number
  success: boolean
}

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
      const backendSessions = await invoke<NeoCodexSessionInfo[]>('neocodex_list_sessions', {
        project_path: projectPath,
      })
      
      const sessions: Session[] = backendSessions.map(s => ({
        id: s.id,
        title: s.name,
        messages: [],
        createdAt: new Date(s.updated_at * 1000),
        updatedAt: new Date(s.updated_at * 1000),
        project: projectFromPath(s.wire_path),
        tags: s.tags ?? tagsStore.tagsForSession(s.id),
      }))
      
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
      const backendMessages = await invoke<NeoCodexMessageItem[]>('neocodex_get_session_messages', {
        session_id: sessionId,
      })
      
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

  const addSession = async (title = '新对话'): Promise<string> => {
    try {
      const backendSession = await invoke<NeoCodexSessionInfo>('neocodex_create_session', {
        name: title,
      })
      
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
      await invoke('neocodex_delete_session', { session_id: id })
    } catch (error) {
      console.error('[chatStore] Failed to delete session:', error)
    }
    
    setState('sessions', produce(s => {
      const idx = s.findIndex(sess => sess.id === id)
      if (idx !== -1) s.splice(idx, 1)
    }))
    
    if (state.currentSessionId === id) {
      setState('currentSessionId', state.sessions[0]?.id || null)
    }

    // 清理标签映射（localStorage）
    tagsStore.clearSessionTags(id)
  }

  const switchSession = async (id: string): Promise<void> => {
    if (state.currentSessionId === id) return
    
    setState('currentSessionId', id)
    
    // Load messages for the new session
    await loadSessionMessages(id)
    
    // Notify backend to switch context
    try {
      await invoke('neocodex_switch_session', { session_id: id })
    } catch (error) {
      console.error('[chatStore] Failed to switch session on backend:', error)
    }
  }

  const updateSessionTitle = async (id: string, title: string): Promise<void> => {
    try {
      await invoke('neocodex_rename_session', { session_id: id, name: title })
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
    setState('sessions', produce(s => {
      const sess = s.find(sess => sess.id === state.currentSessionId)
      if (sess) {
        sess.messages.push(msg)
        sess.updatedAt = new Date()
      }
    }))
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

  const regenerateLast = (): string => {
    const messages = currentMessages()
    const lastUserMsg = [...messages].reverse().find(m => m.role === 'user')
    const lastAssistantMsg = [...messages].reverse().find(m => m.role === 'assistant')
    
    if (lastAssistantMsg) {
      deleteMessage(lastAssistantMsg.id)
    }
    
    return lastUserMsg?.content || ''
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

  /** 给会话打标（自动注册标签；存 localStorage） */
  const tagSession = (sessionId: string, rawName: string): void => {
    tagsStore.addSessionTag(sessionId, rawName)
    // 同步响应式 session.tags（驱动会话行标签展示）
    setState('sessions', produce(s => {
      const sess = s.find(x => x.id === sessionId)
      if (sess) sess.tags = tagsStore.tagsForSession(sessionId)
    }))
  }

  /** 取消会话标签 */
  const untagSession = (sessionId: string, name: string): void => {
    tagsStore.removeSessionTag(sessionId, name)
    setState('sessions', produce(s => {
      const sess = s.find(x => x.id === sessionId)
      if (sess) sess.tags = tagsStore.tagsForSession(sessionId)
    }))
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
    switchSession,
    updateSessionTitle,
    addMessage,
    updateMessage,
    appendMessageContent,
    appendToolCall,
    deleteMessage,
    clearMessages,
    setGenerating,
    abortGeneration,
    regenerateLast,
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