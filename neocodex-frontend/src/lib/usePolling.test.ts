import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { createRoot, createSignal } from 'solid-js'
import { usePolling } from './usePolling'

describe('lib/usePolling — 统一轮询 hook', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.restoreAllMocks()
  })

  const mount = (opts: Parameters<typeof usePolling>[0]) => {
  return new Promise<() => void>((resolve) => {
    createRoot((dispose) => {
      usePolling(opts)
      resolve(dispose)
    })
  })
}

  it('按 intervalMs 周期性触发 run', async () => {
    const run = vi.fn()
    await mount({ intervalMs: 5000, run })
    await vi.advanceTimersByTimeAsync(15000)
    expect(run).toHaveBeenCalledTimes(3)
  })

  it('enabled=false 时不启动轮询', async () => {
    const run = vi.fn()
    const [on, setOn] = createSignal(false)
    await mount({ enabled: on, intervalMs: 5000, run })
    await vi.advanceTimersByTimeAsync(15000)
    expect(run).not.toHaveBeenCalled()
    setOn(true)
    await vi.advanceTimersByTimeAsync(10000)
    expect(run).toHaveBeenCalledTimes(2)
  })

  it('run 返回 Promise 时做 in-flight 去重（重叠 tick 跳过）', async () => {
    let resolveRun: (() => void) | undefined
    const run = vi.fn(() => new Promise<void>((r) => { resolveRun = r }))
    await mount({ intervalMs: 5000, run })
    await vi.advanceTimersByTimeAsync(5000) // 第一次 tick，in-flight
    await vi.advanceTimersByTimeAsync(5000) // 第二次 tick 被跳过
    expect(run).toHaveBeenCalledTimes(1)
    resolveRun!()
    await vi.advanceTimersByTimeAsync(5000) // in-flight 释放后恢复
    expect(run).toHaveBeenCalledTimes(2)
  })

  it('visibilityState=hidden 时跳过 tick', async () => {
    const run = vi.fn()
    Object.defineProperty(document, 'visibilityState', { value: 'hidden', configurable: true })
    await mount({ intervalMs: 5000, run })
    await vi.advanceTimersByTimeAsync(10000)
    expect(run).not.toHaveBeenCalled()
    Object.defineProperty(document, 'visibilityState', { value: 'visible', configurable: true })
    await vi.advanceTimersByTimeAsync(10000)
    expect(run).toHaveBeenCalledTimes(2)
  })

  it('组件卸载时清理 interval', async () => {
    const run = vi.fn()
    const dispose = await mount({ intervalMs: 5000, run })
    await vi.advanceTimersByTimeAsync(10000)
    expect(run).toHaveBeenCalledTimes(2)
    dispose()
    await vi.advanceTimersByTimeAsync(20000)
    expect(run).toHaveBeenCalledTimes(2)
  })

  it('immediate=true 时启动即执行一次 run', async () => {
    const run = vi.fn()
    await mount({ intervalMs: 5000, run, immediate: true })
    await vi.advanceTimersByTimeAsync(0)
    expect(run).toHaveBeenCalledTimes(1)
    await vi.advanceTimersByTimeAsync(5000)
    expect(run).toHaveBeenCalledTimes(2)
  })
})