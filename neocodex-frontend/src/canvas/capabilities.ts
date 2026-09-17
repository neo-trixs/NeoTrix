// ══════════════════════════════════════════════════════════════════════════
//  Capabilities — 极简能力注册表
//  前端只做两件事：对话 + 展示结果。
//  能力逻辑全在后端，前端只渲染核心组件。
// ══════════════════════════════════════════════════════════════════════════

export interface Capability {
  kind: string
  label: string
  icon: string
  route?: string
  category: 'route' | 'canvas'
}

/**
 * 极简能力表
 * - route: 全屏路由页面（设置/知识库/记忆等）
 * - canvas: 画布内嵌节点（对话产生的结果展示）
 */
export const CAPABILITIES: Capability[] = [
  // ── 路由：全屏页面 ──
  { kind: 'route-chat', label: '对话', icon: '💬', route: '/chat', category: 'route' },
  { kind: 'route-settings', label: '设置', icon: '⚙️', route: '/settings', category: 'route' },
  { kind: 'route-activity', label: '活动', icon: '📋', route: '/activity', category: 'route' },
  { kind: 'route-globe', label: '地球', icon: '🌐', route: '/globe', category: 'route' },
  { kind: 'route-kb', label: '知识库', icon: '🧠', route: '/kb', category: 'route' },
  { kind: 'route-skills', label: '技能', icon: '⚡', route: '/skills', category: 'route' },
  { kind: 'route-memory', label: '记忆', icon: '💾', route: '/memory', category: 'route' },
  { kind: 'route-workflows', label: '工作流', icon: '🔄', route: '/workflows', category: 'route' },
  { kind: 'route-insights', label: '洞察', icon: '🔍', route: '/insights', category: 'route' },
  { kind: 'route-plugins', label: '插件市场', icon: '🧩', route: '/plugins', category: 'route' },
  { kind: 'route-knowledge-graph', label: '知识图谱', icon: '🌐', route: '/knowledge-graph', category: 'route' },
  { kind: 'route-context-dashboard', label: '上下文仪表板', icon: '📊', route: '/context-dashboard', category: 'route' },

  // ── 画布：对话产生的结果节点 ──
  // Agent 执行流
  { kind: 'flow', label: '执行流程', icon: '🔗', category: 'canvas' },
  { kind: 'plan', label: '执行计划', icon: '📋', category: 'canvas' },
  { kind: 'activity', label: '活动流', icon: '📋', category: 'canvas' },
  // 数据展示
  { kind: 'code', label: '代码', icon: '💻', category: 'canvas' },
  { kind: 'table', label: '表格', icon: '📊', category: 'canvas' },
  { kind: 'chart', label: '图表', icon: '📈', category: 'canvas' },
  { kind: 'image', label: '图像', icon: '🖼️', category: 'canvas' },
  { kind: 'markdown', label: '文档', icon: '📄', category: 'canvas' },
  { kind: 'mermaid', label: '流程图', icon: '🔗', category: 'canvas' },
  { kind: 'json', label: 'JSON', icon: '{ }', category: 'canvas' },
  { kind: 'diff', label: '差异', icon: '🔀', category: 'canvas' },
]

export function getCapability(kind: string): Capability | undefined {
  return CAPABILITIES.find(c => c.kind === kind)
}

export function getCapabilitiesByCategory(category: Capability['category']): Capability[] {
  return CAPABILITIES.filter(c => c.category === category)
}

export function getRouteCapabilities(): Capability[] {
  return CAPABILITIES.filter(c => c.route)
}

export function getCanvasCapabilities(): Capability[] {
  return CAPABILITIES.filter(c => c.category === 'canvas')
}
