import { For, Show, createMemo } from 'solid-js'
import { clsx } from 'clsx'

/* ════════════════════════════════════════════
   SlashMenu — 斜杠 / 命令菜单（对标 Claude Code）
   输入以 `/cmd` 开头时浮层展示，键盘上下导航 + Enter 执行。
   命令定义由父组件提供（依赖 chatStore），本组件只负责渲染与过滤。
   ════════════════════════════════════════════ */

export interface SlashCommandDef {
  id: string
  /** 主显示名，如 "清除会话" */
  label: string
  /** 一行描述 */
  desc: string
  /** 匹配关键词（不含斜杠），如 ['clear'] */
  keywords: string[]
}

interface Props {
  query: string
  commands: SlashCommandDef[]
  selectedIdx: number
  onSelect: (cmd: SlashCommandDef) => void
}

export function SlashMenu(props: Props) {
  const filtered = createMemo(() => {
    const q = props.query.trim().toLowerCase()
    if (!q) return props.commands
    return props.commands.filter(
      (c) => c.keywords.some((k) => k.includes(q)) || c.label.toLowerCase().includes(q)
    )
  })

  return (
    <Show when={filtered().length > 0}>
      <div class="slash-menu" role="listbox" aria-label="斜杠命令">
        <For each={filtered()}>
          {(cmd, i) => (
            <button
              role="option"
              aria-selected={i() === props.selectedIdx}
              class={clsx('slash-item', i() === props.selectedIdx && 'on')}
              onClick={() => props.onSelect(cmd)}
              onMouseEnter={() => i() /* 悬停由父级同步索引（可选） */}
            >
              <span class="slash-kbd">/</span>
              <span class="slash-label">{cmd.label}</span>
              <span class="slash-desc">{cmd.desc}</span>
            </button>
          )}
        </For>
      </div>
    </Show>
  )
}