import { describe, it, expect } from 'vitest'
import { convertBackendMessage, type NeoCodexMessageItem } from './chat'

describe('convertBackendMessage', () => {
  const base: NeoCodexMessageItem = {
    id: 1,
    role: 'user',
    content: 'hello',
    timestamp: 1700000000,
  }

  it('converts a plain user message', () => {
    const msg = convertBackendMessage(base)
    expect(msg.id).toBe('msg-1')
    expect(msg.role).toBe('user')
    expect(msg.content).toBe('hello')
    expect(msg.isStreaming).toBe(false)
    expect(msg.timestamp.getTime()).toBe(base.timestamp * 1000)
    expect(msg.toolCalls).toBeUndefined()
  })

  it('carries attachments through', () => {
    const msg = convertBackendMessage({
      ...base,
      attachments: [{ name: 'a.png', size: 123, mime_type: 'image/png' }],
    })
    expect(msg.attachments).toHaveLength(1)
    expect(msg.attachments![0].name).toBe('a.png')
  })

  it('turns a tool message with tool_call into a structured ToolCallRecord', () => {
    const msg = convertBackendMessage({
      id: 7,
      role: 'tool',
      content: '',
      timestamp: 1700000000,
      tool_call: {
        name: 'read_file',
        args: '{"path":"x.rs"}',
        result: 'pub fn main(){}',
        duration_ms: 42,
        success: true,
      },
    })
    expect(msg.role).toBe('tool')
    expect(msg.toolCalls).toHaveLength(1)
    const tc = msg.toolCalls![0]
    expect(tc.id).toBe('msg-7-tool')
    expect(tc.name).toBe('read_file')
    expect(tc.args).toBe('{"path":"x.rs"}')
    expect(tc.duration_ms).toBe(42)
    expect(tc.success).toBe(true)
  })

  it('keeps a tool message without tool_call as a plain message', () => {
    const msg = convertBackendMessage({ ...base, role: 'tool', content: 'nope' })
    expect(msg.toolCalls).toBeUndefined()
    expect(msg.content).toBe('nope')
  })
})
