import { createSignal, onMount, For, Show } from 'solid-js'
import { neocodex, system, errText } from '../api'
import type { ProjectTreeItem, ProjectView } from '../api/types'
import { clsx } from 'clsx'
import { GlobeView } from './GlobeView'
import { ProjectView as ProjectViewPanel } from './ProjectView'

/* ════════════════════════════════════════════
   RightBar — 右栏（设计 v2，已接线后端）
   标签切换：文件（Artifact Pane + 文件树） / 地图（shanhai 3D 地球） / 项目（ProjectView）
   上部：Artifact Pane（预览/代码切换 + 格式 tabs + 内容）
   下部：文件树（真实项目树 ← neocodex_project_tree）
   交互：auto-hide（hover/右侧边缘展开）或 collapsed 固定
   ════════════════════════════════════════════ */

type RbTab = 'files' | 'map' | 'project'

const RB_TABS: RbTab[] = ['files', 'map', 'project']

interface FileNode {
  name: string
  type: 'dir' | 'file'
  open?: boolean
  content?: string
  path?: string
  children?: FileNode[]
}

/* 后端 ProjectTreeItem → 前端 FileNode */
function toFileNode(item: ProjectTreeItem): FileNode {
  return {
    name: item.name,
    type: item.is_dir ? 'dir' : 'file',
    path: item.path,
    open: item.is_dir && item.children != null && item.children.length > 0,
    children: item.children?.map(toFileNode),
  }
}

const PREVIEW_FORMATS = [
  { id: 'raw', label: 'Raw' },
  { id: 'rendered', label: 'Rendered' },
] as const

type PreviewMode = (typeof PREVIEW_FORMATS)[number]['id']

const RUST_KEYWORDS = ['pub', 'struct', 'impl', 'fn', 'let', 'mut', 'const', 'Self', 'for', 'in', 'return', 'if', 'else', 'match', 'use', 'mod', 'trait', 'enum', 'type', 'where', 'as', 'async', 'await', 'move']

function escHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

/* 轻量 MD → 安全 HTML（仅用于 artifact 预览）
   顺序：先提取代码围栏为占位符 → 转义 → 行内语法 → 还原围栏
   避免代码块内的 # / ** / ` 被误转成 HTML 标签 */
function renderMd(text: string): string {
  const fences: string[] = []
  let t = text.replace(/```(\w*)\n?([\s\S]*?)```/g, (_m, _lang, code) => {
    fences.push(escHtml(code))
    return `\u0000F${fences.length - 1}\u0000`
  })
  t = t
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/^#### (.+)$/gm, '<h4>$1</h4>')
    .replace(/^### (.+)$/gm, '<h3>$1</h3>')
    .replace(/^## (.+)$/gm, '<h2>$1</h2>')
    .replace(/^# (.+)$/gm, '<h1>$1</h1>')
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/^- (.+)$/gm, '<li>$1</li>')
    .replace(/(<li>.*<\/li>\n?)+/g, '<ul>$&</ul>')
    .replace(/^> (.+)$/gm, '<blockquote>$1</blockquote>')
    .replace(/\n\n/g, '</p><p>')
    .replace(/^(?!<[hpl]|<[uo]l|<pre|<bl|$)/gm, '<p>')
    .replace(/<\/p>\s*<p>/g, '</p><p>')
  return t.replace(/\u0000F(\d+)\u0000/g, (_m, i) => `<pre><code>${fences[+i]}</code></pre>`)
}

function FileTree(props: {
  nodes: FileNode[]
  depth?: number
  onOpenFile: (node: FileNode) => void
  activeFile: string | null
  activePath: string | null
  hasActive: boolean
  onToggleDir: (node: FileNode) => void
  onActivate: (node: FileNode) => void
  onMoveFocus: (dir: 1 | -1) => void
}) {
  const rowKey = (n: FileNode) => n.path ?? n.name
  /* roving tabindex：活动节点 0，其余 -1；无活动节点时首行 0（Tab 可达） */
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
                <FileTree
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

export function RightBar() {
  const [collapsed, setCollapsed] = createSignal(false)
  const [autoHide, setAutoHide] = createSignal(true)
  const [rbHover, setRbHover] = createSignal(false)
  const [rbTab, setRbTab] = createSignal<RbTab>('files')
  const [previewOpen, setPreviewOpen] = createSignal(false)
  const [currentFile, setCurrentFile] = createSignal<FileNode | null>(null)
  const [previewMode, setPreviewMode] = createSignal<PreviewMode>('rendered')
  const [artifactView, setArtifactView] = createSignal<'preview' | 'code'>('preview')
  const [tree, setTree] = createSignal<FileNode[]>([])
  const [rootPath, setRootPath] = createSignal('')
  const [fileCount, setFileCount] = createSignal(0)
  const [treeLoading, setTreeLoading] = createSignal(false)
  const [fileLoading, setFileLoading] = createSignal(false)
  const [treeError, setTreeError] = createSignal<string | null>(null)
  const [copied, setCopied] = createSignal(false)
  const [activePath, setActivePath] = createSignal<string | null>(null)

  /* ── 展开状态保持：收集当前展开目录 → 新树重新应用 ── */
  const collectOpenPaths = (nodes: FileNode[], acc: Set<string> = new Set()): Set<string> => {
    for (const n of nodes) {
      if (n.type === 'dir' && n.open) {
        acc.add(n.path ?? n.name)
        collectOpenPaths(n.children ?? [], acc)
      }
    }
    return acc
  }
  const applyOpen = (nodes: FileNode[], openPaths: Set<string>) => {
    for (const n of nodes) {
      if (n.type === 'dir') {
        n.open = openPaths.has(n.path ?? n.name)
        applyOpen(n.children ?? [], openPaths)
      }
    }
  }

  let treeReqSeq = 0
  /* 加载真实项目树（neocodex_project_tree） */
  const loadTree = async () => {
    const seq = ++treeReqSeq
    const prevOpen = collectOpenPaths(tree())
    setTreeLoading(true)
    setTreeError(null)
    try {
      const pv = await neocodex.projectTree()
      if (seq !== treeReqSeq) return
      const nodes = pv.tree.map(toFileNode)
      /* 刷新保留用户展开状态；首次加载用默认展开（toFileNode 已处理） */
      if (prevOpen.size > 0) applyOpen(nodes, prevOpen)
      setTree(nodes)
      setRootPath(pv.root)
      setFileCount(pv.file_count)
      if (!activePath()) setActivePath(nodes[0]?.path ?? null)
    } catch (e) {
      if (seq !== treeReqSeq) return
      setTreeError(errText(e))
    } finally {
      if (seq === treeReqSeq) setTreeLoading(false)
    }
  }

  /* 🔴 修复：loadTree 读取 tree() 信号并在完成后 setTree（新数组引用），
   * createEffect 会因依赖自写信号陷入「setTree → 重跑 → 再 setTree」无限循环，
   * 对后端 neocodex_project_tree IPC 持续轰炸。改为 onMount 单次加载 +
   * 显式刷新按钮（ap-footer「刷新」已复用 loadTree）。 */
  onMount(loadTree)

  const toggleRb = () => {
    if (autoHide()) {
      setAutoHide(false)
      setCollapsed(false)
      return
    }
    setCollapsed(!collapsed())
  }

  const openPreview = async (node: FileNode) => {
    setCollapsed(false)
    setAutoHide(false)
    setPreviewOpen(true)
    setCurrentFile(node)
    // 文件内容懒加载：真实读取
    if (node.type === 'file' && node.path && !node.content) {
      setFileLoading(true)
      try {
        const content = await system.readFile(node.path)
        node.content = content
        setCurrentFile({ ...node, content })
      } catch (e) {
        node.content = `// 读取失败: ${errText(e)}`
        setCurrentFile({ ...node })
      } finally {
        setFileLoading(false)
      }
    }
  }

  const toggleDir = (node: FileNode) => {
    /* 不可变更新：沿路径重建对象，避免直接 mutate signal 内对象 */
    setTree((t) => {
      const flip = (nodes: FileNode[]): FileNode[] =>
        nodes.map((n) =>
          n === node
            ? { ...n, open: !n.open }
            : n.type === 'dir' && n.children
              ? { ...n, children: flip(n.children) }
              : n,
        )
      return flip(t)
    })
  }

  /* ProjectView onOpenFile → 复用 Artifact Pane 预览 */
  const openPath = (path: string) => {
    const name = path.split('/').pop() ?? path
    openPreview({ name, path, type: 'file' })
  }

  /* 树键盘导航：↑/↓ 在可见 treeitem 间移动焦点 */
  const activateNode = (node: FileNode) => setActivePath(node.path ?? node.name)
  const moveTreeFocus = (dir: 1 | -1) => {
    const items = Array.from(document.querySelectorAll<HTMLElement>('.ft [role="treeitem"]'))
    const idx = items.findIndex((el) => el === document.activeElement)
    const target = idx === -1 ? (dir === 1 ? items[0] : items[items.length - 1]) : items[idx + dir]
    target?.focus()
  }

  /* 右栏标签方向键切换（roving tabindex，← 后退 / → 前进，环绕） */
  const moveRbTab = (dir: 1 | -1) => {
    setRbTab((cur) => RB_TABS[(RB_TABS.indexOf(cur) + dir + RB_TABS.length) % RB_TABS.length])
  }
  const tabKeyDown = (e: KeyboardEvent) => {
    if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return
    e.preventDefault()
    moveRbTab(e.key === 'ArrowRight' ? 1 : -1)
    requestAnimationFrame(() => {
      const tabs = document.querySelectorAll<HTMLElement>('.rb-tabs [role="tab"]')
      tabs[RB_TABS.indexOf(rbTab())]?.focus()
    })
  }

  const closePreview = () => {
    setPreviewOpen(false)
    setCurrentFile(null)
  }

  const copyPreview = async () => {
    const f = currentFile()
    if (f?.content) {
      try {
        await navigator.clipboard.writeText(f.content)
        setCopied(true)
        setTimeout(() => setCopied(false), 1500)
      } catch { /* ignore */ }
    }
  }

  const toggleExpand = () => {
    setPreviewOpen(!previewOpen())
  }

  /* 渲染 artifact 内容：raw 模式带 Rust 语法着色，其余为 MD 渲染 */
  const renderArtifact = (): string => {
    const f = currentFile()
    const text = f?.content ?? '// 点击文件预览'
    if (artifactView() === 'code' || previewMode() === 'raw') {
      /* 逐行处理：注释行整体着色（不再被关键字 span 二次转义），非注释行才高亮关键字 */
      return text
        .split('\n')
        .map((line) => {
          if (line.trimStart().startsWith('//')) {
            return `<span class="cm">${escHtml(line)}</span>`
          }
          let l = escHtml(line)
          RUST_KEYWORDS.forEach((k) => {
            l = l.replace(new RegExp(`\\b${k}\\b`, 'g'), `<span class="kw">${k}</span>`)
          })
          l = l.replace(/\b[A-Z]\w+(?=\s*(?:[({<]|::))/g, (m) => `<span class="fn">${m}</span>`)
          return l
        })
        .join('\n')
    }
    return renderMd(text)
  }

  return (
    <aside
      class={clsx('rb h-screen flex-shrink-0', autoHide() && 'auto-hide', rbHover() && 'rb-hover', !autoHide() && collapsed() && 'collapsed')}
      onMouseEnter={() => autoHide() && setRbHover(true)}
      onMouseLeave={() => autoHide() && setRbHover(false)}
    >
      <button class="rb-float" onClick={toggleRb} title="切换侧栏" aria-label="切换侧栏">
        <svg viewBox="0 0 8 8">
          <line x1="5" y1="2" x2="3" y2="4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          <line x1="5" y1="6" x2="3" y2="4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        </svg>
      </button>

      <div class="rb-content">
        {/* ── 标签切换：文件 / 地图 / 项目 ── */}
        <div class="rb-tabs" role="tablist" aria-label="右栏视图">
          <button
            class={clsx('rb-tab', rbTab() === 'files' && 'on')}
            onClick={() => setRbTab('files')}
            role="tab"
            aria-selected={rbTab() === 'files'}
            tabIndex={rbTab() === 'files' ? 0 : -1}
            onKeyDown={tabKeyDown}
          >
            <svg viewBox="0 0 14 14" class="rb-tab-ic">
              <path d="M2 1.5h10v11H2z" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linejoin="round" />
              <line x1="4.5" y1="4.5" x2="9.5" y2="4.5" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
            </svg>
            文件
          </button>
          <button
            class={clsx('rb-tab', rbTab() === 'map' && 'on')}
            onClick={() => setRbTab('map')}
            role="tab"
            aria-selected={rbTab() === 'map'}
            tabIndex={rbTab() === 'map' ? 0 : -1}
            onKeyDown={tabKeyDown}
          >
            <svg viewBox="0 0 14 14" class="rb-tab-ic">
              <circle cx="7" cy="7" r="5.5" stroke="currentColor" stroke-width="1.1" fill="none" />
              <ellipse cx="7" cy="7" rx="2.6" ry="5.5" stroke="currentColor" stroke-width="1.1" fill="none" />
              <line x1="1.5" y1="7" x2="12.5" y2="7" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
            </svg>
            地图
          </button>
          <button
            class={clsx('rb-tab', rbTab() === 'project' && 'on')}
            onClick={() => setRbTab('project')}
            role="tab"
            aria-selected={rbTab() === 'project'}
            tabIndex={rbTab() === 'project' ? 0 : -1}
            onKeyDown={tabKeyDown}
          >
            <svg viewBox="0 0 14 14" class="rb-tab-ic">
              <path d="M1.5 2.5h4.5a1.5 1.5 0 011.5 1.5v7.5a1.5 1.5 0 00-1.5-1.5H1.5z" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linejoin="round" />
              <path d="M12.5 2.5H8a1.5 1.5 0 00-1.5 1.5v7.5a1.5 1.5 0 011.5-1.5h4.5z" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linejoin="round" />
            </svg>
            项目
          </button>
        </div>

        {/* ── 地图视图：shanhai 3D 地球 ── */}
        <Show when={rbTab() === 'map'}>
          <div class="rb-map">
            <GlobeView limit={3000} height={420} />
          </div>
        </Show>

        {/* ── 项目视图：ProjectView（AGENTS.md 阅读器 + 目录树） ── */}
        <Show when={rbTab() === 'project'}>
          <div class="rb-project flex-1 min-h-0 overflow-hidden flex flex-col">
            <ProjectViewPanel open onClose={() => setRbTab('files')} onOpenFile={openPath} />
          </div>
        </Show>

        {/* ── 文件视图：Artifact Pane + File Tree ── */}
        <Show when={rbTab() === 'files'}>
        {/* ── Artifact Pane ── */}
        <div class={clsx('ap', !previewOpen() && 'mini')} onClick={toggleExpand}>
          <div class="ap-bar">
            <div class="ap-bar-left">
              <button
                class={clsx('ap-view-btn', artifactView() === 'preview' && 'on')}
                onClick={(e) => { e.stopPropagation(); setArtifactView('preview') }}
                title="Preview"
                aria-label="预览视图"
              >
                <svg viewBox="0 0 14 14">
                  <path d="M1.5 7s2.5-4.5 5.5-4.5S12.5 7 12.5 7s-2.5 4.5-5.5 4.5S1.5 7 1.5 7z" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" />
                  <circle cx="7" cy="7" r="2" stroke="currentColor" stroke-width="1.2" fill="none" />
                </svg>
              </button>
              <button
                class={clsx('ap-view-btn', artifactView() === 'code' && 'on')}
                onClick={(e) => { e.stopPropagation(); setArtifactView('code') }}
                title="Code"
                aria-label="代码视图"
              >
                <svg viewBox="0 0 14 14">
                  <polyline points="4,4 1.5,7 4,10" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
                  <polyline points="10,4 12.5,7 10,10" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
                  <line x1="8.5" y1="3.5" x2="5.5" y2="10.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                </svg>
              </button>
            </div>
            <button
              class="ap-title"
              onClick={(e) => { e.stopPropagation(); toggleExpand() }}
              aria-expanded={previewOpen()}
              title={previewOpen() ? '折叠预览区' : '展开预览区'}
            >
              {currentFile()?.name ?? '未选择文件'}
            </button>
          </div>

          <Show when={previewOpen()}>
            {/* ── 格式 tabs ── */}
            <div class="ap-tabs" onClick={(e) => e.stopPropagation()} role="tablist" aria-label="预览格式">
              <For each={PREVIEW_FORMATS}>
                {(f, i) => (
                  <button
                    class={clsx('ap-tab', previewMode() === f.id && 'on')}
                    onClick={() => setPreviewMode(f.id)}
                    role="tab"
                    aria-selected={previewMode() === f.id}
                    tabIndex={previewMode() === f.id ? 0 : -1}
                    onKeyDown={(e) => {
                      if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return
                      e.preventDefault()
                      const dir = e.key === 'ArrowRight' ? 1 : -1
                      const next = PREVIEW_FORMATS[(i() + dir + PREVIEW_FORMATS.length) % PREVIEW_FORMATS.length]
                      setPreviewMode(next.id)
                      // 聚焦新选中的 tab（原生 roving tabindex 规范）
                      requestAnimationFrame(() => {
                        const tabs = document.querySelectorAll<HTMLElement>('.ap-tabs [role="tab"]')
                        tabs[(i() + dir + PREVIEW_FORMATS.length) % PREVIEW_FORMATS.length]?.focus()
                      })
                    }}
                  >
                    {f.label}
                  </button>
                )}
              </For>
            </div>

            {/* ── 内容 ── */}
            <div class="ap-body open" onClick={(e) => e.stopPropagation()}>
              <div
                class={clsx('ap-content', (artifactView() === 'code' || previewMode() === 'raw') && 'raw')}
                innerHTML={renderArtifact()}
              />
            </div>

            {/* ── 操作栏 ── */}
            <div class="ap-footer">
              <button class="ap-action" onClick={(e) => { e.stopPropagation(); copyPreview() }} title="复制" aria-label="复制">
                <Show when={copied()} fallback={
                  <svg viewBox="0 0 12 12">
                    <rect x="3" y="1.5" width="7.5" height="9" rx="1" stroke="currentColor" stroke-width="1.1" fill="none" />
                    <path d="M1.5 4v6.5a1 1 0 001 1H9" stroke="currentColor" stroke-width="1.1" fill="none" />
                  </svg>
                }>
                  <svg viewBox="0 0 12 12">
                    <polyline points="2,6.5 4.5,9 10,3.5" stroke="currentColor" stroke-width="1.4" fill="none" stroke-linecap="round" stroke-linejoin="round" />
                  </svg>
                </Show>
                <span>{copied() ? '已复制' : '复制'}</span>
              </button>
              <button class="ap-action" onClick={(e) => { e.stopPropagation(); loadTree() }} title="刷新" aria-label="刷新">
                <svg viewBox="0 0 12 12">
                  <path d="M1.5 6A4.5 4.5 0 016 1.5 4.5 4.5 0 0110.5 6" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" />
                  <polyline points="9,4.5 10.5,6 9,7.5" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
                <span>{treeLoading() ? '刷新中…' : '刷新'}</span>
              </button>
              <button class="ap-action" onClick={(e) => { e.stopPropagation(); toggleExpand() }} title="展开">
                <svg viewBox="0 0 12 12">
                  <polyline points="2,4.5 2,10 7.5,10" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
                  <polyline points="10,7.5 10,2 4.5,2" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
                <span>展开</span>
              </button>
              <button class="ap-action" onClick={(e) => { e.stopPropagation(); closePreview() }} title="关闭">
                <svg viewBox="0 0 12 12">
                  <line x1="3" y1="3" x2="9" y2="9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                  <line x1="9" y1="3" x2="3" y2="9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                </svg>
                <span>关闭</span>
              </button>
            </div>
          </Show>
        </div>

        {/* ── File Tree ── */}
        <div class="ft">
          <div class="ft-head">
            <span class="ft-root" title={rootPath()}>{rootPath() || '项目'}</span>
            <span class="ft-count">{fileCount()} 文件</span>
          </div>
          <Show when={treeLoading() && tree().length === 0}>
            <div class="ft-loading">加载项目树…</div>
          </Show>
          <Show when={treeError()}>
            <div class="ft-error">{treeError()}</div>
          </Show>
          <Show when={!treeLoading() && !treeError() && tree().length === 0}>
            <div class="ft-empty">项目为空</div>
          </Show>
          <FileTree
            nodes={tree()}
            onOpenFile={openPreview}
            activeFile={currentFile()?.path ?? null}
            activePath={activePath()}
            hasActive={activePath() !== null}
            onToggleDir={toggleDir}
            onActivate={activateNode}
            onMoveFocus={moveTreeFocus}
          />
          <Show when={fileLoading()}>
            <div class="ft-loading">读取文件…</div>
          </Show>
        </div>
        </Show>
      </div>
    </aside>
  )
}
