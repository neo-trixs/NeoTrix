// ══════════════════════════════════════════════════════════════════════════
//  Smart Canvas — 核心类型（能力网基座）
//  设计原则：类型开放（NodeKind = string），无枚举天花板 → "无限制类型"
//  每个 node 自带 salience（驱动智能收缩），source（provenance，对齐吸收纪律）
// ══════════════════════════════════════════════════════════════════════════

/** 节点类型：开放字符串。注册表未知类型永远走 '*' 兜底渲染器。 */
export type NodeKind = string

/** 画板节点：一个结果可视化单元（对齐 PMX "every node carries its own renderer"）。 */
export interface CanvasNode {
  id: string
  kind: NodeKind
  title?: string
  /** 任意载荷；由对应 kind 的渲染器解释。 */
  data: unknown
  /** 空间坐标（画布世界坐标，非屏幕像素）。 */
  x: number
  y: number
  /** 建议尺寸（自动布局用）；缺省取渲染器 defaultSize。 */
  w?: number
  h?: number
  /** 显著性 0..1：驱动智能收缩。缺省 0.5。 */
  salience?: number
  /** 显式折叠（用户手动）。 */
  collapsed?: boolean
  tags?: string[]
  /** 产出来源（agent / 工具 / 会话）：对齐 supacanvas provenance。 */
  source?: string
  createdAt?: number
}

/** 渲染上下文：渲染器可请求折叠 / 派生新节点（agent 现场构建视图）。 */
export interface NodeRenderContext {
  node: CanvasNode
  setCollapsed: (v: boolean) => void
  /** 由当前节点派生一个同画板新节点（空间偏移避免重叠）。 */
  spawn: (partial: Partial<CanvasNode> & { kind: NodeKind; data: unknown }) => void
}

/** 节点渲染器 = 能力网中的一个"能力节点"（可随 SEAL 吸收新增）。 */
export interface NodeRenderer {
  /** 适用类型：具体 kind 列表，或 '*' 兜底。 */
  kind: NodeKind | NodeKind[]
  label: string
  /** 渲染为 SolidJS JSX（返回 Element）。 */
  render: (ctx: NodeRenderContext) => unknown
  /** 自动布局建议尺寸。 */
  defaultSize?: { w: number; h: number }
  /** 自检钩子（T1）：纯函数校验载荷是否可渲染；失败则降级兜底。 */
  validate?: (data: unknown) => boolean
}

/** 视口（世界坐标），用于 viewport 轴折叠。 */
export interface Viewport {
  x: number
  y: number
  w: number
  h: number
}

/** 折叠策略：智能收缩三轴开关。 */
export interface CollapsePolicy {
  /** salience 低于此值自动折叠（0 = 关闭）。 */
  salienceFloor: number
  /** 视口外节点做 culling 折叠（不挂载 DOM）。 */
  cullOffscreen: boolean
  /** 同 kind+data 哈希的重复节点折叠为一条。 */
  collapseDuplicates: boolean
}
