// ══════════════════════════════════════════════════════════════════════════
//  KnowledgeGraph — 知识图谱可视化
//  对标：Obsidian Graph View + Roam Research + Heptabase Knowledge Graph
//  语义搜索 + 知识关联 + 维度筛选 + 节点聚类
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, For, Show, onMount, onCleanup, createMemo } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { ArrowLeft, Search, Filter, Maximize2, ZoomIn, ZoomOut, RotateCcw, Sparkles } from 'lucide-solid'
import { clsx } from 'clsx'

interface KnowledgeNode {
  id: string
  label: string
  type: 'concept' | 'skill' | 'memory' | 'session' | 'file'
  domain?: string
  connections: number
  lastAccessed?: number
}

interface KnowledgeEdge {
  source: string
  target: string
  weight: number
  type: 'semantic' | 'reference' | 'derived'
}

/** Mock data — 后端接线后替换 */
const MOCK_NODES: KnowledgeNode[] = [
  { id: '1', label: 'E8 核心', type: 'concept', domain: 'core', connections: 12, lastAccessed: Date.now() - 3600000 },
  { id: '2', label: 'HyperCube', type: 'concept', domain: 'memory', connections: 8, lastAccessed: Date.now() - 7200000 },
  { id: '3', label: 'GWT 注意力路由', type: 'concept', domain: 'core', connections: 15, lastAccessed: Date.now() - 1800000 },
  { id: '4', label: 'VSA 符号', type: 'concept', domain: 'memory', connections: 6, lastAccessed: Date.now() - 86400000 },
  { id: '5', label: 'nt_core_self', type: 'skill', domain: 'core', connections: 10, lastAccessed: Date.now() - 600000 },
  { id: '6', label: 'nt_mind_background', type: 'skill', domain: 'mind', connections: 7, lastAccessed: Date.now() - 14400000 },
  { id: '7', label: 'experience-tree', type: 'skill', domain: 'memory', connections: 9, lastAccessed: Date.now() - 300000 },
  { id: '8', label: 'rev-officer', type: 'skill', domain: 'shield', connections: 5, lastAccessed: Date.now() - 28800000 },
  { id: '9', label: 'des-architect', type: 'skill', domain: 'mind', connections: 4, lastAccessed: Date.now() - 43200000 },
  { id: '10', label: 'dev-implementer', type: 'skill', domain: 'act', connections: 11, lastAccessed: Date.now() - 900000 },
  { id: '11', label: 'CONSCIOUSNESS_TREE', type: 'memory', domain: 'core', connections: 3, lastAccessed: Date.now() - 172800000 },
  { id: '12', label: 'AGENTS.md', type: 'file', connections: 14, lastAccessed: Date.now() - 300000 },
  { id: '13', label: 'CONTEXT.md', type: 'file', connections: 8, lastAccessed: Date.now() - 600000 },
  { id: '14', label: 'knowledge.db', type: 'memory', domain: 'memory', connections: 6, lastAccessed: Date.now() - 1800000 },
  { id: '15', label: 'consciousness_core', type: 'concept', domain: 'core', connections: 13, lastAccessed: Date.now() - 600000 },
]

const MOCK_EDGES: KnowledgeEdge[] = [
  { source: '1', target: '15', weight: 0.9, type: 'semantic' },
  { source: '3', target: '1', weight: 0.8, type: 'semantic' },
  { source: '2', target: '4', weight: 0.7, type: 'reference' },
  { source: '5', target: '1', weight: 0.85, type: 'derived' },
  { source: '6', target: '3', weight: 0.6, type: 'reference' },
  { source: '7', target: '14', weight: 0.9, type: 'semantic' },
  { source: '8', target: '5', weight: 0.5, type: 'reference' },
  { source: '9', target: '2', weight: 0.4, type: 'derived' },
  { source: '10', target: '5', weight: 0.7, type: 'semantic' },
  { source: '11', target: '15', weight: 0.95, type: 'semantic' },
  { source: '12', target: '13', weight: 0.8, type: 'reference' },
  { source: '12', target: '3', weight: 0.6, type: 'derived' },
  { source: '14', target: '2', weight: 0.5, type: 'reference' },
  { source: '15', target: '12', weight: 0.7, type: 'semantic' },
]

const TYPE_COLORS: Record<string, { bg: string; text: string; border: string }> = {
  concept: { bg: 'bg-nt-core-500/15', text: 'text-nt-core-600', border: 'border-nt-core-500/30' },
  skill: { bg: 'bg-nt-io-500/15', text: 'text-nt-io-600', border: 'border-nt-io-500/30' },
  memory: { bg: 'bg-nt-memory-500/15', text: 'text-nt-memory-600', border: 'border-nt-memory-500/30' },
  session: { bg: 'bg-nt-mind-500/15', text: 'text-nt-mind-600', border: 'border-nt-mind-500/30' },
  file: { bg: 'bg-nt-shield-500/15', text: 'text-nt-shield-600', border: 'border-nt-shield-500/30' },
}

const TYPE_ICONS: Record<string, string> = {
  concept: '💡',
  skill: '⚡',
  memory: '🧠',
  session: '💬',
  file: '📄',
}

const DOMAIN_COLORS: Record<string, string> = {
  core: '#6366f1',
  mind: '#f0913a',
  memory: '#14b8a6',
  shield: '#dc2626',
  act: '#8b5cf6',
}

export function KnowledgeGraph() {
  const navigate = useNavigate()
  const [search, setSearch] = createSignal('')
  const [selectedType, setSelectedType] = createSignal<string | null>(null)
  const [selectedDomain, setSelectedDomain] = createSignal<string | null>(null)
  const [selectedNode, setSelectedNode] = createSignal<KnowledgeNode | null>(null)
  const [zoom, setZoom] = createSignal(1)
  const [pan, setPan] = createSignal({ x: 0, y: 0 })
  const [hoveredNode, setHoveredNode] = createSignal<string | null>(null)

  let canvasRef: HTMLCanvasElement | undefined
  let animFrame: number | undefined

  // 筛选节点
  const filteredNodes = createMemo(() => {
    let nodes = MOCK_NODES
    const q = search().toLowerCase()
    if (q) nodes = nodes.filter(n => n.label.toLowerCase().includes(q))
    if (selectedType()) nodes = nodes.filter(n => n.type === selectedType())
    if (selectedDomain()) nodes = nodes.filter(n => n.domain === selectedDomain())
    return nodes
  })

  const filteredEdges = createMemo(() => {
    const nodeIds = new Set(filteredNodes().map(n => n.id))
    return MOCK_EDGES.filter(e => nodeIds.has(e.source) && nodeIds.has(e.target))
  })

  // Canvas 绘制
  const drawGraph = () => {
    const canvas = canvasRef
    if (!canvas) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const rect = canvas.getBoundingClientRect()
    canvas.width = rect.width * devicePixelRatio
    canvas.height = rect.height * devicePixelRatio
    ctx.scale(devicePixelRatio, devicePixelRatio)

    ctx.clearRect(0, 0, rect.width, rect.height)

    const cx = rect.width / 2 + pan().x
    const cy = rect.height / 2 + pan().y
    const scale = zoom()

    // 布局：力导向简化版（圆形布局）
    const nodes = filteredNodes()
    const nodePositions = new Map<string, { x: number; y: number }>()
    const angleStep = (2 * Math.PI) / Math.max(nodes.length, 1)
    const radius = Math.min(rect.width, rect.height) * 0.32

    nodes.forEach((n, i) => {
      const angle = angleStep * i - Math.PI / 2
      nodePositions.set(n.id, {
        x: cx + Math.cos(angle) * radius * scale,
        y: cy + Math.sin(angle) * radius * scale,
      })
    })

    // 绘制边
    const edges = filteredEdges()
    ctx.lineWidth = 1
    for (const edge of edges) {
      const src = nodePositions.get(edge.source)
      const tgt = nodePositions.get(edge.target)
      if (!src || !tgt) continue

      ctx.beginPath()
      ctx.moveTo(src.x, src.y)
      ctx.lineTo(tgt.x, tgt.y)
      ctx.strokeStyle = edge.type === 'semantic' ? 'rgba(99,102,241,0.4)'
        : edge.type === 'reference' ? 'rgba(20,184,166,0.3)'
        : 'rgba(240,145,58,0.3)'
      ctx.lineWidth = edge.weight * 2
      ctx.stroke()
    }

    // 绘制节点
    for (const node of nodes) {
      const pos = nodePositions.get(node.id)
      if (!pos) continue

      const isHovered = hoveredNode() === node.id
      const isSelected = selectedNode()?.id === node.id
      const r = (12 + node.connections * 0.8) * scale

      // 节点圆
      ctx.beginPath()
      ctx.arc(pos.x, pos.y, r, 0, 2 * Math.PI)
      const color = node.domain ? DOMAIN_COLORS[node.domain] ?? '#6366f1' : '#6366f1'
      ctx.fillStyle = isHovered || isSelected ? color : `${color}40`
      ctx.fill()
      ctx.strokeStyle = color
      ctx.lineWidth = isSelected ? 2.5 : 1.5
      ctx.stroke()

      // 节点标签
      ctx.fillStyle = isHovered || isSelected ? '#fff' : '#666'
      ctx.font = `${isHovered ? '12' : '10'}px -apple-system, BlinkMacSystemFont, sans-serif`
      ctx.textAlign = 'center'
      ctx.textBaseline = 'middle'
      const label = node.label.length > 12 ? node.label.slice(0, 11) + '…' : node.label
      ctx.fillText(label, pos.x, pos.y + r + 12)
    }
  }

  const animate = () => {
    drawGraph()
    animFrame = requestAnimationFrame(animate)
  }

  onMount(() => {
    animate()
  })

  onCleanup(() => {
    if (animFrame) cancelAnimationFrame(animFrame)
  })

  // 鼠标交互
  const handleMouseMove = (e: MouseEvent) => {
    const canvas = canvasRef
    if (!canvas) return
    const rect = canvas.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top

    const nodes = filteredNodes()
    const nodePositions = new Map<string, { x: number; y: number }>()
    const cx = rect.width / 2 + pan().x
    const cy = rect.height / 2 + pan().y
    const scale = zoom()
    const angleStep = (2 * Math.PI) / Math.max(nodes.length, 1)
    const radius = Math.min(rect.width, rect.height) * 0.32

    nodes.forEach((n, i) => {
      const angle = angleStep * i - Math.PI / 2
      nodePositions.set(n.id, {
        x: cx + Math.cos(angle) * radius * scale,
        y: cy + Math.sin(angle) * radius * scale,
      })
    })

    let found: string | null = null
    for (const [id, pos] of nodePositions) {
      const r = (12 + (nodes.find(n => n.id === id)?.connections ?? 0) * 0.8) * scale
      const dist = Math.hypot(mx - pos.x, my - pos.y)
      if (dist < r + 4) { found = id; break }
    }
    setHoveredNode(found)
  }

  const handleClick = () => {
    const hid = hoveredNode()
    if (hid) {
      setSelectedNode(filteredNodes().find(n => n.id === hid) ?? null)
    } else {
      setSelectedNode(null)
    }
  }

  const domains = ['core', 'mind', 'memory', 'shield', 'act']
  const types: { id: string; label: string; icon: string }[] = [
    { id: 'concept', label: '概念', icon: '💡' },
    { id: 'skill', label: '技能', icon: '⚡' },
    { id: 'memory', label: '记忆', icon: '🧠' },
    { id: 'session', label: '会话', icon: '💬' },
    { id: 'file', label: '文件', icon: '📄' },
  ]

  return (
    <div class="h-screen flex flex-col bg-bg-primary">
      {/* Header */}
      <div class="flex items-center gap-3 px-5 h-12 border-b border-border-primary/40 shrink-0">
        <button
          class="flex items-center gap-1.5 text-13px text-text-muted hover:text-text-primary transition-colors"
          onClick={() => navigate('/chat')}
          aria-label="返回对话"
        >
          <ArrowLeft class="w-4 h-4" />
          对话
        </button>
        <h1 class="text-14px font-semibold flex items-center gap-1.5">
          <Sparkles class="w-4 h-4 text-nt-memory-600" />
          知识图谱
        </h1>
        <span class="text-11px text-text-muted">{filteredNodes().length} 节点 · {filteredEdges().length} 连接</span>
        <div class="flex-1" />
        <div class="flex items-center gap-1">
          <button class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60" onClick={() => setZoom(z => Math.min(z + 0.2, 3))} title="放大">
            <ZoomIn class="w-4 h-4" />
          </button>
          <button class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60" onClick={() => setZoom(z => Math.max(z - 0.2, 0.3))} title="缩小">
            <ZoomOut class="w-4 h-4" />
          </button>
          <button class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60" onClick={() => { setZoom(1); setPan({ x: 0, y: 0 }) }} title="重置视图">
            <RotateCcw class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div class="flex flex-1 min-h-0">
        {/* 左侧筛选 */}
        <div class="w-[200px] border-r border-border-primary/30 p-3 flex flex-col gap-4 overflow-y-auto shrink-0">
          <div>
            <div class="relative">
              <Search class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
              <input
                type="search"
                value={search()}
                onInput={(e) => setSearch(e.currentTarget.value)}
                placeholder="搜索节点…"
                class="w-full h-8 pl-8 pr-3 rounded-lg text-12px bg-bg-secondary border border-border-primary/50 focus:border-nt-io-500 focus:outline-none"
              />
            </div>
          </div>

          <div>
            <div class="text-10px uppercase tracking-widest text-text-muted/70 font-semibold mb-2">类型</div>
            <div class="space-y-1">
              <button
                class={clsx('w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-12px transition-colors', !selectedType() ? 'bg-nt-io-500/10 text-nt-io-600' : 'text-text-secondary hover:bg-white/40')}
                onClick={() => setSelectedType(null)}
              >
                全部
              </button>
              <For each={types}>
                {(t) => (
                  <button
                    class={clsx('w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-12px transition-colors', selectedType() === t.id ? 'bg-nt-io-500/10 text-nt-io-600' : 'text-text-secondary hover:bg-white/40')}
                    onClick={() => setSelectedType(selectedType() === t.id ? null : t.id)}
                  >
                    <span>{t.icon}</span>
                    <span>{t.label}</span>
                  </button>
                )}
              </For>
            </div>
          </div>

          <div>
            <div class="text-10px uppercase tracking-widest text-text-muted/70 font-semibold mb-2">域</div>
            <div class="space-y-1">
              <button
                class={clsx('w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-12px transition-colors', !selectedDomain() ? 'bg-nt-io-500/10 text-nt-io-600' : 'text-text-secondary hover:bg-white/40')}
                onClick={() => setSelectedDomain(null)}
              >
                全部
              </button>
              <For each={domains}>
                {(d) => (
                  <button
                    class={clsx('w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-12px transition-colors', selectedDomain() === d ? 'bg-nt-io-500/10 text-nt-io-600' : 'text-text-secondary hover:bg-white/40')}
                    onClick={() => setSelectedDomain(selectedDomain() === d ? null : d)}
                  >
                    <span class="w-2.5 h-2.5 rounded-full" style={{ background: DOMAIN_COLORS[d] }} />
                    <span>{d}</span>
                  </button>
                )}
              </For>
            </div>
          </div>
        </div>

        {/* Canvas */}
        <div class="flex-1 relative">
          <canvas
            ref={canvasRef}
            class="w-full h-full cursor-grab active:cursor-grabbing"
            onMouseMove={handleMouseMove}
            onClick={handleClick}
            onMouseLeave={() => setHoveredNode(null)}
          />
        </div>

        {/* 右侧详情 */}
        <Show when={selectedNode()}>
          {(node) => (
            <div class="w-[240px] border-l border-border-primary/30 p-4 flex flex-col gap-3 overflow-y-auto shrink-0">
              <div class="flex items-start gap-2">
                <span class="text-2xl">{TYPE_ICONS[node().type]}</span>
                <div class="min-w-0 flex-1">
                  <div class="text-14px font-semibold text-text-primary truncate">{node().label}</div>
                  <div class="text-11px text-text-muted capitalize">{node().type}</div>
                </div>
              </div>
              <div class="space-y-2 text-12px">
                <div class="flex justify-between">
                  <span class="text-text-muted">连接数</span>
                  <span class="text-text-primary font-medium">{node().connections}</span>
                </div>
                {node().domain && (
                  <div class="flex justify-between">
                    <span class="text-text-muted">所属域</span>
                    <span class="text-text-primary font-medium">{node().domain}</span>
                  </div>
                )}
                {node().lastAccessed && (
                  <div class="flex justify-between">
                    <span class="text-text-muted">最近访问</span>
                    <span class="text-text-primary font-medium">
                      {formatTime(node().lastAccessed!)}
                    </span>
                  </div>
                )}
              </div>
              <div class="mt-2">
                <div class="text-10px uppercase tracking-widest text-text-muted/70 font-semibold mb-2">相关连接</div>
                <div class="space-y-1">
                  {MOCK_EDGES.filter(e => e.source === node().id || e.target === node().id).slice(0, 5).map(e => {
                    const otherId = e.source === node().id ? e.target : e.source
                    const other = MOCK_NODES.find(n => n.id === otherId)
                    return other ? (
                      <div class="flex items-center gap-2 px-2 py-1 rounded bg-white/40 text-11px">
                        <span class="w-1.5 h-1.5 rounded-full" style={{ background: DOMAIN_COLORS[other.domain ?? 'core'] ?? '#666' }} />
                        <span class="truncate">{other.label}</span>
                        <span class="ml-auto text-text-muted">{Math.round(e.weight * 100)}%</span>
                      </div>
                    ) : null
                  })}
                </div>
              </div>
            </div>
          )}
        </Show>
      </div>
    </div>
  )
}

function formatTime(ts: number): string {
  const diff = Date.now() - ts
  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`
  return `${Math.floor(diff / 86400000)}天前`
}
