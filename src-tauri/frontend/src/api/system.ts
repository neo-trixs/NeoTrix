import { errText, ApiError } from './client'
import { call as domainCall } from './domain'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { isTauriRuntime } from '../lib/env'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ProjectTreeItem, ProjectView, UpdateProgress, VoiceTranscript } from './types'

/* ════════════════════════════════════════════
   api/system.ts — 窗口 / 项目树 / 文件 / 语音 / 开机自启 / 更新事件
   chat-first 迁移后：
   - 窗口操作走 @tauri-apps/api/window（前端原生，无需后端命令）
   - 文件/项目走 file 域插件；开机自启走 autostart 域插件
   ════════════════════════════════════════════ */

function requireRuntime(): void {
  if (!isTauriRuntime()) {
    throw new ApiError('此功能仅在桌面宿主可用')
  }
}

/* ── 窗口（前端原生） ── */
export function windowMinimize(): Promise<void> {
  requireRuntime()
  return getCurrentWindow().minimize()
}

export function windowMaximize(): Promise<void> {
  requireRuntime()
  return getCurrentWindow().toggleMaximize()
}

export function windowClose(): Promise<void> {
  requireRuntime()
  return getCurrentWindow().close()
}

/* ── 项目 / 文件（file 域插件） ── */
export function projectTree(): Promise<ProjectView> {
  return domainCall<ProjectView>('file', 'tree', {})
}

export function readFile(path: string): Promise<string> {
  return domainCall<string>('file', 'read', { path })
}

export function writeFile(path: string, content: string): Promise<void> {
  return domainCall<void>('file', 'write', { path, content })
}

/* ── 语音 ── */
export function voiceGetTranscription(
  _audioData: string,
  _language?: string,
  _model?: string,
): Promise<VoiceTranscript> {
  // 后端 voice_agent 域只有会话/监听/合成动作，无离线转写端点；
  // 保留签名以兼容调用方，明确抛出未实现而非未知命令错误。
  return Promise.reject(new ApiError('语音转写后端未实现', 'NOT_IMPLEMENTED'))
}

/* ── 开机自启（autostart 域插件） ── */
export async function autostartIsEnabled(): Promise<boolean> {
  const r = await domainCall<{ enabled: boolean }>('autostart', 'is_enabled')
  return r.enabled
}

export function autostartEnable(): Promise<void> {
  return domainCall<void>('autostart', 'enable')
}

export function autostartDisable(): Promise<void> {
  return domainCall<void>('autostart', 'disable')
}

export async function autostartToggle(): Promise<boolean> {
  const r = await domainCall<{ enabled: boolean }>('autostart', 'toggle')
  return r.enabled
}

/* ── 更新事件监听（热更新进度） ── */
export interface UpdateEventHandlers {
  onProgress?: (p: UpdateProgress) => void
  onDownloaded?: () => void
  onError?: (msg: string) => void
}

/** 订阅更新进度事件，返回取消函数。任一 listen 被拒绝时释放已注册的部分监听器，避免泄漏 */
export async function listenUpdateEvents(handlers: UpdateEventHandlers): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = []

  // 🟡 修复：listen 可能逐个失败（如事件名未注册）。原实现中第二个 listen 抛错时
  // 第一个监听器永久泄漏（unlisteners[0] 无人取消）。现在逐个 try/catch：
  // 失败即释放已注册部分并重新抛出；onError 回调同步接线。
  try {
    if (handlers.onProgress) {
      unlisteners.push(await listen<UpdateProgress>('neotrix_update_progress', (e) => handlers.onProgress?.(e.payload)))
    }
    if (handlers.onDownloaded) {
      unlisteners.push(await listen('neotrix_update_downloaded', () => handlers.onDownloaded?.()))
    }
  } catch (e) {
    for (const un of unlisteners) un()
    handlers.onError?.(errText(e))
    throw e
  }

  return () => {
    for (const un of unlisteners) un()
  }
}

export type { ProjectTreeItem }
