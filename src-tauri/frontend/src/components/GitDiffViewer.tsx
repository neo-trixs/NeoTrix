import { createSignal, For, Show } from 'solid-js'
import { git, type GitHunk } from '../api/domain'

export function GitDiffViewer() {
  const [hunks, setHunks] = createSignal<GitHunk[]>([])
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [filePath, setFilePath] = createSignal('')

  const fetchDiff = async (path: string) => {
    setLoading(true)
    setError(null)
    try {
      const result = await git.diff(path || undefined)
      setHunks(result.hunks)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  const handleSearch = () => {
    fetchDiff(filePath())
  }

  return (
    <div class="git-diff-viewer">
      <div class="header">
        <h3>Git Diff</h3>
        <div class="search">
          <input
            type="text"
            value={filePath()}
            onInput={(e) => setFilePath(e.currentTarget.value)}
            placeholder="文件路径（可选）..."
          />
          <button onClick={handleSearch} disabled={loading()}>
            {loading() ? '加载中...' : '查看 Diff'}
          </button>
        </div>
      </div>

      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>

      <Show when={hunks().length > 0}>
        <div class="diff-list">
          <For each={hunks()}>
            {(hunk) => (
              <div class="diff-hunk">
                <div class="hunk-header">
                  <span class="hunk-file">{hunk.file}</span>
                  <span class="hunk-stats">
                    <span class="additions">+{hunk.additions}</span>
                    <span class="deletions">-{hunk.deletions}</span>
                  </span>
                </div>
                <pre class="hunk-content">{hunk.content}</pre>
              </div>
            )}
          </For>
        </div>
      </Show>

      <Show when={!loading() && hunks().length === 0 && !error()}>
        <div class="empty">无差异内容</div>
      </Show>
    </div>
  )
}
