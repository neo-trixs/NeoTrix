import { For, Show } from 'solid-js'
import { clsx } from 'clsx'
import type { ProjectTreeItem } from '../api/types'

export interface FileNode {
  name: string
  type: 'dir' | 'file'
  open?: boolean
  content?: string
  path?: string
  children?: FileNode[]
}

export function toFileNode(item: ProjectTreeItem): FileNode {
  return {
    name: item.name,
    type: item.is_dir ? 'dir' : 'file',
    path: item.path,
    open: false,
    children: item.children?.map(toFileNode),
  }
}

/* ── 展开状态保持：收集当前展开目录 → 新树重新应用 ── */
export const collectOpenPaths = (nodes: FileNode[], acc: Set<string> = new Set()): Set<string> => {
  for (const n of nodes) {
    if (n.type === 'dir' && n.open) {
      acc.add(n.path ?? n.name)
      collectOpenPaths(n.children ?? [], acc)
    }
  }
  return acc
}

export const applyOpen = (nodes: FileNode[], openPaths: Set<string>) => {
  for (const n of nodes) {
    if (n.type === 'dir') {
      n.open = openPaths.has(n.path ?? n.name)
      applyOpen(n.children ?? [], openPaths)
    }
  }
}

interface FileTreeProps {
  nodes: FileNode[]
  depth?: number
  onOpenFile: (node: FileNode) => void
  activeFile: string | null
  activePath: string | null
  hasActive: boolean
  onToggleDir: (node: FileNode) => void
  onActivate: (node: FileNode) => void
  onMoveFocus: (dir: 1 | -1) => void
}

function FileTreeInner(props: FileTreeProps) {
  const rowKey = (n: FileNode) => n.path ?? n.name
  const rowTab = (n: FileNode) =>
    props.activePath === rowKey(n) || (!props.hasActive && (props.depth ?? 0) === 0 && props.nodes[0] === n) ? 0 : -1

  const onRowKeyDown = (e: KeyboardEvent, n: FileNode) => {
    if (e.key === 'ArrowDown') { e.preventDefault(); props.onMoveFocus(1) }
    else if (e.key === 'ArrowUp') { e.preventDefault(); props.onMoveFocus(-1) }
    else if (e.key === 'ArrowRight') {
      if (n.type === 'dir' && !n.open) { e.preventDefault(); props.onToggleDir(n) }
    } else if (e.key === 'ArrowLeft') {
      if (n.type === 'dir' && n.open) { e.preventDefault(); props.onToggleDir(n) }
    } else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      if (n.type === 'dir') props.onToggleDir(n)
      else props.onOpenFile(n)
    }
  }

  return (
    <For each={props.nodes}>
      {(n) => (
        <>
          <div
            class={clsx('ft-item', n.open && 'open', props.activeFile === rowKey(n) && 'ft-active')}
            style={{ 'padding-left': `${(props.depth ?? 0) * 14 + 4}px` }}
            role="treeitem"
            aria-expanded={n.type === 'dir' ? n.open : undefined}
            aria-current={props.activeFile === rowKey(n) ? 'true' : undefined}
            tabIndex={rowTab(n)}
            onClick={() => (n.type === 'dir' ? props.onToggleDir(n) : props.onOpenFile(n))}
            onFocus={() => props.onActivate(n)}
            onKeyDown={(e) => onRowKeyDown(e, n)}
          >
            {n.type === 'dir' ? (
              <svg class={clsx('chev', n.open && 'open')} viewBox="0 0 9 9">
                <line x1="3" y1="2.5" x2="6" y2="4.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                <line x1="3" y1="6.5" x2="6" y2="4.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
              </svg>
            ) : null}
            {n.type === 'dir' ? (
              <svg class="fic" viewBox="0 0 14 14">
                <path d="M1.5 4.5h3.5l1-1.5h6a1 1 0 011 1v6a1 1 0 01-1 1h-10a1 1 0 01-1-1v-5.5z" stroke="currentColor" stroke-width="1" fill="none" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            ) : (
              <svg class="fic" viewBox="0 0 14 14">
                <path d="M2 1.5h10v11H2z" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linejoin="round" />
                <line x1="4.5" y1="4.5" x2="9.5" y2="4.5" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
              </svg>
            )}
            {n.name}
          </div>
          <Show when={n.type === 'dir'}>
            <div class={clsx('ft-children', n.open && 'open')} role="group">
              <Show when={n.open}>
                <FileTreeInner
                  nodes={n.children ?? []}
                  depth={(props.depth ?? 0) + 1}
                  onOpenFile={props.onOpenFile}
                  activeFile={props.activeFile}
                  activePath={props.activePath}
                  hasActive={props.hasActive}
                  onToggleDir={props.onToggleDir}
                  onActivate={props.onActivate}
                  onMoveFocus={props.onMoveFocus}
                />
              </Show>
            </div>
          </Show>
        </>
      )}
    </For>
  )
}

interface FileTreeViewProps {
  tree: FileNode[]
  rootPath: string
  fileCount: number
  treeLoading: boolean
  treeError: string | null
  fileLoading: boolean
  currentFile: FileNode | null
  activePath: string | null
  onOpenFile: (node: FileNode) => void
  onToggleDir: (node: FileNode) => void
  onActivate: (node: FileNode) => void
  onMoveFocus: (dir: 1 | -1) => void
}

export function FileTreeView(props: FileTreeViewProps) {
  return (
    <div class="ft">
      <div class="ft-head">
        <span class="ft-root" title={props.rootPath}>{props.rootPath || '项目'}</span>
        <span class="ft-count">{props.fileCount} 文件</span>
      </div>
      <Show when={props.treeLoading && props.tree.length === 0}>
        <div class="ft-loading">加载项目树…</div>
      </Show>
      <Show when={props.treeError}>
        <div class="ft-error">{props.treeError}</div>
      </Show>
      <Show when={!props.treeLoading && !props.treeError && props.tree.length === 0}>
        <div class="ft-empty">项目为空</div>
      </Show>
      <FileTreeInner
        nodes={props.tree}
        onOpenFile={props.onOpenFile}
        activeFile={props.currentFile?.path ?? null}
        activePath={props.activePath}
        hasActive={props.activePath !== null}
        onToggleDir={props.onToggleDir}
        onActivate={props.onActivate}
        onMoveFocus={props.onMoveFocus}
      />
      <Show when={props.fileLoading}>
        <div class="ft-loading">读取文件…</div>
      </Show>
    </div>
  )
}
