/* ════════════════════════════════════════════
   components/settings/settingsIcons.tsx — 设置页共享外扩线条图标
   从 SettingsModal 抽出：Expand/Palette/Info/Plugins/Data/X/Tag。
   供各 section 子组件 + SettingsModal 骨架复用（单一事实源）。
   ════════════════════════════════════════════ */

/* ── 外扩线条图标（open/expand 语义，非内敛） ── */
export function ExpandIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 中央节点 + 四条外射线：向外打开 */}
      <circle cx="8" cy="8" r="1.2" stroke="currentColor" stroke-width="1.3" />
      <line x1="8" y1="3" x2="8" y2="0.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
      <line x1="8" y1="13" x2="8" y2="15.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
      <line x1="3" y1="8" x2="0.5" y2="8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
      <line x1="13" y1="8" x2="15.5" y2="8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
    </svg>
  )
}

export function PaletteIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 色板 + 外扩扇区 */}
      <path d="M8 2.5a5.5 5.5 0 100 11c1.5 0 2-1 1-2-.7-.7-.3-1.5 1-1.5h1.5c1.1 0 2-.9 2-2A5.5 5.5 0 008 2.5z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
      <circle cx="5.5" cy="6.5" r="0.7" fill="currentColor" />
      <circle cx="8" cy="5" r="0.7" fill="currentColor" />
      <circle cx="10.5" cy="6.5" r="0.7" fill="currentColor" />
    </svg>
  )
}

export function InfoIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.2" />
      <line x1="8" y1="7.5" x2="8" y2="11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <circle cx="8" cy="4.8" r="0.7" fill="currentColor" />
    </svg>
  )
}

export function PluginsIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 拼图块 + 外扩射线（插件扩展语义） */}
      <path d="M5 3h4v2.5a1.5 1.5 0 010 3V11H5V8.5a1.5 1.5 0 010-3V3z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
      <line x1="8" y1="1" x2="8" y2="0.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="8" y1="15" x2="8" y2="15.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="1" y1="8" x2="0.5" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="15" y1="8" x2="15.5" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
    </svg>
  )
}

export function DataIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 数据库圆柱 + 外扩箭头（导出语义） */}
      <ellipse cx="8" cy="4" rx="5" ry="2.2" stroke="currentColor" stroke-width="1.2" />
      <path d="M3 4v8c0 1.2 2.2 2.2 5 2.2s5-1 5-2.2V4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="3" y1="8" x2="8" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="13" y1="8" x2="13" y2="3.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="13" y1="3.5" x2="11.5" y2="5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
      <line x1="13" y1="3.5" x2="14.5" y2="5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  )
}

export function XIcon() {
  return (
    <svg viewBox="0 0 12 12" fill="none">
      <line x1="3" y1="3" x2="9" y2="9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="9" y1="3" x2="3" y2="9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
    </svg>
  )
}

export function TagIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 标签签低 + 斜杠孔 + 外扩射线（标签集合语义） */}
      <path d="M2 3.5h8l4 4.5-6 6L2 9V3.5z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
      <circle cx="6.4" cy="6.4" r="0.9" fill="currentColor" stroke="none" />
      <line x1="8" y1="14.5" x2="8" y2="15.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.4" />
      <line x1="15" y1="7" x2="15.5" y2="7" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.4" />
    </svg>
  )
}