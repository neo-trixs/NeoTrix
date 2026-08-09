import { createSignal, onMount, createEffect, For, Show } from 'solid-js'
import { Folder, FolderOpen, File, FileText, ChevronRight, ChevronDown, BookOpen, Loader2, X, RefreshCw } from 'lucide-solid'
import { invoke } from '@tauri-apps/api/core'
import { clsx } from 'clsx'

interface TreeItem {
  name: string
  path: string
  is_dir: boolean
  children?: TreeItem[] | null
}

interface ProjectView {
  root: string
  tree: TreeItem[]
  agents_md: string | null
  file_count: number
}

interface Props {
  open: boolean
  onClose: () => void
  /** Called with the absolute file path when a tree node is clicked. */
  onOpenFile?: (path: string) => void
}

export function ProjectView(props: Props) {
  const [view, setView] = createSignal<ProjectView | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [expanded, setExpanded] = createSignal<Set<string>>(new Set())
  const [tab, setTab] = createSignal<'tree' | 'agents'>('tree')
  let firstBtnRef: HTMLButtonElement | undefined

  // 面板打开时聚焦首个按钮（对标 Codex 面板聚焦规范）
  createEffect(() => {
    if (props.open && firstBtnRef) firstBtnRef.focus()
  })

  const load = async () => {
    setLoading(true)
    setError(null)
    try {
      const v = await invoke<ProjectView>('neocodex_project_tree')
      setView(v)
      // Auto-expand root dirs
      const firstLevel = v.tree.filter(t => t.is_dir).map(t => t.path)
      setExpanded(new Set(firstLevel))
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

  const baseName = () => view()?.root.split('/').filter(Boolean).pop() || '项目'

  const renderNode = (item: TreeItem, depth: number) => {
    const isOpen = expanded().has(item.path)
    return (
      <div>
        <button
          class="flex items-center gap-1.5 px-2 py-1 rounded-md cursor-pointer hover:bg-bg-tertiary/60 transition-colors text-sm w-full text-left bg-transparent border-none font-inherit focus-visible:ring-2 focus-visible:ring-nt-core-500 focus-visible:outline-none"
          style={{ 'padding-left': `${depth * 14 + 8}px` }}
          onClick={() => {
            if (item.is_dir) toggle(item.path)
            else props.onOpenFile?.(item.path)
          }}
          title={item.path}
          aria-expanded={item.is_dir ? isOpen : undefined}
        >
          {item.is_dir ? (
            <>
              {isOpen ? (
                <ChevronDown class="w-3.5 h-3.5 text-text-muted flex-shrink-0" />
              ) : (
                <ChevronRight class="w-3.5 h-3.5 text-text-muted flex-shrink-0" />
              )}
              {isOpen ? (
                <FolderOpen class="w-4 h-4 text-nt-core-600 flex-shrink-0" />
              ) : (
                <Folder class="w-4 h-4 text-nt-core-600 flex-shrink-0" />
              )}
            </>
          ) : (
            <>
              <span class="w-3.5 flex-shrink-0" />
              {item.name.toLowerCase() === 'agents.md' ? (
                <FileText class="w-4 h-4 text-nt-io-600 flex-shrink-0" />
              ) : (
                <File class="w-4 h-4 text-text-muted flex-shrink-0" />
              )}
            </>
          )}
          <span class={clsx('truncate', item.is_dir ? 'font-medium text-text-primary' : 'text-text-secondary')}>
            {item.name}
          </span>
        </button>
        <Show when={item.is_dir && isOpen && item.children}>
          <For each={item.children}>
            {(child) => renderNode(child, depth + 1)}
          </For>
        </Show>
      </div>
    )
  }

  return (
    <Show when={props.open}>
      <div class="panel w-80">
        {/* Header */}
        <div class="panel-head">
          <BookOpen class="panel-head-icon text-nt-core-600" />
          <span class="panel-title">{baseName()}</span>
          <Show when={view()}>
            <span class="panel-sub">({view()!.file_count} 文件)</span>
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
            class="panel-close"
            onClick={props.onClose}
            aria-label="关闭项目视图"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        {/* Tabs */}
        <div class="flex border-b border-border-primary" role="tablist" aria-label="项目视图切换">
          <button
            class={clsx(
              'flex-1 py-2 text-xs font-medium transition-colors',
              tab() === 'tree'
                ? 'text-nt-core-700 border-b-2 border-nt-core-500'
                : 'text-text-muted hover:text-text-primary'
            )}
            onClick={() => setTab('tree')}
            role="tab"
            aria-selected={tab() === 'tree'}
            tabIndex={tab() === 'tree' ? 0 : -1}
            onKeyDown={(e) => {
              if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') { e.preventDefault(); setTab(tab() === 'tree' ? 'agents' : 'tree') }
            }}
          >
            目录树
          </button>
          <button
            class={clsx(
              'flex-1 py-2 text-xs font-medium transition-colors',
              tab() === 'agents'
                ? 'text-nt-core-700 border-b-2 border-nt-core-500'
                : 'text-text-muted hover:text-text-primary'
            )}
            onClick={() => setTab('agents')}
            role="tab"
            aria-selected={tab() === 'agents'}
            tabIndex={tab() === 'agents' ? 0 : -1}
            onKeyDown={(e) => {
              if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') { e.preventDefault(); setTab(tab() === 'tree' ? 'agents' : 'tree') }
            }}
          >
            AGENTS.md
          </button>
        </div>

        {/* Body */}
        <div class="flex-1 overflow-y-auto p-2">
          <Show when={loading}>
            <div class="flex items-center justify-center gap-2 py-8 text-text-muted text-sm">
              <Loader2 class="w-4 h-4 animate-spin" />
              加载项目...
            </div>
          </Show>
          <Show when={error()}>
            <div class="p-3 text-xs text-red-500 bg-red-500/10 rounded-lg">{error()}</div>
          </Show>
          <Show when={!loading && !error() && view()}>
            {tab() === 'tree' ? (
              <For each={view()!.tree}>
                {(item) => renderNode(item, 0)}
              </For>
            ) : view()!.agents_md ? (
              <pre class="whitespace-pre-wrap break-words text-xs text-text-secondary font-mono p-2">
                {view()!.agents_md}
              </pre>
            ) : (
              <div class="p-4 text-xs text-text-muted text-center">项目中没有 AGENTS.md</div>
            )}
          </Show>
        </div>
      </div>
    </Show>
  )
}
