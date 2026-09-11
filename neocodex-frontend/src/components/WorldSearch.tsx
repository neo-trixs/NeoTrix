import { createSignal, For } from 'solid-js'
import { createWorldStore } from '../stores/world'

export function WorldSearch() {
  const store = createWorldStore()
  const [query, setQuery] = createSignal('')

  const handleSearch = async () => {
    if (!query()) return
    await store.search(query())
  }

  return (
    <div class="world-search">
      <input
        type="text"
        value={query()}
        onInput={(e) => setQuery(e.currentTarget.value)}
        placeholder="搜索网页..."
      />
      <button onClick={handleSearch}>搜索</button>

      <For each={store.searchResults()}>
        {(result) => (
          <div class="search-result">
            <a href={result.url} target="_blank">{result.title}</a>
            <p>{result.snippet}</p>
            <span>{result.source}</span>
          </div>
        )}
      </For>

      {store.searchLoading() && <div class="loading">搜索中...</div>}
    </div>
  )
}
