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
  /** 单个事件订阅失败时回调（用于向用户暴露流式降级提示） */
  onSubscribeError?: (event: string, error: unknown) => void
}

/** 订阅聊天流式事件（neocodex_stream_*），返回解除订阅函数 */
export async function subscribeStream(handlers: StreamEventHandlers): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = []
  const subscribe = async (event: string, cb: () => Promise<UnlistenFn>) => {
    try {
      unlisteners.push(await cb())
    } catch (e) {
      handlers.onSubscribeError?.(event, e)
    }
  }
  if (handlers.onStart) {
    await subscribe('neocodex_stream_start', () =>
      listen<string>('neocodex_stream_start', (e) => handlers.onStart?.(e.payload)),
    )
  }
  if (handlers.onToken) {
    await subscribe('neocodex_stream_token', () =>
      listen<string>('neocodex_stream_token', (e) => handlers.onToken?.(e.payload)),
    )
  }
  if (handlers.onEnd) {
    await subscribe('neocodex_stream_end', () =>
      listen<string>('neocodex_stream_end', (e) => handlers.onEnd?.(e.payload)),
    )
  }
  if (handlers.onDone) {
    await subscribe('neocodex_stream_done', () =>
      listen<{ cancelled: boolean; elapsed_ms: number; content: string }>('neocodex_stream_done', (e) =>
        handlers.onDone?.(e.payload),
      ),
    )
  }
  if (handlers.onTool) {
    await subscribe('neocodex_stream_tool', () =>
      listen<StreamToolPayload>('neocodex_stream_tool', (e) => handlers.onTool?.(e.payload)),
    )
  }
  return () => {
    for (const un of unlisteners) un()
  }
}
