/* ════════════════════════════════════════════
   components/settings/McpServerPanel.tsx — MCP 服务器管理面板
   展示已注册的 MCP 服务器列表，支持刷新。
   ════════════════════════════════════════════ */
import { createSignal, onMount, For, Show } from 'solid-js'
import { tool } from '../../api/domain'

interface McpServer {
  name: string
  url: string
  enabled: boolean
}

export function McpServerPanel() {
  const [servers, setServers] = createSignal<McpServer[]>([])
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  const fetchServers = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await tool.mcpList()
      setServers(result.servers || [])
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(fetchServers)

  return (
    <div class="mcp-server-panel">
      <div class="header">
        <h3>MCP 服务器</h3>
        <button onClick={fetchServers} disabled={loading()}>
          {loading() ? '刷新中...' : '刷新'}
        </button>
      </div>

      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>

      <For each={servers()}>
        {(server) => (
          <div class="server-item">
            <div class="name">{server.name}</div>
            <div class="url">{server.url}</div>
            <div class="status">
              <span class={`badge ${server.enabled ? 'green' : 'gray'}`}>
                {server.enabled ? '启用' : '禁用'}
              </span>
            </div>
          </div>
        )}
      </For>

      <Show when={servers().length === 0 && !loading()}>
        <div class="empty">暂无 MCP 服务器</div>
      </Show>
    </div>
  )
}
