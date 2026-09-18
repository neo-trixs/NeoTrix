/**
 * useInputState — 输入框 + 草稿持久化
 *
 * 职责：inputValue / textareaRef / auto-resize / per-session draft 读写
 */
import { createSignal, createEffect } from 'solid-js'
import { chatStore } from '../../../../stores/chat'

const DRAFT_KEY = 'nt_session_drafts'

function readDrafts(): Record<string, string> {
  try { return JSON.parse(localStorage.getItem(DRAFT_KEY) || '{}') } catch { return {} }
}

function saveDraft(id: string, text: string) {
  const d = readDrafts()
  if (text) d[id] = text
  else delete d[id]
  try { localStorage.setItem(DRAFT_KEY, JSON.stringify(d)) } catch { /* 隐私模式忽略 */ }
}

export function useInputState() {
  const [inputValue, setInputValue] = createSignal('')

  // 切换会话时恢复草稿
  createEffect(() => {
    const id = chatStore.state.currentSessionId
    setInputValue(id ? (readDrafts()[id] ?? '') : '')
  })

  // 输入时保存草稿
  createEffect(() => {
    const id = chatStore.state.currentSessionId
    if (id) saveDraft(id, inputValue())
  })

  const [textareaRef, setTextareaRef] = createSignal<HTMLTextAreaElement | null>(null)

  const adjustTextarea = () => {
    const textarea = textareaRef()
    if (textarea) {
      textarea.style.height = 'auto'
      textarea.style.height = `${Math.min(textarea.scrollHeight, 200)}px`
    }
  }

  return {
    inputValue, setInputValue,
    textareaRef, setTextareaRef,
    adjustTextarea,
  }
}
