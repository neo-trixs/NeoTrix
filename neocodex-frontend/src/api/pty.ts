/* ════════════════════════════════════════════
   api/pty.ts — PTY 终端 IPC 封装
   后端: src-tauri pty_spawn/write/resize/close
        事件: pty-output-{id} / pty-exit-{id}
   ════════════════════════════════════════════ */
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/** 启动 PTY 会话，返回 session id */
export function ptySpawn(cols: number, rows: number): Promise<string> {
  return invoke<string>('pty_spawn', { cols, rows })
}

/** 写入数据到 PTY stdin */
export function ptyWrite(sessionId: string, data: string): Promise<void> {
  return invoke('pty_write', { sessionId, data })
}

/** 通知后端终端尺寸变更 */
export function ptyResize(sessionId: string, cols: number, rows: number): Promise<void> {
  return invoke('pty_resize', { sessionId, cols, rows })
}

/** 关闭 PTY 会话 */
export function ptyClose(sessionId: string): Promise<void> {
  return invoke('pty_close', { sessionId })
}

/** 订阅 PTY 输出（服务端 → 前端的字节流），返回取消订阅函数 */
export function onPtyOutput(sessionId: string, handler: (data: string) => void): Promise<UnlistenFn> {
  return listen<string>(`pty-output-${sessionId}`, (event) => handler(event.payload))
}

/** 订阅 PTY 退出事件 */
export function onPtyExit(sessionId: string, handler: (code: number) => void): Promise<UnlistenFn> {
  return listen<number>(`pty-exit-${sessionId}`, (event) => handler(event.payload))
}
