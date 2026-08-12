import { call } from './client'
import type { CoworkAction, CoworkDeliverable, CoworkSession } from './types'

/* ════════════════════════════════════════════
   api/cowork.ts — 协同会话（CW）
   对应 cowork_cmds.rs
   ════════════════════════════════════════════ */

export function coworkList(): Promise<CoworkSession[]> {
  return call('cowork_list', {})
}

export function coworkGet(sessionId: string): Promise<CoworkSession> {
  return call('cowork_get', { session_id: sessionId })
}

export function coworkStatus(sessionId: string): Promise<CoworkSession> {
  return call('cowork_status', { session_id: sessionId })
}

export function coworkStart(params: {
  workspacePath: string
  description: string
  name?: string | null
  tags?: string[] | null
}): Promise<string> {
  return call('cowork_start', {
    workspace_path: params.workspacePath,
    description: params.description,
    name: params.name ?? null,
    tags: params.tags ?? null,
  })
}

export function coworkActions(sessionId: string): Promise<CoworkAction[]> {
  return call('cowork_actions', { session_id: sessionId })
}

export function coworkListDeliverables(sessionId: string): Promise<CoworkDeliverable[]> {
  return call('cowork_list_deliverables', { session_id: sessionId })
}

export function coworkPause(sessionId: string): Promise<void> {
  return call('cowork_pause', { session_id: sessionId })
}

export function coworkResume(sessionId: string): Promise<void> {
  return call('cowork_resume', { session_id: sessionId })
}

export function coworkStop(sessionId: string): Promise<CoworkSession> {
  return call('cowork_stop', { session_id: sessionId })
}

export function coworkDelete(sessionId: string): Promise<void> {
  return call('cowork_delete', { session_id: sessionId })
}

export function coworkReadFile(sessionId: string, path: string): Promise<string> {
  return call('cowork_read_file', { session_id: sessionId, path })
}

export function coworkWriteFile(sessionId: string, path: string, content: string): Promise<void> {
  return call('cowork_write_file', { session_id: sessionId, path, content })
}

export function coworkScanFiles(sessionId: string, pattern?: string): Promise<unknown[]> {
  return call('cowork_scan_files', { session_id: sessionId, pattern: pattern ?? null })
}

export function coworkTemplates(category?: string): Promise<unknown[]> {
  return call('cowork_templates', { category: category ?? null })
}
