// ══════════════════════════════════════════════════════════════════════════
//  Smart Canvas — 节点存储（前端态；未来经 neocodex IPC 接后端 agent 结果流）
//  种子演示 11 个格式族 → 证明"无限制类型"由能力网覆盖。
// ══════════════════════════════════════════════════════════════════════════
import { createSignal } from 'solid-js'
import type { CanvasNode } from '../canvas/types'

let seq = 0
const nid = () => `node-${Date.now().toString(36)}-${seq++}`

function seed(): CanvasNode[] {
  const base: CanvasNode[] = [
    {
      id: nid(), kind: 'kpi', title: '上下文占用', x: 20, y: 20, salience: 0.9,
      data: { value: '63%', label: 'context usage', delta: '↓ 12%' }, source: 'agent_status',
    },
    {
      id: nid(), kind: 'chart', title: 'Token 趋势', x: 260, y: 20, salience: 0.8,
      data: {
        chart: 'line', labels: ['m1', 'm2', 'm3', 'm4'],
        series: [{ name: 'in', values: [12, 19, 14, 24] }, { name: 'out', values: [8, 11, 9, 17] }],
      }, source: 'cost_dashboard',
    },
    {
      id: nid(), kind: 'table', title: '搜索结果', x: 20, y: 200, salience: 0.7,
      data: [
        ['rank', 'title', 'score'],
        ['1', 'tldraw', '0.91'],
        ['2', 'Excalidraw', '0.88'],
        ['3', 'Quickdraw', '0.84'],
      ], source: 'world_search',
    },
    {
      id: nid(), kind: 'code', title: 'snippet.rs', x: 420, y: 200, salience: 0.6,
      data: 'pub fn main() {\n    println!("hi");\n}',
    },
    {
      id: nid(), kind: 'diff', title: 'patch', x: 420, y: 360, salience: 0.5,
      data: '- old_line()\n+ new_line()\n  keep()',
    },
    {
      id: nid(), kind: 'json', title: 'payload', x: 700, y: 20, salience: 0.4,
      data: { a: 1, b: [2, 3, { c: 'x' }], d: { e: true } },
    },
    {
      id: nid(), kind: 'markdown', title: '说明', x: 700, y: 220, salience: 0.45,
      data: '# 画板\n- 无限制类型\n- 智能收缩',
    },
    {
      id: nid(), kind: 'html', title: 'artifact', x: 700, y: 380, salience: 0.55,
      data: '<div style="padding:8px;font:14px sans-serif">交互卡片 <b>live</b></div>',
    },
    {
      id: nid(), kind: 'image', title: '预览', x: 980, y: 20, salience: 0.4,
      data: { src: 'https://www.gstatic.com/webp/gallery/1.jpg', meta: '512×512 png' },
    },
    {
      id: nid(), kind: 'mermaid', title: 'flow', x: 980, y: 220, salience: 0.5,
      data: 'graph TD; A-->B; B-->C;',
    },
    {
      id: nid(), kind: 'heatmap', title: 'matrix', x: 980, y: 400, salience: 0.3,
      data: { cells: [[1, 3, 2], [4, 1, 5], [2, 6, 1]], max: 6 },
    },
    // 未知类型 → 走 '*' 兜底渲染器（无限制类型的硬保证）
    {
      id: nid(), kind: 'unknown-future-format', title: '未知类型', x: 1240, y: 20, salience: 0.2,
      data: { anything: '任意结构' },
    },
  ]
  return base
}

const [nodes, setNodes] = createSignal<CanvasNode[]>(seed())

export const canvasStore = {
  get nodes() {
    return nodes()
  },
  setNodes,
  /** 由 agent/工具结果派生新节点（spawn）。 */
  spawn(n: Partial<CanvasNode> & { kind: string; data: unknown }) {
    setNodes((cur) => [...cur, { id: nid(), x: 0, y: 0, salience: 0.5, ...n } as CanvasNode])
  },
  setCollapsed(id: string, v: boolean) {
    setNodes((cur) => cur.map((n) => (n.id === id ? { ...n, collapsed: v } : n)))
  },
}
