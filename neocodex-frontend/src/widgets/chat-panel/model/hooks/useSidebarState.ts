/**
 * useSidebarState — 侧边栏折叠 + 拖拽宽度
 */
import { createSignal } from 'solid-js'

export function useSidebarState() {
  const [sidebarCollapsed, setSidebarCollapsed] = createSignal(false)
  const [sidebarWidth, setSidebarWidth] = createSignal(280)

  let sbResizing = false
  const onSidebarResizeDown = (e: MouseEvent) => {
    e.preventDefault()
    sbResizing = true
    const onMove = (ev: MouseEvent) => {
      if (!sbResizing) return
      setSidebarWidth(Math.min(460, Math.max(200, ev.clientX)))
    }
    const onUp = () => {
      sbResizing = false
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
  }

  return {
    sidebarCollapsed, setSidebarCollapsed,
    sidebarWidth, setSidebarWidth,
    onSidebarResizeDown,
  }
}
