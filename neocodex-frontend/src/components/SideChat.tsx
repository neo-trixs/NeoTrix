import { createSignal, createEffect, Show, For } from 'solid-js'
import { MessageSquare, X, Send, Loader2, RefreshCw } from 'lucide-solid'
import { neocodex, errText } from '../api'
import type { NeoCodexMessageItem } from '../api/types'
import { clsx } from 'clsx'
import { Markdown } from './Markdown'

interface Props {
  open: boolean
  sessionId: string | null
  onClose: () => void
}

export function SideChat(props: Props) {
  const [messages, setMessages] = createSignal<NeoCodexMessageItem[]>([])
  const [input, setInput] = createSignal('')
  const [sending, setSending] = createSignal(false)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [bodyRef, setBodyRef] = createSignal<HTMLDivElement | null>(null)
  let inputRef: HTMLTextAreaElement | undefined
  let panelRef: HTMLDivElement | undefined
  // 打开面板前的触发元素，关闭后还原焦点
  let lastFocusedEl: HTMLElement | null = null

  // 面板打开时聚焦输入框（对标 Codex 面板聚焦规范），并记录触发元素；
  // 关闭时（Esc/关闭按钮/遮罩点击触发卸载）经 effect 清理还原焦点
  createEffect(() => {
    if (!props.open) return
    lastFocusedEl = document.activeElement as HTMLElement | null
    const raf = requestAnimationFrame(() => {
      if (inputRef) inputRef.focus()
      else panelRef?.focus()
    })
    return () => {
      cancelAnimationFrame(raf)
      if (lastFocusedEl?.isConnected) lastFocusedEl.focus()
    }
  })

  const load = async () => {
    if (!props.sessionId) return
    setLoading(true)
    setError(null)
    try {
      const msgs = await neocodex.getSideChat(props.sessionId)
      setMessages(msgs)
      scrollToBottom()
    } catch (e) {
      setError(errText(e))
    } finally {
      setLoading(false)
    }
  }

  const scrollToBottom = () => {
    const el = bodyRef()
    if (el) el.scrollTop = el.scrollHeight
  }

  // 会话切换时重新加载（面板打开状态 + sessionId 变化）。
  // 组件在面板打开时挂载（见 Chat.tsx Show when=activePanel），createEffect 首轮即覆盖
  // 打开加载，故不再单独 onMount(load)，避免同一 sessionId 触发两次请求。
  createEffect(() => {
    if (props.open && props.sessionId) {
      load()
    }
  })

  const send = async () => {
    const content = input().trim()
    if (!content || sending() || !props.sessionId) return
    setSending(true)
    setError(null)
    try {
      const msgs = await neocodex.sendSideChat(props.sessionId, content)
      setMessages(msgs)
      setInput('')
      scrollToBottom()
    } catch (e) {
      setError(errText(e))
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
      <div
        ref={panelRef}
        class="panel w-[24rem]"
        role="dialog"
        aria-modal="true"
        aria-label="侧聊"
        tabIndex={-1}
        onKeyDown={(e) => {
          if (e.key === 'Escape') {
            e.preventDefault()
            props.onClose()
            return
          }
          if (e.key === 'Tab' && panelRef) {
            const focusables = panelRef.querySelectorAll<HTMLElement>(
              'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
            )
            if (focusables.length === 0) return
            const first = focusables[0]
            const last = focusables[focusables.length - 1]
            const active = document.activeElement
            if (e.shiftKey && (active === first || active === panelRef)) {
              e.preventDefault()
              last.focus()
            } else if (!e.shiftKey && active === last) {
              e.preventDefault()
              first.focus()
            }
          }
        }}
      >
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
            <RefreshCw class={clsx('w-4 h-4', loading() && 'animate-spin')} />
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
        <div ref={setBodyRef} class="flex-1 overflow-y-auto p-3 space-y-3" aria-live="polite">
          <Show when={loading() && messages().length === 0 && !error()}>
            <div class="py-10 text-center text-xs text-text-muted flex items-center justify-center gap-2">
              <Loader2 class="w-4 h-4 animate-spin" />
              加载中…
            </div>
          </Show>
          <Show when={messages().length === 0 && !error() && !loading()}>
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
                    'max-w-[85%] px-3 py-2 rounded-2xl text-sm break-words',
                    msg.role === 'user'
                      ? 'bg-nt-mind-500/20 text-text-primary rounded-br-sm whitespace-pre-wrap'
                      : 'bg-bg-tertiary text-text-primary rounded-bl-sm'
                  )}
                >
                  <div class="text-[10px] text-text-muted mb-1">{formatTime(msg.timestamp)}</div>
                  {/* assistant 回答接入 Markdown 渲染（代码块/列表/链接，XSS 已转义）；
                      user 输入保持纯文本等宽回显 */}
                  {msg.role === 'user' ? (
                    msg.content
                  ) : (
                    <Markdown content={msg.content} />
                  )}
                </div>
              </div>
            )}
          </For>
          {/* 发送中占位：请求未返回时显示「思考中…」气泡，避免无任何进行中反馈 */}
          <Show when={sending()}>
            <div class="flex justify-start" role="status">
              <div class="max-w-[85%] px-3 py-2 rounded-2xl text-sm bg-bg-tertiary text-text-muted rounded-bl-sm flex items-center gap-2">
                <Loader2 class="w-3.5 h-3.5 animate-spin" />
                思考中…
              </div>
            </div>
          </Show>
        </div>

        {/* Input */}
        <div class="border-t border-border-primary p-3">
          <div class="flex items-end gap-2">
            <textarea
              ref={inputRef}
              value={input()}
              onInput={(e) => setInput(e.currentTarget.value)}
              onKeyDown={(e) => {
                // 中文 IME 组合态（候选词上屏）按 Enter 不发送，避免半截拼音误发
                if (e.isComposing || e.keyCode === 229) return
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  send()
                }
              }}
              placeholder="向侧向对话提问..."
              rows={2}
              class="flex-1 resize-none bg-white/40 border border-white/40 rounded-xl px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-nt-io-500 backdrop-blur-sm"
            />
            <button
              class="p-3 rounded-xl bg-nt-mind-500/20 text-nt-mind-700 hover:bg-nt-mind-500/30 transition-colors disabled:opacity-50"
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