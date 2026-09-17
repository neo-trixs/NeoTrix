import { createSignal, createEffect, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import { Package, Search, Download, Trash2, RefreshCw, ExternalLink, Check, AlertCircle, Loader2, FolderOpen, Shield } from 'lucide-solid'

/* ════════════════════════════════════════════════════════════
   ExtensionManager — MCP/.ntb 扩展包管理器
   
   融合产品模式：
   - Claude: MCP Server 管理界面
   - Cursor: Extension Marketplace
   - OpenHands: Plugin 系统
   
   NeoTrix 特有：
   - .ntb 包格式（NeoTrix Binary Package）
   - 能力网注册（CapabilityRegistry 自动发现）
   - 安全沙箱隔离验证
   ════════════════════════════════════════════════════════════ */

export type ExtensionStatus = 'installed' | 'available' | 'updating' | 'error'

export interface Extension {
  id: string
  name: string
  description: string
  version: string
  author: string
  status: ExtensionStatus
  capabilities: string[]  // 该扩展提供的能力标签
  downloads?: number
  rating?: number
  lastUpdated?: Date
  enabled: boolean
  securityLevel?: 'trusted' | 'sandboxed' | 'unverified'
}

export interface ExtensionManagerProps {
  extensions: () => Extension[]
  onInstall: (id: string) => void
  onUninstall: (id: string) => void
  onToggleEnabled: (id: string) => void
  onUpdate: (id: string) => void
  onRefresh: () => void
  loading?: () => boolean
}

const SECURITY_COLORS: Record<string, string> = {
  trusted: 'text-emerald-500 bg-emerald-50 border-emerald-200/50',
  sandboxed: 'text-amber-500 bg-amber-50 border-amber-200/50',
  unverified: 'text-red-500 bg-red-50 border-red-200/50',
}

const SECURITY_LABELS: Record<string, string> = {
  trusted: '已验证',
  sandboxed: '沙箱隔离',
  unverified: '未验证',
}

export function ExtensionManager(props: ExtensionManagerProps) {
  const [searchQuery, setSearchQuery] = createSignal('')
  const [activeTab, setActiveTab] = createSignal<'installed' | 'available'>('installed')

  const filtered = () => {
    const q = searchQuery().trim().toLowerCase()
    const list = activeTab() === 'installed'
      ? props.extensions().filter(e => e.status === 'installed' || e.status === 'updating' || e.status === 'error')
      : props.extensions().filter(e => e.status === 'available')
    if (!q) return list
    return list.filter(e =>
      e.name.toLowerCase().includes(q) ||
      e.description.toLowerCase().includes(q) ||
      e.capabilities.some(c => c.toLowerCase().includes(q))
    )
  }

  const installedCount = () => props.extensions().filter(e => e.status === 'installed').length
  const availableCount = () => props.extensions().filter(e => e.status === 'available').length

  return (
    <div class="flex flex-col h-full">
      {/* 头部 */}
      <div class="px-4 py-3 border-b border-border-primary/40">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <Package class="w-4 h-4 text-text-muted" />
            <span class="text-[13px] font-medium text-text-primary">扩展管理</span>
          </div>
          <button
            class="p-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors"
            onClick={props.onRefresh}
            title="刷新"
          >
            <RefreshCw class={clsx('w-3.5 h-3.5', props.loading?.() && 'animate-spin')} />
          </button>
        </div>

        {/* 搜索框 */}
        <div class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border border-border-primary/40 bg-white/40">
          <Search class="w-3.5 h-3.5 text-text-muted/50 flex-shrink-0" />
          <input
            class="flex-1 min-w-0 bg-transparent border-none outline-none text-[12px] text-text-primary placeholder-text-muted/50"
            placeholder="搜索扩展..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
          />
        </div>
      </div>

      {/* 标签页 */}
      <div class="flex border-b border-border-primary/40">
        <button
          class={clsx(
            'flex-1 px-3 py-2 text-[11px] font-medium transition-colors',
            activeTab() === 'installed'
              ? 'text-nt-io-600 border-b-2 border-nt-io-500'
              : 'text-text-muted hover:text-text-primary'
          )}
          onClick={() => setActiveTab('installed')}
        >
          已安装 ({installedCount()})
        </button>
        <button
          class={clsx(
            'flex-1 px-3 py-2 text-[11px] font-medium transition-colors',
            activeTab() === 'available'
              ? 'text-nt-io-600 border-b-2 border-nt-io-500'
              : 'text-text-muted hover:text-text-primary'
          )}
          onClick={() => setActiveTab('available')}
        >
          可用 ({availableCount()})
        </button>
      </div>

      {/* 扩展列表 */}
      <div class="flex-1 overflow-y-auto p-3 space-y-2">
        <Show
          when={filtered().length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-32 text-text-muted text-[12px]">
              <Package class="w-6 h-6 mb-2 opacity-30" />
              <span>{searchQuery() ? '未找到匹配的扩展' : '暂无扩展'}</span>
            </div>
          }
        >
          <For each={filtered()}>
            {(ext) => (
              <div class={clsx(
                'p-3 rounded-xl border transition-all',
                ext.status === 'error'
                  ? 'border-red-200 bg-red-50/30'
                  : 'border-border-primary/40 bg-white/30 hover:bg-white/50'
              )}>
                {/* 扩展信息 */}
                <div class="flex items-start gap-3">
                  <div class={clsx(
                    'w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0 text-[14px]',
                    ext.status === 'installed' ? 'bg-nt-io-500/10 text-nt-io-600' : 'bg-white/60 text-text-muted'
                  )}>
                    {ext.name.charAt(0).toUpperCase()}
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="text-[12px] font-medium text-text-primary truncate">{ext.name}</span>
                      <span class="text-[10px] text-text-muted">v{ext.version}</span>
                    </div>
                    <div class="text-[11px] text-text-muted truncate mt-0.5">{ext.description}</div>
                    <div class="text-[10px] text-text-muted/60 mt-1">by {ext.author}</div>
                  </div>
                </div>

                {/* 能力标签 */}
                <Show when={ext.capabilities.length > 0}>
                  <div class="flex flex-wrap gap-1 mt-2 ml-11">
                    <For each={ext.capabilities.slice(0, 4)}>
                      {(cap) => (
                        <span class="px-1.5 py-0.5 rounded bg-white/60 text-[9px] text-text-muted border border-black/5">
                          {cap}
                        </span>
                      )}
                    </For>
                    <Show when={ext.capabilities.length > 4}>
                      <span class="text-[9px] text-text-muted/50">+{ext.capabilities.length - 4}</span>
                    </Show>
                  </div>
                </Show>

                {/* 操作行 */}
                <div class="flex items-center justify-between mt-2 ml-11">
                  {/* 安全级别 */}
                  <Show when={ext.securityLevel}>
                    <span class={clsx(
                      'px-1.5 py-0.5 rounded text-[9px] border',
                      SECURITY_COLORS[ext.securityLevel!]
                    )}>
                      {SECURITY_LABELS[ext.securityLevel!]}
                    </span>
                  </Show>

                  {/* 操作按钮 */}
                  <div class="flex gap-1.5">
                    <Show when={ext.status === 'installed'}>
                      <button
                        class={clsx(
                          'px-2 py-1 rounded-lg text-[10px] font-medium transition-colors',
                          ext.enabled
                            ? 'text-amber-600 bg-amber-50 hover:bg-amber-100'
                            : 'text-emerald-600 bg-emerald-50 hover:bg-emerald-100'
                        )}
                        onClick={() => props.onToggleEnabled(ext.id)}
                      >
                        {ext.enabled ? '禁用' : '启用'}
                      </button>
                      <button
                        class="px-2 py-1 rounded-lg text-[10px] font-medium text-red-500 bg-red-50 hover:bg-red-100 transition-colors"
                        onClick={() => props.onUninstall(ext.id)}
                      >
                        卸载
                      </button>
                    </Show>
                    <Show when={ext.status === 'available'}>
                      <button
                        class="px-2 py-1 rounded-lg text-[10px] font-medium text-nt-io-600 bg-nt-io-500/10 hover:bg-nt-io-500/20 transition-colors"
                        onClick={() => props.onInstall(ext.id)}
                      >
                        安装
                      </button>
                    </Show>
                    <Show when={ext.status === 'updating'}>
                      <button disabled class="px-2 py-1 rounded-lg text-[10px] font-medium text-text-muted bg-white/40">
                        <Loader2 class="w-3 h-3 animate-spin inline" />
                      </button>
                    </Show>
                    <Show when={ext.status === 'error'}>
                      <button
                        class="px-2 py-1 rounded-lg text-[10px] font-medium text-red-500 bg-red-50 hover:bg-red-100 transition-colors"
                        onClick={() => props.onUpdate(ext.id)}
                      >
                        重试
                      </button>
                    </Show>
                  </div>
                </div>
              </div>
            )}
          </For>
        </Show>
      </div>
    </div>
  )
}
