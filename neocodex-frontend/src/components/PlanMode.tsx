// ══════════════════════════════════════════════════════════════════════════
//  PlanMode — 交互式规划面板（对标 Devin Interactive Planning + Cursor Plan Mode）
//  Agent 先研究代码库 → 生成计划 → 用户审批 → 执行
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, For, Show } from 'solid-js'
import { clsx } from 'clsx'

export interface PlanStep {
  id: string
  title: string
  description?: string
  status: 'pending' | 'approved' | 'running' | 'completed' | 'skipped'
  files?: string[]
  estimatedMs?: number
}

export interface PlanModeProps {
  steps: () => PlanStep[]
  onApprove: (stepId: string) => void
  onReject: (stepId: string) => void
  onApproveAll: () => void
  onStart: () => void
  onCancel: () => void
  isRunning: () => boolean
  title?: string
  description?: string
}

const STATUS_CONFIG: Record<string, { icon: string; color: string; bg: string }> = {
  pending: { icon: '○', color: 'text-text-muted', bg: 'bg-slate-100' },
  approved: { icon: '✓', color: 'text-emerald-600', bg: 'bg-emerald-50' },
  running: { icon: '⟳', color: 'text-amber-600', bg: 'bg-amber-50' },
  completed: { icon: '●', color: 'text-emerald-600', bg: 'bg-emerald-50' },
  skipped: { icon: '–', color: 'text-text-muted', bg: 'bg-slate-50' },
}

export function PlanMode(props: PlanModeProps) {
  const [expanded, setExpanded] = createSignal<string | null>(null)

  const approvedCount = () => props.steps().filter(s => s.status === 'approved' || s.status === 'completed').length
  const totalCount = () => props.steps().length

  return (
    <div class="border border-border-primary/20 rounded-xl bg-white/80 backdrop-blur-sm overflow-hidden">
      {/* Header */}
      <div class="flex items-center justify-between px-4 py-3 border-b border-border-primary/10 bg-gradient-to-r from-nt-io-500/5 to-transparent">
        <div class="flex items-center gap-2">
          <span class="text-lg">📋</span>
          <div>
            <h3 class="text-sm font-semibold text-text-primary">{props.title ?? '执行计划'}</h3>
            <Show when={props.description}>
              <p class="text-xs text-text-muted mt-0.5">{props.description}</p>
            </Show>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-xs text-text-muted">{approvedCount()}/{totalCount()} 已审批</span>
          <Show when={!props.isRunning()}>
            <button
              class="px-3 py-1.5 text-xs font-medium rounded-lg bg-nt-io-500 text-white hover:bg-nt-io-600 transition-colors"
              onClick={props.onApproveAll}
            >
              全部审批
            </button>
            <button
              class="px-3 py-1.5 text-xs font-medium rounded-lg bg-emerald-500 text-white hover:bg-emerald-600 transition-colors"
              onClick={props.onStart}
              disabled={approvedCount() === 0}
            >
              开始执行
            </button>
          </Show>
          <Show when={props.isRunning()}>
            <button
              class="px-3 py-1.5 text-xs font-medium rounded-lg bg-red-500 text-white hover:bg-red-600 transition-colors"
              onClick={props.onCancel}
            >
              取消
            </button>
          </Show>
        </div>
      </div>

      {/* Steps */}
      <div class="divide-y divide-border-primary/10">
        <For each={props.steps()}>
          {(step) => {
            const cfg = STATUS_CONFIG[step.status]
            const isExpanded = () => expanded() === step.id
            return (
              <div class="px-4 py-2.5 hover:bg-white/60 transition-colors">
                <div class="flex items-center gap-3">
                  {/* Status icon */}
                  <span class={clsx('text-sm w-5 text-center', cfg.color, step.status === 'running' && 'animate-spin')}>
                    {cfg.icon}
                  </span>

                  {/* Title */}
                  <span class={clsx('text-sm flex-1', step.status === 'completed' ? 'text-text-muted line-through' : 'text-text-primary')}>
                    {step.title}
                  </span>

                  {/* Files count */}
                  <Show when={step.files?.length}>
                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-slate-100 text-text-muted">
                      {step.files!.length} 文件
                    </span>
                  </Show>

                  {/* Estimated time */}
                  <Show when={step.estimatedMs}>
                    <span class="text-[10px] text-text-muted">~{Math.round(step.estimatedMs! / 1000)}s</span>
                  </Show>

                  {/* Actions */}
                  <Show when={step.status === 'pending' && !props.isRunning()}>
                    <button
                      class="text-xs px-2 py-1 rounded bg-emerald-50 text-emerald-600 hover:bg-emerald-100 transition-colors"
                      onClick={() => props.onApprove(step.id)}
                    >
                      审批
                    </button>
                    <button
                      class="text-xs px-2 py-1 rounded bg-slate-50 text-text-muted hover:bg-slate-100 transition-colors"
                      onClick={() => props.onReject(step.id)}
                    >
                      跳过
                    </button>
                  </Show>

                  {/* Expand toggle */}
                  <Show when={step.description || step.files?.length}>
                    <button
                      class="text-text-muted hover:text-text-primary text-xs"
                      onClick={() => setExpanded(isExpanded() ? null : step.id)}
                    >
                      {isExpanded() ? '▾' : '▸'}
                    </button>
                  </Show>
                </div>

                {/* Expanded detail */}
                <Show when={isExpanded()}>
                  <div class="mt-2 ml-8 text-xs text-text-muted space-y-1">
                    <Show when={step.description}>
                      <p>{step.description}</p>
                    </Show>
                    <Show when={step.files?.length}>
                      <div class="flex flex-wrap gap-1 mt-1">
                        <For each={step.files!}>
                          {(f) => (
                            <span class="px-1.5 py-0.5 rounded bg-slate-100 text-text-muted font-mono text-[10px]">
                              {f}
                            </span>
                          )}
                        </For>
                      </div>
                    </Show>
                  </div>
                </Show>
              </div>
            )
          }}
        </For>
      </div>
    </div>
  )
}
