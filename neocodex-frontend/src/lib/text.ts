/* ════════════════════════════════════════════
   lib/text.ts — 纯文本工具函数（从 routes/Chat.tsx 抽出）
   无任何 Solid/Tauri 依赖，可独立单测。
   ════════════════════════════════════════════ */

/** 折叠预览：前 N 字符 + 若截断点处未闭合的 ``` 围栏则自动补闭合，保证预览 markdown 完整 */
export function foldPreview(content: string, limit: number): string {
  const snippet = content.slice(0, limit)
  const fenceCount = (snippet.match(/```/g) || []).length
  const closed = fenceCount % 2 === 0
  return closed ? `${snippet}\n\n…` : `${snippet}\n\`\`\`\n\n…`
}

/** 根据扩展名猜测 MIME（附件预览用） */
export function guessMime(name: string): string {
  const ext = name.split('.').pop()?.toLowerCase() || ''
  const map: Record<string, string> = {
    png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg', gif: 'image/gif', webp: 'image/webp', svg: 'image/svg+xml',
    rs: 'text/rust', ts: 'text/typescript', tsx: 'text/typescript', js: 'text/javascript', jsx: 'text/javascript',
    py: 'text/python', go: 'text/plain', java: 'text/plain', c: 'text/plain', cpp: 'text/plain', h: 'text/plain',
    rb: 'text/plain', sh: 'text/plain', json: 'application/json', yaml: 'text/yaml', yml: 'text/yaml',
    toml: 'text/plain', md: 'text/markdown', sql: 'text/plain', html: 'text/html', css: 'text/css',
    csv: 'text/csv', txt: 'text/plain', pdf: 'application/pdf',
  }
  return map[ext] ?? 'application/octet-stream'
}

/** 字节数人性化格式 */
export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

/** 估算 token 数：CJK 每字符约 1 token，拉丁按 4 字符/token（对标 Claude 输入计数） */
export function estimateTokens(text: string): number {
  if (!text) return 0
  const cjk = text.match(/[\u4e00-\u9fff\u3000-\u303f\uff00-\uffef]/g)?.length ?? 0
  const latin = text.length - cjk
  return Math.ceil(cjk + latin / 4)
}

/** 时间自适应问候语 */
export function greeting(): string {
  const h = new Date().getHours()
  if (h < 6) return '夜深了'
  if (h < 12) return '上午好'
  if (h < 14) return '中午好'
  if (h < 18) return '下午好'
  return '晚上好'
}