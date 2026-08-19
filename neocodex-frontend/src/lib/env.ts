/* ════════════════════════════════════════════
   lib/env.ts — 运行环境能力探测（最小收敛层）
   设计决策：不做超前传输抽象（当前仅 Tauri 单一宿主，抽象无消费方
   即死代码）。只收敛真实存在的平台差异判断点：
   - 组件能否安全使用 @tauri-apps/api/*（TrafficLights 的 try/catch 兜底）
   - localStorage 是否可用（tags.ts / TaskList 的兜底）
   未来若出现 web/HTTP 双宿主，再扩展为 transport 抽象。
   ════════════════════════════════════════════ */

/** 是否运行在 Tauri 桌面宿主（window.__TAURI_INTERNALS__ 存在） */
export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

/** localStorage 安全访问：jsdom 等环境可能无 Storage 实现；异常回退 null */
export function safeLocalStorage(): Storage | null {
  try {
    if (typeof window === 'undefined') return null
    const s = window.localStorage
    if (typeof s?.getItem !== 'function') return null
    return s
  } catch {
    return null
  }
}

/** 从 localStorage 读值（不存在/不可用返回 null） */
export function storageGet(key: string): string | null {
  return safeLocalStorage()?.getItem(key) ?? null
}

/** 写 localStorage（不可用静默跳过，不抛错） */
export function storageSet(key: string, value: string): void {
  try {
    safeLocalStorage()?.setItem(key, value)
  } catch {
    /* 静默：非关键路径 */
  }
}