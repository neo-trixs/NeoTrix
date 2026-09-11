/* ════════════════════════════════════════════
   stores/world.ts — NT-WORLD 域前端状态

   连接 domain.world.* API，提供搜索/抓取/提取能力。
   ════════════════════════════════════════════ */
import { createSignal } from 'solid-js'
import * as domain from '../api/domain'
import type { WorldSearchResult, WorldFetchResult } from '../api/domain'

export type { WorldSearchResult, WorldFetchResult }

export function createWorldStore() {
  const [searchResults, setSearchResults] = createSignal<WorldSearchResult[]>([])
  const [searchLoading, setSearchLoading] = createSignal(false)
  const [searchQuery, setSearchQuery] = createSignal('')
  const [error, setError] = createSignal<string | null>(null)

  async function search(query: string, count?: number) {
    setSearchLoading(true)
    setError(null)
    setSearchQuery(query)
    try {
      const result = await domain.world.webSearch(query, count)
      setSearchResults(result.results)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setSearchLoading(false)
    }
  }

  async function fetchUrl(url: string): Promise<WorldFetchResult> {
    setError(null)
    try {
      return await domain.world.fetchUrl(url)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
      throw e
    }
  }

  async function extractContent(url: string, format?: string) {
    setError(null)
    try {
      return await domain.world.extractContent(url, format)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
      throw e
    }
  }

  function clear() {
    setSearchResults([])
    setSearchQuery('')
    setError(null)
  }

  return {
    searchResults,
    searchLoading,
    searchQuery,
    error,
    search,
    fetchUrl,
    extractContent,
    clear,
  }
}

export type WorldStore = ReturnType<typeof createWorldStore>
