import { scoreMatch } from '../model'

export { scoreMatch }

export function highlightMatch(text: string, query: string): string {
  if (!query.trim()) return text
  const esc = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  return text.replace(new RegExp(`(${esc})`, 'gi'), '<mark>$1</mark>')
}
