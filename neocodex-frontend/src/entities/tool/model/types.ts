/**
 * entities/tool/model/types.ts — 工具执行实体类型定义
 * 统一事实源：从 api/types 重新导出，保持单一定位
 */
export type { ToolCallRecord } from '../../../api/types'

export interface ToolResult {
  toolCallId: string
  output: string
  isError: boolean
  durationMs?: number
}

export type ToolStatus = 'pending' | 'running' | 'completed' | 'error' | 'approval_required'

export interface ToolExecutionState {
  toolCallId: string
  status: ToolStatus
  startedAt: number
  durationMs?: number
}
