import { invoke } from '@tauri-apps/api/core'

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

/** 统一执行（对话即OS）— 唯一生产入口，自动路由 */
export function harnessExecute(req: HarnessExecuteRequest): Promise<HarnessExecuteResponse> {
  return call<HarnessExecuteResponse>('harness_execute', {
    instruction: req.instruction,
    capability_tag: req.capability_tag ?? null,
    project: req.project ?? null,
  })
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

export function harnessRun(req: HarnessExecuteRequest): Promise<HarnessRunResponse> {
  return call<HarnessRunResponse>('harness_run', {
    instruction: req.instruction,
    run_id: req.run_id ?? null,
    capability_tag: req.capability_tag ?? null,
    project: req.project ?? null,
  })
}

/** 能力标签API地图（调试用） */
export function harnessApiMap(): Promise<CapabilityApiEntry[]> {
  return call<CapabilityApiEntry[]>('harness_api_map')
}

export function harnessToolCatalog(): Promise<{ name: string; description: string }[]> {
  return call<{ name: string; description: string }[]>('harness_tool_catalog')
}

export function harnessResolve(instruction: string): Promise<CapabilityApiEntry | null> {
  return call<CapabilityApiEntry | null>('harness_resolve', { instruction })
}

/** 推理路由 */
export function harnessRouterStatus(): Promise<{ default_provider: string; auto_routing: boolean }> {
  return call<{ default_provider: string; auto_routing: boolean }>('harness_router_status')
}

export function harnessRouterSetProvider(provider: string): Promise<{ default_provider: string }> {
  return call<{ default_provider: string }>('harness_router_set_provider', { provider })
}

/** 沙箱 */
export function harnessSandboxStatus(): Promise<{ config: { use_local_docker: boolean }; state: string }> {
  return call<{ config: { use_local_docker: boolean }; state: string }>('harness_sandbox_status')
}

/** App-Server 线程/turn（对标 codex app-server） */
export function harnessThreadCreate(project?: string): Promise<HarnessThread> {
  return call<HarnessThread>('harness_thread_create', { project: project ?? null })
}

export function harnessThreadList(): Promise<HarnessThread[]> {
  return call<HarnessThread[]>('harness_thread_list')
}

export function harnessTurnStart(thread_id: string, instruction: string): Promise<HarnessTurn> {
  return call<HarnessTurn>('harness_turn_start', { thread_id, instruction })
}

/** 审批流：Harness 外部动作待人工确认队列（只读；后端暂未暴露 approve/reject 端点） */
export interface HarnessApproval {
  id: string
  action: string
  state: string
}

export function harnessApprovalList(): Promise<HarnessApproval[]> {
  return call<HarnessApproval[]>('harness_approval_list')
}

/** 审批交互：approve / reject 写回 app_server（决策 -> Approved / Denied） */
export function harnessApprovalResolve(id: string, decision: 'approve' | 'reject'): Promise<HarnessApproval> {
  return call<HarnessApproval>('harness_approval_resolve', { id, decision })
}
