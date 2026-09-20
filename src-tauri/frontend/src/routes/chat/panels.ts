/* ════════════════════════════════════════════
   routes/chat/panels.ts — 面板注册表 (拆分刀 2)
   PanelId / 面板顺序 / 快捷键解析, 与宿主解耦。
   ════════════════════════════════════════════ */

export type PanelId = 'git' | 'tasks' | 'cost' | 'terminal' | 'timeline' | 'sidechat' | 'preview'

/** ⌘1..6 面板切换顺序 (cost 已收敛至 /insights 页, 保留位兼容旧键) */
export const PANEL_ORDER: PanelId[] = ['git', 'cost', 'terminal', 'tasks', 'timeline', 'sidechat', 'preview']

/** 数字快捷键 → 面板; 超界返回 null (调用方决定是否 preventDefault) */
export function resolvePanelShortcut(key: string): PanelId | null {
  const idx = Number(key) - 1
  if (!Number.isInteger(idx) || idx < 0 || idx >= PANEL_ORDER.length) return null
  return PANEL_ORDER[idx]
}
