/**
 * Chat API — 对话管理
 * 
 * 通过 domain_call('chat', action, args) 调用后端 ChatPlugin。
 * 流式响应通过 Tauri events 订阅。
 */
import { domainCall } from './domain-client'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp?: number
}

export interface SendMessageOptions {
  session_id: string
  content: string
  temperature?: number
  max_tokens?: number
  stream?: boolean
}

/**
 * 发送消息 (非流式)
 */
export async function sendMessage(options: SendMessageOptions): Promise<string> {
  const result = await domainCall<{ content: string }>('chat', 'send', {
    session_id: options.session_id,
    content: options.content,
    temperature: options.temperature ?? 0.7,
    max_tokens: options.max_tokens ?? 2048,
    stream: options.stream ?? false,
  })
  return result.content
}

/**
 * 发送消息 (流式)
 */
export async function sendMessageStream(
  options: SendMessageOptions,
  onToken: (token: string) => void,
  onDone: (content: string) => void,
  onError: (error: string) => void
): Promise<() => void> {
  // 启动流式请求
  const streamId = await domainCall<string>('chat', 'send', {
    session_id: options.session_id,
    content: options.content,
    temperature: options.temperature ?? 0.7,
    max_tokens: options.max_tokens ?? 2048,
    stream: true,
  })

  let fullContent = ''
  
  // 订阅流式事件
  const unlistenToken = await listen<string>(`chat_stream_token_${streamId}`, (event) => {
    fullContent += event.payload
    onToken(event.payload)
  })

  const unlistenDone = await listen<{ content: string }>(`chat_stream_done_${streamId}`, (event) => {
    unlistenToken()
    unlistenDone()
    unlistenError()
    onDone(event.payload.content || fullContent)
  })

  const unlistenError = await listen<{ message: string }>(`chat_stream_error_${streamId}`, (event) => {
    unlistenToken()
    unlistenDone()
    unlistenError()
    onError(event.payload.message)
  })

  // 返回取消函数
  return () => {
    unlistenToken()
    unlistenDone()
    unlistenError()
  }
}

/**
 * 停止生成
 */
export async function stopStream(sessionId: string): Promise<void> {
  await domainCall('chat', 'stop', { session_id: sessionId })
}

/**
 * 获取历史消息
 */
export async function getHistory(sessionId: string): Promise<ChatMessage[]> {
  return domainCall<ChatMessage[]>('chat', 'history', { session_id: sessionId })
}

/**
 * 压缩上下文
 */
export async function compactSession(sessionId: string): Promise<void> {
  await domainCall('chat', 'compact', { session_id: sessionId })
}

/**
 * 导出会话
 */
export async function exportSession(sessionId: string): Promise<string> {
  return domainCall<string>('chat', 'export', { session_id: sessionId })
}
