/* ════════════════════════════════════════════
   api/kb.ts — KB 文档级 CRUD 命令入口
   契约镜像 src-tauri/commands/kb_cmds.rs B2 段:
   KbDocSummary / KbDocIngestResult
   kb_doc_ingest / kb_doc_list / kb_doc_delete / kb_doc_reindex
   ════════════════════════════════════════════ */
import { call } from './client'

export interface KbDocSummary {
  doc_id: string
  title: string
  library: string
  chunk_count: number
  total_chars: number
  status: 'ready' | 'empty'
  created_at: number
}

export interface KbDocIngestResult {
  doc_id: string
  title: string
  library: string
  chunk_count: number
  status: string
}

export function kbDocIngest(title: string, text: string, library?: string): Promise<KbDocIngestResult> {
  return call('kb_doc_ingest', { title, text, library: library ?? null })
}

export function kbDocList(): Promise<KbDocSummary[]> {
  return call('kb_doc_list', {})
}

export function kbDocDelete(docId: string): Promise<number> {
  return call('kb_doc_delete', { doc_id: docId })
}

export function kbDocReindex(docId: string): Promise<number> {
  return call('kb_doc_reindex', { doc_id: docId })
}
