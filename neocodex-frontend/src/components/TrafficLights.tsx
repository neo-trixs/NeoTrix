import { createSignal, onCleanup, onMount } from 'solid-js'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { system } from '../api'
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

  const minimize = () => system.windowMinimize()
  const maximize = () => system.windowMaximize()
  const close = () => system.windowClose()

  // 双击防抖：window_maximize 是 toggle 语义，双击会触发 click+click+dblclick 共 3 次 toggle，
  // 净效果正确但产生闪烁。改用标准 click-timer：单击延迟 250ms 判断是否双击，
  // 250ms 内无第二次点击才执行 toggle；双击则取消待执行的单击并在第二次点击时只 toggle 一次。
  let maxTimer: ReturnType<typeof setTimeout> | null = null

  const handleMaximizeClick = () => {
    if (maxTimer !== null) {
      clearTimeout(maxTimer)
      maxTimer = null
      maximize()
      return
    }
    maxTimer = setTimeout(() => {
      maxTimer = null
      maximize()
    }, 250)
  }

  onCleanup(() => {
    if (maxTimer !== null) {
      clearTimeout(maxTimer)
      maxTimer = null
    }
  })

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
        onDblClick={(e) => e.preventDefault()}
        onClick={handleMaximizeClick}
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
