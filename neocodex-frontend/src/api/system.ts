import { call } from './client'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ProjectTreeItem, ProjectView, UpdateProgress, VoiceTranscript } from './types'

/* ════════════════════════════════════════════
   api/system.ts — 窗口 / 项目树 / 文件 / 语音 / 更新事件
   对应 desktop_cmds.rs / project_cmds.rs / voice_cmds.rs / neocodex_cmds.rs(update)
   ════════════════════════════════════════════ */

/* ── 窗口 ── */
export function windowMinimize(): Promise<void> {
  return call('window_minimize', {})
}

export function windowMaximize(): Promise<void> {
  return call('window_maximize', {})
}

export function windowClose(): Promise<void> {
  return call('window_close', {})
}

/* ── 项目 / 文件 ── */
export function projectTree(): Promise<ProjectView> {
  return call('neocodex_project_tree', {})
}

export function readFile(path: string): Promise<string> {
  return call('read_file', { path })
}

/* ── 语音 ── */
export function voiceGetTranscription(audioData: string, language?: string, model?: string): Promise<VoiceTranscript> {
  return call('voice_get_transcription', {
    audioData,
    language: language ?? null,
    model: model ?? null,
  })
}

/* ── 更新事件监听（热更新进度） ── */
export interface UpdateEventHandlers {
  onProgress?: (p: UpdateProgress) => void
  onDownloaded?: () => void
  onError?: (msg: string) => void
}

/** 订阅更新进度事件，返回取消函数。注意：listen 调用本身可能被拒绝，错误回调会收到 { type: 'listen_error' } 之外的消息 */
export async function listenUpdateEvents(handlers: UpdateEventHandlers): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = []

  if (handlers.onProgress) {
    unlisteners.push(await listen<UpdateProgress>('neocodex_update_progress', (e) => handlers.onProgress?.(e.payload)))
  }
  if (handlers.onDownloaded) {
    unlisteners.push(await listen('neocodex_update_downloaded', () => handlers.onDownloaded?.()))
  }

  return () => {
    for (const un of unlisteners) un()
  }
}

export type { ProjectTreeItem }
