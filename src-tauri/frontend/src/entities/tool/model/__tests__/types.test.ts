import { describe, it, expect } from 'vitest'
import type { ToolCallRecord, ToolResult, ToolExecutionState } from '../types'

describe('entities/tool — 类型契约', () => {
  it('ToolCallRecord 应满足 api 契约', () => {
    const tc: ToolCallRecord = {
      id: 'tool-1',
      name: 'read_file',
      args: '{"path":"/tmp/a.txt"}',
      result: 'content',
      duration_ms: 12,
      success: true,
      domain: 'nt_core',
    }
    expect(tc.name).toBe('read_file')
    expect(tc.success).toBe(true)
  })

  it('ToolResult 应包含输出', () => {
    const r: ToolResult = { toolCallId: 'tool-1', output: 'ok', isError: false, durationMs: 5 }
    expect(r.isError).toBe(false)
  })

  it('ToolExecutionState 应支持 pending', () => {
    const s: ToolExecutionState = { toolCallId: 'tool-1', status: 'pending', startedAt: Date.now() }
    expect(s.status).toBe('pending')
  })
})
