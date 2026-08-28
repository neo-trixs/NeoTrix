// ══════════════════════════════════════════════════════════════════════════
//  Smart Canvas — 无限画布表面（SolidJS 原生，零重依赖，对齐 R-P1/Dark Forest）
//  能力网渲染：每个 node 经 nodeRegistry 取渲染器；智能收缩经 smartCollapse。
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, createMemo, onMount, onCleanup, Show, For, type JSX } from 'solid-js'
import type { CanvasNode, CollapsePolicy, Viewport } from './types'
import { getRenderer, listCapabilities } from './nodeRegistry'
import { computeCollapse, DEFAULT_POLICY } from './smartCollapse'

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
        <span class="sc-cap-count">能力网 {listCapabilities().length} 族</span>
      </div>

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
      </div>
    </div>
  )
}
