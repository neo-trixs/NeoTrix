// ══════════════════════════════════════════════════════════════════════════
//  MessageBubble — 统一消息气泡组件
//  支持：文本/代码/图像/视频/音频/工具结果/流式输出
//  参考：iMessage/WhatsApp/Telegram/ChatGPT 最新设计
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, Show, For, Match, Switch, createEffect, onCleanup } from 'solid-js'
import { clsx } from 'clsx'

export type MessageRole = 'user' | 'assistant' | 'system' | 'tool'

export interface MediaAttachment {
  type: 'image' | 'video' | 'audio' | 'file'
  url: string
  name?: string
  size?: number
  thumbnail?: string
  duration?: number
}

export interface ToolCall {
  id: string
  name: string
  arguments?: string
  result?: string
  success?: boolean
  duration_ms?: number
}

export interface Message {
  id: string
  role: MessageRole
  content: string
  timestamp: Date
  /** 流式输出状态 */
  streaming?: boolean
  /** 附件列表 */
  attachments?: MediaAttachment[]
  /** 工具调用 */
  toolCalls?: ToolCall[]
  /** 是否已编辑 */
  edited?: boolean
  /** 引用消息 */
  replyTo?: string
}

interface MessageBubbleProps {
  message: Message
  isLast?: boolean
  onCopy?: (text: string) => void
  onRetry?: (id: string) => void
  onDelete?: (id: string) => void
}

export function MessageBubble(props: MessageBubbleProps) {
  const [showActions, setShowActions] = createSignal(false)
  const [copied, setCopied] = createSignal(false)

  const isUser = () => props.message.role === 'user'
  const isAssistant = () => props.message.role === 'assistant'
  const isTool = () => props.message.role === 'tool'
  const isSystem = () => props.message.role === 'system'

  const handleCopy = () => {
    navigator.clipboard.writeText(props.message.content)
    setCopied(true)
    props.onCopy?.(props.message.content)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div
      class={clsx(
        'msg-row',
        isUser() && 'msg-row-user',
        isAssistant() && 'msg-row-assistant',
        isTool() && 'msg-row-tool',
        isSystem() && 'msg-row-system',
      )}
      onMouseEnter={() => setShowActions(true)}
      onMouseLeave={() => setShowActions(false)}
    >
      {/* 头像 */}
      <Show when={isAssistant()}>
        <div class="msg-avatar msg-avatar-ai">
          <span>✦</span>
        </div>
      </Show>

      {/* 消息主体 */}
      <div class="msg-body">
        {/* 气泡 */}
        <div class={clsx(
          'msg-bubble',
          isUser() && 'msg-bubble-user',
          isAssistant() && 'msg-bubble-assistant',
          isTool() && 'msg-bubble-tool',
          isSystem() && 'msg-bubble-system',
          props.message.streaming && 'msg-bubble-streaming',
        )}>
          {/* 流式输出光标 */}
          <Show when={props.message.streaming && isAssistant()}>
            <span class="msg-cursor" />
          </Show>

          {/* 文本内容 */}
          <Show when={props.message.content}>
            <div class="msg-text">{props.message.content}</div>
          </Show>

          {/* 图像附件 */}
          <Show when={props.message.attachments?.some(a => a.type === 'image')}>
            <div class="msg-media-grid">
              <For each={props.message.attachments?.filter(a => a.type === 'image') ?? []}>
                {(img) => (
                  <div class="msg-image-wrap">
                    <img
                      src={img.url}
                      alt={img.name ?? '图像'}
                      class="msg-image"
                      loading="lazy"
                    />
                  </div>
                )}
              </For>
            </div>
          </Show>

          {/* 视频附件 */}
          <Show when={props.message.attachments?.some(a => a.type === 'video')}>
            <For each={props.message.attachments?.filter(a => a.type === 'video') ?? []}>
              {(vid) => (
                <div class="msg-video-wrap">
                  <video
                    src={vid.url}
                    controls
                    preload="metadata"
                    class="msg-video"
                    poster={vid.thumbnail}
                  />
                </div>
              )}
            </For>
          </Show>

          {/* 音频附件 */}
          <Show when={props.message.attachments?.some(a => a.type === 'audio')}>
            <For each={props.message.attachments?.filter(a => a.type === 'audio') ?? []}>
              {(aud) => (
                <div class="msg-audio-wrap">
                  <div class="msg-audio-icon">🎵</div>
                  <audio src={aud.url} controls preload="metadata" class="msg-audio" />
                  <Show when={aud.name}>
                    <span class="msg-audio-name">{aud.name}</span>
                  </Show>
                </div>
              )}
            </For>
          </Show>

          {/* 文件附件 */}
          <Show when={props.message.attachments?.some(a => a.type === 'file')}>
            <For each={props.message.attachments?.filter(a => a.type === 'file') ?? []}>
              {(file) => (
                <div class="msg-file-wrap">
                  <span class="msg-file-icon">📎</span>
                  <span class="msg-file-name">{file.name}</span>
                  <Show when={file.size}>
                    <span class="msg-file-size">{formatSize(file.size!)}</span>
                  </Show>
                </div>
              )}
            </For>
          </Show>

          {/* 工具调用 */}
          <Show when={props.message.toolCalls && props.message.toolCalls.length > 0}>
            <div class="msg-tools">
              <For each={props.message.toolCalls ?? []}>
                {(tc) => (
                  <div class={clsx('msg-tool', tc.success ? 'msg-tool-ok' : 'msg-tool-fail')}>
                    <span class="msg-tool-icon">{tc.success ? '✓' : '✗'}</span>
                    <span class="msg-tool-name">{tc.name}</span>
                    <Show when={tc.duration_ms !== undefined}>
                      <span class="msg-tool-time">{tc.duration_ms}ms</span>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>

        {/* 操作按钮 */}
        <Show when={showActions() && !props.message.streaming}>
          <div class="msg-actions">
            <button class="msg-action-btn" onClick={handleCopy} title="复制">
              {copied() ? '✓' : '📋'}
            </button>
            <Show when={isAssistant()}>
              <button class="msg-action-btn" onClick={() => props.onRetry?.(props.message.id)} title="重试">
                🔄
              </button>
            </Show>
            <button class="msg-action-btn msg-action-delete" onClick={() => props.onDelete?.(props.message.id)} title="删除">
              🗑️
            </button>
          </div>
        </Show>

        {/* 时间戳 */}
        <div class="msg-time">
          {formatTime(props.message.timestamp)}
          <Show when={props.message.edited}>
            <span class="msg-edited">(已编辑)</span>
          </Show>
        </div>
      </div>

      {/* 用户头像 */}
      <Show when={isUser()}>
        <div class="msg-avatar msg-avatar-user">
          <span>N</span>
        </div>
      </Show>
    </div>
  )
}

function formatTime(date: Date): string {
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1048576).toFixed(1)} MB`
}
