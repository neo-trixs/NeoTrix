// ══════════════════════════════════════════════════════════════════════════
//  Smart Canvas — 无限画布表面（SolidJS 原生，零重依赖，对齐 R-P1/Dark Forest）
//  能力网渲染：每个 node 经 nodeRegistry 取渲染器；智能收缩经 smartCollapse。
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, createMemo, onMount, onCleanup, Show, For, type JSX } from 'solid-js'
import type { CanvasNode, CollapsePolicy, Viewport } from './types'
import { getRenderer, listCapabilities } from './nodeRegistry'
import { computeCollapse, DEFAULT_POLICY } from './smartCollapse'
import { evolutionRoute, pruneCandidates, treeStatus, pruneNode, setDesired, applyEvolutionRoute, syncToCapabilityTree } from './evolution'

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
  // 能力地图维度筛选（按 kind 维度过滤可见节点，对标 2026 标签筛选）
  const [kindFilter, setKindFilter] = createSignal<Set<string>>(new Set())
  const toggleKind = (kind: string) =>
    setKindFilter((s) => {
      const n = new Set(s)
      if (n.has(kind)) n.delete(kind)
      else n.add(kind)
      return n
    })
  // 节点可拖拽移动（画布内重定位，前端本地覆盖；对标 2026 无限画布）
  const [nodePos, setNodePos] = createSignal<Record<string, { x: number; y: number }>>({})
  const dragNode = (id: string, e: PointerEvent) => {
    if ((e.target as HTMLElement).closest('button')) return // 按钮不触发拖拽
    e.stopPropagation()
    const ox = e.clientX
    const oy = e.clientY
    const base = (() => {
      const n = props.nodes().find((x) => x.id === id)
      return n ? { x: n.x, y: n.y } : { x: 0, y: 0 }
    })()
    const move = (ev: PointerEvent) => {
      setNodePos((p) => ({ ...p, [id]: { x: base.x + (ev.clientX - ox), y: base.y + (ev.clientY - oy) } }))
    }
    const up = () => {
      window.removeEventListener('pointermove', move)
      window.removeEventListener('pointerup', up)
    }
    window.addEventListener('pointermove', move)
    window.addEventListener('pointerup', up)
  }
  const [evoOpen, setEvoOpen] = createSignal(false)
  const [routeResult, setRouteResult] = createSignal<{ matured: number; pruned: number; applied: string[] } | null>(null)
  const matched = createMemo(() => {
    const q = search().toLowerCase().trim()
    const caps = listCapabilities()
    if (!q) return caps
    return caps.filter((c) => c.kind.toLowerCase().includes(q) || c.label.toLowerCase().includes(q))
  })
  // 搜索联想：键盘上下导航 + 空查询时热门能力推荐
  const [activeIdx, setActiveIdx] = createSignal(0)
  const resetActive = () => setActiveIdx(0)
  const onSearchKey = (e: KeyboardEvent) => {
    if (e.key === 'ArrowDown') { e.preventDefault(); setActiveIdx((i) => Math.min(matched().length - 1, i + 1)) }
    else if (e.key === 'ArrowUp') { e.preventDefault(); setActiveIdx((i) => Math.max(0, i - 1)) }
    else if (e.key === 'Enter') {
      const c = matched()[activeIdx()]
      if (c) { e.preventDefault(); addBySearch(c.kind, c.label) }
    }
  }
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
          onInput={(e) => { setSearch(e.currentTarget.value); setPaletteOpen(true); resetActive() }}
          onKeyDown={onSearchKey}
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

      {/* 维度筛选：按 kind 过滤可见节点 */}
      <div class="sc-dimbar">
        <For each={[...new Set(props.nodes().map((n) => n.kind))]}>
          {(kind) => (
            <button
              class="sc-dim"
              classList={{ on: kindFilter().has(kind) }}
              onClick={() => toggleKind(kind)}
            >{kind}</button>
          )}
        </For>
        <Show when={kindFilter().size > 0}>
          <button class="sc-dim sc-dim-clear" onClick={() => setKindFilter(new Set())}>清除</button>
        </Show>
      </div>

      {/* 能力搜索面板 */}
      <Show when={paletteOpen() && matched().length}>
        <div class="sc-palette">
          <Show when={!search().trim()}>
            <div class="sc-palette-hint">热门能力（↑↓ 选择 · ↵ 添加）</div>
          </Show>
          <For each={matched()}>
            {(c, i) => (
              <button
                class="sc-palette-item"
                classList={{ on: i() === activeIdx() }}
                onMouseEnter={() => setActiveIdx(i())}
                onClick={() => addBySearch(c.kind, c.label)}
              >
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
                  classList={{
                    'sc-collapsed': isCollapsed(),
                    'sc-dimmed': kindFilter().size > 0 && !kindFilter().has(node.kind),
                  }}
                  style={{
                    left: `${nodePos()[node.id]?.x ?? node.x}px`,
                    top: `${nodePos()[node.id]?.y ?? node.y}px`,
                    width: `${size().w}px`,
                  }}
                >
                  <div
                    class="sc-node-head"
                    onPointerDown={(e) => dragNode(node.id, e)}
                  >
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
              <button
                class="sc-evo-auto"
                title="按能力树自身 SEAL 路线自动进化 (树提议, 画板执行)"
                onClick={() => applyEvolutionRoute().then(setRouteResult)}
              >⚡ 自动进化</button>
              <button class="sc-evo-close" onClick={() => setEvoOpen(false)}>×</button>
            </div>
            <Show when={routeResult()}>
              {(r) => (
                <div class="sc-evo-route">
                  自动进化完成 · 晋升 {r().matured} · 回收 {r().pruned}
                  <For each={r().applied}>{(a) => <div class="sc-evo-route-item">↳ {a}</div>}</For>
                </div>
              )}
            </Show>
            <div class="sc-evo-list">
              <For each={evolutionRoute()}>
                {(c) => (
                  <div class="sc-evo-row" classList={{ dead: c.count === 0 && c.userAdded }}>
                    <span class="sc-evo-stage">{c.stage}</span>
                    <span class="sc-evo-label">{c.label}</span>
                    <span class="sc-evo-kind">{c.kind}</span>
                    <span class="sc-evo-count">×{c.count}</span>
                    <Show when={treeStatus()?.canonical[c.kind]}>
                      {(cn) => (
                        <>
                          <span
                            class="sc-evo-canon"
                            classList={{ dead: cn().deprecated }}
                            title="NeoTrix 能力树 canonical 成熟度 (SEAL 实算)"
                          >NeoTrix {cn().constellation}</span>
                          <button
                            class="sc-evo-star"
                            classList={{ on: cn().desired !== undefined }}
                            title={cn().desired !== undefined ? `期望成熟度 C${cn().desired} · 点按清除` : '设为基石目标 (C5)'}
                            onClick={() => setDesired(c.kind, cn().desired !== undefined ? null : 5)}
                          >{cn().desired !== undefined ? '★' : '☆'}</button>
                        </>
                      )}
                    </Show>
                    <Show when={c.count === 0 && c.userAdded}>
                      <button class="sc-evo-prune-btn" onClick={() => pruneNode(c.kind)}>回收</button>
                    </Show>
                  </div>
                )}
              </For>
            </div>
            <Show when={treeStatus()}>
              {(st) => (
                <>
                  <div class="sc-evo-prune">
                    已并入 NeoTrix 能力树 · {st().canvasNodes} 节点 (cycle {st().cycle})
                    {st().matured > 0 ? ` · SEAL 晋升 ${st().matured}` : ''}
                    {st().deprecated > 0 ? ` · Dark Forest 回收 ${st().deprecated}` : ''}
                  </div>
                  <Show when={st().plans.length > 0}>
                    <div class="sc-evo-plans">
                      <For each={st().plans}>
                        {(p) => (
                          <div class="sc-evo-plan">
                            <span class="sc-evo-plan-act">{p.action}</span>
                            <span class="sc-evo-plan-node">{p.nodeId.replace('canvas::', '')}</span>
                            <span class="sc-evo-plan-why">{p.rationale}</span>
                          </div>
                        )}
                      </For>
                    </div>
                  </Show>
                </>
              )}
            </Show>
            <Show when={pruneCandidates().length}>
              <div class="sc-evo-prune">
                Dark Forest 回收候选：{pruneCandidates().join('，')}
                <button class="sc-evo-prune-all" onClick={() => pruneCandidates().forEach((k) => pruneNode(k))}>
                  回收全部死节点
                </button>
              </div>
            </Show>
            <button class="sc-evo-resync" onClick={() => syncToCapabilityTree()}>↻ 重新同步能力树</button>
          </div>
        </Show>
      </div>
    </div>
  )
}
