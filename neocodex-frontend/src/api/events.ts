import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/* 透传 Tauri 的取消订阅函数类型 */
export type { UnlistenFn }

/* ════════════════════════════════════════════
   api/events.ts — Tauri 事件订阅（流式生成 / 更新进度）
   组件禁止直接 import '@tauri-apps/api/event'，统一经此层。
   ════════════════════════════════════════════ */

/** neocodex_stream_tool 事件负载：后端无 id，id 由前端生成 */
export interface StreamToolPayload {
  name: string
  args: string
  result: string
  duration_ms: number
  success: boolean
}

export interface StreamEventHandlers {
  onStart?: (sessionId?: string) => void
  onToken?: (delta: string) => void
  onEnd?: (content: string) => void
  onDone?: (payload: { cancelled: boolean; elapsed_ms: number; content: string }) => void
  onTool?: (payload: StreamToolPayload) => void
}

/** 订阅聊天流式事件（neocodex_stream_*），返回解除订阅函数 */
export async function subscribeStream(handlers: StreamEventHandlers): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = []
  try {
    if (handlers.onStart) {
      unlisteners.push(await listen<string>('neocodex_stream_start', (e) => handlers.onStart?.(e.payload)))
    }
    if (handlers.onToken) {
      unlisteners.push(await listen<string>('neocodex_stream_token', (e) => handlers.onToken?.(e.payload)))
    }
    if (handlers.onEnd) {
      unlisteners.push(await listen<string>('neocodex_stream_end', (e) => handlers.onEnd?.(e.payload)))
    }
    if (handlers.onDone) {
      unlisteners.push(
        await listen<{ cancelled: boolean; elapsed_ms: number; content: string }>('neocodex_stream_done', (e) =>
          handlers.onDone?.(e.payload),
        ),
      )
    }
    if (handlers.onTool) {
      unlisteners.push(await listen<StreamToolPayload>('neocodex_stream_tool', (e) => handlers.onTool?.(e.payload)))
    }
  } catch {
    // 部分订阅失败：尽力而为，返回已注册的解除函数
  }
  return () => {
    for (const un of unlisteners) un()
  }
}
