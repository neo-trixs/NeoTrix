import { For, Show } from 'solid-js'

/* ══════════════════════════════════════════════
   ShortcutHelp — 快捷键帮助面板（⌘? 唤起，对标 Raycast/命令行帮助）。
   罗列 App Bot 全部键盘快捷键，提升「对话即操作系统」可发现性。
   ════════════════════════════════════════════ */

interface Shortcut {
  keys: string
  desc: string
}

const SHORTCUTS: Shortcut[] = [
  { keys: '⌘K', desc: '打开命令面板' },
  { keys: '⌘N', desc: '新建会话' },
  { keys: '⌘1 – ⌘6', desc: '切换功能面板（能力地图 / 标注 / 预览 …）' },
  { keys: '⌘7', desc: '切换电脑控制视图' },
  { keys: 'Enter', desc: '发送消息' },
  { keys: 'Shift + Enter', desc: '换行（不发送）' },
  { keys: '⌘Enter', desc: '编辑态重发修改' },
  { keys: 'Shift + Tab', desc: '循环权限模式（手动 / 规划 / 自动 / 接受编辑）' },
  { keys: '@', desc: '输入区唤起上下文引用' },
  { keys: 'Esc', desc: '关闭浮层（命令面板 / 面板 / 设置 / 帮助）' },
  { keys: '⌘?', desc: '打开本帮助' },
]

export function ShortcutHelp(props: { open: boolean; onClose: () => void }) {
  return (
    <Show when={props.open}>
      <div class="cmd-palette" role="dialog" aria-modal="true" aria-label="键盘快捷键帮助">
        <div class="cmd-palette-backdrop" onClick={props.onClose} aria-hidden="true" />
        <div class="cmd-palette-card shortcut-help-card">
          <div class="shortcut-help-head">
            <span>键盘快捷键</span>
            <button class="shortcut-help-close" onClick={props.onClose} aria-label="关闭帮助">
              ×
            </button>
          </div>
          <div class="shortcut-help-list">
            <For each={SHORTCUTS}>
              {(s) => (
                <div class="shortcut-help-row">
                  <span class="shortcut-help-desc">{s.desc}</span>
                  <kbd class="shortcut-help-kbd">{s.keys}</kbd>
                </div>
              )}
            </For>
          </div>
          <div class="cmd-palette-footer">
            <span><kbd>esc</kbd> 关闭</span>
          </div>
        </div>
      </div>
    </Show>
  )
}
