import { call } from './client'

/* ════════════════════════════════════════════
   api/unified.ts — CLI ↔ app 统一命令桥
   L3 前端消费层: 命令目录 (单一真源) + CLI 命令执行。
   对应 src-tauri unified_cmds.rs。
   ════════════════════════════════════════════ */

export type UnifiedBackend = 'cli' | 'tauri'

export interface UnifiedCommandSpec {
  name: string
  aliases: string[]
  category: string
  description: string
  backend: UnifiedBackend
  json_support: boolean
  internal: boolean
}

export interface UnifiedCommandResult {
  success: boolean
  message: string
  exit_code: number
  json: unknown | null
}

/** 全量统一命令目录 (CLI 动态 + Tauri 自动生成) — 前端命令面板单一真源 */
export function fullCatalog(): Promise<UnifiedCommandSpec[]> {
  return call<UnifiedCommandSpec[]>('unified_tauri_full_catalog')
}

/** CLI 侧命令目录 */
export function cliList(): Promise<UnifiedCommandSpec[]> {
  return call<UnifiedCommandSpec[]>('unified_cli_list')
}

/** Tauri 侧命令目录 (核心子集) */
export function tauriList(): Promise<UnifiedCommandSpec[]> {
  return call<UnifiedCommandSpec[]>('unified_tauri_list')
}

/** 统一命令目录 (core 静态) */
export function unifiedCatalog(): Promise<UnifiedCommandSpec[]> {
  return call<UnifiedCommandSpec[]>('unified_command_catalog')
}

/** 执行一条 CLI 命令 (如 '/help' 或 'help') */
export function execCli(input: string): Promise<UnifiedCommandResult> {
  return call<UnifiedCommandResult>('unified_cli_execute', { input })
}

/** 查询单条 CLI 命令详情 */
export function cliLookup(name: string): Promise<UnifiedCommandSpec | null> {
  return call<UnifiedCommandSpec | null>('unified_cli_lookup', { name })
}