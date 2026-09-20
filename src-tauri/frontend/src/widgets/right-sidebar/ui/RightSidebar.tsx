// widgets/right-sidebar/ui/RightSidebar.tsx — 因果链画板（FSD 正确位置）
import { createSignal, createMemo } from 'solid-js'
import { clsx } from 'clsx'
import { CausalMap } from '../../../components/CausalMap'
import type { CausalNode } from '../../../components/CausalMap'
import { chatStore } from '../../../stores/chat'

function extractCausalNodes(): CausalNode[] {
  const messages = chatStore.currentMessages
  const nodes: CausalNode[] = []
  for (const msg of messages) {
    if (msg.role === 'user') {
      nodes.push({
        id: msg.id,
        kind: 'input',
        title: '你的请求',
        summary: msg.content.slice(0, 120) + (msg.content.length > 120 ? '…' : ''),
        timestamp: msg.timestamp,
      })
    }
    if (msg.role === 'assistant' && !msg.isStreaming) {
      nodes.push({
        id: msg.id,
        kind: 'output',
        title: 'AI 回复',
        summary: msg.content.slice(0, 120) + (msg.content.length > 120 ? '…' : ''),
        content: msg.content,
        timestamp: msg.timestamp,
        result_type: 'text',
      })
    }
    if (msg.toolCalls && msg.toolCalls.length > 0) {
      for (const tc of msg.toolCalls) {
        nodes.push({
          id: tc.id,
          kind: 'execute',
          title: `工具调用：${tc.name}`,
          summary: tc.success ? '成功' : '失败',
          timestamp: msg.timestamp,
          duration_ms: tc.duration_ms,
        })
      }
    }
  }
  return nodes
}

export function RightSidebar() {
  const [collapsed, setCollapsed] = createSignal(false)
  const [autoHide, setAutoHide] = createSignal(true)
  const [rbHover, setRbHover] = createSignal(false)
  const nodes = createMemo(() => extractCausalNodes())
  const toggleRb = () => {
    if (autoHide()) { setAutoHide(false); setCollapsed(false); return }
    setCollapsed(!collapsed())
  }
  const handleApprove = (id: string) => console.log('[RightSidebar] Approve:', id)
  const handleReject = (id: string, reason?: string) => console.log('[RightSidebar] Reject:', id, reason)
  const handleEdit = (id: string, content: string) => console.log('[RightSidebar] Edit:', id)
  return (
    <aside
      class={clsx('rb h-screen flex-shrink-0', autoHide() && 'auto-hide', rbHover() && 'rb-hover', !autoHide() && collapsed() && 'collapsed')}
      onMouseEnter={() => autoHide() && setRbHover(true)}
      onMouseLeave={() => autoHide() && setRbHover(false)}
    >
      <button class="rb-float" onClick={toggleRb} title="切换侧栏" aria-label="切换侧栏">
        <svg viewBox="0 0 8 8"><line x1="5" y1="2" x2="3" y2="4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" /><line x1="5" y1="6" x2="3" y2="4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" /></svg>
      </button>
      <div class="rb-content"><CausalMap nodes={nodes()} onApprove={handleApprove} onReject={handleReject} onEdit={handleEdit} /></div>
    </aside>
  )
}
