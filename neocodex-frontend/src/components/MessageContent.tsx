// ══════════════════════════════════════════════════════════════════════════
//  MessageContent — 消息内容渲染器
//  统一处理：文本/流式输出/图像/视频/音频/代码块/工具结果
//  替代 Chat.tsx 中分散的内容渲染逻辑
// ══════════════════════════════════════════════════════════════════════════
import { Show, For, createSignal, createEffect, onCleanup, Match, Switch } from 'solid-js'
import { clsx } from 'clsx'
import { Markdown } from './Markdown'
import { StreamingText } from './StreamingText'
import { CodeBlock } from './CodeBlock'
import { ToolResult } from './ToolResult'
import { FilePreview } from './FilePreview'
import type { Message } from '../stores/chat'

interface MessageContentProps {
  message: Message
  /** 是否折叠（超长消息） */
  collapsed?: boolean
  /** 折叠预览文本 */
  foldedPreview?: string
  /** 展开/折叠回调 */
  onToggleExpand?: () => void
  /** 是否可展开 */
  collapsible?: boolean
  /** 是否展开 */
  expanded?: boolean
  /** 注解提示回调 */
  onAnnotate?: (hint: string | null) => void
}

/**
 * 智能内容渲染：
 * - 用户消息：纯文本 + 附件
 * - AI 消息：Markdown + 流式输出 + 代码块 + 工具结果 + 附件
 * - 工具消息：工具调用卡片
 */
export function MessageContent(props: MessageContentProps) {
  const isUser = () => props.message.role === 'user'
  const isAssistant = () => props.message.role === 'assistant'
  const isTool = () => props.message.role === 'tool'
  const isStreaming = () => props.message.isStreaming

  return (
    <div class="msg-content">
      {/* 工具消息：仅显示工具调用 */}
      <Show when={isTool() && props.message.toolCalls && props.message.toolCalls.length > 0}>
        <ToolResult tools={props.message.toolCalls!} />
      </Show>

      {/* 用户消息 */}
      <Show when={isUser()}>
        <p class="msg-text-user whitespace-pre-wrap">{props.message.content}</p>
      </Show>

      {/* AI 消息：Markdown + 流式输出 */}
      <Show when={isAssistant()}>
        <Switch>
          {/* 流式输出：打字机效果 */}
          <Match when={isStreaming()}>
            <StreamingText
              text={props.message.content}
              streaming={true}
              speed={20}
            />
          </Match>

          {/* 静态消息：Markdown 渲染 */}
          <Match when={true}>
            <div
              class="relative"
              style={props.collapsed ? { 'max-height': '360px', overflow: 'hidden' } : undefined}
            >
              <Markdown content={props.collapsed ? (props.foldedPreview ?? props.message.content) : props.message.content} />
              {props.collapsed && (
                <div
                  class="absolute inset-x-0 bottom-0 h-16 pointer-events-none"
                  style={{
                    background: 'linear-gradient(180deg, rgba(255,255,255,0) 0%, rgba(255,255,255,0.92) 100%)',
                  }}
                />
              )}
            </div>
            {/* 展开/折叠按钮 */}
            <Show when={props.collapsible}>
              <button
                class="msg-expand-btn"
                onClick={props.onToggleExpand}
              >
                {props.expanded ? '收起' : '展开全文'}
              </button>
            </Show>
          </Match>
        </Switch>
      </Show>

      {/* 工具调用（非 tool 角色消息中的 toolCalls） */}
      <Show when={!isTool() && props.message.toolCalls && props.message.toolCalls.length > 0}>
        <div class="msg-tool-calls-inline">
          <For each={props.message.toolCalls!}>
            {(tc) => (
              <div class={clsx('msg-tool-inline', tc.success !== false ? 'tool-ok' : 'tool-fail')}>
                <span class="msg-tool-inline-icon">{tc.success !== false ? '✓' : '✗'}</span>
                <span class="msg-tool-inline-name">{tc.name}</span>
                <Show when={tc.duration_ms !== undefined}>
                  <span class="msg-tool-inline-time">{tc.duration_ms}ms</span>
                </Show>
              </div>
            )}
          </For>
        </div>
      </Show>

      {/* 附件 */}
      <Show when={props.message.attachments && props.message.attachments.length > 0}>
        <div class="msg-attachments">
          <For each={props.message.attachments!}>
            {(att) => (
              <FilePreview
                attachment={att}
                onAnnotate={props.onAnnotate}
              />
            )}
          </For>
        </div>
      </Show>
    </div>
  )
}
