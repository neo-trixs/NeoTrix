import { For, Show } from 'solid-js'
import { clsx } from 'clsx'
import type { FileNode } from './FileTreeView'

export const PREVIEW_FORMATS = [
  { id: 'raw', label: 'Raw' },
  { id: 'rendered', label: 'Rendered' },
] as const

export type PreviewMode = (typeof PREVIEW_FORMATS)[number]['id']

const RUST_KEYWORDS = ['pub', 'struct', 'impl', 'fn', 'let', 'mut', 'const', 'Self', 'for', 'in', 'return', 'if', 'else', 'match', 'use', 'mod', 'trait', 'enum', 'type', 'where', 'as', 'async', 'await', 'move']

function escHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

/* 轻量 MD → 安全 HTML（仅用于 artifact 预览） */
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

function renderArtifact(currentFile: FileNode | null, artifactView: 'preview' | 'code', previewMode: PreviewMode): string {
  const text = currentFile?.content ?? '// 点击文件预览'
  if (artifactView === 'code' || previewMode === 'raw') {
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

interface ArtifactPaneProps {
  previewOpen: boolean
  currentFile: FileNode | null
  previewMode: PreviewMode
  artifactView: 'preview' | 'code'
  copied: boolean
  treeLoading: boolean
  onSetPreviewMode: (mode: PreviewMode) => void
  onSetArtifactView: (view: 'preview' | 'code') => void
  onToggleExpand: () => void
  onCopy: () => void
  onClose: () => void
  onRefresh: () => void
}

export function ArtifactPane(props: ArtifactPaneProps) {
  return (
    <div class={clsx('ap', !props.previewOpen && 'mini')} onClick={props.onToggleExpand}>
      <div class="ap-bar">
        <div class="ap-bar-left">
          <button
            class={clsx('ap-view-btn', props.artifactView === 'preview' && 'on')}
            onClick={(e) => { e.stopPropagation(); props.onSetArtifactView('preview') }}
            title="Preview"
            aria-label="预览视图"
          >
            <svg viewBox="0 0 14 14">
              <path d="M1.5 7s2.5-4.5 5.5-4.5S12.5 7 12.5 7s-2.5 4.5-5.5 4.5S1.5 7 1.5 7z" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" />
              <circle cx="7" cy="7" r="2" stroke="currentColor" stroke-width="1.2" fill="none" />
            </svg>
          </button>
          <button
            class={clsx('ap-view-btn', props.artifactView === 'code' && 'on')}
            onClick={(e) => { e.stopPropagation(); props.onSetArtifactView('code') }}
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
          onClick={(e) => { e.stopPropagation(); props.onToggleExpand() }}
          aria-expanded={props.previewOpen}
          title={props.previewOpen ? '折叠预览区' : '展开预览区'}
        >
          {props.currentFile?.name ?? '未选择文件'}
        </button>
      </div>

      <Show when={props.previewOpen}>
        <div class="ap-tabs" onClick={(e) => e.stopPropagation()} role="tablist" aria-label="预览格式">
          <For each={[...PREVIEW_FORMATS]}>
            {(f, i) => (
              <button
                class={clsx('ap-tab', props.previewMode === f.id && 'on')}
                onClick={() => props.onSetPreviewMode(f.id)}
                role="tab"
                aria-selected={props.previewMode === f.id}
                tabIndex={props.previewMode === f.id ? 0 : -1}
                onKeyDown={(e) => {
                  if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return
                  e.preventDefault()
                  const dir = e.key === 'ArrowRight' ? 1 : -1
                  const next = PREVIEW_FORMATS[(i() + dir + PREVIEW_FORMATS.length) % PREVIEW_FORMATS.length]
                  props.onSetPreviewMode(next.id)
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

        <div class="ap-body open" onClick={(e) => e.stopPropagation()}>
          <div
            class={clsx('ap-content', (props.artifactView === 'code' || props.previewMode === 'raw') && 'raw')}
            innerHTML={renderArtifact(props.currentFile, props.artifactView, props.previewMode)}
          />
        </div>

        <div class="ap-footer">
          <button class="ap-action" onClick={(e) => { e.stopPropagation(); props.onCopy() }} title="复制" aria-label="复制">
            <Show when={props.copied} fallback={
              <svg viewBox="0 0 12 12">
                <rect x="3" y="1.5" width="7.5" height="9" rx="1" stroke="currentColor" stroke-width="1.1" fill="none" />
                <path d="M1.5 4v6.5a1 1 0 001 1H9" stroke="currentColor" stroke-width="1.1" fill="none" />
              </svg>
            }>
              <svg viewBox="0 0 12 12">
                <polyline points="2,6.5 4.5,9 10,3.5" stroke="currentColor" stroke-width="1.4" fill="none" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </Show>
            <span>{props.copied ? '已复制' : '复制'}</span>
          </button>
          <button class="ap-action" onClick={(e) => { e.stopPropagation(); props.onRefresh() }} title="刷新" aria-label="刷新">
            <svg viewBox="0 0 12 12">
              <path d="M1.5 6A4.5 4.5 0 016 1.5 4.5 4.5 0 0110.5 6" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" />
              <polyline points="9,4.5 10.5,6 9,7.5" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <span>{props.treeLoading ? '刷新中…' : '刷新'}</span>
          </button>
          <button class="ap-action" onClick={(e) => { e.stopPropagation(); props.onToggleExpand() }} title="展开">
            <svg viewBox="0 0 12 12">
              <polyline points="2,4.5 2,10 7.5,10" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
              <polyline points="10,7.5 10,2 4.5,2" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <span>展开</span>
          </button>
          <button class="ap-action" onClick={(e) => { e.stopPropagation(); props.onClose() }} title="关闭">
            <svg viewBox="0 0 12 12">
              <line x1="3" y1="3" x2="9" y2="9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
              <line x1="9" y1="3" x2="3" y2="9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
            </svg>
            <span>关闭</span>
          </button>
        </div>
      </Show>
    </div>
  )
}
