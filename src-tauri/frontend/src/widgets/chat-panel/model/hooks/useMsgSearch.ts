/**
 * useMsgSearch — 消息搜索 + 匹配跳转
 */
import { createSignal } from 'solid-js'
import { chatStore } from '../../../../stores/chat'

export function useMsgSearch() {
  const [msgSearch, setMsgSearch] = createSignal('')
  const [msgSearchOpen, setMsgSearchOpen] = createSignal(false)
  const [msgSearchInput, setMsgSearchInput] = createSignal<HTMLInputElement | null>(null)
  const [matchCursor, setMatchCursor] = createSignal(0)

  const matchCount = () => {
    const q = msgSearch().trim().toLowerCase()
    if (!q) return 0
    return chatStore.currentMessages.filter(
      (m) => m.content.toLowerCase().includes(q),
    ).length
  }

  const matchedIds = () => {
    const q = msgSearch().trim().toLowerCase()
    if (!q) return []
    return chatStore.currentMessages
      .filter((m) => m.content.toLowerCase().includes(q))
      .map((m) => m.id)
  }

  const messageEls = new Map<string, HTMLElement>()

  const jumpMatch = (dir: 1 | -1) => {
    const ids = matchedIds()
    if (!ids.length) return
    const i = (matchCursor() + dir + ids.length) % ids.length
    setMatchCursor(i)
    messageEls.get(ids[i])?.scrollIntoView({ block: 'center', behavior: 'smooth' })
  }

  return {
    msgSearch, setMsgSearch,
    msgSearchOpen, setMsgSearchOpen,
    msgSearchInput, setMsgSearchInput,
    matchCount, matchCursor, setMatchCursor,
    matchedIds, jumpMatch,
    messageEls,
  }
}
