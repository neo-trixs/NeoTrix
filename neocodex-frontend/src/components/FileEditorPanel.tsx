import { createSignal, Show } from 'solid-js'
import { readFile, writeFile } from '../api/system'

/* ════════════════════════════════════════════
   FileEditorPanel — 左栏 bot「文件内联编辑」
   对标 Claude Code 的 /edit：载入文件 → 编辑 → 写回（read_file + write_file）。
   仅做轻量内联编辑（无语法高亮），满足 bot 直接改文件的闭环。
   ════════════════════════════════════════════ */

interface Props {
  onClose: () => void
}

export function FileEditorPanel(props: Props) {
  const [path, setPath] = createSignal('')
  const [content, setContent] = createSignal('')
  const [loading, setLoading] = createSignal(false)
  const [saving, setSaving] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [dirty, setDirty] = createSignal(false)

  const load = async () => {
    const p = path().trim()
    if (!p) return
    setLoading(true)
    setError(null)
    try {
      setContent(await readFile(p))
      setDirty(false)
    } catch (e) {
      setError(`读取失败：${String(e)}`)
      setContent('')
    } finally {
      setLoading(false)
    }
  }

  const save = async () => {
    const p = path().trim()
    if (!p) return
    setSaving(true)
    setError(null)
    try {
      await writeFile(p, content())
      setDirty(false)
    } catch (e) {
      setError(`写入失败：${String(e)}`)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div class="file-editor rounded-xl border border-nt-io-500/20 bg-white/55 backdrop-blur-sm shadow-sm overflow-hidden">
      <div class="flex items-center gap-2 px-3 py-2">
        <span class="w-1.5 h-1.5 rounded-full bg-nt-io-500 flex-shrink-0" />
        <span class="text-12px font-semibold text-text-primary flex-shrink-0">文件内联编辑</span>
        <Show when={dirty()}>
          <span class="text-10px text-amber-600">● 未保存</span>
        </Show>
        <button
          class="ml-auto p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70 focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
          onClick={props.onClose}
          aria-label="关闭文件编辑"
          title="关闭"
        >
          <span class="text-11px">✕</span>
        </button>
      </div>
      <div class="px-3 pb-3 space-y-2">
        <div class="flex gap-2">
          <input
            class="flex-1 px-2 py-1 text-11px rounded border border-border-primary/50 bg-white/70 font-mono focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
            placeholder="文件路径，如 ./src/main.rs"
            value={path()}
            onInput={(e) => setPath(e.currentTarget.value)}
            onKeyDown={(e) => e.key === 'Enter' && load()}
          />
          <button
            class="px-2 py-1 rounded bg-nt-io-500/15 text-nt-io-600 hover:bg-nt-io-500/25 text-11px disabled:opacity-50"
            onClick={load}
            disabled={loading()}
          >
            载入
          </button>
        </div>
        <Show when={error()}>
          <div class="text-10px text-red-500">{error()}</div>
        </Show>
        <textarea
          class="w-full h-40 px-2 py-1 text-11px rounded border border-border-primary/50 bg-white/70 font-mono resize-y focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
          placeholder="载入文件后在此编辑…"
          value={content()}
          onInput={(e) => {
            setContent(e.currentTarget.value)
            setDirty(true)
          }}
        />
        <button
          class="w-full px-2 py-1 rounded bg-emerald-500/15 text-emerald-600 hover:bg-emerald-500/25 text-11px disabled:opacity-50"
          onClick={save}
          disabled={saving() || !dirty()}
        >
          写回文件
        </button>
      </div>
    </div>
  )
}
