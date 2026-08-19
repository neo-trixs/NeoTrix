import { invoke } from '@tauri-apps/api/core'

/* ════════════════════════════════════════════
   api/client.ts — Tauri invoke 统一封装
   职责：
   1. 统一错误归一化（后端 Err(String) → ApiError）
   2. 类型化调用入口（IPC 契约单一事实源见 api/types.ts）
   组件禁止直接 import '@tauri-apps/api/core'，一律经 api/* 域模块。
   ════════════════════════════════════════════ */

/** invoke 拒绝值形态（Tauri 2）：command 失败时为一个含 code/message 的对象。
 *  对齐 iPolloWorkServerError 三要素：status/code/details。 */
export interface InvokeErrorShape {
  code?: string
  message?: string
  details?: unknown
}

/** 统一 IPC 错误：后端 Err(message) 或传输异常均归一为此类。
 *  code 从 invoke 拒绝值提取（Tauri 内建如 ERR_COMMAND_FAILURE，或后端未来
 *  返回结构化错误时透传的语义码）；message 恒为人类可读文本。 */
export class ApiError extends Error {
  readonly code: string | null
  readonly details: unknown

  constructor(message: string, code?: string | null, details?: unknown) {
    super(message)
    this.name = 'ApiError'
    this.code = code ?? null
    this.details = details
  }
}

/** 从 catch 到的任意值归一为 ApiError（保留 code/details 三要素） */
export function toApiError(e: unknown): ApiError {
  if (e instanceof ApiError) return e
  if (e instanceof Error) {
    const shape = (e as unknown as { details?: unknown }).details as InvokeErrorShape | undefined
    if (shape && typeof shape.message === 'string') {
      return new ApiError(shape.message, shape.code ?? null, shape.details)
    }
    return new ApiError(e.message)
  }
  if (e && typeof e === 'object') {
    const shape = e as InvokeErrorShape
    if (typeof shape.message === 'string') {
      return new ApiError(shape.message, shape.code ?? null, shape.details)
    }
  }
  return new ApiError(String(e))
}

/** 类型化调用。args 键名保持 snake_case（对齐 Rust 参数名） */
export async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args)
  } catch (e) {
    throw toApiError(e)
  }
}

/** 静默调用：失败返回 fallback（非关键路径用，避免 try/catch 噪音） */
export async function callOr<T>(cmd: string, args: Record<string, unknown> | undefined, fallback: T): Promise<T> {
  try {
    return await invoke<T>(cmd, args)
  } catch {
    return fallback
  }
}

/** 从后端 Err(String) 提取人类可读信息（透传） */
export function errText(e: unknown): string {
  if (e instanceof ApiError) return e.message
  return e instanceof Error ? e.message : String(e)
}
