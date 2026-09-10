/* ════════════════════════════════════════════
   api/memory.ts — KB 记忆 + API Key
   记忆操作走 domain plugin (memory domain)
   API Key 操作走直接 invoke
   ════════════════════════════════════════════ */
import { invoke } from '@tauri-apps/api/core'
import * as domain from './domain'
import type { MemoryStats } from './types'

/* ── KB 记忆（domain plugin） ── */

/** 镜像 memory_mgr_cmds.rs:12 MemoryEntry */
export interface MemoryEntry {
  id: string
  kind: string
  content: string
  summary: string
  source: string
  confidence: number
  created_at: number
  last_accessed_at: number
  access_count: number
  tags: string[]
  is_pinned: boolean
}

/** 镜像 memory_mgr_cmds.rs:45 MemoryTimelineEntry */
export interface MemoryTimelineEntry {
  date: string
  entries_created: number
  entries_accessed: number
  top_topic: string
}

export function memoryStats(): Promise<MemoryStats> {
  return domain.memory.stats() as Promise<MemoryStats>
}

export function memoryExport(format?: string): Promise<string> {
  return domain.memory.export(format) as Promise<string>
}

export function memoryImport(content: string, format?: string): Promise<number> {
  return domain.memory.import(content).then(r => r.imported)
}

export function memoryClear(kind?: string | null): Promise<number> {
  return domain.memory.clear(kind) as Promise<number>
}

export function memoryList(category?: string): Promise<MemoryEntry[]> {
  return domain.memory.list(category) as Promise<MemoryEntry[]>
}

export function memorySearch(query: string, kind?: string | null): Promise<MemoryEntry[]> {
  return domain.memory.search(query) as Promise<MemoryEntry[]>
}

export function memoryTimeline(days?: number): Promise<MemoryTimelineEntry[]> {
  return domain.memory.timeline(days) as Promise<MemoryTimelineEntry[]>
}

/* ── API Key（直接 invoke，非 domain plugin） ── */
export function saveApiKey(key: string): Promise<void> {
  return invoke('save_api_key', { key })
}

export function hasApiKey(): Promise<boolean> {
  return invoke('has_api_key', {})
}

export function deleteApiKey(): Promise<void> {
  return invoke('delete_api_key', {})
}
