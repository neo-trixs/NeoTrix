import { call } from './client'
import type { MemoryStats } from './types'

/* ════════════════════════════════════════════
   api/memory.ts — KB 记忆（stats/export/clear）+ API Key
   对应 memory_mgr_cmds.rs / chat_cmds.rs
   ════════════════════════════════════════════ */

/* ── KB 记忆 ── */
export function memoryStats(): Promise<MemoryStats> {
  return call('memory_stats', {})
}

export function memoryExport(format?: string): Promise<string> {
  return call('memory_export', { format: format ?? null })
}

export function memoryClear(kind?: string | null): Promise<number> {
  return call('memory_clear', { kind: kind ?? null })
}

export function memoryList(category?: string): Promise<unknown[]> {
  return call('memory_list', { category: category ?? null })
}

export function memorySearch(query: string): Promise<unknown[]> {
  return call('memory_search', { query })
}

export function memoryTimeline(days?: number): Promise<unknown[]> {
  return call('memory_timeline', { days: days ?? null })
}

/* ── API Key ── */
export function saveApiKey(key: string): Promise<void> {
  return call('save_api_key', { key })
}

export function hasApiKey(): Promise<boolean> {
  return call('has_api_key', {})
}

export function deleteApiKey(): Promise<void> {
  return call('delete_api_key', {})
}
