/* ════════════════════════════════════════════
   components/settings/TagRow.tsx — 设置内标签行
   （对标 Obsidian 标签设置 + Linear label row）
   色点 + 名称(层级缩进) + 使用计数 + 色板快捷选色 + 自定义取色 + 重命名/删除
   ════════════════════════════════════════════ */
import { createSignal, Show, For } from 'solid-js'
import { normalizeTagName, TAG_PALETTE, tagDepth } from '../../stores/tags'

interface Props {
  name: string
  color: string
  count?: number
  onColorChange: (color: string) => void
  onRename: (next: string) => void
  onDelete: () => void
}

export function TagRow(props: Props) {
  const [editing, setEditing] = createSignal(false)
  const [editValue, setEditValue] = createSignal(props.name)
  const [pickerOpen, setPickerOpen] = createSignal(false)

  const confirmRename = () => {
    const next = normalizeTagName(editValue())
    if (next) props.onRename(next)
    else setEditValue(props.name)
    setEditing(false)
  }

  const indent = () => (tagDepth(props.name) - 1) * 16

  return (
    <li
      class="flex items-center gap-3 py-2.5 bg-white/30 hover:bg-white/55 transition-colors"
      style={{ 'padding-left': `${12 + indent()}px` }}
    >
      {/* 层级缩进导轨 */}
      <Show when={indent() > 0}>
        <span class="w-px h-6 bg-border-primary/60 flex-shrink-0 -ml-3" aria-hidden="true" />
      </Show>
      {/* 色点（点击展开快捷取色） */}
      <button
        class="w-5 h-5 rounded-full border border-white/80 shadow-sm flex-shrink-0 cursor-pointer transition-transform hover:scale-110"
        style={{ background: props.color }}
        onClick={() => setPickerOpen((v) => !v)}
        aria-label={`标签 ${props.name} 选色`}
        title="选择颜色"
      />
      <Show when={pickerOpen()}>
        <div class="flex items-center gap-1.5 px-2 py-1.5 rounded-lg bg-white/80 border border-border-primary shadow-sm">
          <For each={TAG_PALETTE}>
            {(c) => (
              <button
                class="w-4 h-4 rounded-full border border-white/70 transition-transform hover:scale-115"
                style={{ background: c, 'box-shadow': c === props.color ? '0 0 0 2px #fff, 0 0 0 4px rgba(240,145,58,0.6)' : undefined }}
                onClick={() => { props.onColorChange(c); setPickerOpen(false) }}
                aria-label={`设为 ${c}`}
              />
            )}
          </For>
          <label class="relative flex-shrink-0">
            <span class="w-4 h-4 rounded-full border border-dashed border-text-muted flex items-center justify-center text-[9px] text-text-muted cursor-pointer" title="自定义颜色">
              +
            </span>
            <input
              type="color"
              class="opacity-0 absolute inset-0 w-4 h-4 cursor-pointer"
              value={props.color}
              onInput={(e) => props.onColorChange(e.currentTarget.value)}
              aria-label="自定义颜色"
            />
          </label>
        </div>
      </Show>

      {/* 名称（编辑 / 展示） */}
      <Show
        when={editing()}
        fallback={
          <button
            class="flex-1 min-w-0 text-left group"
            onClick={() => setEditing(true)}
            title="重命名"
          >
            <span class="inline-flex items-center gap-1 text-[12px] text-text-primary truncate">
              <span class="font-mono text-text-muted">#</span>
              <span class="truncate">{props.name}</span>
            </span>
          </button>
        }
      >
        <span class="flex-1 min-w-0 flex items-center gap-1.5">
          <input
            class="flex-1 min-w-0 px-2 py-1 rounded-md bg-white/80 border border-nt-io-500/50 text-[12px] text-text-primary font-mono focus:outline-none focus:ring-1 focus:ring-nt-io-500"
            value={editValue()}
            onInput={(e) => setEditValue(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') confirmRename()
              else if (e.key === 'Escape') { setEditValue(props.name); setEditing(false) }
            }}
            autofocus
          />
          <button
            class="px-2 py-1 rounded-md bg-nt-io-500 text-white text-[10.5px] font-medium hover:bg-nt-io-600 transition-colors flex-shrink-0"
            onClick={confirmRename}
          >
            保存
          </button>
        </span>
      </Show>

      {/* 使用计数（Obsidian 风格徽章） */}
      <Show when={!editing() && (props.count ?? 0) > 0}>
        <span
          class="text-[9px] font-mono font-semibold px-1.5 py-0.5 rounded-full bg-black/4 text-text-muted tabular-nums flex-shrink-0"
          title={`${props.count} 个会话使用此标签`}
        >
          {props.count}
        </span>
      </Show>

      {/* 操作：重命名 / 删除 */}
      <div class="flex items-center gap-1 flex-shrink-0">
        <Show when={!editing()}>
          <button
            class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-black/5 transition-colors"
            onClick={() => { setEditValue(props.name); setEditing(true) }}
            aria-label={`重命名标签 ${props.name}`}
            title="重命名"
          >
            <svg viewBox="0 0 14 14" fill="none" class="w-3.5 h-3.5">
              <path d="M3 11l.5-2.5L9 3 11 5l-5.5 5.5L3 11z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
            </svg>
          </button>
          <button
            class="p-1.5 rounded-md text-text-muted hover:text-red-600 hover:bg-red-500/10 transition-colors"
            onClick={props.onDelete}
            aria-label={`删除标签 ${props.name}`}
            title="删除标签"
          >
            <svg viewBox="0 0 14 14" fill="none" class="w-3.5 h-3.5">
              <path d="M3.5 4.5h7v6a1 1 0 01-1 1h-5a1 1 0 01-1-1v-6z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
              <line x1="2.5" y1="4.5" x2="11.5" y2="4.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
            </svg>
          </button>
        </Show>
      </div>
    </li>
  )
}