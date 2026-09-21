import { call as domainCall } from './domain'
import { ApiError } from './client'

/* ════════════════════════════════════════════
   api/harness.ts — Harness 统一网关前端 SDK
   对标：openai/codex SDK + harness/mcp-server 11工具
   前端仅经对话调用，零学习成本；高级能力按需直调。
   ════════════════════════════════════════════ */

export interface HarnessExecuteRequest {
  instruction: string
  capability_tag?: string
  project?: string
  /** 流式进度关联 id (harness_run 推送 harness-progress 事件时用) */
  run_id?: string
}

/** harness_run 真·流式进度的单步载荷 (phase=step 时携带) */
export interface HarnessStep {
  index: number
  total: number
  /** "internal" (能力网命中) | "external" (外部缺口求解) */
  kind: string
  capability_tag: string
  summary: string
  /** "running" | "done" | "failed" */
  status: string
  output: string
}

/** harness_run 真·流式进度事件载荷 (后端经 Tauri `harness-progress` 推送) */
export interface HarnessProgressEvent {
  run_id?: string
  phase: 'allocated' | 'step' | 'done'
  /** 阶段1(allocated)/阶段3(done) 携带全量报告；step 阶段为空 */
  report?: HarnessRunResponse
  /** 阶段2(step) 携带单步进度；allocated/done 阶段为空 */
  step?: HarnessStep
}

export interface HarnessExecuteResponse {
  instruction: string
  capability_tag: string
  domain: string
  specialist: string
  harness_tool: string
  allocations: string[]
  internal_count: number
  external_gap_count: number
  message: string
}

export interface CapabilityApiEntry {
  capability_tag: string
  domain: string
  specialist: string
  keywords: string[]
  harness_tool: string
  description: string
}

export interface HarnessThread {
  id: string
  project: string | null
  created_at: number
  turn_count: number
}

export interface HarnessTurn {
  id: string
  thread_id: string
  instruction: string
  capability_tag: string
  status: string
}

function notImplemented(op: string): ApiError {
  return new ApiError(`harness 操作 ${op} 后端未实现`, 'NOT_IMPLEMENTED')
}

/** 统一执行（对话即OS）— 唯一生产入口，自动路由（经 tool 域） */
export async function harnessExecute(req: HarnessExecuteRequest): Promise<HarnessExecuteResponse> {
  const r = await domainCall<{ ok?: boolean; stub?: boolean; message?: string }>('tool', 'harness_execute', {
    instruction: req.instruction,
    capability_tag: req.capability_tag ?? null,
    project: req.project ?? null,
  })
  return {
    instruction: req.instruction,
    capability_tag: req.capability_tag ?? 'orchestration',
    domain: '',
    specialist: '',
    harness_tool: '',
    allocations: [],
    internal_count: 0,
    external_gap_count: 0,
    message: r.message ?? '',
  }
}

/** 重路径真实执行（按需触发）— 完整闭环: 内置执行 + 外部缺口 LLM 试错求解 */
export interface HarnessRunResponse {
  instruction: string
  allocations: { task: { capability_tag: string; domain: string; specialist: string; summary: string }; provider: unknown }[]
  internal_count: number
  external_gap_count: number
  strengthening_actions: number
  external_gaps: string[]
  internal_results: { task_id: string; summary: string; provider_path: string[]; executed: boolean; output: string }[]
  external_closures: { solved: boolean; solution: string; knowledge_acquired: boolean }[]
}

export async function harnessRun(_req: HarnessExecuteRequest): Promise<HarnessRunResponse> {
  throw notImplemented('harness_run')
}

/** 能力标签API地图（调试用） */
export async function harnessApiMap(): Promise<CapabilityApiEntry[]> {
  throw notImplemented('harness_api_map')
}

export async function harnessToolCatalog(): Promise<{ name: string; description: string }[]> {
  throw notImplemented('harness_tool_catalog')
}

export async function harnessResolve(instruction: string): Promise<CapabilityApiEntry | null> {
  const r = await domainCall<CapabilityApiEntry | { ok?: boolean }>('tool', 'harness_resolve', { instruction })
  if (r && typeof r === 'object' && 'capability_tag' in (r as object)) {
    return r as CapabilityApiEntry
  }
  return null
}

/** 推理路由 */
export async function harnessRouterStatus(): Promise<{ default_provider: string; auto_routing: boolean }> {
  throw notImplemented('harness_router_status')
}

export async function harnessRouterSetProvider(_provider: string): Promise<{ default_provider: string }> {
  throw notImplemented('harness_router_set_provider')
}

/** 沙箱 */
export async function harnessSandboxStatus(): Promise<{ config: { use_local_docker: boolean }; state: string }> {
  throw notImplemented('harness_sandbox_status')
}

/** App-Server 线程/turn（对标 codex app-server） */
export async function harnessThreadCreate(_project?: string): Promise<HarnessThread> {
  throw notImplemented('harness_thread_create')
}

export async function harnessThreadList(): Promise<HarnessThread[]> {
  throw notImplemented('harness_thread_list')
}

export async function harnessTurnStart(_thread_id: string, _instruction: string): Promise<HarnessTurn> {
  throw notImplemented('harness_turn_start')
}

/** 审批流：Harness 外部动作待人工确认队列（只读；后端暂未暴露 approve/reject 端点） */
export interface HarnessApproval {
  id: string
  action: string
  state: string
}

export async function harnessApprovalList(): Promise<HarnessApproval[]> {
  // tool 域审批队列（被禁 cli 二进制、显式申请等在此排队）
  const r = await domainCall<{ approvals: HarnessApproval[] }>('tool', 'approval_list')
  return r.approvals ?? []
}

/** 审批交互：approve / reject 写回 tool 域审批队列 */
export async function harnessApprovalResolve(id: string, decision: 'approve' | 'reject'): Promise<HarnessApproval> {
  return domainCall<HarnessApproval>('tool', 'approval_resolve', { id, decision })
}

/** 显式申请审批（高风险操作先审批后执行） */
export async function harnessApprovalRequest(action: string, detail: string): Promise<HarnessApproval> {
  return domainCall<HarnessApproval>('tool', 'approval_request', { action, detail })
}
