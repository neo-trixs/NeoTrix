/* ════════════════════════════════════════════
   components/PageShortcuts.tsx — ⌘1..7 全局页面切换 (Phase 4)

   吸收 VSCode/Raycast 工作区快捷键模式：
   ⌘1 对话 · ⌘2 知识库 · ⌘3 插件 · ⌘4 洞察
   ⌘5 技能 · ⌘6 记忆 · ⌘7 流程
   必须渲染在 <Router> 内部 (useNavigate 上下文要求)。
   ════════════════════════════════════════════ */
import { onMount, onCleanup, type JSX } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { DeepLinkBridge } from './DeepLinkBridge'

/** 顺序即编号语义 — 新页面追加到尾部，勿插入中间（用户肌肉记忆） */
export const PAGE_PATHS = ['/chat', '/kb', '/plugins', '/insights', '/skills', '/memory', '/workflows']

export function PageShortcuts(props: { children?: JSX.Element }) {
  const navigate = useNavigate()
  const onKey = (e: KeyboardEvent) => {
    if (!(e.metaKey || e.ctrlKey)) return
    if (e.shiftKey || e.altKey) return
    // 输入焦点在文本框时放行数字输入
    const tag = (e.target as HTMLElement | null)?.tagName
    if (tag === 'INPUT' || tag === 'TEXTAREA') return
    const n = Number(e.key)
    if (!Number.isInteger(n) || n < 1 || n > PAGE_PATHS.length) return
    e.preventDefault()
    navigate(PAGE_PATHS[n - 1])
  }
  onMount(() => window.addEventListener('keydown', onKey))
  onCleanup(() => window.removeEventListener('keydown', onKey))
  // 通配路由唯一性: 本组件作为 catch-all 挂载并透传子路由内容
  return (<>{props.children}<DeepLinkBridge /></>)
}
