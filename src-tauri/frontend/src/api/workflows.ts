/* ════════════════════════════════════════════
   api/workflows.ts — 工作流命令入口
   契约镜像 src-tauri/commands/workflow_cmds.rs:
   Workflow { id,name,description,version,steps,created_at,updated_at,tags }
   workflow_list / workflow_run / workflow_run_status / workflow_run_cancel
   ════════════════════════════════════════════ */
import { invoke } from '@tauri-apps/api/core'

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
  return invoke('workflow_list')
}

export function workflowRun(workflowId: string): Promise<string> {
  return invoke('workflow_run', { workflow_id: workflowId })
}

export function workflowRunStatus(runId: string): Promise<WorkflowRun> {
  return invoke('workflow_run_status', { run_id: runId })
}

export function workflowRunCancel(runId: string): Promise<void> {
  return invoke('workflow_run_cancel', { run_id: runId })
}
