// features/search-knowledge/model — 搜索知识状态
export interface SearchQuery {
  text: string
  tags?: string[]
  limit?: number
}

export function normalizeQuery(q: string): string {
  return q.trim().toLowerCase().replace(/\s+/g, ' ')
}

export function buildSearchParams(query: SearchQuery): Record<string, string> {
  const p: Record<string, string> = { q: normalizeQuery(query.text) }
  if (query.tags?.length) p.tags = query.tags.join(',')
  if (query.limit) p.limit = String(query.limit)
  return p
}

export function scoreMatch(content: string, query: string): number {
  const nq = normalizeQuery(query)
  if (!nq) return 0
  const nc = content.toLowerCase()
  if (nc === nq) return 1
  if (nc.includes(nq)) return 0.8
  const terms = nq.split(' ')
  const hits = terms.filter(t => nc.includes(t)).length
  return hits / terms.length * 0.5
}
