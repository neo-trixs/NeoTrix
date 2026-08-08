import { createSignal, onMount, createEffect, Show, For } from 'solid-js'
import { GitBranch, FileCode2, Check, X, Loader2, RefreshCw, ChevronDown, ChevronRight } from 'lucide-solid'
import { invoke } from '@tauri-apps/api/core'
import { clsx } from 'clsx'

interface DiffLine {
  t: 'add' | 'del' | 'ctx'
  o: number | null
  n: number | null
  s: string
}

interface Hunk {
  lines: DiffLine[]
}

interface DiffFile {
  path: string
  hunks: Hunk[]
}

interface DiffResponse {
  files: DiffFile[]
}

interface GitStatus {
  branch: string
  dirty: boolean
}

interface Props {
  open: boolean
  onClose: () => void
}

export function GitPanel(props: Props) {
  const [status, setStatus] = createSignal<GitStatus | null>(null)
  const [diff, setDiff] = createSignal<DiffResponse | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [expanded, setExpanded] = createSignal<Set<string>>(new Set())
  const [busy, setBusy] = createSignal<string | null>(null)
  const [error, setError] = createSignal<string | null>(null)
  let firstBtnRef: HTMLButtonElement | undefined

  // 面板打开时聚焦首个按钮（对标 Codex 面板聚焦规范）
  createEffect(() => {
    if (props.open && firstBtnRef) firstBtnRef.focus()
  })

  const load = async () => {
    setLoading(true)
    setError(null)
    try {
      const [st, df] = await Promise.all([
        invoke<GitStatus | null>('neocodex_git_status'),
        invoke<DiffResponse>('neocodex_get_diff'),
      ])
      setStatus(st)
      setDiff(df)
      // Auto-expand first file
      if (df.files.length > 0) setExpanded(new Set([df.files[0].path]))
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(load)

  const toggle = (path: string) => {
    setExpanded(prev => {
      const next = new Set(prev)
      if (next.has(path)) next.delete(path)
      else next.add(path)
      return next
    })
  }

  const apply = async (path: string, action: 'accept' | 'reject') => {
    setBusy(`${path}:${action}`)
    setError(null)
    try {
      await invoke('neocodex_apply_diff', { path, action })
      await load()
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(null)
    }
  }

  const renderLine = (line: DiffLine) => {
    const num = line.t === 'add' ? line.n : line.t === 'del' ? line.o : line.n
    return (
      <div
        class={clsx(
          'flex items-start gap-2 px-2 py-0.5 text-xs font-mono leading-relaxed',
          line.t === 'add' && 'bg-emerald-500/10 text-emerald-300',
          line.t === 'del' && 'bg-red-500/10 text-red-300',
          line.t === 'ctx' && 'text-text-secondary'
        )}
      >
        <span class="w-8 flex-shrink-0 text-right text-text-muted/50 select-none">
          {line.t === 'add' ? '+' : line.t === 'del' ? '-' : ' '}
          {num ?? ''}
        </span>
        <span class="whitespace-pre-wrap break-all">{line.s}</span>
      </div>
    )
  }

  const totalChanges = () => {
    const d = diff()
    if (!d) return 0
    return d.files.reduce((acc, f) => acc + f.hunks.reduce((a, h) => a + h.lines.filter(l => l.t !== 'ctx').length, 0), 0)
  }

  return (
    <Show when={props.open}>
      <div class="panel w-[30rem]">
        {/* Header */}
        <div class="panel-head">
          <GitBranch class="panel-head-icon text-nt-act-600" />
          <span class="panel-title">Git 变更</span>
          <Show when={status()}>
            <span class="panel-sub font-mono">{status()!.branch}</span>
            <span class={clsx(status()!.dirty ? 'badge-warn' : 'badge-success')}>
              {status()!.dirty ? `${totalChanges()} 处变更` : '干净'}
            </span>
          </Show>
          <button
            ref={firstBtnRef}
            class="panel-close"
            onClick={load}
            aria-label="刷新"
            title="刷新"
          >
            <RefreshCw class={clsx('w-4 h-4', loading() && 'animate-spin')} />
          </button>
          <button
            class="p-1.5 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
            onClick={props.onClose}
            aria-label="关闭"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div class="flex-1 overflow-y-auto p-3">
          <Show when={loading}>
            <div class="flex items-center justify-center gap-2 py-8 text-text-muted text-sm">
              <Loader2 class="w-4 h-4 animate-spin" />
              加载变更...
            </div>
          </Show>
          <Show when={error()}>
            <div class="p-3 text-xs text-red-500 bg-red-500/10 rounded-lg mb-2">{error()}</div>
          </Show>
          <Show when={!loading && diff() && diff()!.files.length === 0 && !error()}>
            <div class="py-10 text-center text-xs text-text-muted">工作区干净，没有未提交的变更</div>
          </Show>
          <Show when={!loading && diff() && diff()!.files.length > 0}>
            <div class="space-y-2">
              <For each={diff()!.files}>
                {(file) => {
                  const isOpen = () => expanded().has(file.path)
                  const addCount = file.hunks.reduce((a, h) => a + h.lines.filter(l => l.t === 'add').length, 0)
                  const delCount = file.hunks.reduce((a, h) => a + h.lines.filter(l => l.t === 'del').length, 0)
                  return (
                    <div class="rounded-lg border border-border-primary overflow-hidden bg-bg-primary/40">
                      {/* File header */}
                      <div class="flex items-center gap-2 px-3 py-2 bg-bg-secondary/60">
                        <button
                          class="flex items-center gap-2 flex-1 min-w-0 text-left"
                          onClick={() => toggle(file.path)}
                        >
                          {isOpen() ? (
                            <ChevronDown class="w-4 h-4 text-text-muted flex-shrink-0" />
                          ) : (
                            <ChevronRight class="w-4 h-4 text-text-muted flex-shrink-0" />
                          )}
                          <FileCode2 class="w-4 h-4 text-nt-act-600 flex-shrink-0" />
                          <span class="text-sm text-text-primary truncate font-mono">{file.path}</span>
                        </button>
                        <span class="text-xs text-emerald-600 flex-shrink-0">+{addCount}</span>
                        <span class="text-xs text-red-500 flex-shrink-0">-{delCount}</span>
                        <button
                          class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs text-emerald-600 hover:bg-emerald-500/10 border border-emerald-500/30 flex-shrink-0"
                          onClick={() => apply(file.path, 'accept')}
                          disabled={busy() !== null}
                          aria-label="接受变更"
                          title="git add"
                        >
                          {busy() === `${file.path}:accept` ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <Check class="w-3.5 h-3.5" />}
                          接受
                        </button>
                        <button
                          class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs text-red-500 hover:bg-red-500/10 border border-red-500/30 flex-shrink-0"
                          onClick={() => apply(file.path, 'reject')}
                          disabled={busy() !== null}
                          aria-label="拒绝此变更"
                          title="git restore"
                        >
                          {busy() === `${file.path}:reject` ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <X class="w-3.5 h-3.5" />}
                          拒绝
                        </button>
                      </div>
                      {/* Diff body */}
                      <Show when={isOpen()}>
                        <div class="border-t border-border-primary">
                          <For each={file.hunks}>
                            {(hunk) => (
                              <div class="py-0.5">
                                <For each={hunk.lines}>
                                  {(line) => renderLine(line)}
                                </For>
                              </div>
                            )}
                          </For>
                        </div>
                      </Show>
                    </div>
                  )
                }}
              </For>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  )
}