import { createMemo, createSignal, createEffect, For, Show } from 'solid-js'
import { CheckSquare, Square, ListTodo } from 'lucide-solid'
import { clsx } from 'clsx'

/* 批次4：消息内 TODO 追踪 —— 从 assistant markdown 回复解析 `- [ ]` / `- [x]` checklist，
 * 渲染为可勾选任务组。勾选状态本地持久化（localStorage，按消息 id 隔离），不写后端。
 * 依据: agentic 会话中任务状态与对话绑定的 UX 实践（计划-执行闭环的可见追踪）。 */

export interface TodoItem {
  key: number
  text: string
  done: boolean
}

export function parseTodoItems(content: string): TodoItem[] {
  if (!content) return []
  const items: TodoItem[] = []
  let key = 0
  const re = /^\s*[-*]\s+\[([ xX])\]\s+(.+)$/gm
  let m: RegExpExecArray | null
  while ((m = re.exec(content)) !== null) {
    items.push({ key: key++, text: m[2].trim(), done: m[1].toLowerCase() === 'x' })
  }
  return items
}

const STORAGE_PREFIX = 'neotrix:todo:'

export function loadTodoState(msgId: string, count: number): boolean[] | null {
  try {
    const raw = localStorage.getItem(STORAGE_PREFIX + msgId)
    if (!raw) return null
    const arr = JSON.parse(raw)
    if (!Array.isArray(arr) || arr.length !== count) return null
    return arr.map(Boolean)
  } catch {
    return null
  }
}

export function saveTodoState(msgId: string, done: boolean[]): void {
  try {
    localStorage.setItem(STORAGE_PREFIX + msgId, JSON.stringify(done))
  } catch {
    /* localStorage 不可用/满时静默降级为会话内状态 */
  }
}

export function TaskList(props: { content: string; messageId: string }) {
  const items = createMemo(() => parseTodoItems(props.content))
  const [done, setDone] = createSignal<boolean[]>(
    loadTodoState(props.messageId, items().length) ?? items().map(i => i.done)
  )

  // 🟡 修复：assistant 消息流式追加内容时 items 长度增长，done 数组必须同步扩容。
  // 此前 done 仅在首次按初始长度初始化：流式期间 toggle 会写入越界稀疏数组，
  // 且 loadTodoState 长度校验失败导致持久化/恢复失效。
  createEffect(() => {
    const n = items().length
    const persisted = loadTodoState(props.messageId, n)
    setDone(prev => {
      if (persisted) return persisted
      if (prev.length === n) return prev
      const next = prev.slice(0, n)
      while (next.length < n) next.push(items()[next.length].done)
      return next
    })
  })

  const toggle = (idx: number) => {
    setDone(prev => {
      const next = prev.slice()
      next[idx] = !next[idx]
      saveTodoState(props.messageId, next)
      return next
    })
  }

  const completed = () => done().filter(Boolean).length

  return (
    <Show when={items().length > 0}>
      <div class="mt-2 rounded-xl border border-nt-act-500/20 bg-nt-act-500/5 overflow-hidden">
        <div class="flex items-center gap-2 px-3 py-2 border-b border-nt-act-500/15">
          <ListTodo class="w-3.5 h-3.5 text-nt-act-600 flex-shrink-0" />
          <span class="text-xs font-medium text-text-primary flex-1 min-w-0 truncate">任务清单</span>
          <span class={clsx('text-[10px] font-mono flex-shrink-0', completed() === items().length ? 'text-emerald-600' : 'text-text-muted')}>
            {completed()}/{items().length}
          </span>
        </div>
        <ul class="divide-y divide-nt-act-500/10">
          <For each={items()}>
            {(item, i) => (
              <li>
                <button
                  class={clsx(
                    'w-full flex items-start gap-2 px-3 py-2 text-left hover:bg-nt-act-500/5 transition-colors focus-visible:ring-2 focus-visible:ring-nt-act-500 focus-visible:outline-none',
                    done()[i()] ? 'opacity-60' : 'opacity-100'
                  )}
                  onClick={() => toggle(i())}
                  aria-pressed={done()[i()]}
                  aria-label={`${done()[i()] ? '取消完成' : '标记完成'}: ${item.text}`}
                >
                  <span class="mt-0.5 flex-shrink-0">
                    {done()[i()] ? (
                      <CheckSquare class="w-4 h-4 text-emerald-600" />
                    ) : (
                      <Square class="w-4 h-4 text-text-muted" />
                    )}
                  </span>
                  <span class={clsx('text-[13px] leading-relaxed break-words', done()[i()] && 'line-through text-text-muted')}>
                    {item.text}
                  </span>
                </button>
              </li>
            )}
          </For>
        </ul>
      </div>
    </Show>
  )
}