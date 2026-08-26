/* ════════════════════════════════════════════
   api/files.ts — 文件选择 + doc-parse 解析 (W1)
   契约镜像 project_cmds.rs::parse_doc_file / ParsedDocFile
   ════════════════════════════════════════════ */
import { openFileDialog } from './fs'
import { call } from './client'

export interface ParsedDocFile {
  path: string
  title: string
  format: string
  text: string
  tables: unknown[] | null
}

const DOC_FILTERS = [{ name: '文档', extensions: ['md', 'txt', 'pdf', 'docx', 'xlsx', 'csv'] }]

/** 打开选择器并解析为统一文本; 取消返回 null */
export async function pickAndParseDoc(): Promise<ParsedDocFile | null> {
  const path = await openFileDialog({ filters: DOC_FILTERS })
  if (!path) return null
  return parseDocAt(path)
}

export function parseDocAt(path: string): Promise<ParsedDocFile> {
  return call('parse_doc_file', { path })
}
