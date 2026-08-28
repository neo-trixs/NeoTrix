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

/** neocodex_stream_error 事件负载：provider 阶段错误（F1） */
export interface StreamErrorPayload {
  message: string
  partial: string
  // 结构化错误三段式（后端可选填充；前端缺失时按规则推导 what/why/next）
  what?: string
  why?: string
  next?: string
}

// 推理/思考流（OS 正在推理，非最终回复）——后端可选 emit，前端就绪层
export interface StreamReasoningPayload {
  text: string
}

export interface StreamEventHandlers {
  onStart?: (sessionId?: string) => void
  onToken?: (delta: string) => void
  onEnd?: (content: string) => void
  onDone?: (payload: { cancelled: boolean; elapsed_ms: number; content: string }) => void
  onTool?: (payload: StreamToolPayload) => void
  /** F1: provider 阶段错误（不落盘，保留 partial） */
  onError?: (payload: StreamErrorPayload) => void
  /** OS 推理流（可选）：把意识核心的推理步骤实时透出，对抗 black-box */
  onReasoning?: (payload: StreamReasoningPayload) => void
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
  if (handlers.onError) {
    await subscribe('neocodex_stream_error', () =>
      listen<StreamErrorPayload>('neocodex_stream_error', (e) => handlers.onError?.(e.payload)),
    )
  }
  if (handlers.onReasoning) {
    await subscribe('neocodex_stream_reasoning', () =>
      listen<StreamReasoningPayload>('neocodex_stream_reasoning', (e) => handlers.onReasoning?.(e.payload)),
    )
  }
  return () => {
    for (const un of unlisteners) un()
  }
}

/** 桌面菜单/全局事件桥（Rust 菜单 emit → 前端动作）
 *  Rust 侧 SubmenuBuilder（src-tauri/src/lib.rs）emit 以下事件，
 *  此前零前端订阅 → 菜单项 UI 全部无效。此层统一桥接。
 *  组件禁止直接 import '@tauri-apps/api/event'，统一经此层。 */
export interface MenuEventHandlers {
  onNewSession?: () => void
  onOpenSettings?: () => void
  onOpenPalette?: () => void
  onCheckUpdates?: () => void
}

export async function subscribeMenuEvents(handlers: MenuEventHandlers): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = []
  const entries: Array<[string, (() => void) | undefined]> = [
    ['neotrix:new-session', handlers.onNewSession],
    ['open-settings', handlers.onOpenSettings],
    ['neocodex-open-palette', handlers.onOpenPalette],
    ['neocodex-check-updates', handlers.onCheckUpdates],
  ]
  for (const [event, cb] of entries) {
    if (!cb) continue
    try {
      unlisteners.push(await listen<void>(event, () => cb()))
    } catch {
      // 菜单事件订阅失败不阻塞主流程（非核心能力）
    }
  }
  return () => {
    for (const un of unlisteners) un()
  }
}
