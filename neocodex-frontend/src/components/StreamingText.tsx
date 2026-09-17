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
  let timer: ReturnType<typeof setTimeout> | undefined

  const tick = () => {
    if (index < props.text.length) {
      // 每次追加 1-3 个字符，模拟自然打字节奏
      const chunk = Math.random() > 0.7 ? 2 : 1
      index = Math.min(index + chunk, props.text.length)
      setDisplayed(props.text.slice(0, index))
      timer = setTimeout(tick, props.speed ?? 15)
    } else {
      setDone(true)
      props.onComplete?.()
    }
  }

  createEffect(() => {
    // 文本变化时重新开始
    const text = props.text
    index = 0
    setDisplayed('')
    setDone(false)

    if (props.streaming !== false && text) {
      tick()
    } else {
      setDisplayed(text)
      setDone(true)
    }
  })

  onCleanup(() => {
    if (timer) clearTimeout(timer)
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
