import { createSignal, onMount, createEffect, Show, For } from 'solid-js'
import { Puzzle, X, RefreshCw, Loader2, Download, Trash2, Power, PowerOff, ListTree, Box } from 'lucide-solid'
import { plugins as pluginsApi, errText, fs as fsApi } from '../api'
import type { PluginEvent, PluginStatus } from '../api/types'
import { clsx } from 'clsx'
import { ConfirmModal, type ModalReq } from './ConfirmModal'

interface Props {
  open: boolean
  onClose: () => void
  /** 内嵌模式：渲染在设置弹窗内（无浮层/无滑入动画/无关闭按钮） */
  embedded?: boolean
}

export function PluginMarketplace(props: Props) {
  const [plugins, setPlugins] = createSignal<PluginStatus[]>([])
  const [events, setEvents] = createSignal<PluginEvent[]>([])
  const [loading, setLoading] = createSignal(false)
  const [busy, setBusy] = createSignal<string | null>(null)
  const [error, setError] = createSignal<string | null>(null)
  // 统一确认模态（替换原生 confirm）
  const [modalReq, setModalReq] = createSignal<ModalReq | null>(null)
  const [pendingUninstallId, setPendingUninstallId] = createSignal<string | null>(null)
  let firstBtnRef: HTMLButtonElement | undefined

  // 面板打开时聚焦首个按钮（对标 Codex 面板聚焦规范）
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
    } catch (e) {
      setError(errText(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(load)

  const install = async () => {
    // Pick a plugin manifest .json via dialog
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
    // 破坏性操作确认（对标 Codex）— 统一模态
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
      await pluginsApi.pluginUninstall(id)
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

  const formatEventTime = (ts: number) => {
    const d = new Date(ts)
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  }

  const eventKindClass = (kind: string) => {
    if (kind === 'loaded') return 'text-emerald-600'
    if (kind === 'unloaded') return 'text-red-500'
    return 'text-nt-memory-600'
  }

  return (
    <Show when={props.open}>
      <div class={props.embedded ? 'flex-1 h-full flex flex-col min-h-0' : 'panel w-[28rem]'}>
        {/* Header */}
        <div class="panel-head">
          <Puzzle class="panel-head-icon text-nt-mind-600" />
          <span class="panel-title">插件市场</span>
          <span class="panel-sub">{plugins().length} 个插件</span>
          <button
            ref={firstBtnRef}
            class="panel-close"
            onClick={load}
            aria-label="刷新"
          >
            <RefreshCw class={clsx('w-4 h-4', loading() && 'animate-spin')} />
          </button>
          <Show when={!props.embedded}>
            <button
              class="p-2 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
              onClick={props.onClose}
              aria-label="关闭"
            >
              <X class="w-4 h-4" />
            </button>
          </Show>
        </div>

        {/* Body */}
        <div class="flex-1 overflow-y-auto p-4 space-y-4">
          <Show when={error()}>
            <div class="p-3 text-xs text-red-500 bg-red-500/10 rounded-lg">{error()}</div>
          </Show>

          {/* Install button */}
          <button
            class="w-full flex items-center justify-center gap-2 px-3 py-3 rounded-xl bg-nt-mind-500/20 text-nt-mind-700 hover:bg-nt-mind-500/30 transition-colors text-sm font-medium"
            onClick={install}
            disabled={busy() !== null}
          >
            {busy() === 'install' ? <Loader2 class="w-4 h-4 animate-spin" /> : <Download class="w-4 h-4" />}
            安装插件（选择 manifest.json）
          </button>

          {/* Plugin list */}
          <Show when={loading() && plugins().length === 0}>
            <div class="flex items-center justify-center gap-2 py-8 text-text-muted text-sm">
              <Loader2 class="w-4 h-4 animate-spin" />
              加载插件...
            </div>
          </Show>
          <Show when={!loading() && plugins().length === 0 && !error()}>
            <div class="py-8 text-center text-xs text-text-muted space-y-2">
              <Box class="w-8 h-8 mx-auto text-text-muted/40" />
              <p>暂无插件</p>
              <p class="text-[10px]">将插件 manifest.json 放入 {`${pluginsDirLabel()}`} 或点击上方安装</p>
            </div>
          </Show>

          <div class="space-y-2">
            <For each={plugins()}>
              {(p) => (
                <div class={clsx('rounded-xl border p-3', p.enabled ? 'border-border-primary bg-bg-primary/40' : 'border-border-primary/50 bg-bg-primary/20 opacity-70')}>
                  <div class="flex items-center gap-2">
                    <Puzzle class={clsx('w-4 h-4 flex-shrink-0', p.enabled ? 'text-nt-mind-600' : 'text-text-muted')} />
                    <span class="text-sm font-medium text-text-primary truncate flex-1">{p.name}</span>
                    <span class="text-[10px] text-text-muted font-mono">{p.version}</span>
                    <span
                      class={clsx('badge', p.enabled ? 'badge-success' : 'badge-warn')}
                      title={p.enabled ? '已启用' : '已禁用'}
                    >
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
                      aria-pressed={p.enabled}
                    >
                      {busy() === `toggle:${p.id}` ? (
                        <Loader2 class="w-3.5 h-3.5 animate-spin" />
                      ) : p.enabled ? (
                        <PowerOff class="w-3.5 h-3.5" />
                      ) : (
                        <Power class="w-3.5 h-3.5" />
                      )}
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
                    <span class="ml-auto text-[10px] text-text-muted font-mono">{p.id}</span>
                  </div>
                </div>
              )}
            </For>
          </div>

          {/* Event log */}
          <Show when={events().length > 0}>
            <div>
              <div class="flex items-center gap-2 text-xs text-text-muted mb-2">
                <ListTree class="w-3.5 h-3.5" />
                事件日志
              </div>
              <div class="space-y-1">
                <For each={events()}>
                  {(ev) => (
                    <div class="flex items-center gap-2 text-[11px] font-mono px-2 py-1 rounded bg-bg-primary/40">
                      <span class="text-text-muted/60 flex-shrink-0">{formatEventTime(ev.timestamp)}</span>
                      <span class={clsx('flex-shrink-0', eventKindClass(ev.kind))}>{ev.kind}</span>
                      <span class="text-text-muted flex-shrink-0">{ev.plugin_id}</span>
                      <span class="text-text-secondary truncate">{ev.message}</span>
                    </div>
                  )}
                </For>
              </div>
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

// Label helper for the default plugins dir (kept simple — matches backend default).
function pluginsDirLabel(): string {
  const home = typeof window !== 'undefined' ? '' : ''
  return `${home}.config/neotrix/plugins`
}
