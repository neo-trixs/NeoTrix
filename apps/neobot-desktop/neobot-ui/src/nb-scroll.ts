/**
 * 溢出提示 —— 「下面还有内容」的可见 affordance。
 *
 * # 为什么需要 JS（纯 CSS 做不到）
 *
 * 「内容是否超出容器」是**布局事实**，CSS 选择器无法查询。
 * 且不能用 `overflow: auto` 的默认表现兜底：Tauri/macOS 的 WKWebView
 * 用**覆盖式滚动条**，不滚动时完全不可见 ⇒ 溢出对用户**无声**。
 *
 * # 为什么不引虚拟化 / 第三方滚动条库
 *
 * · 虚拟化（TanStack/virtual 等）：会话量级下全量渲染尚未实测成瓶颈，
 *   先加依赖不合算（商用路径上每个依赖都要过许可 + 体积两道闸）。
 * · 第三方滚动条库（OverlayScrollbars 等）：同上，且要替换原生行为，
 *   收益（一个渐隐提示）小于引入面。
 * ⇒ 本轮只补**最缺的那一环**：让溢出**可见**。
 *
 * 详见 `nb-scroll.css` 顶部注释（设计来源与取舍）。
 */
import { useCallback, useEffect, useRef, useState } from 'react'

/**
 * 给可滚动容器加「还有更多」渐隐提示。
 *
 * @returns 传给容器的 ref；同时在容器上设 `data-overflow` 属性
 */
export function useOverflowHint<T extends HTMLElement>() {
  const ref = useRef<T | null>(null)
  const [overflowing, setOverflowing] = useState(false)

  const measure = useCallback(() => {
    const el = ref.current
    if (!el) return
    // 容差 1px：避免亚像素导致的抖动（scrollHeight 偶尔差 0.5px）
    setOverflowing(el.scrollHeight - el.clientHeight > 1)
  }, [])

  useEffect(() => {
    const el = ref.current
    if (!el) return
    measure()
    // 内容变化（会话增删、消息追加）与尺寸变化（窗口缩放）都要重量
    const ro = new ResizeObserver(measure)
    ro.observe(el)
    for (const child of Array.from(el.children)) ro.observe(child)
    el.addEventListener('scroll', measure, { passive: true })
    return () => {
      ro.disconnect()
      el.removeEventListener('scroll', measure)
    }
  }, [measure])

  return { ref, overflowing, measure }
}
