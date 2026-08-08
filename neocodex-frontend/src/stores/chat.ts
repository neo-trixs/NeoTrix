
import { createStore, produce } from 'solid-js/store'
import { invoke } from '@tauri-apps/api/core'

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
      }))
      
      setState('sessions', sessions)
      
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
  }
}

export const chatStore = createChatStore()