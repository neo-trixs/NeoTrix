/**
 * entities/tool/ — 工具执行实体
 *
 * 职责：工具调用类型、工具执行状态、工具 UI 组件
 * 依赖：shared/api
 */
export type { ToolCallRecord, ToolResult } from './model/types'
export { ToolResult as ToolResultComponent, ToolCallCard } from './ui'
