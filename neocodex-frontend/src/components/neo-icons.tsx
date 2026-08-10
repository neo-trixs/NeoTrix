import { clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'
import type { JSX } from 'solid-js'

/* ════════════════════════════════════════════
   NeoIcons — NeoTrix 特性线条图标集
   图标语言：极简线条 · 外扩（open/expand）而非内敛
   —— 开阔心态：中心节点 + 向外射线（对标 E8 / 外扩语义）
   ════════════════════════════════════════════ */

interface IconProps {
  class?: string
}

const base = (props: IconProps) => ({
  viewBox: '0 0 16 16',
  fill: 'none',
  class: twMerge(clsx('w-4 h-4', props.class)),
} as JSX.GSVGAttributes<SVGSVGElement>)

/** 发送：中心点 + 四外射线 + 右主射线强化（E8 外扩语义） */
export function NeoSend(props: IconProps) {
  return (
    <svg {...base(props)}>
      <circle cx="8" cy="8" r="1.2" stroke="currentColor" stroke-width="1.4" />
      <line x1="8" y1="2.5" x2="8" y2="0.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" opacity="0.5" />
      <line x1="13.5" y1="8" x2="15.5" y2="8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <line x1="8" y1="13.5" x2="8" y2="15.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" opacity="0.5" />
      <line x1="2.5" y1="8" x2="0.5" y2="8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" opacity="0.5" />
      <path d="M8 8l3-3M8 8l3 3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" opacity="0.35" />
    </svg>
  )
}

/** 新建：外扩十字射线 */
export function NeoPlus(props: IconProps) {
  return (
    <svg {...base(props)}>
      <line x1="8" y1="3.5" x2="8" y2="12.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      <line x1="3.5" y1="8" x2="12.5" y2="8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      <line x1="8" y1="1" x2="8" y2="0.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" opacity="0.4" />
      <line x1="8" y1="15.5" x2="8" y2="15" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" opacity="0.4" />
      <line x1="1" y1="8" x2="0.5" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" opacity="0.4" />
      <line x1="15.5" y1="8" x2="15" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" opacity="0.4" />
    </svg>
  )
}

/** 会话/消息：中心消息块 + 外扩射线（对话扩展语义） */
export function NeoMessage(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M3.5 3.5h9v6.5H7.5L4.5 12.5v-2.5h-1v-6.5z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
      <line x1="8" y1="1.5" x2="8" y2="0.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" opacity="0.45" />
      <line x1="8" y1="14.5" x2="8" y2="15.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" opacity="0.45" />
      <line x1="1.5" y1="7" x2="0.5" y2="7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" opacity="0.45" />
      <line x1="15.5" y1="7" x2="14.5" y2="7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" opacity="0.45" />
    </svg>
  )
}

/** 搜索：外扩同心圆（探索语义） */
export function NeoSearch(props: IconProps) {
  return (
    <svg {...base(props)}>
      <circle cx="7" cy="7" r="4" stroke="currentColor" stroke-width="1.3" />
      <line x1="10" y1="10" x2="13.5" y2="13.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      <line x1="7" y1="1.5" x2="7" y2="0.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.4" />
      <line x1="13" y1="4.5" x2="14" y2="3.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.4" />
    </svg>
  )
}

/** 设置：齿轮 + 外扩刻度（配置语义） */
export function NeoGear(props: IconProps) {
  return (
    <svg {...base(props)}>
      <circle cx="8" cy="8" r="2.8" stroke="currentColor" stroke-width="1.3" />
      <path d="M8 2.8v-1M8 14.2v-1M2.8 8h-1M14.2 8h-1M4.3 4.3L3.6 3.6M12.4 12.4l-.7-.7M12.4 3.6l-.7.7M4.3 11.7l-.7.7" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.5" />
    </svg>
  )
}

/** 折叠：外扩右箭头（展开/打开语义） */
export function NeoChevronRight(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M5.5 3l4.5 5-4.5 5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
      <line x1="12.5" y1="3" x2="12.5" y2="13" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.4" />
    </svg>
  )
}

/** 删除：垃圾桶 + 外扩警示线 */
export function NeoTrash(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M4 5h8v7a1 1 0 01-1 1H5a1 1 0 01-1-1V5z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
      <line x1="3" y1="5" x2="13" y2="5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="6" y1="2.5" x2="10" y2="2.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="6.5" y1="7.5" x2="6.5" y2="10" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.6" />
      <line x1="9.5" y1="7.5" x2="9.5" y2="10" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.6" />
    </svg>
  )
}

/** 重命名：铅笔 + 外扩记录线 */
export function NeoPencil(props: IconProps) {
  return (
    <svg {...base(props)}>
      <path d="M3.5 12.5l.8-3.2L10 3.6l2.4 2.4-5.7 5.7-3.2.8z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
      <line x1="9" y1="4.5" x2="11.5" y2="7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" opacity="0.5" />
      <line x1="3" y1="14.5" x2="2" y2="15.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.4" />
    </svg>
  )
}

/** 关闭：外扩 X（取消/关闭语义） */
export function NeoClose(props: IconProps) {
  return (
    <svg {...base(props)}>
      <line x1="4.5" y1="4.5" x2="11.5" y2="11.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      <line x1="11.5" y1="4.5" x2="4.5" y2="11.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      <line x1="8" y1="2.5" x2="8" y2="1" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.4" />
      <line x1="8" y1="15" x2="8" y2="13.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.4" />
    </svg>
  )
}
