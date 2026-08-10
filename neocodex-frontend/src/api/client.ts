import { invoke } from '@tauri-apps/api/core'

/* ════════════════════════════════════════════
   api/client.ts — Tauri invoke 统一封装
   职责：
   1. 统一错误归一化（后端 Err(String) → ApiError）
   2. 类型化调用入口（IPC 契约单一事实源见 api/types.ts）
   组件禁止直接 import '@tauri-apps/api/core'，一律经 api/* 域模块。
   ════════════════════════════════════════════ */

/** 统一 IPC 错误：后端 Err(message) 或传输异常均归一为此类 */
export class ApiError extends Error {
  /** 错误类别（扩展位，供 UI 按 code 差异化展示） */
  code: string

  constructor(message: string, code = 'ERR_IPC') {
    super(message)
    this.name = 'ApiError'
    this.code = code
  }
}

/** 类型化调用。args 键名保持 snake_case（对齐 Rust 参数名） */
export async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args)
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    throw new ApiError(msg)
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
  return e instanceof Error ? e.message : String(e)
}
