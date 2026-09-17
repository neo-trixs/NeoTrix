import { describe, it, expect } from 'vitest'
import type { Message, Session, ChatState } from '../types'

describe('entities/message — 类型契约', () => {
  it('Message 应满足基本结构', () => {
    const msg: Message = {
      id: 'm1',
      role: 'user',
      content: 'hello',
      timestamp: new Date(),
    }
    expect(msg.id).toBe('m1')
    expect(msg.role).toBe('user')
  })

  it('Message 支持可选字段', () => {
    const msg: Message = {
      id: 'm2',
      role: 'assistant',
      content: 'hi',
      timestamp: new Date(),
      isStreaming: true,
      toolCalls: [{ id: 't1', name: 'read_file', args: '{}', result: 'ok', duration_ms: 10, success: true }],
      reasoning: 'thinking...',
    }
    expect(msg.isStreaming).toBe(true)
    expect(msg.toolCalls?.length).toBe(1)
  })

  it('Session 应包含 messages', () => {
    const sess: Session = {
      id: 's1',
      title: 'test',
      messages: [],
      createdAt: new Date(),
      updatedAt: new Date(),
      tags: [],
    }
    expect(sess.messages).toEqual([])
  })

  it('ChatState 应包含 sessions', () => {
    const state: ChatState = {
      sessions: [],
      currentSessionId: null,
      isGenerating: false,
      isLoadingSessions: false,
      isLoadingMessages: false,
    }
    expect(state.isGenerating).toBe(false)
  })
})
