/* ════════════════════════════════════════════
   components/settings/McpSection.tsx — 设置页 MCP 服务器管理
   从 SettingsModal 抽出（原 846-947 行区块）：stdio 注册 + 工具一览。
   自包含状态（servers/tools/loading/busy/表单），仅依赖 showNotice。
   ════════════════════════════════════════════ */
import { createSignal, createEffect, Show, For } from 'solid-js'
import { neocodex, errText } from '../../api'
import type { McpServerInfo, McpToolInfo } from '../../api/types'
import { clsx } from 'clsx'

interface Props {
  /** 通知回调（由父级注入，非关键） */
  showNotice: (msg: string) => void
}

function DataIcon() {
  return (
    <svg viewBox="0 0 14 14" class="w-3.5 h-3.5 text-nt-io-600" fill="none">
      <ellipse cx="7" cy="3.5" rx="4.5" ry="1.8" stroke="currentColor" stroke-width="1.2" />
      <path d="M2.5 3.5v3.5c0 1 2 1.8 4.5 1.8s4.5-.8 4.5-1.8V3.5" stroke="currentColor" stroke-width="1.2" />
      <path d="M2.5 7v3.5c0 1 2 1.8 4.5 1.8s4.5-.8 4.5-1.8V7" stroke="currentColor" stroke-width="1.2" />
    </svg>
  )
}

export function McpSection(props: Props) {
  const [mcpServers, setMcpServers] = createSignal<McpServerInfo[]>([])
  const [mcpToolList, setMcpToolList] = createSignal<McpToolInfo[]>([])
  const [mcpLoading, setMcpLoading] = createSignal(false)
  const [mcpBusy, setMcpBusy] = createSignal(false)
  const [showMcpTools, setShowMcpTools] = createSignal(false)
  const [mcpName, setMcpName] = createSignal('')
  const [mcpCommand, setMcpCommand] = createSignal('')
  const [mcpArgs, setMcpArgs] = createSignal('')

  const loadMcp = async () => {
    setMcpLoading(true)
    try {
      const [servers, tools] = await Promise.all([neocodex.mcpList(), neocodex.mcpTools()])
      setMcpServers(servers)
      setMcpToolList(tools)
    } catch (e) {
      console.error('[McpSection] load failed:', e)
    } finally {
      setMcpLoading(false)
    }
  }

  const registerMcp = async () => {
    const name = mcpName().trim()
    const command = mcpCommand().trim()
    if (!name || !command) {
      props.showNotice('请填写服务器名称与启动命令')
      return
    }
    const args = mcpArgs().split(',').map((s) => s.trim()).filter(Boolean)
    setMcpBusy(true)
    try {
      const servers = await neocodex.mcpRegister(name, command, args)
      setMcpServers(servers)
      setMcpName('')
      setMcpCommand('')
      setMcpArgs('')
      props.showNotice(`已注册 MCP 服务器 ${name}`)
      void loadMcp()
    } catch (e) {
      console.error('[McpSection] register failed:', e)
      props.showNotice(`注册失败：${errText(e)}`)
    } finally {
      setMcpBusy(false)
    }
  }

  createEffect(() => {
    void loadMcp()
  })

  return (
    <div class="ss-card">
      <div class="ss-card-header">
        <DataIcon />
        MCP 服务器
        <span class="ml-auto text-[10px] text-text-muted font-mono">{mcpServers().length} 个</span>
      </div>
      <div class="ss-card-body space-y-3">
        <p class="text-[11px] text-text-muted leading-relaxed -mt-1">
          注册本地 stdio MCP 服务器，为代理附加外部工具（如文件系统 / 数据库 / 浏览器）。
          当前会话内生效，重启后重新注册。
        </p>

        {/* 服务器列表 */}
        <Show when={mcpLoading() && mcpServers().length === 0}>
          <div class="text-xs text-text-muted py-2 text-center">加载 MCP 服务器…</div>
        </Show>
        <Show when={!mcpLoading() && mcpServers().length === 0}>
          <div class="text-[11px] text-text-muted py-3 text-center border border-dashed border-border-primary/60 rounded-lg">
            暂无 MCP 服务器，填写下方表单注册
          </div>
        </Show>
        <div class="space-y-1.5">
          <For each={mcpServers()}>
            {(srv) => (
              <div class="flex items-center gap-2 px-3 py-2 rounded-lg border border-border-primary/40 bg-white/40">
                <span class={clsx('w-2 h-2 rounded-full flex-shrink-0', srv.healthy ? 'bg-emerald-500' : 'bg-red-500')} />
                <span class="text-[12px] text-text-primary font-medium truncate flex-1">{srv.name}</span>
                <span class="text-[10px] text-text-muted font-mono flex-shrink-0">{srv.transport}</span>
                <span class="text-[10px] text-text-muted font-mono flex-shrink-0">{srv.tool_count} 工具</span>
                <span class={clsx('text-[10px] px-1.5 py-0.5 rounded-full font-medium flex-shrink-0', srv.healthy ? 'bg-emerald-500/10 text-emerald-600' : 'bg-red-500/10 text-red-500')}>
                  {srv.healthy ? '健康' : '异常'}
                </span>
              </div>
            )}
          </For>
        </div>

        {/* 工具一览（可折叠） */}
        <Show when={mcpToolList().length > 0}>
          <button
            class="flex items-center gap-1 text-[11px] text-nt-io-600 hover:text-nt-io-700"
            onClick={() => setShowMcpTools(!showMcpTools())}
            aria-expanded={showMcpTools()}
          >
            {showMcpTools() ? '▾' : '▸'} 查看工具（{mcpToolList().length}）
          </button>
          <Show when={showMcpTools()}>
            <div class="space-y-1 max-h-40 overflow-y-auto">
              <For each={mcpToolList()}>
                {(tool) => (
                  <div class="px-2 py-1 rounded bg-bg-primary/40 text-[11px] font-mono break-all">
                    <span class="text-nt-io-600">{tool.server}.</span>
                    <span class="text-text-primary">{tool.name}</span>
                    <span class="text-text-muted"> — {tool.description}</span>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>

        {/* 添加表单 */}
        <div class="border-t border-border-primary/40 pt-3 space-y-2">
          <div class="grid grid-cols-[1fr_1fr] gap-2">
            <input
              class="px-3 py-2 rounded-lg bg-white/70 border border-border-primary text-[12.5px] text-text-primary placeholder:text-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
              placeholder="服务器名称（如 filesystem）"
              value={mcpName()}
              onInput={(e) => setMcpName(e.currentTarget.value)}
              aria-label="MCP 服务器名称"
            />
            <input
              class="px-3 py-2 rounded-lg bg-white/70 border border-border-primary text-[12.5px] text-text-primary placeholder:text-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
              placeholder="启动命令（如 npx）"
              value={mcpCommand()}
              onInput={(e) => setMcpCommand(e.currentTarget.value)}
              onKeyDown={(e) => { if (e.key === 'Enter') registerMcp() }}
              aria-label="MCP 启动命令"
            />
          </div>
          <div class="flex items-center gap-2">
            <input
              class="flex-1 min-w-0 px-3 py-2 rounded-lg bg-white/70 border border-border-primary text-[12.5px] text-text-primary placeholder:text-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500 font-mono"
              placeholder="参数（逗号分隔，如 @modelcontextprotocol/server-filesystem, /tmp）"
              value={mcpArgs()}
              onInput={(e) => setMcpArgs(e.currentTarget.value)}
              onKeyDown={(e) => { if (e.key === 'Enter') registerMcp() }}
              aria-label="MCP 启动参数"
            />
            <button
              class="px-3 py-2 rounded-lg bg-nt-io-500 text-text-primary text-[12px] font-medium hover:bg-nt-io-600 disabled:opacity-50 transition-colors flex-shrink-0"
              onClick={registerMcp}
              disabled={mcpBusy()}
              aria-label="注册 MCP 服务器"
            >
              {mcpBusy() ? '注册中…' : '注册'}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}