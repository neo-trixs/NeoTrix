import { lazy, Show, Suspense } from 'solid-js'
import { clsx } from 'clsx'
import { ProjectView as ProjectViewPanel } from './ProjectView'
import type { RbTab } from './RightBarTabs'
import type { FileNode } from './FileTreeView'
import type { PreviewMode } from './ArtifactPane'
import { ArtifactPane } from './ArtifactPane'
import { FileTreeView } from './FileTreeView'
import { canvasStore } from '../stores/canvas'

const GlobeView = lazy(() => import('./GlobeView').then((m) => ({ default: m.GlobeView })))
const SmartCanvas = lazy(() => import('../canvas').then((m) => ({ default: m.SmartCanvas })))
const KanbanAgentBoard = lazy(() => import('./KanbanAgentBoard').then((m) => ({ default: m.KanbanAgentBoard })))
const OfficeFloor = lazy(() => import('./OfficeFloor').then((m) => ({ default: m.OfficeFloor })))

function TabFallback() {
  return (
    <div class="flex items-center justify-center h-full text-[11px] text-text-muted animate-pulse">
      加载中…
    </div>
  )
}

interface RightBarContentProps {
  rbTab: RbTab
  setRbTab: (tab: RbTab) => void
  previewOpen: boolean
  currentFile: FileNode | null
  previewMode: PreviewMode
  artifactView: 'preview' | 'code'
  copied: boolean
  tree: FileNode[]
  rootPath: string
  fileCount: number
  treeLoading: boolean
  treeError: string | null
  fileLoading: boolean
  activePath: string | null
  agentView: 'kanban' | 'floor'
  setAgentView: (view: 'kanban' | 'floor') => void
  setPreviewMode: (mode: PreviewMode) => void
  setArtifactView: (view: 'preview' | 'code') => void
  onOpenFile: (node: FileNode) => void
  onOpenPath: (path: string) => void
  onToggleDir: (node: FileNode) => void
  onActivateNode: (node: FileNode) => void
  onMoveTreeFocus: (dir: 1 | -1) => void
  onToggleExpand: () => void
  onCopy: () => void
  onClosePreview: () => void
  onRefresh: () => void
}

export function RightBarContent(props: RightBarContentProps) {
  return (
    <>
      <Show when={props.rbTab === 'map'}>
        <div class="rb-map">
          <Suspense fallback={<TabFallback />}>
            <GlobeView limit={3000} height={420} />
          </Suspense>
        </div>
      </Show>

      <Show when={props.rbTab === 'project'}>
        <div class="rb-project flex-1 min-h-0 overflow-hidden flex flex-col">
          <ProjectViewPanel open onClose={() => props.setRbTab('files')} onOpenFile={props.onOpenPath} />
        </div>
      </Show>

      <Show when={props.rbTab === 'canvas'}>
        <div class="rb-canvas flex-1 min-h-0 flex flex-col">
          <Suspense fallback={<TabFallback />}>
            <SmartCanvas
              nodes={() => canvasStore.nodes}
              spawn={(n) => canvasStore.spawn(n)}
              setCollapsed={(id, v) => canvasStore.setCollapsed(id, v)}
            />
          </Suspense>
        </div>
      </Show>

      <Show when={props.rbTab === 'agents'}>
        <div class="flex-1 min-h-0 flex flex-col overflow-hidden">
          <div class="flex items-center gap-1 px-3 pt-2 pb-1">
            <button
              class={clsx('text-[10px] px-2 py-0.5 rounded-md transition-colors', props.agentView === 'kanban' ? 'bg-white/10 text-white' : 'text-white/40 hover:text-white/60')}
              onClick={() => props.setAgentView('kanban')}
            >看板</button>
            <button
              class={clsx('text-[10px] px-2 py-0.5 rounded-md transition-colors', props.agentView === 'floor' ? 'bg-white/10 text-white' : 'text-white/40 hover:text-white/60')}
              onClick={() => props.setAgentView('floor')}
            >地图</button>
          </div>
          <Show when={props.agentView === 'kanban'}>
            <div class="flex-1 min-h-0 overflow-hidden">
              <Suspense fallback={<TabFallback />}>
                <KanbanAgentBoard
                  agents={() => [
                    { id: 'god-agent', name: 'GOD Agent', status: 'running', domain: 'orchestration', mode: 'chat', model: 'claude-sonnet-4-20250514', filesChanged: 0, toolCalls: 3, startedAt: new Date(), phi: 0.85 },
                    { id: 'coder-1', name: '编码员', status: 'idle', domain: 'code_generation', mode: 'code', model: 'gpt-4o', filesChanged: 0, toolCalls: 0, startedAt: new Date() },
                    { id: 'reviewer-1', name: '审查员', status: 'idle', domain: 'code_review', mode: 'work', model: 'claude-sonnet-4-20250514', filesChanged: 0, toolCalls: 0, startedAt: new Date() },
                  ]}
                  onSelectAgent={(id) => console.log('select agent:', id)}
                  onPauseAgent={(id) => console.log('pause agent:', id)}
                  onStopAgent={(id) => console.log('stop agent:', id)}
                  onMoveAgent={(id, target) => console.log('move agent:', id, target)}
                />
              </Suspense>
            </div>
          </Show>
          <Show when={props.agentView === 'floor'}>
            <div class="flex-1 min-h-0 p-2">
              <Suspense fallback={<TabFallback />}>
                <OfficeFloor
                  agents={() => [
                    { id: 'god-agent', name: 'GOD Agent', avatar: '🧠', status: 'running', specialty: 'orchestration', x: 0.5, y: 0.3 },
                    { id: 'coder-1', name: '编码员', avatar: '💻', status: 'idle', specialty: 'code_generation', x: 0.2, y: 0.6 },
                    { id: 'reviewer-1', name: '审查员', avatar: '🔍', status: 'idle', specialty: 'code_review', x: 0.8, y: 0.6 },
                  ]}
                  messages={() => []}
                  onSelectAgent={(id) => console.log('select agent:', id)}
                  useRealData={true}
                />
              </Suspense>
            </div>
          </Show>
        </div>
      </Show>

      <Show when={props.rbTab === 'files'}>
        <ArtifactPane
          previewOpen={props.previewOpen}
          currentFile={props.currentFile}
          previewMode={props.previewMode}
          artifactView={props.artifactView}
          copied={props.copied}
          treeLoading={props.treeLoading}
          onSetPreviewMode={props.setPreviewMode}
          onSetArtifactView={props.setArtifactView}
          onToggleExpand={props.onToggleExpand}
          onCopy={props.onCopy}
          onClose={props.onClosePreview}
          onRefresh={props.onRefresh}
        />
        <FileTreeView
          tree={props.tree}
          rootPath={props.rootPath}
          fileCount={props.fileCount}
          treeLoading={props.treeLoading}
          treeError={props.treeError}
          fileLoading={props.fileLoading}
          currentFile={props.currentFile}
          activePath={props.activePath}
          onOpenFile={props.onOpenFile}
          onToggleDir={props.onToggleDir}
          onActivate={props.onActivateNode}
          onMoveFocus={props.onMoveTreeFocus}
        />
      </Show>
    </>
  )
}
