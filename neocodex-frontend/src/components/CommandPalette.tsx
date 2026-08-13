import { createSignal, createEffect, For, Show } from 'solid-js'
import { clsx } from 'clsx'

/* ════════════════════════════════════════════
   CommandPalette — ⌘K 命令面板（对标 Claude Code / Osaurus 命令菜单）
   全局 ⌘K 唤起，模糊过滤 + 键盘上下导航 + Enter 执行。
   命令与动作由父组件注入（复用 Chat 既有 handler，单一事实源）。
   ════════════════════════════════════════════ */

export interface PaletteCommand {
  id: string
  label: string
  desc: string
  keywords: string[]
  run: () => void
}

interface Props {
  open: boolean
  commands: PaletteCommand[]
  onClose: () => void
}

export function CommandPalette(props: Props) {
  const [query, setQuery] = createSignal('')
  const [selectedIdx, setSelectedIdx] = createSignal(0)
  let inputRef: HTMLInputElement | undefined

  const filtered = () => {
    const q = query().trim().toLowerCase()
    if (!q) return props.commands
    return props.commands.filter(
      (c) => c.keywords.some((k) => k.includes(q)) || c.label.toLowerCase().includes(q),
    )
  }

  // 每次打开重置查询并聚焦输入
  createEffect(() => {
    if (props.open) {
      setQuery('')
      setSelectedIdx(0)
      requestAnimationFrame(() => inputRef?.focus())
    }
  })

  const run = (cmd: PaletteCommand) => {
    props.onClose()
    cmd.run()
  }

  const onKeyDown = (e: KeyboardEvent) => {
    const list = filtered()
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault()
      // 🔵 修复：空列表时 `% 0` 产生 NaN，污染 selectedIdx（Enter 有 guard 但状态已坏）
      if (list.length === 0) return
      const dir = e.key === 'ArrowDown' ? 1 : -1
      setSelectedIdx((p) => (p + dir + list.length) % list.length)
    } else if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      const cmd = list[Math.min(selectedIdx(), list.length - 1)]
      if (cmd) run(cmd)
    }
  }

  return (
    <Show when={props.open}>
      <div class="cmd-palette" role="dialog" aria-modal="true" aria-label="命令面板">
        <div class="cmd-palette-backdrop" onClick={props.onClose} aria-hidden="true" />
        <div class="cmd-palette-card">
          <input
            ref={inputRef}
            class="cmd-palette-input"
            placeholder="输入命令…"
            aria-label="搜索命令"
            value={query()}
            onInput={(e) => {
              setQuery(e.currentTarget.value)
              setSelectedIdx(0)
            }}
            onKeyDown={onKeyDown}
          />
          <div class="cmd-palette-list" role="listbox">
            <For each={filtered()}>
              {(cmd, i) => (
                <button
                  class={clsx('slash-item', i() === selectedIdx() && 'on')}
                  role="option"
                  aria-selected={i() === selectedIdx()}
                  onClick={() => run(cmd)}
                  onMouseEnter={() => setSelectedIdx(i())}
                >
                  <span class="slash-kbd">›</span>
                  <span class="slash-label">{cmd.label}</span>
                  <span class="slash-desc">{cmd.desc}</span>
                </button>
              )}
            </For>
            <Show when={filtered().length === 0}>
              <div class="cmd-palette-empty">无匹配命令</div>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  )
}
