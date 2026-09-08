/**
 * ArtifactPreview — 对话内联制品预览
 * 
 * 在聊天区域显示最近产生的制品（代码块、图表、图片等），
 * 点击可展开到完整画板。
 */
import { createSignal, Show, For, createMemo } from 'solid-js'
import { clsx } from 'clsx'
import { canvasStore } from '../stores/canvas'
import type { CanvasNode } from '../canvas/types'

interface Props {
  maxItems?: number
  onExpand?: () => void
}

/** 制品类型 → 图标/颜色映射 */
const ARTIFACT_META: Record<string, { icon: string; color: string; label: string }> = {
  code: { icon: '💻', color: 'bg-blue-500/10 text-blue-600 border-blue-200', label: '代码' },
  diff: { icon: '🔄', color: 'bg-amber-500/10 text-amber-600 border-amber-200', label: '差异' },
  json: { icon: '📊', color: 'bg-purple-500/10 text-purple-600 border-purple-200', label: 'JSON' },
  table: { icon: '📋', color: 'bg-emerald-500/10 text-emerald-600 border-emerald-200', label: '表格' },
  mermaid: { icon: '🔗', color: 'bg-cyan-500/10 text-cyan-600 border-cyan-200', label: '流程图' },
  image: { icon: '🖼️', color: 'bg-pink-500/10 text-pink-600 border-pink-200', label: '图片' },
  html: { icon: '🌐', color: 'bg-orange-500/10 text-orange-600 border-orange-200', label: 'HTML' },
  markdown: { icon: '📝', color: 'bg-slate-500/10 text-slate-600 border-slate-200', label: '文档' },
  webpage: { icon: '🔗', color: 'bg-indigo-500/10 text-indigo-600 border-indigo-200', label: '网页' },
  kpi: { icon: '📈', color: 'bg-teal-500/10 text-teal-600 border-teal-200', label: '指标' },
  video: { icon: '🎬', color: 'bg-red-500/10 text-red-600 border-red-200', label: '视频' },
}

const DEFAULT_META = { icon: '📦', color: 'bg-zinc-500/10 text-zinc-600 border-zinc-200', label: '制品' }

function truncateText(text: string, maxLen: number): string {
  if (text.length <= maxLen) return text
  return text.slice(0, maxLen) + '…'
}

function ArtifactCard(props: { node: CanvasNode; onExpand?: () => void }) {
  const meta = ARTIFACT_META[props.node.kind] ?? DEFAULT_META
  const preview = createMemo(() => {
    const d = props.node.data
    if (typeof d === 'string') return truncateText(d, 120)
    if (typeof d === 'object' && d !== null) {
      if ('src' in d) return `[${props.node.kind}] ${(d as any).src?.slice(0, 60) ?? ''}`
      return truncateText(JSON.stringify(d), 120)
    }
    return String(d)
  })

  return (
    <div
      class={clsx(
        'flex-shrink-0 w-48 p-2.5 rounded-xl border cursor-pointer transition-all',
        'hover:shadow-md hover:scale-[1.02]',
        meta.color
      )}
      onClick={() => props.onExpand?.()}
      title="点击展开到画板"
    >
      <div class="flex items-center gap-1.5 mb-1">
        <span class="text-sm">{meta.icon}</span>
        <span class="text-[11px] font-medium truncate">{props.node.title ?? meta.label}</span>
      </div>
      <div class="text-[10px] opacity-70 leading-relaxed font-mono">
        {preview()}
      </div>
    </div>
  )
}

export function ArtifactPreview(props: Props) {
  const maxItems = props.maxItems ?? 6

  // 获取最近的制品节点（按 salience 降序）
  const recentArtifacts = createMemo(() => {
    return canvasStore.nodes
      .filter(n => n.source === 'assistant' || n.source === 'tool' || n.source === 'attachment')
      .sort((a, b) => (b.salience ?? 0) - (a.salience ?? 0))
      .slice(0, maxItems)
  })

  const hasArtifacts = createMemo(() => recentArtifacts().length > 0)

  return (
    <Show when={hasArtifacts()}>
      <div class="artifact-preview-section mb-3">
        <div class="flex items-center gap-2 mb-2 px-1">
          <span class="text-[11px] font-medium text-text-secondary">对话制品</span>
          <span class="text-[10px] text-text-muted bg-bg-tertiary/50 px-1.5 py-0.5 rounded">
            {recentArtifacts().length}
          </span>
          <button
            class="ml-auto text-[10px] text-nt-io-600 hover:text-nt-io-700 transition-colors"
            onClick={() => props.onExpand?.()}
          >
            展开画板 →
          </button>
        </div>
        <div class="flex gap-2 overflow-x-auto pb-2 scrollbar-thin">
          <For each={recentArtifacts()}>
            {(node) => (
              <ArtifactCard node={node} onExpand={props.onExpand} />
            )}
          </For>
        </div>
      </div>
    </Show>
  )
}
