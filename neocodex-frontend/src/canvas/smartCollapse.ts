// ══════════════════════════════════════════════════════════════════════════
//  Smart Canvas — 智能收缩 (Smart Collapse)
//  三轴驱动：Salience / Viewport / Redundancy（熔炼自 PMX 空间策展、
//  Jetro C2 通道、supacanvas 密度管理）。返回"应折叠节点 id 集合"。
// ══════════════════════════════════════════════════════════════════════════
import type { CanvasNode, CollapsePolicy, Viewport } from './types'

/** 稳定哈希：kind + 载荷（去 createdAt/salience 等易变字段）。 */
export function hashNode(node: CanvasNode): string {
  const payload = JSON.stringify({ k: node.kind, d: node.data, t: node.title })
  // 轻量 FNV-1a
  let h = 0x811c9dc5
  for (let i = 0; i < payload.length; i++) {
    h ^= payload.charCodeAt(i)
    h = Math.imul(h, 0x01000193)
  }
  return (h >>> 0).toString(36)
}

function inViewport(node: CanvasNode, vp: Viewport): boolean {
  const w = node.w ?? 300
  const h = node.h ?? 200
  return !(node.x + w < vp.x || node.x > vp.x + vp.w || node.y + h < vp.y || node.y > vp.y + vp.h)
}

/**
 * 计算应折叠节点集合。
 * - 用户显式 collapsed 优先。
 * - salienceFloor：显著性低于阈值。
 * - cullOffscreen：完全在视口外 → culling（不挂载 DOM）。
 * - collapseDuplicates：首个之外同哈希节点折叠为一条 sticky cluster。
 */
export function computeCollapse(
  nodes: CanvasNode[],
  policy: CollapsePolicy,
  vp: Viewport,
): Set<string> {
  const collapsed = new Set<string>()
  const seen = new Map<string, string>() // hash -> 首个 id

  for (const n of nodes) {
    if (n.collapsed) {
      collapsed.add(n.id)
      continue
    }
    const sal = n.salience ?? 0.5
    if (policy.salienceFloor > 0 && sal < policy.salienceFloor) {
      collapsed.add(n.id)
      continue
    }
    if (policy.cullOffscreen && !inViewport(n, vp)) {
      collapsed.add(n.id)
      continue
    }
    if (policy.collapseDuplicates) {
      const key = hashNode(n)
      const first = seen.get(key)
      if (first && first !== n.id) {
        collapsed.add(n.id)
        continue
      }
      seen.set(key, n.id)
    }
  }
  return collapsed
}

/** 默认策略：仅 salience 轴开启（保守，避免误伤）。 */
export const DEFAULT_POLICY: CollapsePolicy = {
  salienceFloor: 0,
  cullOffscreen: true,
  collapseDuplicates: false,
}
