/* ════════════════════════════════════════════
   lib/errorMonitor.ts — 前端 JS 错误上报
   全局捕获 window error / unhandledrejection，经后端
   neocodex_report_frontend_error 桥接到 nt_shield_sentry。
   DSN 未配置时后端静默，前端不阻塞主流程。
   ════════════════════════════════════════════ */
import { callOr } from '../api/client'

let registered = false

/** 幂等注册全局错误捕获（main.tsx 启动时调用一次） */
export function initErrorMonitor(): void {
  if (registered) return
  registered = true

  window.addEventListener('error', (ev: ErrorEvent) => {
    void report(ev.message ?? 'unknown', ev.filename, ev.error?.stack)
  })

  window.addEventListener('unhandledrejection', (ev: PromiseRejectionEvent) => {
    const reason = ev.reason
    const message = reason instanceof Error ? reason.message : String(reason)
    const stack = reason instanceof Error ? reason.stack : undefined
    void report(message, 'unhandledrejection', stack)
  })
}

function report(message: string, source: string, stack?: string): Promise<unknown> {
  // 静默上报：后端未配置 DSN / IPC 失败均不抛回
  return callOr(
    'neocodex_report_frontend_error',
    { source, message, stack: stack ?? null },
    null,
  ).catch(() => null)
}
