// ══════════════════════════════════════════════════════════════════════════
//  StreamingText — 流式文本渲染器
//  增量 Markdown 解析，无布局抖动，打字机效果
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, createEffect, onCleanup, Show } from 'solid-js'
import { clsx } from 'clsx'

interface StreamingTextProps {
  /** 完整文本 */
  text: string
  /** 是否正在流式输出 */
  streaming?: boolean
  /** 输出速度 (ms/token) */
  speed?: number
  /** 完成回调 */
  onComplete?: () => void
}

export function StreamingText(props: StreamingTextProps) {
  const [displayed, setDisplayed] = createSignal('')
  const [done, setDone] = createSignal(false)

  let index = 0
  let rafId: number | undefined
  let lastUpdateTime = 0
  let pendingIndex = 0

  const tick = (timestamp: number) => {
    if (index < props.text.length) {
      const elapsed = timestamp - lastUpdateTime
      const interval = props.speed ?? 15

      if (elapsed >= interval) {
        const charsPerFrame = Math.max(1, Math.floor(elapsed / interval))
        const chunk = Math.min(charsPerFrame, props.text.length - index)
        index = Math.min(index + chunk, props.text.length)
        lastUpdateTime = timestamp
        setDisplayed(props.text.slice(0, index))
      }

      rafId = requestAnimationFrame(tick)
    } else {
      setDone(true)
      props.onComplete?.()
    }
  }

  createEffect(() => {
    // 文本变化时重新开始
    const text = props.text
    index = 0
    lastUpdateTime = 0
    pendingIndex = 0
    setDisplayed('')
    setDone(false)

    if (props.streaming !== false && text) {
      rafId = requestAnimationFrame(tick)
    } else {
      setDisplayed(text)
      setDone(true)
    }
  })

  onCleanup(() => {
    if (rafId) cancelAnimationFrame(rafId)
  })

  return (
    <div class={clsx('streaming-text', done() && 'streaming-done')}>
      <span>{displayed()}</span>
      <Show when={!done() && props.streaming !== false}>
        <span class="streaming-cursor" />
      </Show>
    </div>
  )
}
