import { createSignal, onMount, Show, For } from 'solid-js'
import { MessageSquare, X, Send, Loader2, RefreshCw } from 'lucide-solid'
import { invoke } from '@tauri-apps/api/core'
import { clsx } from 'clsx'

interface SideChatMsg {
  id: number
  role: string
  content: string
  timestamp: number
}

interface Props {
  open: boolean
  sessionId: string | null
  onClose: () => void
}

export function SideChat(props: Props) {
  const [messages, setMessages] = createSignal<SideChatMsg[]>([])
  const [input, setInput] = createSignal('')
  const [sending, setSending] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [bodyRef, setBodyRef] = createSignal<HTMLDivElement | null>(null)

  const load = async () => {
    if (!props.sessionId) return
    try {
      const msgs = await invoke<SideChatMsg[]>('neocodex_get_side_chat', { session_id: props.sessionId })
      setMessages(msgs)
      scrollToBottom()
    } catch (e) {
      setError(String(e))
    }
  }

  const scrollToBottom = () => {
    const el = bodyRef()
    if (el) el.scrollTop = el.scrollHeight
  }

  onMount(load)

  const send = async () => {
    const content = input().trim()
    if (!content || sending() || !props.sessionId) return
    setSending(true)
    setError(null)
    try {
      const msgs = await invoke<SideChatMsg[]>('neocodex_send_side_chat', {
        session_id: props.sessionId,
        content,
      })
      setMessages(msgs)
      setInput('')
      scrollToBottom()
    } catch (e) {
      setError(String(e))
    } finally {
      setSending(false)
    }
  }

  const formatTime = (ts: number) => {
    const d = new Date(ts * 1000)
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }

  return (
    <Show when={props.open}>
      <div class="panel w-[24rem]">
        {/* Header */}
        <div class="panel-head">
          <MessageSquare class="panel-head-icon text-nt-mind-600" />
          <span class="panel-title">侧向对话</span>
          <span class="panel-sub">独立于主上下文</span>
          <button
            class="panel-close"
            onClick={load}
            aria-label="刷新"
            title="刷新"
          >
            <RefreshCw class="w-4 h-4" />
          </button>
          <button
            class="panel-close"
            onClick={props.onClose}
            aria-label="关闭"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        {/* Messages */}
        <div ref={setBodyRef} class="flex-1 overflow-y-auto p-3 space-y-3">
          <Show when={messages().length === 0 && !error()}>
            <div class="py-10 text-center text-xs text-text-muted">
              侧向对话与主上下文隔离，适合提问、澄清、探索而不污染主线程
            </div>
          </Show>
          <Show when={error()}>
            <div class="p-2 text-xs text-red-500 bg-red-500/10 rounded-lg">{error()}</div>
          </Show>
          <For each={messages()}>
            {(msg) => (
              <div class={clsx('flex', msg.role === 'user' ? 'justify-end' : 'justify-start')}>
                <div
                  class={clsx(
                    'max-w-[85%] px-3 py-2 rounded-2xl text-sm whitespace-pre-wrap break-words',
                    msg.role === 'user'
                      ? 'bg-nt-mind-500/20 text-text-primary rounded-br-sm'
                      : 'bg-bg-tertiary text-text-primary rounded-bl-sm'
                  )}
                >
                  <div class="text-[10px] text-text-muted mb-1">{formatTime(msg.timestamp)}</div>
                  {msg.content}
                </div>
              </div>
            )}
          </For>
        </div>

        {/* Input */}
        <div class="border-t border-border-primary p-3">
          <div class="flex items-end gap-2">
            <textarea
              value={input()}
              onInput={(e) => setInput(e.currentTarget.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  send()
                }
              }}
              placeholder="向侧向对话提问..."
              rows={2}
              class="flex-1 resize-none bg-bg-primary border border-border-primary rounded-xl px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-nt-mind-400/50"
            />
            <button
              class="p-2.5 rounded-xl bg-nt-mind-500/20 text-nt-mind-700 hover:bg-nt-mind-500/30 transition-colors disabled:opacity-50"
              onClick={send}
              disabled={sending() || !input().trim()}
              aria-label="发送"
            >
              {sending() ? <Loader2 class="w-4 h-4 animate-spin" /> : <Send class="w-4 h-4" />}
            </button>
          </div>
        </div>
      </div>
    </Show>
  )
}