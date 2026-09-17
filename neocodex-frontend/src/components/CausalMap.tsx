// ══════════════════════════════════════════════════════════════════════════
//  CausalMap — 对话因果链智能画板
//  唯一目标：优雅知性地展示对话中所有因果链条的结构化信息
//  信息隔离：内部 Agent 名称不暴露，只展示用户可见的流程信息
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, Show, For, Switch, Match, createMemo } from 'solid-js'
import { clsx } from 'clsx'
import { Icon, type IconName } from './Icon'

// ── 因果链节点类型 ──

export type CausalKind =
  | 'input'       // 用户输入
  | 'analyze'     // 分析理解
  | 'search'      // 检索信息
  | 'execute'     // 执行操作
  | 'output'      // 产出结果
  | 'decision'    // 决策点
  | 'approval'    // 审批门控
  | 'error'       // 错误
  | 'context'     // 上下文注入

export interface CausalNode {
  id: string
  kind: CausalKind
  /** 用户可见的标题（如「分析需求」「修改文件」） */
  title: string
  /** 用户可见的摘要（如「找到 3 篇相关文档」） */
  summary?: string
  /** 完整内容（展开时显示） */
  content?: string
  /** 时间戳 */
  timestamp: Date
  /** 耗时 */
  duration_ms?: number
  /** 修改的文件列表 */
  files_changed?: string[]
  /** 产出物 */
  outputs?: string[]
  /** 审批状态 */
  approval_status?: 'pending' | 'approved' | 'rejected'
  /** 审批问题（用户可见） */
  approval_question?: string
  /** 结果类型 */
  result_type?: 'text' | 'code' | 'json' | 'image'
  /** 因果连接 */
  causes?: string[]
  caused_by?: string[]
}

// ── 节点类型配置（用户可见） ──

const KIND_CFG: Record<CausalKind, { icon: IconName; color: string }> = {
  input:    { icon: 'input',    color: '#f0913a' },
  analyze:  { icon: 'analyze',  color: '#6366f1' },
  search:   { icon: 'search',   color: '#10b981' },
  execute:  { icon: 'execute',  color: '#f59e0b' },
  output:   { icon: 'output',   color: '#10b981' },
  decision: { icon: 'decision', color: '#8b5cf6' },
  approval: { icon: 'approval', color: '#f59e0b' },
  error:    { icon: 'error',    color: '#ef4444' },
  context:  { icon: 'context',  color: '#14b8a6' },
}

interface CausalMapProps {
  nodes: CausalNode[]
  /** 审批回调 */
  onApprove?: (id: string) => void
  onReject?: (id: string, reason?: string) => void
  /** 结果编辑 */
  onEdit?: (id: string, content: string) => void
}

// ── 主组件 ──

export function CausalMap(props: CausalMapProps) {
  const [expandedId, setExpandedId] = createSignal<string | null>(null)
  const [editingId, setEditingId] = createSignal<string | null>(null)
  const [editContent, setEditContent] = createSignal('')
  const [rejectTarget, setRejectTarget] = createSignal<string | null>(null)
  const [rejectReason, setRejectReason] = createSignal('')

  const sortedNodes = createMemo(() =>
    [...props.nodes].sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime())
  )

  // 统计（内部使用，不展示）
  const stats = createMemo(() => {
    const nodes = sortedNodes()
    return {
      total: nodes.length,
      pending_approval: nodes.filter(n => n.kind === 'approval' && n.approval_status === 'pending').length,
      errors: nodes.filter(n => n.kind === 'error').length,
    }
  })

  // 因果链分组：以 input 节点为链头
  const causalChains = createMemo(() => {
    const chains: CausalNode[][] = []
    const nodes = sortedNodes()
    let current: CausalNode[] = []

    for (const node of nodes) {
      if (node.kind === 'input') {
        if (current.length > 0) chains.push(current)
        current = [node]
      } else {
        current.push(node)
      }
    }
    if (current.length > 0) chains.push(current)
    return chains
  })

  const toggleExpand = (id: string) => {
    setExpandedId(expandedId() === id ? null : id)
  }

  const startEdit = (node: CausalNode) => {
    setEditingId(node.id)
    setEditContent(node.content ?? node.summary ?? '')
  }

  return (
    <div class="cm-root">
      {/* ── 顶部：会话摘要 ── */}
      <div class="cm-header">
        <div class="cm-title-row">
          <span class="cm-title">对话流程</span>
          <Show when={stats().pending_approval > 0}>
            <span class="cm-badge cm-badge-approval">待审批</span>
          </Show>
          <Show when={stats().errors > 0}>
            <span class="cm-badge cm-badge-error">异常</span>
          </Show>
        </div>
      </div>

      {/* ── 因果链视图 ── */}
      <div class="cm-chains">
        <For each={causalChains()}>
          {(chain, chainIdx) => (
            <div class="cm-chain">
              {/* 链头：用户输入 */}
              <Show when={chain[0]}>
                <NodeCard
                  node={chain[0]}
                  expanded={expandedId() === chain[0].id}
                  editing={editingId() === chain[0].id}
                  editContent={editContent()}
                  showReject={rejectTarget() === chain[0].id}
                  rejectReason={rejectReason()}
                  onToggle={() => toggleExpand(chain[0].id)}
                  onStartEdit={() => startEdit(chain[0])}
                  onEditInput={setEditContent}
                  onSaveEdit={() => { props.onEdit?.(chain[0].id, editContent()); setEditingId(null) }}
                  onCancelEdit={() => setEditingId(null)}
                  onApprove={() => props.onApprove?.(chain[0].id)}
                  onReject={() => { props.onReject?.(chain[0].id, rejectReason()); setRejectTarget(null); setRejectReason('') }}
                  onShowReject={() => setRejectTarget(chain[0].id)}
                  onHideReject={() => { setRejectTarget(null); setRejectReason('') }}
                  isChainHead
                />
              </Show>

              {/* 链内节点 */}
              <Show when={chain.length > 1}>
                <div class="cm-chain-body">
                  <For each={chain.slice(1)}>
                    {(node) => (
                      <>
                        <div class="cm-edge">
                          <div class="cm-edge-line" />
                        </div>
                        <NodeCard
                          node={node}
                          expanded={expandedId() === node.id}
                          editing={editingId() === node.id}
                          editContent={editContent()}
                          showReject={rejectTarget() === node.id}
                          rejectReason={rejectReason()}
                          onToggle={() => toggleExpand(node.id)}
                          onStartEdit={() => startEdit(node)}
                          onEditInput={setEditContent}
                          onSaveEdit={() => { props.onEdit?.(node.id, editContent()); setEditingId(null) }}
                          onCancelEdit={() => setEditingId(null)}
                          onApprove={() => props.onApprove?.(node.id)}
                          onReject={() => { props.onReject?.(node.id, rejectReason()); setRejectTarget(null); setRejectReason('') }}
                          onShowReject={() => setRejectTarget(node.id)}
                          onHideReject={() => { setRejectTarget(null); setRejectReason('') }}
                        />
                      </>
                    )}
                  </For>
                </div>
              </Show>

              {/* 链间分隔 */}
              <Show when={chainIdx() < causalChains().length - 1}>
                <div class="cm-chain-sep">
                  <div class="cm-chain-sep-line" />
                  <span class="cm-chain-sep-dot" />
                  <div class="cm-chain-sep-line" />
                </div>
              </Show>
            </div>
          )}
        </For>
      </div>
    </div>
  )
}

// ── 节点卡片 ──

function NodeCard(props: {
  node: CausalNode
  expanded: boolean
  editing: boolean
  editContent: string
  showReject: boolean
  rejectReason: string
  isChainHead?: boolean
  onToggle: () => void
  onStartEdit: () => void
  onEditInput: (v: string) => void
  onSaveEdit: () => void
  onCancelEdit: () => void
  onApprove: () => void
  onReject: () => void
  onShowReject: () => void
  onHideReject: () => void
}) {
  const cfg = () => KIND_CFG[props.node.kind]
  const hasContent = () => props.node.content || props.node.summary
  const isApproval = () => props.node.kind === 'approval' && props.node.approval_status === 'pending'

  return (
    <div
      class={clsx(
        'cm-node',
        `cm-node-${props.node.kind}`,
        props.isChainHead && 'cm-node-head',
        props.expanded && 'cm-node-expanded',
      )}
      onClick={props.onToggle}
    >
      {/* 时间线标记 */}
      <div class="cm-node-marker">
        <div class="cm-node-dot" style={{ background: cfg().color }}>
          <Icon name={cfg().icon} size={10} color="white" stroke-width={2} />
        </div>
        <div class="cm-node-line" />
      </div>

      {/* 内容区 */}
      <div class="cm-node-body">
        {/* 标题行 */}
        <div class="cm-node-header">
          <span class="cm-node-title">{props.node.title}</span>
          <Show when={props.node.duration_ms !== undefined}>
            <span class="cm-node-time">{fmtMs(props.node.duration_ms!)}</span>
          </Show>
        </div>

        {/* 摘要 */}
        <Show when={props.node.summary && !props.expanded}>
          <div class="cm-node-summary">{props.node.summary}</div>
        </Show>

        {/* 修改的文件 */}
        <Show when={props.node.files_changed && props.node.files_changed!.length > 0}>
          <div class="cm-node-files">
            <For each={props.node.files_changed!}>
              {(f) => <span class="cm-file-tag">{f}</span>}
            </For>
          </div>
        </Show>

        {/* 产出物 */}
        <Show when={props.node.outputs && props.node.outputs!.length > 0 && !props.expanded}>
          <div class="cm-node-outputs">
            <For each={props.node.outputs!}>
              {(o) => <span class="cm-output-item">→ {o}</span>}
            </For>
          </div>
        </Show>

        {/* 展开内容 */}
        <Show when={props.expanded && hasContent()}>
          <div class="cm-node-detail" onClick={(e) => e.stopPropagation()}>
            <Switch>
              <Match when={props.editing}>
                <textarea
                  class="cm-detail-editor"
                  value={props.editContent}
                  onInput={(e) => props.onEditInput(e.target.value)}
                />
                <div class="cm-detail-actions">
                  <button class="cm-btn cm-btn-sm cm-btn-save" onClick={props.onSaveEdit}>保存</button>
                  <button class="cm-btn cm-btn-sm cm-btn-cancel" onClick={props.onCancelEdit}>取消</button>
                </div>
              </Match>
              <Match when={props.node.result_type === 'code'}>
                <pre class="cm-detail-code">{props.node.content}</pre>
              </Match>
              <Match when={props.node.result_type === 'json'}>
                <pre class="cm-detail-code">{fmtJson(props.node.content ?? '')}</pre>
              </Match>
              <Match when={true}>
                <div class="cm-detail-text">{props.node.content ?? props.node.summary}</div>
              </Match>
            </Switch>
            <Show when={!props.editing}>
              <button class="cm-detail-edit" onClick={props.onStartEdit}>✏️</button>
            </Show>
          </div>
        </Show>

        {/* 审批（行内） */}
        <Show when={isApproval()}>
          <div class="cm-approval" onClick={(e) => e.stopPropagation()}>
            <Show when={props.node.approval_question}>
              <div class="cm-approval-q">{props.node.approval_question}</div>
            </Show>
            <div class="cm-approval-btns">
              <button class="cm-btn cm-btn-approve" onClick={props.onApprove}>✓ 批准</button>
              <button class="cm-btn cm-btn-reject" onClick={props.onShowReject}>✗ 拒绝</button>
            </div>
            <Show when={props.showReject}>
              <div class="cm-reject-row">
                <input
                  class="cm-reject-input"
                  placeholder="原因（可选）"
                  value={props.rejectReason}
                  onInput={(e) => props.onEditInput(e.target.value)}
                  onKeyDown={(e) => { if (e.key === 'Enter') props.onReject() }}
                />
                <button class="cm-btn cm-btn-sm cm-btn-confirm" onClick={props.onReject}>确认</button>
                <button class="cm-btn cm-btn-sm cm-btn-cancel" onClick={props.onHideReject}>取消</button>
              </div>
            </Show>
          </div>
        </Show>

        {/* 审批结果 */}
        <Show when={props.node.kind === 'approval' && props.node.approval_status === 'approved'}>
          <div class="cm-approval-result cm-approved">✓ 已批准</div>
        </Show>
        <Show when={props.node.kind === 'approval' && props.node.approval_status === 'rejected'}>
          <div class="cm-approval-result cm-rejected">✗ 已拒绝</div>
        </Show>

        {/* 错误 */}
        <Show when={props.node.kind === 'error'}>
          <div class="cm-error" onClick={(e) => e.stopPropagation()}>
            <span>{props.node.summary}</span>
          </div>
        </Show>
      </div>
    </div>
  )
}

// ── 工具函数 ──

function fmtMs(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`
  return `${Math.floor(ms / 60000)}m${Math.round((ms % 60000) / 1000)}s`
}

function fmtJson(s: string): string {
  try { return JSON.stringify(JSON.parse(s), null, 2) } catch { return s }
}
