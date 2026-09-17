// ══════════════════════════════════════════════════════════════════════════
//  Smart Canvas — 极简渲染器
//  只保留对话产生的结果节点：执行流/计划/数据格式。
//  能力逻辑全在后端，前端只做渲染。
// ══════════════════════════════════════════════════════════════════════════
import { type NodeRenderer, type NodeRenderContext } from './types'
import { registerNodeRenderer } from './nodeRegistry'
import { CAPABILITIES } from './capabilities'

/** 注册所有画布节点渲染器 */
export function registerPanelRenderers(): void {
  // ── 执行流程 (Agent tool call chain) ──
  registerNodeRenderer({
    kind: 'flow',
    label: '执行流程',
    defaultSize: { w: 380, h: 240 },
    render: (ctx) => {
      const data = ctx.node.data as {
        tools?: { name: string; success?: boolean; duration_ms?: number; domain?: string }[]
        status?: string
        failed?: number
      }
      const tools = data.tools ?? []
      const statusColor = data.status === 'running'
        ? 'var(--nt-color-gold-500, #f0913a)'
        : data.status === 'failed'
          ? 'var(--nt-color-danger, #dc2626)'
          : 'var(--nt-color-teal-500, #14b8a6)'

      return (
        <div class="sc-body">
          <div class="sc-flow-pipeline">
            {tools.map((t, i) => (
              <div class="sc-flow-step">
                <div
                  class="sc-flow-dot"
                  style={{ background: t.success === undefined ? '#f59e0b' : t.success ? statusColor : '#dc2626' }}
                />
                <div class="sc-flow-info">
                  <span class="sc-flow-name">{t.name}</span>
                  {t.duration_ms !== undefined && (
                    <span class="sc-flow-time">{t.duration_ms}ms</span>
                  )}
                  {t.domain && (
                    <span class="sc-flow-domain">{t.domain}</span>
                  )}
                </div>
                {i < tools.length - 1 && <div class="sc-flow-line" />}
              </div>
            ))}
          </div>
          {data.failed !== undefined && data.failed > 0 && (
            <div class="sc-flow-summary" style={{ color: '#dc2626' }}>
              ✗ {data.failed} failed
            </div>
          )}
        </div>
      )
    },
  })

  // ── 执行计划 (Plan mode) ──
  registerNodeRenderer({
    kind: 'plan',
    label: '执行计划',
    defaultSize: { w: 400, h: 280 },
    render: (ctx) => {
      const data = ctx.node.data as {
        title?: string
        steps?: { title: string; status: string; files?: string[] }[]
        approved?: number
        total?: number
      }
      const steps = data.steps ?? []
      return (
        <div class="sc-body">
          <div class="sc-panel-header">
            <span class="sc-panel-icon">📋</span>
            <span class="sc-panel-title">{data.title ?? '执行计划'}</span>
            <span class="sc-panel-badge">{data.approved ?? 0}/{data.total ?? steps.length}</span>
          </div>
          <div class="sc-plan-steps">
            {steps.map((s) => (
              <div class="sc-plan-step">
                <span class={`sc-plan-dot sc-plan-dot-${s.status}`} />
                <span class="sc-plan-label">{s.title}</span>
                {s.files && s.files.length > 0 && (
                  <span class="sc-plan-files">{s.files.length} 文件</span>
                )}
              </div>
            ))}
          </div>
        </div>
      )
    },
  })

  // ── 活动流 (Activity feed) ──
  registerNodeRenderer({
    kind: 'activity',
    label: '活动流',
    defaultSize: { w: 360, h: 300 },
    render: (ctx) => {
      const data = ctx.node.data as {
        items?: { title: string; type: string; status: string; time?: string }[]
        count?: number
      }
      const items = data.items ?? []
      return (
        <div class="sc-body">
          <div class="sc-panel-header">
            <span class="sc-panel-icon">📋</span>
            <span class="sc-panel-title">活动流</span>
            <span class="sc-panel-badge">{data.count ?? items.length}</span>
          </div>
          <div class="sc-activity-list">
            {items.slice(0, 8).map((item) => (
              <div class="sc-activity-item">
                <span class={`sc-activity-dot sc-activity-dot-${item.status}`} />
                <span class="sc-activity-title">{item.title}</span>
                <span class="sc-activity-type">{item.type}</span>
                {item.time && <span class="sc-activity-time">{item.time}</span>}
              </div>
            ))}
          </div>
        </div>
      )
    },
  })

  // ── 代码块 ──
  registerNodeRenderer({
    kind: 'code',
    label: '代码',
    defaultSize: { w: 420, h: 320 },
    render: (ctx) => {
      const data = ctx.node.data as { language?: string; content?: string; filename?: string }
      return (
        <div class="sc-body">
          <div class="sc-panel-header">
            <span class="sc-panel-icon">💻</span>
            <span class="sc-panel-title">{data.filename ?? data.language ?? '代码'}</span>
          </div>
          <pre class="sc-code" style={{ 'max-height': '280px', 'overflow': 'auto' }}>
            {data.content ?? '// 代码'}
          </pre>
        </div>
      )
    },
  })

  // ── 表格 ──
  registerNodeRenderer({
    kind: 'table',
    label: '表格',
    defaultSize: { w: 440, h: 320 },
    render: (ctx) => {
      const data = ctx.node.data as { headers?: string[]; rows?: string[][]; title?: string }
      const headers = data.headers ?? []
      const rows = data.rows ?? []
      return (
        <div class="sc-body">
          <div class="sc-panel-header">
            <span class="sc-panel-icon">📊</span>
            <span class="sc-panel-title">{data.title ?? '表格'}</span>
            <span class="sc-panel-badge">{rows.length} 行</span>
          </div>
          <div class="sc-table-wrap">
            <table class="sc-table">
              <thead>
                <tr>{headers.map(h => <th>{h}</th>)}</tr>
              </thead>
              <tbody>
                {rows.map(row => (
                  <tr>{row.map(cell => <td>{cell}</td>)}</tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )
    },
  })

  // ── 图表 ──
  registerNodeRenderer({
    kind: 'chart',
    label: '图表',
    defaultSize: { w: 400, h: 280 },
    render: (ctx) => {
      const data = ctx.node.data as { type?: string; title?: string; values?: number[] }
      return (
        <div class="sc-body">
          <div class="sc-panel-header">
            <span class="sc-panel-icon">📈</span>
            <span class="sc-panel-title">{data.title ?? '图表'}</span>
          </div>
          <div class="sc-chart-placeholder">
            {data.type ?? 'bar'} 图表 — 后端接线
          </div>
        </div>
      )
    },
  })

  // ── 图像 ──
  registerNodeRenderer({
    kind: 'image',
    label: '图像',
    defaultSize: { w: 360, h: 280 },
    render: (ctx) => {
      const data = ctx.node.data as { url?: string; alt?: string; width?: number; height?: number }
      return (
        <div class="sc-body">
          <div class="sc-panel-header">
            <span class="sc-panel-icon">🖼️</span>
            <span class="sc-panel-title">{data.alt ?? '图像'}</span>
          </div>
          {data.url ? (
            <img src={data.url} alt={data.alt ?? ''} class="sc-image" style={{ 'max-height': '240px', 'object-fit': 'contain' }} />
          ) : (
            <div class="sc-chart-placeholder">图像占位</div>
          )}
        </div>
      )
    },
  })

  // ── Markdown 文档 ──
  registerNodeRenderer({
    kind: 'markdown',
    label: '文档',
    defaultSize: { w: 400, h: 320 },
    render: (ctx) => {
      const data = ctx.node.data as { content?: string; title?: string }
      return (
        <div class="sc-body">
          <div class="sc-panel-header">
            <span class="sc-panel-icon">📄</span>
            <span class="sc-panel-title">{data.title ?? '文档'}</span>
          </div>
          <div class="sc-markdown" style={{ 'max-height': '280px', 'overflow': 'auto' }}>
            {data.content ?? '文档内容'}
          </div>
        </div>
      )
    },
  })

  // ── Mermaid 流程图 ──
  registerNodeRenderer({
    kind: 'mermaid',
    label: '流程图',
    defaultSize: { w: 400, h: 300 },
    render: (ctx) => {
      const data = ctx.node.data as { definition?: string; title?: string }
      return (
        <div class="sc-body">
          <div class="sc-panel-header">
            <span class="sc-panel-icon">🔗</span>
            <span class="sc-panel-title">{data.title ?? '流程图'}</span>
          </div>
          <div class="sc-chart-placeholder">
            Mermaid 渲染 — 后端接线
          </div>
        </div>
      )
    },
  })

  // ── JSON ──
  registerNodeRenderer({
    kind: 'json',
    label: 'JSON',
    defaultSize: { w: 400, h: 320 },
    render: (ctx) => {
      const data = ctx.node.data as { content?: string; title?: string }
      return (
        <div class="sc-body">
          <div class="sc-panel-header">
            <span class="sc-panel-icon">{'{ }'}</span>
            <span class="sc-panel-title">{data.title ?? 'JSON'}</span>
          </div>
          <pre class="sc-code" style={{ 'max-height': '280px', 'overflow': 'auto' }}>
            {data.content ?? '{}'}
          </pre>
        </div>
      )
    },
  })

  // ── Diff 差异 ──
  registerNodeRenderer({
    kind: 'diff',
    label: '差异',
    defaultSize: { w: 420, h: 320 },
    render: (ctx) => {
      const data = ctx.node.data as { hunks?: { oldStart: number; newStart: number; lines: string[] }[]; filename?: string }
      const hunks = data.hunks ?? []
      return (
        <div class="sc-body">
          <div class="sc-panel-header">
            <span class="sc-panel-icon">🔀</span>
            <span class="sc-panel-title">{data.filename ?? '差异'}</span>
          </div>
          <pre class="sc-code sc-diff" style={{ 'max-height': '280px', 'overflow': 'auto' }}>
            {hunks.flatMap(h => h.lines).join('\n') || '// 无差异'}
          </pre>
        </div>
      )
    },
  })

  // ── 路由节点（点击打开全屏路由） ──
  const routeCapabilities = CAPABILITIES.filter(c => c.route)
  for (const cap of routeCapabilities) {
    registerNodeRenderer({
      kind: cap.kind,
      label: cap.label,
      defaultSize: { w: 200, h: 120 },
      render: (ctx) => (
        <div
          class="sc-body sc-route-node"
          style={{ cursor: 'pointer' }}
          onClick={() => { if (cap.route) window.location.hash = cap.route }}
        >
          <div class="sc-route-icon">{cap.icon}</div>
          <div class="sc-route-title">{cap.label}</div>
          <div class="sc-route-hint">点击打开 →</div>
        </div>
      ),
    })
  }
}
