/* ════════════════════════════════════════════
   api/kb.ts — KB 文档级 CRUD
   所有操作走 domain plugin (kb domain)
   ════════════════════════════════════════════ */
import * as domain from './domain'

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
  return domain.kb.docIngest(title, title, 'document', text, library) as Promise<KbDocIngestResult>
}

export function kbDocList(): Promise<KbDocSummary[]> {
  return domain.kb.docList() as Promise<KbDocSummary[]>
}

export function kbDocDelete(docId: string): Promise<number> {
  return domain.kb.docDelete(docId).then(() => 1)
}

export function kbDocReindex(docId: string): Promise<number> {
  return domain.kb.docReindex().then(r => r.reindexed)
}
