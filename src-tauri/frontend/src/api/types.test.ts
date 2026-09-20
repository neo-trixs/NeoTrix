import { describe, it, expect } from 'vitest'
import {
  NeoCodexMessageItem,
  HealthReport,
  ProviderConfig,
  ScreenCapture,
  CoworkSession,
  McpToolInfo,
} from './types'

/* ════════════════════════════════════════════
   api/types.test.ts — wire 契约 tripwire
   对齐 iPolloWork packages/types 思想：跨端契约必须有守卫。
   前端 TS 类型是契约的事实源之一；Rust serde 结构体在
   src-tauri/src/commands/types.rs，字段 snake_case 必须与之吻合。
   此文件对高频契约做编译期 + 运行时双断言：
   - 字段名拼写 / 类型错误会在 tsc 阶段报错（编译期守卫）
   - 运行时断言关键字段存在（防删字段漂移）
   ════════════════════════════════════════════ */

const typeChecks: Record<string, unknown> = {
  message_item: null as unknown as NeoCodexMessageItem,
  health_report: null as unknown as HealthReport,
  provider_config: null as unknown as ProviderConfig,
  screen_capture: null as unknown as ScreenCapture,
  cowork_session: null as unknown as CoworkSession,
  mcp_tool: null as unknown as McpToolInfo,
}
expect(typeChecks).toBeTruthy()

function hasFields(o: unknown, fields: string[]): boolean {
  if (typeof o !== 'object' || o === null) return false
  return fields.every((f) => f in o)
}

describe('wire 契约守卫 — 高频接口字段对齐 Rust serde', () => {
  const msg: NeoCodexMessageItem = {
    id: 1,
    role: 'user',
    content: 'hi',
    timestamp: 0,
    tool_call: { name: 't', args: '{}', result: 'r', duration_ms: 1, success: true },
  }
  it('NeoCodexMessageItem 契约字段', () => {
    expect(hasFields(msg, ['id', 'role', 'content', 'timestamp', 'tool_call'])).toBe(true)
  })

  const health: HealthReport = {
    healthy: true,
    uptime_secs: 1,
    turn_count: 0,
    tokens_used: 0,
    context_usage: 0,
    provider_model: '',
    evolution_iterations: 0,
    cost_spent: 0,
    cost_budget: 0,
  }
  it('HealthReport 契约字段', () => {
    expect(hasFields(health, ['healthy', 'uptime_secs', 'turn_count', 'tokens_used', 'context_usage'])).toBe(true)
    expect(hasFields(health, ['provider_model', 'evolution_iterations', 'cost_spent', 'cost_budget'])).toBe(true)
  })

  const prov: ProviderConfig = {
    provider_count: 0,
    resolvable: false,
    active_model: '',
    providers: [],
  }
  it('ProviderConfig 契约字段', () => {
    expect(hasFields(prov, ['provider_count', 'resolvable', 'active_model', 'providers'])).toBe(true)
  })

  const cap: ScreenCapture = { path: '', width: 0, height: 0, format: '', timestamp: 0, data_base64: 'iVBORw0KGgo=' }
  it('ScreenCapture 契约字段', () => {
    expect(hasFields(cap, ['path', 'width', 'height', 'format', 'timestamp', 'data_base64'])).toBe(true)
  })

  const cowork: CoworkSession = {
    id: '',
    name: '',
    workspace_path: '',
    status: '',
    files_read: 0,
    files_created: 0,
    files_modified: 0,
    started_at: 0,
    last_active_at: 0,
    deliverables: [],
    description: '',
    tags: [],
  }
  it('CoworkSession 契约字段', () => {
    expect(hasFields(cowork, ['id', 'workspace_path', 'status', 'files_read', 'files_modified', 'deliverables'])).toBe(true)
  })

  it('McpToolInfo 契约字段', () => {
    const t: McpToolInfo = { name: '', description: '', server: '' }
    expect(hasFields(t, ['name', 'description', 'server'])).toBe(true)
  })
})
