// widgets/canvas-viewer/model — 画布视图模型
export type CanvasNode = { id: string; x: number; y: number; label: string }
export type CanvasEdge = { from: string; to: string }

export function layoutNodes(nodes: CanvasNode[], gap = 120): CanvasNode[] {
  return nodes.map((n, i) => ({ ...n, x: (i % 4) * gap, y: Math.floor(i / 4) * gap }))
}

export function bounds(nodes: CanvasNode[]): { minX: number; minY: number; maxX: number; maxY: number } {
  if (nodes.length === 0) return { minX: 0, minY: 0, maxX: 0, maxY: 0 }
  return {
    minX: Math.min(...nodes.map(n => n.x)),
    minY: Math.min(...nodes.map(n => n.y)),
    maxX: Math.max(...nodes.map(n => n.x)),
    maxY: Math.max(...nodes.map(n => n.y)),
  }
}
