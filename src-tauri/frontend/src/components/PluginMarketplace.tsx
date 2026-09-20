import { createSignal, onMount, createEffect, Show, For } from 'solid-js'
import { Puzzle, X, RefreshCw, Loader2, Download, Trash2, Power, PowerOff, ListTree, Box, Search, Globe, Tag, Star, ArrowDownToLine, ExternalLink, ChevronDown, ChevronRight } from 'lucide-solid'
import { plugins as pluginsApi, market as marketApi, errText, fs as fsApi } from '../api'
import type { PluginEvent, PluginStatus } from '../api/types'
import type { MarketEntry, PluginManifest } from '../api/market'
import { clsx } from 'clsx'
import { ConfirmModal, type ModalReq } from './ConfirmModal'

interface Props {
  open: boolean
  onClose: () => void
  /** 内嵌模式：渲染在设置弹窗内（无浮层/无滑入动画/无关闭按钮） */
  embedded?: boolean
}

type TabType = 'installed' | 'discover'

export function PluginMarketplace(props: Props) {
  const [plugins, setPlugins] = createSignal<PluginStatus[]>([])
  const [events, setEvents] = createSignal<PluginEvent[]>([])
  const [loading, setLoading] = createSignal(false)
  const [busy, setBusy] = createSignal<string | null>(null)
  const [error, setError] = createSignal<string | null>(null)
  const [modalReq, setModalReq] = createSignal<ModalReq | null>(null)
  const [pendingUninstallId, setPendingUninstallId] = createSignal<string | null>(null)

  // Market state
  const [tab, setTab] = createSignal<TabType>('installed')
  const [searchQuery, setSearchQuery] = createSignal('')
  const [searchResults, setSearchResults] = createSignal<MarketEntry[]>([])
  const [searchLoading, setSearchLoading] = createSignal(false)
  const [installedMarket, setInstalledMarket] = createSignal<PluginManifest[]>([])
  const [expandedEntry, setExpandedEntry] = createSignal<string | null>(null)
  const [installingId, setInstallingId] = createSignal<string | null>(null)

  let firstBtnRef: HTMLButtonElement | undefined
  let searchInputRef: HTMLInputElement | undefined

  createEffect(() => {
    if (props.open && firstBtnRef) firstBtnRef.focus()
  })

  const load = async () => {
    setLoading(true)
    setError(null)
    try {
      const [pl, ev] = await Promise.all([
        pluginsApi.pluginList(),
        pluginsApi.pluginEventLog(30),
      ])
      setPlugins(pl)
      setEvents(ev)

      // Load market installed
      try {
        const installed = await marketApi.marketListInstalled()
        setInstalledMarket(installed)
      } catch {}
    } catch (e) {
      setError(errText(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(load)

  // ── Installed tab actions ──

  const installLocal = async () => {
    try {
      const path = await fsApi.openFileDialog({ filters: [{ name: 'Plugin Manifest', extensions: ['json'] }] })
      if (typeof path === 'string') {
        setBusy('install')
        await pluginsApi.pluginInstall(path)
        await load()
      }
    } catch (e) {
      setError(errText(e))
    } finally {
      setBusy(null)
    }
  }

  const uninstall = async (id: string) => {
    setPendingUninstallId(id)
    setModalReq({
      title: '卸载插件',
      message: '确定卸载该插件？',
      danger: true,
      confirmLabel: '卸载',
    })
  }

  const doUninstall = async (id: string) => {
    setPendingUninstallId(null)
    setModalReq(null)
    setBusy(`uninstall:${id}`)
    setError(null)
    try {
      // Try market uninstall first, fallback to native plugin
      try {
        await marketApi.marketUninstall(id)
      } catch {
        await pluginsApi.pluginUninstall(id)
      }
      await load()
    } catch (e) {
      setError(errText(e))
    } finally {
      setBusy(null)
    }
  }

  const toggleEnabled = async (p: PluginStatus) => {
    setBusy(`toggle:${p.id}`)
    setError(null)
    try {
      if (p.enabled) await pluginsApi.pluginDisable(p.id)
      else await pluginsApi.pluginEnable(p.id)
      await load()
    } catch (e) {
      setError(errText(e))
    } finally {
      setBusy(null)
    }
  }

  // ── Discover tab actions ──

  const doSearch = async () => {
    const q = searchQuery().trim()
    if (!q) return

    setSearchLoading(true)
    setError(null)
    try {
      const results = await marketApi.marketSearch(q)
      // Merge all sources
      const merged: MarketEntry[] = []
      for (const r of results) {
        merged.push(...r.entries)
      }
      // Dedupe by id, keep highest rated
      const seen = new Map<string, MarketEntry>()
      for (const entry of merged) {
        const existing = seen.get(entry.id)
        if (!existing || entry.rating > existing.rating) {
          seen.set(entry.id, entry)
        }
      }
      setSearchResults(Array.from(seen.values()).sort((a, b) => b.rating - a.rating))
    } catch (e) {
      setError(errText(e))
    } finally {
      setSearchLoading(false)
    }
  }

  const installRemote = async (entry: MarketEntry) => {
    setInstallingId(entry.id)
    setError(null)
    try {
      await marketApi.marketInstall(entry.id, entry.source_type, entry.latest_version)
      await load()
    } catch (e) {
      setError(errText(e))
    } finally {
      setInstallingId(null)
    }
  }

  const sourceIcon = (source: string) => {
    switch (source) {
      case 'dsh-market': return <Globe class="w-3.5 h-3.5" />
      case 'github': return <Globe class="w-3.5 h-3.5" />
      default: return <Puzzle class="w-3.5 h-3.5" />
    }
  }

  const sourceColor = (source: string) => {
    switch (source) {
      case 'dsh-market': return 'text-emerald-600 bg-emerald-500/10'
      case 'github': return 'text-gray-600 bg-gray-500/10'
      default: return 'text-nt-io-600 bg-nt-io-500/10'
    }
  }

  const isInstalled = (id: string) => {
    return plugins().some(p => p.id === id) || installedMarket().some(m => m.plugin.id === id)
  }

  // ── Render ──

  const tabs: { id: TabType; label: string; icon: typeof Puzzle }[] = [
    { id: 'installed', label: '已安装', icon: Puzzle },
    { id: 'discover', label: '发现', icon: Search },
  ]

  return (
    <Show when={props.open}>
      <div class={props.embedded ? 'flex-1 h-full flex flex-col min-h-0' : 'panel w-[32rem]'}>
        {/* Header */}
        <div class="panel-head">
          <Puzzle class="panel-head-icon text-nt-mind-600" />
          <span class="panel-title">插件市场</span>
          <span class="panel-sub">{plugins().length} 本地 · {installedMarket().length} 市场</span>
          <button ref={firstBtnRef} class="panel-close" onClick={load} aria-label="刷新">
            <RefreshCw class={clsx('w-4 h-4', loading() && 'animate-spin')} />
          </button>
          <Show when={!props.embedded}>
            <button class="p-2 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors" onClick={props.onClose} aria-label="关闭">
              <X class="w-4 h-4" />
            </button>
          </Show>
        </div>

        {/* Tab bar */}
        <div class="flex border-b border-border-primary">
          <For each={tabs}>
            {(t) => (
              <button
                class={clsx(
                  'flex-1 flex items-center justify-center gap-1.5 px-3 py-2.5 text-xs font-medium transition-colors',
                  tab() === t.id
                    ? 'text-nt-mind-700 border-b-2 border-nt-mind-500 bg-nt-mind-500/5'
                    : 'text-text-muted hover:text-text-primary hover:bg-bg-tertiary'
                )}
                onClick={() => setTab(t.id)}
              >
                <t.icon class="w-3.5 h-3.5" />
                {t.label}
              </button>
            )}
          </For>
        </div>

        {/* Body */}
        <div class="flex-1 overflow-y-auto p-4 space-y-3">
          <Show when={error()}>
            <div class="p-3 text-xs text-red-500 bg-red-500/10 rounded-lg">{error()}</div>
          </Show>

          {/* ── Installed Tab ── */}
          <Show when={tab() === 'installed'}>
            <button
              class="w-full flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl bg-nt-mind-500/20 text-nt-mind-700 hover:bg-nt-mind-500/30 transition-colors text-sm font-medium"
              onClick={installLocal}
              disabled={busy() !== null}
            >
              {busy() === 'install' ? <Loader2 class="w-4 h-4 animate-spin" /> : <Download class="w-4 h-4" />}
              本地安装（选择 manifest.json）
            </button>

            <Show when={loading() && plugins().length === 0}>
              <div class="flex items-center justify-center gap-2 py-8 text-text-muted text-sm">
                <Loader2 class="w-4 h-4 animate-spin" />加载中...
              </div>
            </Show>

            <Show when={!loading() && plugins().length === 0 && installedMarket().length === 0 && !error()}>
              <div class="py-8 text-center text-xs text-text-muted space-y-2">
                <Box class="w-8 h-8 mx-auto text-text-muted/40" />
                <p>暂无插件</p>
                <p class="text-10px">切换到「发现」标签搜索在线插件</p>
              </div>
            </Show>

            {/* Native plugins */}
            <div class="space-y-2">
              <For each={plugins()}>
                {(p) => (
                  <div class={clsx('rounded-xl border p-3', p.enabled ? 'border-border-primary bg-bg-primary/40' : 'border-border-primary/50 bg-bg-primary/20 opacity-70')}>
                    <div class="flex items-center gap-2">
                      <Puzzle class={clsx('w-4 h-4 flex-shrink-0', p.enabled ? 'text-nt-mind-600' : 'text-text-muted')} />
                      <span class="text-sm font-medium text-text-primary truncate flex-1">{p.name}</span>
                      <span class="text-10px text-text-muted font-mono">{p.version}</span>
                      <span class={clsx('badge', p.enabled ? 'badge-success' : 'badge-warn')}>
                        {p.enabled ? '启用' : '禁用'}
                      </span>
                    </div>
                    <div class="flex items-center gap-1 mt-2 ml-6">
                      <button
                        class={clsx(
                          'flex items-center gap-1 px-2 py-1 rounded-lg text-xs transition-colors',
                          p.enabled
                            ? 'text-amber-600 hover:bg-amber-500/10 border border-amber-500/30'
                            : 'text-emerald-600 hover:bg-emerald-500/10 border border-emerald-500/30'
                        )}
                        onClick={() => toggleEnabled(p)}
                        disabled={busy() !== null}
                      >
                        {busy() === `toggle:${p.id}` ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : p.enabled ? <PowerOff class="w-3.5 h-3.5" /> : <Power class="w-3.5 h-3.5" />}
                        {p.enabled ? '禁用' : '启用'}
                      </button>
                      <button
                        class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs text-red-500 hover:bg-red-500/10 border border-red-500/30 transition-colors"
                        onClick={() => uninstall(p.id)}
                        disabled={busy() !== null}
                      >
                        {busy() === `uninstall:${p.id}` ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <Trash2 class="w-3.5 h-3.5" />}
                        卸载
                      </button>
                      <span class="ml-auto text-10px text-text-muted font-mono">{p.id}</span>
                    </div>
                  </div>
                )}
              </For>
            </div>

            {/* Market-installed plugins */}
            <Show when={installedMarket().length > 0}>
              <div class="text-xs text-text-muted font-medium mt-4 mb-2 flex items-center gap-1.5">
                <Globe class="w-3.5 h-3.5" /> 市场安装
              </div>
              <div class="space-y-2">
                <For each={installedMarket()}>
                  {(m) => (
                    <div class="rounded-xl border border-border-primary bg-bg-primary/40 p-3">
                      <div class="flex items-center gap-2">
                        <Globe class="w-4 h-4 flex-shrink-0 text-emerald-600" />
                        <span class="text-sm font-medium text-text-primary truncate flex-1">{m.plugin.name}</span>
                        <span class="text-10px text-text-muted font-mono">v{m.plugin.version}</span>
                        <span class="badge badge-success">已安装</span>
                      </div>
                      <p class="text-xs text-text-secondary mt-1 ml-6 line-clamp-2">{m.plugin.description}</p>
                      <div class="flex items-center gap-1 mt-2 ml-6">
                        <button
                          class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs text-red-500 hover:bg-red-500/10 border border-red-500/30 transition-colors"
                          onClick={() => uninstall(m.plugin.id)}
                          disabled={busy() !== null}
                        >
                          <Trash2 class="w-3.5 h-3.5" /> 卸载
                        </button>
                        <span class="ml-auto text-10px text-text-muted">{m.plugin.category}</span>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            {/* Event log */}
            <Show when={events().length > 0}>
              <div class="mt-4">
                <div class="flex items-center gap-2 text-xs text-text-muted mb-2">
                  <ListTree class="w-3.5 h-3.5" /> 事件日志
                </div>
                <div class="space-y-1 max-h-40 overflow-y-auto">
                  <For each={events()}>
                    {(ev) => (
                      <div class="flex items-center gap-2 text-[11px] font-mono px-2 py-1 rounded bg-bg-primary/40">
                        <span class="text-text-muted/60 flex-shrink-0">{new Date(ev.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}</span>
                        <span class={clsx('flex-shrink-0', ev.kind === 'loaded' ? 'text-emerald-600' : ev.kind === 'unloaded' ? 'text-red-500' : 'text-nt-memory-600')}>{ev.kind}</span>
                        <span class="text-text-muted flex-shrink-0">{ev.plugin_id}</span>
                        <span class="text-text-secondary truncate">{ev.message}</span>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            </Show>
          </Show>

          {/* ── Discover Tab ── */}
          <Show when={tab() === 'discover'}>
            {/* Search bar */}
            <div class="flex gap-2">
              <div class="flex-1 relative">
                <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted" />
                <input
                  ref={searchInputRef}
                  type="text"
                  value={searchQuery()}
                  onInput={(e) => setSearchQuery(e.currentTarget.value)}
                  onKeyDown={(e) => e.key === 'Enter' && doSearch()}
                  placeholder="搜索插件（DSH 市场 + GitHub）..."
                  class="w-full pl-9 pr-3 py-2 rounded-xl bg-bg-primary/60 border border-border-primary text-sm text-text-primary placeholder:text-text-muted/50 focus:outline-none focus:ring-2 focus:ring-nt-mind-500/50"
                />
              </div>
              <button
                class="px-4 py-2 rounded-xl bg-nt-mind-500 text-white text-sm font-medium hover:bg-nt-mind-600 transition-colors disabled:opacity-50"
                onClick={doSearch}
                disabled={searchLoading() || !searchQuery().trim()}
              >
                {searchLoading() ? <Loader2 class="w-4 h-4 animate-spin" /> : '搜索'}
              </button>
            </div>

            {/* Source filters */}
            <div class="flex items-center gap-2 text-xs text-text-muted">
              <span>来源:</span>
              <span class={clsx('px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-600', 'ring-1 ring-emerald-500/30')}>DSH 市场</span>
              <span class={clsx('px-2 py-0.5 rounded-full bg-gray-500/10 text-gray-600', 'ring-1 ring-gray-500/30')}>GitHub</span>
            </div>

            {/* Search results */}
            <Show when={searchLoading()}>
              <div class="flex items-center justify-center gap-2 py-8 text-text-muted text-sm">
                <Loader2 class="w-4 h-4 animate-spin" />搜索中...
              </div>
            </Show>

            <Show when={!searchLoading() && searchResults().length === 0 && searchQuery().trim()}>
              <div class="py-8 text-center text-xs text-text-muted">
                <p>未找到匹配的插件</p>
                <p class="mt-1">试试其他关键词</p>
              </div>
            </Show>

            <div class="space-y-2">
              <For each={searchResults()}>
                {(entry) => (
                  <div class="rounded-xl border border-border-primary bg-bg-primary/40 p-3 hover:border-nt-mind-500/30 transition-colors">
                    <div class="flex items-start gap-2">
                      <div class={clsx('w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0', sourceColor(entry.source_type))}>
                        {sourceIcon(entry.source_type)}
                      </div>
                      <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-2">
                          <span class="text-sm font-medium text-text-primary truncate">{entry.name}</span>
                          <span class="text-10px text-text-muted font-mono">v{entry.version}</span>
                          <span class={clsx('text-10px px-1.5 py-0.5 rounded-full', sourceColor(entry.source_type))}>
                            {entry.source_type}
                          </span>
                        </div>
                        <p class="text-xs text-text-secondary mt-0.5 line-clamp-2">{entry.description}</p>
                        <div class="flex items-center gap-3 mt-1.5 text-10px text-text-muted">
                          <span class="flex items-center gap-0.5"><Star class="w-3 h-3" />{entry.rating.toFixed(1)}</span>
                          <span class="flex items-center gap-0.5"><ArrowDownToLine class="w-3 h-3" />{entry.downloads}</span>
                          <span>{entry.author}</span>
                        </div>
                        {/* Tags */}
                        <Show when={entry.tags.length > 0}>
                          <div class="flex flex-wrap gap-1 mt-1.5">
                            <For each={entry.tags.slice(0, 5)}>
                              {(tag) => (
                                <span class="text-10px px-1.5 py-0.5 rounded bg-bg-tertiary text-text-muted flex items-center gap-0.5">
                                  <Tag class="w-2.5 h-2.5" />{tag}
                                </span>
                              )}
                            </For>
                          </div>
                        </Show>
                      </div>
                      <div class="flex flex-col items-end gap-1 flex-shrink-0">
                        <Show when={isInstalled(entry.id)}>
                          <span class="badge badge-success text-10px">已安装</span>
                        </Show>
                        <Show when={!isInstalled(entry.id)}>
                          <button
                            class="flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs bg-nt-mind-500 text-white hover:bg-nt-mind-600 transition-colors disabled:opacity-50"
                            onClick={() => installRemote(entry)}
                            disabled={installingId() === entry.id}
                          >
                            {installingId() === entry.id ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <Download class="w-3.5 h-3.5" />}
                            安装
                          </button>
                        </Show>
                        {/* Expand toggle */}
                        <button
                          class="text-text-muted hover:text-text-primary transition-colors"
                          onClick={() => setExpandedEntry(expandedEntry() === entry.id ? null : entry.id)}
                        >
                          {expandedEntry() === entry.id ? <ChevronDown class="w-3.5 h-3.5" /> : <ChevronRight class="w-3.5 h-3.5" />}
                        </button>
                      </div>
                    </div>
                    {/* Expanded details */}
                    <Show when={expandedEntry() === entry.id}>
                      <div class="mt-3 pt-3 border-t border-border-primary space-y-2 text-xs text-text-secondary">
                        <div class="grid grid-cols-2 gap-2">
                          <div><span class="text-text-muted">ID:</span> {entry.id}</div>
                          <div><span class="text-text-muted">分类:</span> {entry.category}</div>
                          <div><span class="text-text-muted">作者:</span> {entry.author}</div>
                          <div><span class="text-text-muted">版本:</span> {entry.latest_version}</div>
                        </div>
                        <Show when={entry.homepage}>
                          <a href={entry.homepage!} target="_blank" rel="noopener" class="flex items-center gap-1 text-nt-io-600 hover:underline">
                            <ExternalLink class="w-3 h-3" />主页
                          </a>
                        </Show>
                        <Show when={entry.source_repo}>
                          <a href={`https://github.com/${entry.source_repo}`} target="_blank" rel="noopener" class="flex items-center gap-1 text-nt-io-600 hover:underline">
                            <ExternalLink class="w-3 h-3" />{entry.source_repo}
                          </a>
                        </Show>
                      </div>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </div>

      <ConfirmModal
        req={modalReq()}
        onConfirm={() => pendingUninstallId() && doUninstall(pendingUninstallId()!)}
        onClose={() => {
          setPendingUninstallId(null)
          setModalReq(null)
        }}
      />
    </Show>
  )
}
