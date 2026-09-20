/* ════════════════════════════════════════════
   routes/chat/avatars.tsx — Chat 头像/标记图标（从 Chat.tsx 抽出）
   纯展示组件：无状态、无副作用，供消息气泡与空态复用。
   ════════════════════════════════════════════ */

/** 设计 v2 图标：E8 六芒星（hero / 空态标记） */
export function HeroMark() {
  return (
    <svg viewBox="0 0 32 32" fill="none">
      <path d="M16 2l4 8 8 4-8 4-4 8-4-8-8-4 8-4 4-8z" fill="#E85454" opacity="0.25" />
      <path d="M16 6l2.5 5 5.5 2.5-5.5 2.5-2.5 5-2.5-5L8 13.5l5.5-2.5 2.5-5z" fill="#E85454" />
      <circle cx="16" cy="13.5" r="2.5" fill="#E85454" stroke="none" />
      <circle cx="16" cy="13.5" r="1" fill="#fff" stroke="none" />
      <path d="M4 20q4-4 8 0t8-8 8 4" stroke="#D04040" stroke-width="0.8" stroke-linecap="round" opacity="0.4" fill="none" />
    </svg>
  )
}

/** 设计 v2 头像图标：用户（人形） */
export function UserIcon() {
  return (
    <svg viewBox="0 0 14 14">
      <circle cx="7" cy="4.5" r="2.5" stroke="currentColor" stroke-width="1.2" fill="none" />
      <path d="M2 12.5a5 5 0 0110 0" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" />
    </svg>
  )
}

/** 设计 v2 头像图标：助手（方框·意识） */
export function BotIcon() {
  return (
    <svg viewBox="0 0 14 14">
      <rect x="2" y="3" width="10" height="8" rx="1.5" stroke="currentColor" stroke-width="1.2" fill="none" />
      <circle cx="7" cy="7" r="1.5" stroke="currentColor" stroke-width="1" fill="none" />
    </svg>
  )
}
