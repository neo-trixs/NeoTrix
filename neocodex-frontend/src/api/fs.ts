/* ════════════════════════════════════════════
   api/fs.ts — 文件系统/对话框封装
   封装 @tauri-apps/plugin-dialog + plugin-fs，供组件经统一 api 层调用，
   组件禁止直接 import tauri 插件（对齐 api/client.ts 的 IPC 收敛约定）。
   在非 Tauri 宿主（测试/浏览器）返回 null/抛错，由调用方兜底。
   ════════════════════════════════════════════ */
import { open, save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { isTauriRuntime } from '../lib/env'

export interface SaveDialogOptions {
  defaultPath?: string
  filters?: { name: string; extensions: string[] }[]
}

export interface OpenDialogOptions {
  filters?: { name: string; extensions: string[] }[]
}

/** 弹出保存对话框，返回选中路径（用户取消返回 null；非 Tauri 宿主返回 null） */
export async function saveFileDialog(options: SaveDialogOptions = {}): Promise<string | null> {
  if (!isTauriRuntime()) return null
  const path = await save({
    defaultPath: options.defaultPath,
    filters: options.filters,
  })
  return path ?? null
}

/** 弹出打开文件对话框，返回选中路径（用户取消返回 null；非 Tauri 宿主返回 null） */
export async function openFileDialog(options: OpenDialogOptions = {}): Promise<string | null> {
  if (!isTauriRuntime()) return null
  const path = await open({
    filters: options.filters,
  })
  return typeof path === 'string' ? path : null
}

/** 写文本文件到指定路径（非 Tauri 宿主抛错） */
export async function writeTextFileAt(path: string, contents: string): Promise<void> {
  if (!isTauriRuntime()) throw new Error('文件写入仅在桌面宿主可用')
  await writeTextFile(path, contents)
}
