import { createSignal, onCleanup, onMount } from 'solid-js'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { clsx } from 'clsx'

/**
 * 自绘 macOS 交通灯（红/黄/绿窗口控制点）。
 * 生产窗口使用 decorations:false，原生交通灯被隐藏，由本组件自绘并接入窗口控制命令。
 * 补齐 macOS 原生语义：
 *  - 窗口失焦时三灯变灰（macOS 标准行为）
 *  - hover 时显示符号（红× / 黄− / 绿+）
 *  - 双击绿色灯最大化/还原
 */
export function TrafficLights() {
  const [focused, setFocused] = createSignal(true)

  onMount(async () => {
    try {
      const win = getCurrentWindow()
      setFocused(await win.isFocused())
      const un = await win.onFocusChanged(({ payload: isFocused }) => {
        setFocused(isFocused)
      })
      onCleanup(un)
    } catch {
      /* 非 Tauri 环境（测试/浏览器）静默 */
    }
  })

  const minimize = () => invoke('window_minimize')
  const maximize = () => invoke('window_maximize')
  const close = () => invoke('window_close')

  return (
    <div class={clsx('traffic', !focused() && 'blurred')} data-tauri-drag-region>
      <button
        class="t-dot t-c"
        onClick={close}
        aria-label="关闭窗口"
        title="关闭"
      >
        <svg viewBox="0 0 12 12" class="t-sym">
          <line x1="3.5" y1="3.5" x2="8.5" y2="8.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          <line x1="8.5" y1="3.5" x2="3.5" y2="8.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        </svg>
      </button>
      <button
        class="t-dot t-m"
        onClick={minimize}
        aria-label="最小化"
        title="最小化"
      >
        <svg viewBox="0 0 12 12" class="t-sym">
          <line x1="3" y1="6" x2="9" y2="6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      </button>
      <button
        class="t-dot t-x"
        onDblClick={(e) => { e.preventDefault(); maximize() }}
        onClick={maximize}
        aria-label="最大化"
        title="最大化"
      >
        <svg viewBox="0 0 12 12" class="t-sym">
          <line x1="3" y1="6" x2="9" y2="6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
          <line x1="6" y1="3" x2="6" y2="9" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  )
}
