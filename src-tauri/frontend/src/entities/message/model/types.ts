/**
 * entities/message/model/types.ts — 消息实体类型定义
 *
 * 从 stores/chat.ts 提取，建立单一事实源
 */
import type { NeoCodexAttachmentDto, ToolCallRecord } from '../../../api/types'

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
  reasoning?: string
}

export interface Session {
  id: string
  title: string
  messages: Message[]
  createdAt: Date
  updatedAt: Date
  checkpointId?: string
  project?: string
  tags: string[]
}

export interface ChatState {
  sessions: Session[]
  currentSessionId: string | null
  isGenerating: boolean
  isLoadingSessions: boolean
  isLoadingMessages: boolean
}
