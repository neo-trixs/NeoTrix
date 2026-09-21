/* ════════════════════════════════════════════
   api/workflows.ts — 工作流命令入口
   Routes through domain_call (no direct IPC registered).
   ════════════════════════════════════════════ */
import { call } from './domain'

export interface WorkflowStep {
  id: string
  kind: string
  name: string
  params: Record<string, string>
  depends_on: string[]
  timeout_secs: number
  retry_count: number
}

export interface Workflow {
  id: string
  name: string
  description: string
  version: number
  steps: WorkflowStep[]
  created_at: number
  updated_at: number
  tags: string[]
}

export interface WorkflowRun {
  id: string
  workflow_id: string
  status: string
  current_step: number
  progress_pct: number
  started_at: number
}

export function workflowList(): Promise<Workflow[]> {
  return call<Workflow[]>('workflow', 'list')
}

export async function workflowRun(workflowId: string): Promise<string> {
  // 后端 run 返回完整 WorkflowRun 对象，此处提取 id 以保持调用方契约
  const run = await call<WorkflowRun>('workflow', 'run', { workflow_id: workflowId })
  return run.id
}

export function workflowRunStatus(runId: string): Promise<WorkflowRun> {
  return call<WorkflowRun>('workflow', 'status', { run_id: runId })
}

export function workflowRunCancel(runId: string): Promise<void> {
  return call<void>('workflow', 'cancel', { run_id: runId })
}
