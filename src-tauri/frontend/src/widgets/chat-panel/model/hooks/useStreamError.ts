/**
 * useStreamError — 流式错误状态 + 错误详情展示
 */
import { createSignal } from 'solid-js'

export function useStreamError() {
  const [streamError, setStreamError] = createSignal<string | null>(null)
  const [showErrRaw, setShowErrRaw] = createSignal(false)
  const [streamErrorDetail, setStreamErrorDetail] = createSignal<{
    what: string; why: string; next: string
  } | null>(null)

  return {
    streamError, setStreamError,
    showErrRaw, setShowErrRaw,
    streamErrorDetail, setStreamErrorDetail,
  }
}
