/* ════════════════════════════════════════════
   lib/usePolling.ts — 统一轮询 hook（I-1）
   对照 osaurus 单 Loop Driver 经验：集中轮询生命周期，
   统一 in-flight 去重 + 页面不可见守卫 + cleanup + enabled 守卫。
   消除 Chat(15s)/CostDashboard(5s)/CoworkView(10s)/ScheduledTasks(10s)
   四处手写 setInterval 重复样板。
   ════════════════════════════════════════════ */
import { createEffect, onCleanup } from 'solid-js'

interface UsePollingOptions {
  /** 轮询开关守卫：返回 false 时不启动轮询（如面板未打开） */
  enabled?: () => boolean
  /** 轮询间隔毫秒 */
  intervalMs: number
  /** 每 tick 执行的动作；返回 Promise 时自动做 in-flight 去重 */
  run: () => void | Promise<void>
  /** 是否启用页面不可见守卫（默认 true）：document.hidden 时跳过 tick */
  visibilityGuard?: boolean
  /** 是否在启动时立即执行一次 run（默认 false；等价于 onMount 里的首次调用） */
  immediate?: boolean
}

/**
 * 统一轮询生命周期 hook。
 * - 仅当 enabled() 为真时创建 interval；enabled 翻转为假时自动清理。
 * - run 返回 Promise 时，重叠 tick 自动跳过（in-flight 去重）。
 * - 默认在 document.hidden 时跳过 tick（省电量 + 避免后台空转）。
 * - 组件卸载时自动 clearInterval。
 */
export function usePolling(opts: UsePollingOptions) {
  const visibilityGuard = opts.visibilityGuard ?? true

  createEffect(() => {
    if (opts.enabled && !opts.enabled()) return

    let inFlight = false
    let timer: ReturnType<typeof setInterval> | undefined

    const tick = () => {
      if (visibilityGuard && document.visibilityState === 'hidden') return
      if (inFlight) return
      const result = opts.run()
      if (result && typeof (result as Promise<void>).then === 'function') {
        inFlight = true
        ;(result as Promise<void>).finally(() => {
          inFlight = false
        })
      }
    }

    if (opts.immediate) tick()
    timer = setInterval(tick, opts.intervalMs)
    onCleanup(() => clearInterval(timer))
  })
}