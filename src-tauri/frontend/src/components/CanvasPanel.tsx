import { createSignal, For, Show, onMount } from 'solid-js'
import { clsx } from 'clsx'
import { LayoutDashboard, ExternalLink, Copy, RefreshCw, Maximize2, Minimize2, Code2, Eye } from 'lucide-solid'

/* ════════════════════════════════════════════════════════════
   CanvasPanel — Cursor 3 风格交互式 Artifacts
   
   融合产品模式：
   - Cursor 3: Canvas (agent 生成交互式 artifacts)
   - Cursor 3.7: Canvas Design Mode + Clickable Prompt Buttons
   - Claude: Artifacts (双向同步)
   
   NeoTrix 特有：
   - E8 意识状态与 artifact 质量关联
   - Constellation 成熟度标记
   ════════════════════════════════════════════════════════════ */

export type CanvasType = 'dashboard' | 'report' | 'tool' | 'code' | 'unknown'

export interface CanvasArtifact {
  id: string
  title: string
  type: CanvasType
  content: string  // HTML/React content
  createdAt: Date
  agentId?: string
  promptButtons?: { label: string; prompt: string }[]
  phi?: number
}

export interface CanvasPanelProps {
  artifacts: () => CanvasArtifact[]
  activeArtifactId: () => string | null
  onSelectArtifact: (id: string) => void
  onRefresh: (id: string) => void
  onCopyContent: (id: string) => void
  onOpenExternal: (id: string) => void
  onPromptButtonClick: (artifactId: string, prompt: string) => void
}

const TYPE_CONFIG: Record<CanvasType, { icon: any; color: string; label: string }> = {
  dashboard: { icon: LayoutDashboard, color: 'text-blue-500', label: '仪表盘' },
  report:    { icon: Eye,             color: 'text-emerald-500', label: '报告' },
  tool:      { icon: Code2,           color: 'text-amber-500', label: '工具' },
  code:      { icon: Code2,           color: 'text-purple-500', label: '代码' },
  unknown:   { icon: LayoutDashboard, color: 'text-zinc-500', label: 'Artifact' },
}

export function CanvasPanel(props: CanvasPanelProps) {
  const [viewMode, setViewMode] = createSignal<'preview' | 'code'>('preview')
  const [fullscreen, setFullscreen] = createSignal(false)

  const activeArtifact = () =>
    props.artifacts().find(a => a.id === props.activeArtifactId()) ?? null

  return (
    <div class={clsx(
      'flex flex-col h-full',
      fullscreen() && 'fixed inset-0 z-50 bg-white'
    )}>
      {/* Header */}
      <div class="flex items-center justify-between px-4 py-3 border-b border-border-primary/40">
        <div class="flex items-center gap-2">
          <LayoutDashboard class="w-4 h-4 text-text-muted" />
          <span class="text-[13px] font-medium text-text-primary">Canvas</span>
          <span class="text-[11px] text-text-muted bg-white/60 px-1.5 py-0.5 rounded-full">
            {props.artifacts().length} 个 Artifact
          </span>
        </div>
        <div class="flex items-center gap-1.5">
          <button
            class={clsx(
              'p-1.5 rounded-lg text-[10px] transition-colors',
              viewMode() === 'preview'
                ? 'bg-nt-io-500/10 text-nt-io-600'
                : 'text-text-muted hover:text-text-primary hover:bg-white/60'
            )}
            onClick={() => setViewMode('preview')}
            title="预览"
          >
            <Eye class="w-3.5 h-3.5" />
          </button>
          <button
            class={clsx(
              'p-1.5 rounded-lg text-[10px] transition-colors',
              viewMode() === 'code'
                ? 'bg-nt-io-500/10 text-nt-io-600'
                : 'text-text-muted hover:text-text-primary hover:bg-white/60'
            )}
            onClick={() => setViewMode('code')}
            title="源码"
          >
            <Code2 class="w-3.5 h-3.5" />
          </button>
          <button
            class="p-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors"
            onClick={() => setFullscreen(!fullscreen())}
            title={fullscreen() ? '退出全屏' : '全屏'}
          >
            {fullscreen() ? <Minimize2 class="w-3.5 h-3.5" /> : <Maximize2 class="w-3.5 h-3.5" />}
          </button>
        </div>
      </div>

      {/* Artifact list + preview */}
      <div class="flex-1 flex overflow-hidden">
        {/* Artifact list (left) */}
        <div class="w-[200px] border-r border-border-primary/20 overflow-y-auto">
          <Show
            when={props.artifacts().length > 0}
            fallback={
              <div class="flex flex-col items-center justify-center h-32 text-text-muted text-[11px]">
                <LayoutDashboard class="w-5 h-5 mb-1 opacity-30" />
                <span>暂无 Artifacts</span>
              </div>
            }
          >
            <For each={props.artifacts()}>
              {(artifact) => {
                const active = () => props.activeArtifactId() === artifact.id
                const config = TYPE_CONFIG[artifact.type]
                const TypeIcon = config.icon
                return (
                  <button
                    class={clsx(
                      'w-full text-left px-3 py-2.5 border-b border-border-primary/10 transition-colors',
                      active()
                        ? 'bg-nt-io-500/5 border-l-2 border-l-nt-io-500'
                        : 'hover:bg-white/40 border-l-2 border-l-transparent'
                    )}
                    onClick={() => props.onSelectArtifact(artifact.id)}
                  >
                    <div class="flex items-center gap-1.5 mb-0.5">
                      <TypeIcon class={clsx('w-3 h-3', config.color)} />
                      <span class="text-[11px] font-medium text-text-primary truncate flex-1">
                        {artifact.title}
                      </span>
                    </div>
                    <div class="flex items-center gap-2 text-[9px] text-text-muted">
                      <span>{config.label}</span>
                      <span>{formatRelativeTime(artifact.createdAt)}</span>
                      <Show when={artifact.phi !== undefined}>
                        <span class="ml-auto">φ {(artifact.phi! * 100).toFixed(0)}%</span>
                      </Show>
                    </div>
                  </button>
                )
              }}
            </For>
          </Show>
        </div>

        {/* Preview area (right) */}
        <div class="flex-1 overflow-hidden">
          <Show
            when={activeArtifact()}
            fallback={
              <div class="flex flex-col items-center justify-center h-full text-text-muted">
                <LayoutDashboard class="w-8 h-8 mb-2 opacity-20" />
                <span class="text-[12px]">选择一个 Artifact 查看</span>
              </div>
            }
          >
            {(artifact) => (
              <div class="flex flex-col h-full">
                {/* Artifact toolbar */}
                <div class="flex items-center justify-between px-3 py-2 border-b border-border-primary/20 bg-white/30">
                  <div class="flex items-center gap-2">
                    <span class="text-[11px] font-medium text-text-primary">{artifact().title}</span>
                    <span class={clsx(
                      'text-[9px] px-1.5 py-0.5 rounded-full',
                      TYPE_CONFIG[artifact().type].color.replace('text-', 'bg-').replace('500', '100'),
                      TYPE_CONFIG[artifact().type].color
                    )}>
                      {TYPE_CONFIG[artifact().type].label}
                    </span>
                  </div>
                  <div class="flex items-center gap-1">
                    <button
                      class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors"
                      onClick={() => props.onRefresh(artifact().id)}
                      title="刷新"
                    >
                      <RefreshCw class="w-3 h-3" />
                    </button>
                    <button
                      class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors"
                      onClick={() => props.onCopyContent(artifact().id)}
                      title="复制内容"
                    >
                      <Copy class="w-3 h-3" />
                    </button>
                    <button
                      class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors"
                      onClick={() => props.onOpenExternal(artifact().id)}
                      title="在浏览器中打开"
                    >
                      <ExternalLink class="w-3 h-3" />
                    </button>
                  </div>
                </div>

                {/* Content */}
                <div class="flex-1 overflow-auto">
                  <Show
                    when={viewMode() === 'preview'}
                    fallback={
                      <pre class="p-4 text-[11px] font-mono text-text-primary bg-zinc-50 h-full overflow-auto">
                        <code>{artifact().content}</code>
                      </pre>
                    }
                  >
                    <div
                      class="w-full h-full"
                      innerHTML={artifact().content}
                    />
                  </Show>
                </div>

                {/* Prompt buttons (Cursor 3.7 feature) */}
                <Show when={artifact().promptButtons && artifact().promptButtons!.length > 0}>
                  <div class="px-3 py-2 border-t border-border-primary/20 bg-white/30">
                    <div class="flex flex-wrap gap-1.5">
                      <For each={artifact().promptButtons!}>
                        {(btn) => (
                          <button
                            class="px-2.5 py-1 rounded-lg text-[10px] font-medium bg-violet-50 text-violet-600 border border-violet-200/50 hover:bg-violet-100 transition-colors"
                            onClick={() => props.onPromptButtonClick(artifact().id, btn.prompt)}
                          >
                            {btn.label}
                          </button>
                        )}
                      </For>
                    </div>
                  </div>
                </Show>
              </div>
            )}
          </Show>
        </div>
      </div>
    </div>
  )
}

function formatRelativeTime(date: Date): string {
  const diff = Date.now() - date.getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes}m`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h`
  return `${Math.floor(hours / 24)}d`
}
