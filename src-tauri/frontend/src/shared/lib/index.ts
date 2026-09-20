/**
 * shared/lib/ — 工具函数
 *
 * 职责：通用工具（cn, format, env, Tauri bridge）
 * 依赖：无
 */
export { cn } from './cn'
export { isTauriRuntime, safeLocalStorage, storageGet, storageSet } from '../../lib/env'
