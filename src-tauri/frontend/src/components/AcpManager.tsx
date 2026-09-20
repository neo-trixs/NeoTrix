import { createSignal, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import { Plug, Plus, Trash2, RefreshCw, ExternalLink, Check, X, Terminal, Globe, Cpu } from 'lucide-solid'

/* ════════════════════════════════════════════════════════════
   AcpManager — Devin 2.0 风格 ACP 协议管理器
   
   融合产品模式：
   - Devin Desktop 2.0: ACP (Agent Client Protocol)
   - Devin Desktop: ACP Registry + Third-party Agents
   - NeoTrix: ExtensionManager + CapabilityRegistry
   
   NeoTrix 特有：
   - ACP Registry 配置可视化
   - Constellation 成熟度标记
   - E8 意识状态与 agent 健康关联
   ════════════════════════════════════════════════════════════ */

export type AcpAgentStatus = 'available' | 'running' | 'error' | 'disabled'

export interface AcpAgent {
  id: string
  name: string
  description?: string
  status: AcpAgentStatus
  command: string
  args?: string[]
  version?: string
  capabilities?: string[]
  registry: 'local' | 'team'
  lastUsed?: Date
  phi?: number
}

export interface AcpManagerProps {
  agents: () => AcpAgent[]
  onEnableAgent: (id: string) => void
  onDisableAgent: (id: string) => void
  onRemoveAgent: (id: string) => void
  onRefreshRegistry: () => void
  onAddAgent: () => void
  onLaunchAgent: (id: string) => void
}

const STATUS_CONFIG: Record<AcpAgentStatus, { icon: any; color: string; label: string; bgColor: string }> = {
  available: { icon: Check,    color: 'text-emerald-600', bgColor: 'bg-emerald-50', label: '可用' },
  running:   { icon: Terminal, color: 'text-blue-600',    bgColor: 'bg-blue-50',    label: '运行中' },
  error:     { icon: X,        color: 'text-red-600',     bgColor: 'bg-red-50',     label: '错误' },
  disabled:  { icon: X,        color: 'text-zinc-400',    bgColor: 'bg-zinc-50',    label: '已禁用' },
}

const REGISTRY_ICONS: Record<string, any> = {
  local: Terminal,
  team:  Globe,
}

export function AcpManager(props: AcpManagerProps) {
  const [filter, setFilter] = createSignal<'all' | 'local' | 'team'>('all')
  const [showAddModal, setShowAddModal] = createSignal(false)

  const filteredAgents = () => {
    const f = filter()
    return props.agents().filter(a => f === 'all' || a.registry === f)
  }

  const enabledCount = () => props.agents().filter(a => a.status !== 'disabled').length
  const runningCount = () => props.agents().filter(a => a.status === 'running').length

  return (
    <div class="flex flex-col h-full">
      {/* Header */}
      <div class="flex items-center justify-between px-4 py-3 border-b border-border-primary/40">
        <div class="flex items-center gap-2">
          <Plug class="w-4 h-4 text-text-muted" />
          <span class="text-[13px] font-medium text-text-primary">ACP Agents</span>
          <span class="text-[11px] text-text-muted bg-white/60 px-1.5 py-0.5 rounded-full">
            {enabledCount()} 启用 / {props.agents().length} 总计
          </span>
          <Show when={runningCount() > 0}>
            <span class="text-[10px] text-blue-600 bg-blue-50 px-1.5 py-0.5 rounded-full">
              {runningCount()} 运行中
            </span>
          </Show>
        </div>
        <div class="flex items-center gap-1.5">
          <button
            class="p-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors"
            onClick={props.onRefreshRegistry}
            title="刷新 Registry"
          >
            <RefreshCw class="w-3.5 h-3.5" />
          </button>
          <button
            class="p-1.5 rounded-lg text-nt-io-600 bg-nt-io-500/10 hover:bg-nt-io-500/20 transition-colors"
            onClick={props.onAddAgent}
            title="添加 ACP Agent"
          >
            <Plus class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Filter tabs */}
      <div class="flex items-center gap-1 px-4 py-2 border-b border-border-primary/20">
        <For each={[
          { id: 'all' as const, label: '全部' },
          { id: 'local' as const, label: '本地' },
          { id: 'team' as const, label: '团队' },
        ]}>
          {(tab) => (
            <button
              class={clsx(
                'px-2.5 py-1 rounded-lg text-[10px] font-medium transition-colors',
                filter() === tab.id
                  ? 'bg-nt-io-500/10 text-nt-io-600'
                  : 'text-text-muted hover:text-text-primary hover:bg-white/60'
              )}
              onClick={() => setFilter(tab.id)}
            >
              {tab.label}
            </button>
          )}
        </For>
      </div>

      {/* Agent list */}
      <div class="flex-1 overflow-y-auto px-4 py-2 space-y-2">
        <Show
          when={filteredAgents().length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center py-12 text-text-muted">
              <Plug class="w-8 h-8 mb-2 opacity-30" />
              <span class="text-[12px]">暂无 ACP Agent</span>
              <span class="text-[11px] mt-1 opacity-60">点击 + 添加第三方 Agent</span>
            </div>
          }
        >
          <For each={filteredAgents()}>
            {(agent) => {
              const statusConf = STATUS_CONFIG[agent.status]
              const StatusIcon = statusConf.icon
              const RegIcon = REGISTRY_ICONS[agent.registry] ?? Terminal
              return (
                <div class={clsx(
                  'p-3 rounded-xl border transition-all',
                  agent.status === 'running'
                    ? 'border-blue-200/50 bg-blue-50/30'
                    : agent.status === 'error'
                    ? 'border-red-200/50 bg-red-50/30'
                    : 'border-black/5 bg-white/40 hover:bg-white/60'
                )}>
                  {/* Title row */}
                  <div class="flex items-center gap-2 mb-1.5">
                    <div class={clsx('w-2 h-2 rounded-full', statusConf.bgColor, statusConf.color)} />
                    <span class="text-[12px] font-medium text-text-primary flex-1 min-w-0 truncate">
                      {agent.name}
                    </span>
                    <div class="flex items-center gap-1">
                      <RegIcon class="w-3 h-3 text-text-muted/50" title={agent.registry === 'local' ? '本地 Registry' : '团队 Registry'} />
                      <span class={clsx('text-[9px] px-1.5 py-0.5 rounded-full', statusConf.bgColor, statusConf.color)}>
                        {statusConf.label}
                      </span>
                    </div>
                  </div>

                  {/* Description */}
                  <Show when={agent.description}>
                    <div class="text-[10px] text-text-muted mb-1.5 line-clamp-1">{agent.description}</div>
                  </Show>

                  {/* Command */}
                  <div class="flex items-center gap-1 text-[9px] font-mono text-text-muted/60 mb-1.5">
                    <Terminal class="w-2.5 h-2.5" />
                    <span class="truncate">{agent.command} {agent.args?.join(' ') ?? ''}</span>
                  </div>

                  {/* Capabilities */}
                  <Show when={agent.capabilities && agent.capabilities.length > 0}>
                    <div class="flex flex-wrap gap-1 mb-1.5">
                      <For each={agent.capabilities!}>
                        {(cap) => (
                          <span class="text-[8px] px-1.5 py-0.5 rounded-full bg-white/60 border border-black/5 text-text-muted">
                            {cap}
                          </span>
                        )}
                      </For>
                    </div>
                  </Show>

                  {/* Version + Phi + Actions */}
                  <div class="flex items-center gap-2 text-[9px] text-text-muted">
                    <Show when={agent.version}>
                      <span>v{agent.version}</span>
                    </Show>
                    <Show when={agent.lastUsed}>
                      <span>{formatRelativeTime(agent.lastUsed!)}</span>
                    </Show>
                    <Show when={agent.phi !== undefined}>
                      <span class="flex items-center gap-0.5 ml-auto">
                        <Cpu class="w-2.5 h-2.5" />
                        φ {(agent.phi! * 100).toFixed(0)}%
                      </span>
                    </Show>
                    <div class="flex gap-1 ml-auto">
                      <Show when={agent.status === 'available'}>
                        <button
                          class="px-1.5 py-0.5 rounded text-[9px] bg-blue-50 text-blue-600 hover:bg-blue-100 transition-colors"
                          onClick={() => props.onLaunchAgent(agent.id)}
                        >
                          启动
                        </button>
                      </Show>
                      <Show when={agent.status === 'disabled'}>
                        <button
                          class="px-1.5 py-0.5 rounded text-[9px] bg-emerald-50 text-emerald-600 hover:bg-emerald-100 transition-colors"
                          onClick={() => props.onEnableAgent(agent.id)}
                        >
                          启用
                        </button>
                      </Show>
                      <Show when={agent.status !== 'disabled' && agent.status !== 'running'}>
                        <button
                          class="px-1.5 py-0.5 rounded text-[9px] bg-zinc-50 text-zinc-500 hover:bg-zinc-100 transition-colors"
                          onClick={() => props.onDisableAgent(agent.id)}
                        >
                          禁用
                        </button>
                      </Show>
                      <button
                        class="p-0.5 rounded text-text-muted/40 hover:text-red-500 transition-colors"
                        onClick={() => props.onRemoveAgent(agent.id)}
                        title="移除"
                      >
                        <Trash2 class="w-2.5 h-2.5" />
                      </button>
                    </div>
                  </div>
                </div>
              )
            }}
          </For>
        </Show>
      </div>

      {/* ACP info footer */}
      <div class="px-4 py-2 border-t border-border-primary/20">
        <div class="flex items-center gap-2 text-[9px] text-text-muted/50">
          <Plug class="w-2.5 h-2.5" />
          <span>Agent Client Protocol — 开放协议，支持 Codex CLI / Claude Agent / OpenCode / Gemini CLI</span>
        </div>
      </div>
    </div>
  )
}

function formatRelativeTime(date: Date): string {
  const diff = Date.now() - date.getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes}m`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h`
  return `${Math.floor(hours / 24)}d`
}
