import { createSignal, onMount } from 'solid-js'
import { system, errText } from '../api'
import { call as domainCall } from '../api/domain'
import type { ProjectView } from '../api/types'
import { clsx } from 'clsx'
import { startCanvasBridge, initCanvasEvolution } from '../canvas'
import { initCanvasPersistence } from '../stores/canvas'
import { RightBarTabs } from './RightBarTabs'
import { RightBarContent } from './RightBarContent'
import { toFileNode, collectOpenPaths, applyOpen } from './FileTreeView'
import type { FileNode } from './FileTreeView'
import type { RbTab } from './RightBarTabs'
import type { PreviewMode } from './ArtifactPane'

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
  const [agentView, setAgentView] = createSignal<'kanban' | 'floor'>('kanban')

  let treeReqSeq = 0
  const loadTree = async () => {
    const seq = ++treeReqSeq
    const prevOpen = collectOpenPaths(tree())
    setTreeLoading(true)
    setTreeError(null)
    try {
      const pv = await domainCall<ProjectView>('file', 'project_tree')
      if (seq !== treeReqSeq) return
      const nodes = pv.tree.map(toFileNode)
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

  onMount(loadTree)
  startCanvasBridge()
  initCanvasPersistence()
  initCanvasEvolution()

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

  const openPath = (path: string) => {
    const name = path.split('/').pop() ?? path
    openPreview({ name, path, type: 'file' })
  }

  const activateNode = (node: FileNode) => setActivePath(node.path ?? node.name)
  const moveTreeFocus = (dir: 1 | -1) => {
    const items = Array.from(document.querySelectorAll<HTMLElement>('.ft [role="treeitem"]'))
    const idx = items.findIndex((el) => el === document.activeElement)
    const target = idx === -1 ? (dir === 1 ? items[0] : items[items.length - 1]) : items[idx + dir]
    target?.focus()
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
        <RightBarTabs rbTab={rbTab()} onSetTab={setRbTab} />

        <RightBarContent
          rbTab={rbTab()}
          setRbTab={setRbTab}
          previewOpen={previewOpen()}
          currentFile={currentFile()}
          previewMode={previewMode()}
          artifactView={artifactView()}
          copied={copied()}
          tree={tree()}
          rootPath={rootPath()}
          fileCount={fileCount()}
          treeLoading={treeLoading()}
          treeError={treeError()}
          fileLoading={fileLoading()}
          activePath={activePath()}
          agentView={agentView()}
          setAgentView={setAgentView}
          setPreviewMode={setPreviewMode}
          setArtifactView={setArtifactView}
          onOpenFile={openPreview}
          onOpenPath={openPath}
          onToggleDir={toggleDir}
          onActivateNode={activateNode}
          onMoveTreeFocus={moveTreeFocus}
          onToggleExpand={toggleExpand}
          onCopy={copyPreview}
          onClosePreview={closePreview}
          onRefresh={loadTree}
        />
      </div>
    </aside>
  )
}
