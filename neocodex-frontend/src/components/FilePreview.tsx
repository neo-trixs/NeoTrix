import { createSignal, Show, For } from 'solid-js'
import { FileText, Image as ImageIcon, FileCode2, Table as TableIcon, File, X, ChevronDown, ChevronUp, Highlighter } from 'lucide-solid'
import type { NeoCodexAttachmentDto } from '../stores/chat'
import { AnnotatedImage, type Annotation } from './AnnotatedImage'
import { clsx } from 'clsx'

interface Props {
  attachment: NeoCodexAttachmentDto
  onRemove?: () => void
  /** Called with the serialized annotation hint (empty string = none). */
  onAnnotate?: (hint: string) => void
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

function isImage(mime: string): boolean {
  return mime.startsWith('image/')
}

function isCode(mime: string, name: string): boolean {
  const codeExts = ['rs', 'ts', 'tsx', 'js', 'jsx', 'py', 'go', 'java', 'c', 'cpp', 'h', 'rb', 'sh', 'json', 'yaml', 'yml', 'toml', 'md', 'sql', 'html', 'css', 'vue', 'svelte']
  const ext = name.split('.').pop()?.toLowerCase() || ''
  return codeExts.includes(ext) || mime.startsWith('text/') || mime.includes('json') || mime.includes('javascript')
}

function isTable(mime: string, name: string): boolean {
  return mime.includes('csv') || mime.includes('excel') || name.toLowerCase().endsWith('.csv')
}

function toDataUrl(mime: string, data: string): string {
  // data may already be a full data URL (e.g. from capture); if raw base64, wrap it.
  if (data.startsWith('data:')) return data
  return `data:${mime};base64,${data}`
}

export function FilePreview(props: Props) {
  const [expanded, setExpanded] = createSignal(false)
  const [annotating, setAnnotating] = createSignal(false)
  const att = () => props.attachment
  const dataUrl = () => (att().data ? toDataUrl(att().mime_type, att().data!) : null)

  const handleConfirm = (annotations: Annotation[]) => {
    if (annotations.length === 0) {
      props.onAnnotate?.('')
    } else {
      const parts = annotations.map(a => {
        if (a.kind === 'box') {
          return `矩形${a.id}(x:${(a.x * 100).toFixed(1)}%,y:${(a.y * 100).toFixed(1)}%,w:${(a.w * 100).toFixed(1)}%,h:${(a.h * 100).toFixed(1)}%)`
        }
        return `箭头${a.id}(from:${(a.x * 100).toFixed(1)}%,${(a.y * 100).toFixed(1)}%→to:${((a.ex ?? 0) * 100).toFixed(1)}%,${((a.ey ?? 0) * 100).toFixed(1)}%)`
      })
      props.onAnnotate?.(`[图片标注 ${att().name}] ${parts.join('; ')}`)
    }
    setAnnotating(false)
  }

  const renderBody = () => {
    const a = att()
    const url = dataUrl()

    // Image: inline thumbnail, with annotation overlay mode
    if (isImage(a.mime_type) && url) {
      if (annotating()) {
        return (
          <AnnotatedImage
            imageUrl={url}
            imageName={a.name}
            onConfirm={handleConfirm}
            onCancel={() => setAnnotating(false)}
          />
        )
      }
      return (
        <img
          src={url}
          alt={a.name}
          class="max-h-72 max-w-full rounded-md object-contain bg-bg-primary/40"
        />
      )
    }

    // PDF: embed via iframe (data URL works in WebKit/Tauri)
    if (a.mime_type.includes('pdf') && url) {
      return (
        <div class="rounded-md overflow-hidden bg-bg-primary/40 border border-border-primary">
          <iframe
            src={url}
            title={a.name}
            class="w-full h-72"
            sandbox=""
          />
        </div>
      )
    }

    // Code / text: raw view with toggle
    if (isCode(a.mime_type, a.name) && a.data) {
      const text = (() => {
        try {
          return atob(a.data!)
        } catch {
          return a.data!
        }
      })()
      return (
        <div class="rounded-md bg-bg-primary/60 border border-border-primary overflow-hidden">
          <button
            class="w-full flex items-center gap-2 px-3 py-2 text-xs text-text-muted hover:text-text-primary hover:bg-bg-tertiary/50"
            onClick={() => setExpanded(!expanded())}
            aria-expanded={expanded()}
          >
            {expanded() ? <ChevronUp class="w-3.5 h-3.5" /> : <ChevronDown class="w-3.5 h-3.5" />}
            <span class="font-mono">查看内容 ({formatSize(text.length)})</span>
          </button>
          <Show when={expanded()}>
            <pre class="max-h-72 overflow-y-auto px-3 py-2 text-xs font-mono text-text-secondary whitespace-pre-wrap break-words">
              {text.length > 12000 ? text.slice(0, 12000) + '\n... (已截断)' : text}
            </pre>
          </Show>
        </div>
      )
    }

    // CSV: render as a simple table
    if (isTable(a.mime_type, a.name) && a.data) {
      const rows = (() => {
        try {
          const text = atob(a.data!)
          return text.split('\n').filter(l => l.trim()).slice(0, 30).map(l => l.split(','))
        } catch {
          return []
        }
      })()
      return (
        <Show when={rows.length > 0} fallback={<FallbackName name={a.name} />}>
          <div class="rounded-md bg-bg-primary/60 border border-border-primary overflow-hidden max-h-72 overflow-y-auto">
            <table class="w-full text-xs font-mono">
              <For each={rows}>
                {(row, i) => (
                  <tr class={clsx(i() === 0 ? 'bg-bg-secondary/80 font-semibold' : 'border-t border-border-primary/50')}>
                    <For each={row}>
                      {(cell) => <td class="px-2 py-1 text-text-secondary truncate max-w-[200px]">{cell.trim()}</td>}
                    </For>
                  </tr>
                )}
              </For>
            </table>
          </div>
        </Show>
      )
    }

    return <FallbackName name={a.name} />
  }

  return (
    <div class="rounded-lg border border-border-primary bg-bg-secondary/60 overflow-hidden">
      <div class="flex items-center gap-2 px-3 py-2">
        {fileIcon(att())}
        <div class="flex-1 min-w-0">
          <div class="text-sm text-text-primary truncate">{att().name}</div>
          <div class="text-xs text-text-muted">{formatSize(att().size)}</div>
        </div>
        <Show when={isImage(att().mime_type) && att().data && props.onAnnotate}>
          <button
            class={clsx(
              'p-2 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors',
              annotating() && 'bg-nt-core-500/20 text-nt-core-700'
            )}
            onClick={() => setAnnotating(!annotating())}
            aria-label="标注图片"
            title="在图片上标注区域"
          >
            <Highlighter class="w-4 h-4" />
          </button>
        </Show>
        <Show when={props.onRemove}>
          <button
            class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
            onClick={props.onRemove}
            aria-label="移除附件"
          >
            <X class="w-4 h-4" />
          </button>
        </Show>
      </div>
      <div class="px-3 pb-3">{renderBody()}</div>
    </div>
  )
}

function fileIcon(a: NeoCodexAttachmentDto) {
  const cls = 'w-8 h-8 p-2 rounded-md flex-shrink-0'
  if (isImage(a.mime_type)) {
    return <div class={`${cls} bg-nt-io-500/10 text-nt-io-600`}><ImageIcon class="w-5 h-5" /></div>
  }
  if (a.mime_type.includes('pdf')) {
    return <div class={`${cls} bg-red-500/10 text-red-500`}><FileText class="w-5 h-5" /></div>
  }
  if (isCode(a.mime_type, a.name)) {
    return <div class={`${cls} bg-nt-core-500/10 text-nt-core-600`}><FileCode2 class="w-5 h-5" /></div>
  }
  if (isTable(a.mime_type, a.name)) {
    return <div class={`${cls} bg-emerald-500/10 text-emerald-600`}><TableIcon class="w-5 h-5" /></div>
  }
  return <div class={`${cls} bg-bg-tertiary/50 text-text-muted`}><File class="w-5 h-5" /></div>
}

function FallbackName(props: { name: string }) {
  return <div class="text-xs text-text-muted py-1">{props.name} — 无可用预览</div>
}
