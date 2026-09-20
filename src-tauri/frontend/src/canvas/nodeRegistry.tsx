// ══════════════════════════════════════════════════════════════════════════
//  Smart Canvas — 能力网 (Capability Network)
//  Map<kind, NodeRenderer> 注册表。内置 11 个格式族默认渲染器 + '*' 兜底。
//  新格式随 SEAL 吸收经 registerNodeRenderer 自然长出 → 无限制类型。
// ══════════════════════════════════════════════════════════════════════════
import { For, Show, createSignal, type JSX } from 'solid-js'
import type { NodeKind, NodeRenderer, CanvasNode, NodeRenderContext } from './types'

// ── 通用：安全文本 ──
function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

// ── 通用：可折叠 JSON 树（json 族 + '*' 兜底复用） ──
function JsonTree(props: { value: unknown; depth?: number }) {
  const d = props.depth ?? 0
  const [open, setOpen] = createSignal(d < 2)
  const isObj = typeof props.value === 'object' && props.value !== null
  return (
    <Show
      when={isObj}
      fallback={<span class="sc-jval">{esc(String(props.value))}</span>}
    >
      <div class="sc-jnode">
        <Show when={(props.value as object[]).constructor === Array}>
          <button class="sc-jtoggle" onClick={() => setOpen(!open())}>
            {open() ? '▾' : '▸'} [{Object.keys(props.value as object).length}]
          </button>
          <Show when={open()}>
            <For each={Object.entries(props.value as Record<string, unknown>)}>
              {([k, v]) => (
                <div class="sc-jrow" style={{ 'padding-left': `${d * 12 + 8}px` }}>
                  <span class="sc-jkey">{esc(k)}</span>
                  <JsonTree value={v} depth={d + 1} />
                </div>
              )}
            </For>
          </Show>
        </Show>
        <Show when={(props.value as object[]).constructor !== Array}>
          <button class="sc-jtoggle" onClick={() => setOpen(!open())}>
            {open() ? '▾' : '▸'} {'{' + Object.keys(props.value as object).length + '}'}
          </button>
          <Show when={open()}>
            <For each={Object.entries(props.value as Record<string, unknown>)}>
              {([k, v]) => (
                <div class="sc-jrow" style={{ 'padding-left': `${d * 12 + 8}px` }}>
                  <span class="sc-jkey">{esc(k)}</span>
                  <JsonTree value={v} depth={d + 1} />
                </div>
              )}
            </For>
          </Show>
        </Show>
      </div>
    </Show>
  )
}

// ── 1. text / markdown ──
const textRenderer: NodeRenderer = {
  kind: ['text', 'markdown'],
  label: '文本',
  defaultSize: { w: 280, h: 160 },
  render: (ctx) => {
    const body = typeof ctx.node.data === 'string' ? ctx.node.data : JSON.stringify(ctx.node.data)
    // 极简 MD：标题/列表/代码围栏（仅预览级，避免重依赖）
    const html = esc(body)
      .replace(/^### (.+)$/gm, '<h4>$1</h4>')
      .replace(/^## (.+)$/gm, '<h3>$1</h3>')
      .replace(/^# (.+)$/gm, '<h2>$1</h2>')
      .replace(/^- (.+)$/gm, '<li>$1</li>')
      .replace(/(<li>.*<\/li>)+/g, '<ul>$&</ul>')
      .replace(/\n/g, '<br/>')
    return <div class="sc-body sc-md" innerHTML={html} />
  },
}

// ── 2. code / diff ──
const codeRenderer: NodeRenderer = {
  kind: ['code', 'diff'],
  label: '代码',
  defaultSize: { w: 360, h: 220 },
  render: (ctx) => {
    const raw = typeof ctx.node.data === 'string' ? ctx.node.data : JSON.stringify(ctx.node.data, null, 2)
    const isDiff = ctx.node.kind === 'diff'
    return (
      <pre class="sc-body sc-code">
        <For each={raw.split('\n')}>
          {(line) => (
            <div
              classList={{
                'sc-d-add': isDiff && line.startsWith('+'),
                'sc-d-del': isDiff && line.startsWith('-'),
              }}
            >
              {esc(line)}
            </div>
          )}
        </For>
      </pre>
    )
  },
}

// ── 3. table ──
const tableRenderer: NodeRenderer = {
  kind: 'table',
  label: '表格',
  defaultSize: { w: 360, h: 240 },
  validate: (d) => Array.isArray(d) && Array.isArray((d as unknown[])[0]),
  render: (ctx) => {
    const rows = ctx.node.data as unknown[][]
    const head = rows[0] as unknown[]
    const body = rows.slice(1)
    return (
      <div class="sc-body sc-table-wrap">
        <table class="sc-table">
          <thead>
            <tr>
              <For each={head}>{(h) => <th>{esc(String(h))}</th>}</For>
            </tr>
          </thead>
          <tbody>
            <For each={body}>
              {(r) => (
                <tr>
                  <For each={r as unknown[]}>{(c) => <td>{esc(String(c))}</td>}</For>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>
    )
  },
}

// ── 4/5. chart（cartesian + composite，内联 SVG，零依赖） ──
interface ChartSpec {
  chart?: 'line' | 'bar' | 'pie'
  labels?: string[]
  series?: { name: string; color?: string; values: number[] }[]
}
const PALETTE = ['#f0913a', '#14b8a6', '#a855f7', '#fb923c', '#2dd4bf', '#c084fc']
const chartRenderer: NodeRenderer = {
  kind: ['chart', 'chart-cartesian', 'chart-composite'],
  label: '图表',
  defaultSize: { w: 320, h: 220 },
  validate: (d) => !!(d as ChartSpec)?.series,
  render: (ctx) => {
    const spec = ctx.node.data as ChartSpec
    const W = 300
    const H = 180
    const pad = 24
    const series = spec.series ?? []
    const labels = spec.labels ?? []
    const maxV = Math.max(1, ...series.flatMap((s) => s.values))
    const type = spec.chart ?? 'bar'
    return (
      <div class="sc-body sc-chart">
        <svg viewBox={`0 0 ${W} ${H}`} class="sc-chart-svg">
          <For each={series}>
            {(s, si) => {
              const color = s.color ?? PALETTE[si() % PALETTE.length]
              const n = s.values.length
              if (type === 'line') {
                const pts = s.values
                  .map((v, i) => {
                    const x = pad + (i / Math.max(1, n - 1)) * (W - pad * 2)
                    const y = H - pad - (v / maxV) * (H - pad * 2)
                    return `${x},${y}`
                  })
                  .join(' ')
                return <polyline points={pts} fill="none" stroke={color} stroke-width="2" />
              }
              // bar：多序列分组
              const bw = ((W - pad * 2) / n) / Math.max(1, series.length)
              return (
                <For each={s.values}>
                  {(v, i) => {
                    const x = pad + i() * ((W - pad * 2) / n) + si() * bw
                    const h = (v / maxV) * (H - pad * 2)
                    return <rect x={x} y={H - pad - h} width={bw - 2} height={h} fill={color} opacity="0.85" />
                  }}
                </For>
              )
            }}
          </For>
          <line x1={pad} y1={H - pad} x2={W - pad} y2={H - pad} stroke="currentColor" stroke-width="1" opacity="0.3" />
        </svg>
        <div class="sc-chart-legend">
          <For each={series}>{(s, si) => (
            <span class="sc-legend-item">
              <i style={{ background: s.color ?? PALETTE[si() % PALETTE.length] }} />
              {esc(s.name)}
            </span>
          )}</For>
        </div>
      </div>
    )
  },
}

// ── 6. chart-matrix（heatmap 骨架；graph/tree/sankey 经 json 兜底，留接线点） ──
const matrixRenderer: NodeRenderer = {
  kind: ['chart-matrix', 'heatmap'],
  label: '矩阵',
  defaultSize: { w: 280, h: 240 },
  render: (ctx) => {
    const cells = (ctx.node.data as { cells?: number[][]; max?: number })?.cells ?? []
    const mx = (ctx.node.data as { max?: number })?.max ?? Math.max(1, ...cells.flat())
    return (
      <div class="sc-body sc-heat">
        <For each={cells}>{(row) => (
          <div class="sc-heat-row">
            <For each={row}>{(v) => (
              <span class="sc-heat-cell" style={{ opacity: `${0.15 + (v / mx) * 0.85}` }} />
            )}</For>
          </div>
        )}</For>
      </div>
    )
  },
}

// ── 7. diagram（mermaid 留接线点：当前降级为代码块，未来接 mermaid/Excalidraw MCP） ──
const diagramRenderer: NodeRenderer = {
  kind: ['diagram', 'mermaid'],
  label: '图',
  defaultSize: { w: 320, h: 200 },
  render: (ctx) => {
    const src = typeof ctx.node.data === 'string' ? ctx.node.data : JSON.stringify(ctx.node.data)
    return (
      <div class="sc-body sc-diagram">
        <div class="sc-note">mermaid/excalidraw 渲染器接线点（依赖注入；当前以源码展示）</div>
        <pre class="sc-code">{esc(src)}</pre>
      </div>
    )
  },
}

// ── 8. image ──
const imageRenderer: NodeRenderer = {
  kind: 'image',
  label: '图像',
  defaultSize: { w: 280, h: 220 },
  render: (ctx) => {
    const d = ctx.node.data as { src: string; alt?: string; meta?: string }
    return (
      <div class="sc-body sc-image">
        <img src={d.src} alt={d.alt ?? ''} loading="lazy" />
        <Show when={d.meta}><div class="sc-meta">{esc(d.meta!)}</div></Show>
      </div>
    )
  },
}

// ── 9. html（沙箱 iframe，srcdoc 隔离；对齐 agent-sidecar 隔离原则） ──
const htmlRenderer: NodeRenderer = {
  kind: ['html', 'artifact'],
  label: 'HTML',
  defaultSize: { w: 360, h: 260 },
  render: (ctx) => {
    const html = typeof ctx.node.data === 'string' ? ctx.node.data : JSON.stringify(ctx.node.data)
    return (
      <div class="sc-body sc-html">
        <iframe class="sc-iframe" sandbox="allow-scripts" srcdoc={html} title={ctx.node.title ?? 'html'} />
      </div>
    )
  },
}

// ── 8b. video（对话附件中的视频结果） ──
const videoRenderer: NodeRenderer = {
  kind: 'video',
  label: '视频',
  defaultSize: { w: 320, h: 240 },
  render: (ctx) => {
    const d = ctx.node.data as { src: string; poster?: string }
    return (
      <div class="sc-body sc-video">
        <video src={d.src} poster={d.poster} controls class="sc-video-el" />
      </div>
    )
  },
}

// ── 9b. webpage（消息中的 URL → 嵌入式网页卡片） ──
const webpageRenderer: NodeRenderer = {
  kind: 'webpage',
  label: '网页',
  defaultSize: { w: 380, h: 300 },
  render: (ctx) => {
    const d = ctx.node.data as { url: string; title?: string }
    return (
      <div class="sc-body sc-web">
        <div class="sc-web-head">
          <span class="sc-web-title">{esc(d.title ?? d.url)}</span>
          <a class="sc-web-open" href={d.url} target="_blank" rel="noopener noreferrer">↗</a>
        </div>
        <iframe class="sc-web-iframe" src={d.url} title={d.title ?? 'web'} referrerpolicy="no-referrer" />
      </div>
    )
  },
}

// ── 10. map（占位；复用 RightBar GlobeView 为接线点） ──
const mapRenderer: NodeRenderer = {
  kind: 'map',
  label: '地图',
  defaultSize: { w: 320, h: 240 },
  render: () => <div class="sc-body sc-note">地理层渲染器接线点（复用 api/geo + GlobeView）</div>,
}

// ── 11. kpi / status ──
const kpiRenderer: NodeRenderer = {
  kind: ['kpi', 'status'],
  label: '指标',
  defaultSize: { w: 200, h: 120 },
  render: (ctx) => {
    const d = ctx.node.data as { value: string | number; label?: string; delta?: string }
    return (
      <div class="sc-body sc-kpi">
        <div class="sc-kpi-val">{esc(String(d.value))}</div>
        <Show when={d.label}><div class="sc-kpi-label">{esc(d.label!)}</div></Show>
        <Show when={d.delta}><div class="sc-kpi-delta">{esc(d.delta!)}</div></Show>
      </div>
    )
  },
}

// ── '*' 兜底：未知类型永远有渲染器 → 无限制类型的硬保证 ──
const fallbackRenderer: NodeRenderer = {
  kind: '*',
  label: '原始',
  defaultSize: { w: 300, h: 200 },
  render: (ctx) => (
    <div class="sc-body sc-fallback">
      <div class="sc-note">未注册类型 <code>{esc(ctx.node.kind)}</code> → 原始 JSON</div>
      <JsonTree value={ctx.node.data} />
    </div>
  ),
}

// ════════════ 注册表 ════════════
const REGISTRY = new Map<NodeKind, NodeRenderer>()
const FALLBACK = fallbackRenderer

function seed(r: NodeRenderer) {
  const kinds = Array.isArray(r.kind) ? r.kind : [r.kind]
  for (const k of kinds) REGISTRY.set(k, r)
}
;[textRenderer, codeRenderer, tableRenderer, chartRenderer, matrixRenderer, diagramRenderer, imageRenderer, htmlRenderer, videoRenderer, webpageRenderer, mapRenderer, kpiRenderer, fallbackRenderer].forEach(seed)

/** 注册新能力节点（能力网演进入口）。覆盖同名 kind。 */
export function registerNodeRenderer(r: NodeRenderer): void {
  const kinds = Array.isArray(r.kind) ? r.kind : [r.kind]
  for (const k of kinds) REGISTRY.set(k, r)
}

/** 注销（Dark Forest：无消费者的能力可回收）。 */
export function unregisterNodeRenderer(kind: NodeKind): void {
  if (kind !== '*') REGISTRY.delete(kind)
}

/** 按 kind 取渲染器；校验失败降级兜底；未知类型走兜底。 */
export function getRenderer(node: CanvasNode): NodeRenderer {
  const exact = REGISTRY.get(node.kind)
  if (exact && (!exact.validate || exact.validate(node.data))) return exact
  return FALLBACK
}

/** 列出已注册能力（用于调试 / 能力网可视）。 */
export function listCapabilities(): { kind: NodeKind; label: string }[] {
  return [...REGISTRY.values()].map((r) => ({ kind: Array.isArray(r.kind) ? r.kind.join('|') : r.kind, label: r.label }))
}

export type { JSX }
