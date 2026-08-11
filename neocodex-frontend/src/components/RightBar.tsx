import { createSignal, createEffect, For, Show } from 'solid-js'
import { neocodex, system } from '../api'
import type { ProjectTreeItem, ProjectView } from '../api/types'
import { clsx } from 'clsx'
import { GlobeView } from './GlobeView'

/* ════════════════════════════════════════════
   RightBar — 右栏（设计 v2，已接线后端）
   标签切换：文件（Artifact Pane + 文件树） / 地图（shanhai 3D 地球）
   上部：Artifact Pane（预览/代码切换 + 格式 tabs + 内容）
   下部：文件树（真实项目树 ← neocodex_project_tree）
   交互：auto-hide（hover/右侧边缘展开）或 collapsed 固定
   ════════════════════════════════════════════ */

type RbTab = 'files' | 'map'

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
  { id: 'wechat', label: 'WeChat' },
  { id: 'zhihu', label: 'Zhihu' },
  { id: 'juejin', label: 'Juejin' },
  { id: 'web', label: 'Web' },
] as const

type PreviewMode = (typeof PREVIEW_FORMATS)[number]['id']

const RUST_KEYWORDS = ['pub', 'struct', 'impl', 'fn', 'let', 'mut', 'const', 'Self', 'for', 'in', 'return', 'if', 'else', 'match', 'use', 'mod', 'trait', 'enum', 'type', 'where', 'as', 'async', 'await', 'move']

function escHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

/* 轻量 MD → 安全 HTML（仅用于 artifact 预览） */
function renderMd(text: string): string {
  return text
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/^#### (.+)$/gm, '<h4>$1</h4>')
    .replace(/^### (.+)$/gm, '<h3>$1</h3>')
    .replace(/^## (.+)$/gm, '<h2>$1</h2>')
    .replace(/^# (.+)$/gm, '<h1>$1</h1>')
    .replace(/```(\w*)\n([\s\S]*?)```/g, (m, lang, code) => `<pre><code>${escHtml(code)}</code></pre>`)
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/^- (.+)$/gm, '<li>$1</li>')
    .replace(/(<li>.*<\/li>\n?)+/g, '<ul>$&</ul>')
    .replace(/^> (.+)$/gm, '<blockquote>$1</blockquote>')
    .replace(/\n\n/g, '</p><p>')
    .replace(/^(?!<[hpl]|<[uo]l|<pre|<bl|$)/gm, '<p>')
    .replace(/<\/p>\s*<p>/g, '</p><p>')
}

function FileTree(props: {
  nodes: FileNode[]
  depth?: number
  onOpenFile: (node: FileNode) => void
  activeFile: string | null
  onToggleDir: (node: FileNode) => void
}) {
  return (
    <For each={props.nodes}>
      {(n) => (
        <>
          {n.type === 'dir' ? (
            <>
              <div
                class="ft-item"
                style={{ 'padding-left': `${(props.depth ?? 0) * 14 + 4}px` }}
                onClick={() => props.onToggleDir(n)}
              >
                <svg class={clsx('chev', n.open && 'open')} viewBox="0 0 9 9">
                  <line x1="3" y1="2.5" x2="6" y2="4.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                  <line x1="3" y1="6.5" x2="6" y2="4.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                </svg>
                <svg class="fic" viewBox="0 0 14 14">
                  <path d="M1.5 4.5h3.5l1-1.5h6a1 1 0 011 1v6a1 1 0 01-1 1h-10a1 1 0 01-1-1v-5.5z" stroke="currentColor" stroke-width="1" fill="none" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
                {n.name}
              </div>
              <div class={clsx('ft-children', n.open && 'open')}>
                <Show when={n.open}>
                  <FileTree
                    nodes={n.children ?? []}
                    depth={(props.depth ?? 0) + 1}
                    onOpenFile={props.onOpenFile}
                    activeFile={props.activeFile}
                    onToggleDir={props.onToggleDir}
                  />
                </Show>
              </div>
            </>
          ) : (
            <div
              class={clsx('ft-item ft-file', props.activeFile === n.name && 'ft-active')}
              style={{ 'padding-left': `${(props.depth ?? 0) * 14 + 4}px` }}
              onClick={() => props.onOpenFile(n)}
            >
              <svg class="fic" viewBox="0 0 14 14">
                <path d="M2 1.5h10v11H2z" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linejoin="round" />
                <line x1="4.5" y1="4.5" x2="9.5" y2="4.5" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
              </svg>
              {n.name}
            </div>
          )}
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

  /* 加载真实项目树（neocodex_project_tree） */
  const loadTree = async () => {
    setTreeLoading(true)
    setTreeError(null)
    try {
      const pv = await neocodex.projectTree()
      setTree(pv.tree.map(toFileNode))
      setRootPath(pv.root)
      setFileCount(pv.file_count)
    } catch (e) {
      setTreeError(String(e))
    } finally {
      setTreeLoading(false)
    }
  }

  createEffect(() => {
    loadTree()
  })

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
        node.content = `// 读取失败: ${e}`
        setCurrentFile({ ...node })
      } finally {
        setFileLoading(false)
      }
    }
  }

  const toggleDir = (node: FileNode) => {
    node.open = !node.open
    setTree([...tree()])
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
      let html = escHtml(text)
      RUST_KEYWORDS.forEach((k) => {
        html = html.replace(new RegExp(`\\b${k}\\b`, 'g'), `<span class="kw">${k}</span>`)
      })
      html = html.replace(/\/\/.*/g, (m) => `<span class="cm">${escHtml(m)}</span>`)
      html = html.replace(/\b[A-Z]\w+(?=\s*(?:[({<]|::))/g, (m) => `<span class="fn">${m}</span>`)
      return html
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
        {/* ── 标签切换：文件 / 地图 ── */}
        <div class="rb-tabs" role="tablist" aria-label="右栏视图">
          <button
            class={clsx('rb-tab', rbTab() === 'files' && 'on')}
            onClick={() => setRbTab('files')}
            role="tab"
            aria-selected={rbTab() === 'files'}
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
          >
            <svg viewBox="0 0 14 14" class="rb-tab-ic">
              <circle cx="7" cy="7" r="5.5" stroke="currentColor" stroke-width="1.1" fill="none" />
              <ellipse cx="7" cy="7" rx="2.6" ry="5.5" stroke="currentColor" stroke-width="1.1" fill="none" />
              <line x1="1.5" y1="7" x2="12.5" y2="7" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
            </svg>
            地图
          </button>
        </div>

        {/* ── 地图视图：shanhai 3D 地球 ── */}
        <Show when={rbTab() === 'map'}>
          <div class="rb-map">
            <GlobeView limit={3000} height={420} />
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
              <button class="ap-action" onClick={(e) => { e.stopPropagation(); copyPreview() }} title="复制">
                <svg viewBox="0 0 12 12">
                  <rect x="3" y="1.5" width="7.5" height="9" rx="1" stroke="currentColor" stroke-width="1.1" fill="none" />
                  <path d="M1.5 4v6.5a1 1 0 001 1H9" stroke="currentColor" stroke-width="1.1" fill="none" />
                </svg>
                <span>复制</span>
              </button>
              <button class="ap-action" onClick={(e) => e.stopPropagation()} title="刷新">
                <svg viewBox="0 0 12 12">
                  <path d="M1.5 6A4.5 4.5 0 016 1.5 4.5 4.5 0 0110.5 6" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" />
                  <polyline points="9,4.5 10.5,6 9,7.5" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
                <span>刷新</span>
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
            activeFile={currentFile()?.name ?? null}
            onToggleDir={toggleDir}
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
