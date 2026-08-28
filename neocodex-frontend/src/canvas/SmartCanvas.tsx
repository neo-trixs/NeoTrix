// ══════════════════════════════════════════════════════════════════════════
//  Smart Canvas — 无限画布表面（SolidJS 原生，零重依赖，对齐 R-P1/Dark Forest）
//  能力网渲染：每个 node 经 nodeRegistry 取渲染器；智能收缩经 smartCollapse。
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, createMemo, onMount, onCleanup, Show, For, type JSX } from 'solid-js'
import type { CanvasNode, CollapsePolicy, Viewport } from './types'
import { getRenderer, listCapabilities } from './nodeRegistry'
import { computeCollapse, DEFAULT_POLICY } from './smartCollapse'
import { evolutionRoute, pruneCandidates } from './evolution'

/** 搜索添加时用的示例载荷，保证节点可渲染。 */
function sampleData(kind: string): unknown {
  if (kind.startsWith('chart')) return { chart: 'bar', labels: ['a', 'b', 'c'], series: [{ name: '示例', values: [3, 7, 4] }] }
  if (kind === 'table') return [['col1', 'col2'], ['x', '1'], ['y', '2']]
  if (kind === 'image') return { src: 'https://www.gstatic.com/webp/gallery/1.jpg', meta: '示例' }
  if (kind === 'mermaid') return 'graph TD; A-->B; B-->C;'
  if (kind === 'json' || kind === '*') return { hello: 'world', nested: { a: 1 } }
  if (kind === 'markdown' || kind === 'text') return '# 示例\n通过搜索添加的能力节点。'
  if (kind === 'diff') return '- old\n+ new'
  if (kind === 'code') return 'fn main() {}'
  if (kind === 'kpi') return { value: '42', label: '示例指标' }
  if (kind === 'heatmap') return { cells: [[1, 2], [3, 1]], max: 3 }
  if (kind === 'webpage') return { url: 'https://example.com' }
  if (kind === 'html') return '<div style="padding:8px">示例 HTML</div>'
  return { note: '自定义能力示例' }
}

export interface SmartCanvasProps {
  nodes: () => CanvasNode[]
  spawn: (n: Partial<CanvasNode> & { kind: string; data: unknown }) => void
  setCollapsed?: (id: string, v: boolean) => void
}

export function SmartCanvas(props: SmartCanvasProps) {
  // ── 视口变换（世界 ↔ 屏幕） ──
  const [tx, setTx] = createSignal(0)
  const [ty, setTy] = createSignal(0)
  const [k, setK] = createSignal(1)
  const [vp, setVp] = createSignal<Viewport>({ x: -400, y: -300, w: 800, h: 600 })
  const [policy, setPolicy] = createSignal<CollapsePolicy>({ ...DEFAULT_POLICY })
  const [surfaceSize, setSurfaceSize] = createSignal({ w: 800, h: 600 })

  let surface: HTMLDivElement | undefined
  let dragging: { x: number; y: number; tx: number; ty: number } | null = null

  const syncViewport = () => {
    if (!surface) return
    const r = surface.getBoundingClientRect()
    setSurfaceSize({ w: r.width, h: r.height })
    setVp({
      x: -tx() / k(),
      y: -ty() / k(),
      w: r.width / k(),
      h: r.height / k(),
    })
  }

  onMount(() => {
    syncViewport()
    const ro = new ResizeObserver(syncViewport)
    if (surface) ro.observe(surface)
    onCleanup(() => ro.disconnect())
  })

  // ── 智能收缩：节点/策略/视口变化即重算 ──
  const collapsedSet = createMemo(() => computeCollapse(props.nodes(), policy(), vp()))

  // ── 平移 ──
  const onPointerDown = (e: PointerEvent) => {
    if ((e.target as HTMLElement).closest('.sc-node')) return // 节点内不平移
    dragging = { x: e.clientX, y: e.clientY, tx: tx(), ty: ty() }
    ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  }
  const onPointerMove = (e: PointerEvent) => {
    if (!dragging) return
    setTx(dragging.tx + (e.clientX - dragging.x))
    setTy(dragging.ty + (e.clientY - dragging.y))
    syncViewport()
  }
  const onPointerUp = (e: PointerEvent) => {
    dragging = null
    ;(e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId)
  }

  // ── 缩放（光标锚定） ──
  const onWheel = (e: WheelEvent) => {
    e.preventDefault()
    const r = surface!.getBoundingClientRect()
    const cx = e.clientX - r.left
    const cy = e.clientY - r.top
    const wx = (cx - tx()) / k()
    const wy = (cy - ty()) / k()
    const nk = Math.min(3, Math.max(0.2, k() * (e.deltaY < 0 ? 1.1 : 0.9)))
    setK(nk)
    setTx(cx - wx * nk)
    setTy(cy - wy * nk)
    syncViewport()
  }

  const zoomBy = (f: number) => {
    const cx = surfaceSize().w / 2
    const cy = surfaceSize().h / 2
    const wx = (cx - tx()) / k()
    const wy = (cy - ty()) / k()
    const nk = Math.min(3, Math.max(0.2, k() * f))
    setK(nk)
    setTx(cx - wx * nk)
    setTy(cy - wy * nk)
    syncViewport()
  }
  const fit = () => {
    setTx(0)
    setTy(0)
    setK(1)
    syncViewport()
  }
  const toggleAutoCollapse = () =>
    setPolicy((p) => ({ ...p, salienceFloor: p.salienceFloor > 0 ? 0 : 0.35 }))

  // spawn 包装：新节点落在当前视图中心偏移
  const doSpawn: SmartCanvasProps['spawn'] = (n) => {
    props.spawn({ x: -tx() / k() + 40, y: -ty() / k() + 40, ...n })
  }

  // ── 能力网搜索 / 进化路线 ──
  const [search, setSearch] = createSignal('')
  const [paletteOpen, setPaletteOpen] = createSignal(false)
  const [evoOpen, setEvoOpen] = createSignal(false)
  const matched = createMemo(() => {
    const q = search().toLowerCase().trim()
    const caps = listCapabilities()
    if (!q) return caps
    return caps.filter((c) => c.kind.toLowerCase().includes(q) || c.label.toLowerCase().includes(q))
  })
  const addBySearch = (kind: string, label: string) => {
    const k = kind.split('|')[0] // 多 kind renderer 取首个
    doSpawn({ kind: k, data: sampleData(k), title: `${label} 示例`, salience: 0.6, source: 'search' })
    setPaletteOpen(false)
    setSearch('')
  }

  return (
    <div class="sc-root">
      {/* 工具条 */}
      <div class="sc-toolbar">
        <button class="sc-tbtn" title="缩小" onClick={() => zoomBy(0.9)}>−</button>
        <span class="sc-zoom">{Math.round(k() * 100)}%</span>
        <button class="sc-tbtn" title="放大" onClick={() => zoomBy(1.1)}>+</button>
        <button class="sc-tbtn" title="适应" onClick={fit}>⤢</button>
        <button
          class="sc-tbtn"
          classList={{ on: policy().salienceFloor > 0 }}
          title="智能收缩（salience 轴）"
          onClick={toggleAutoCollapse}
        >智能收缩</button>
        <input
          class="sc-search"
          placeholder="搜索能力…"
          value={search()}
          onInput={(e) => { setSearch(e.currentTarget.value); setPaletteOpen(true) }}
          onFocus={() => setPaletteOpen(true)}
        />
        <button
          class="sc-tbtn"
          classList={{ on: evoOpen() }}
          title="能力网进化路线"
          onClick={() => setEvoOpen(!evoOpen())}
        >进化路线</button>
        <span class="sc-cap-count">能力网 {listCapabilities().length} 族</span>
      </div>

      {/* 能力搜索面板 */}
      <Show when={paletteOpen() && matched().length}>
        <div class="sc-palette">
          <For each={matched()}>
            {(c) => (
              <button class="sc-palette-item" onClick={() => addBySearch(c.kind, c.label)}>
                <span class="sc-kind">{c.label}</span>
                <span class="sc-palette-kind">{c.kind}</span>
              </button>
            )}
          </For>
        </div>
      </Show>

      {/* 画布表面 */}
      <div
        class="sc-surface"
        ref={surface}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onWheel={onWheel}
      >
        <div
          class="sc-world"
          style={{ transform: `translate(${tx()}px, ${ty()}px) scale(${k()})` }}
        >
          <For each={props.nodes()}>
            {(node) => {
              const r = getRenderer(node)
              const isCollapsed = () => collapsedSet().has(node.id)
              const size = () => ({ w: node.w ?? r.defaultSize?.w ?? 300, h: node.h ?? r.defaultSize?.h ?? 200 })
              return (
                <div
                  class="sc-node"
                  classList={{ 'sc-collapsed': isCollapsed() }}
                  style={{
                    left: `${node.x}px`,
                    top: `${node.y}px`,
                    width: `${size().w}px`,
                  }}
                >
                  <div class="sc-node-head">
                    <span class="sc-kind">{r.label}</span>
                    <span class="sc-title">{node.title ?? node.kind}</span>
                    <Show when={node.source}><span class="sc-src" title={node.source}>◆</span></Show>
                    <button
                      class="sc-collapse"
                      onClick={() => props.setCollapsed?.(node.id, !isCollapsed())}
                    >
                      {isCollapsed() ? '▸' : '▾'}
                    </button>
                  </div>
                  <Show when={!isCollapsed()}>
                    <div class="sc-node-body">
                      {r.render({
                        node,
                        setCollapsed: (v) => props.setCollapsed?.(node.id, v),
                        spawn: doSpawn,
                      }) as JSX.Element}
                    </div>
                  </Show>
                  <Show when={isCollapsed()}>
                    <div class="sc-node-capsule">
                      显著性 {Math.round((node.salience ?? 0.5) * 100)}% · 点击展开
                    </div>
                  </Show>
                </div>
              )
            }}
          </For>
        </div>

        {/* 能力网进化路线覆盖层 */}
        <Show when={evoOpen()}>
          <div class="sc-evo">
            <div class="sc-evo-head">
              <span>能力网进化路线</span>
              <button class="sc-evo-close" onClick={() => setEvoOpen(false)}>×</button>
            </div>
            <div class="sc-evo-list">
              <For each={evolutionRoute()}>
                {(c) => (
                  <div class="sc-evo-row" classList={{ dead: c.count === 0 && c.userAdded }}>
                    <span class="sc-evo-stage">{c.stage}</span>
                    <span class="sc-evo-label">{c.label}</span>
                    <span class="sc-evo-kind">{c.kind}</span>
                    <span class="sc-evo-count">×{c.count}</span>
                  </div>
                )}
              </For>
            </div>
            <Show when={pruneCandidates().length}>
              <div class="sc-evo-prune">Dark Forest 回收候选：{pruneCandidates().join('，')}</div>
            </Show>
          </div>
        </Show>
      </div>
    </div>
  )
}
