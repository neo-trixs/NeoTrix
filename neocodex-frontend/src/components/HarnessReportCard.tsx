import { createSignal, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import type { HarnessRunResponse } from '../api/harness'

/* ════════════════════════════════════════════
   HarnessReportCard — Harness 重路径执行结果报告面板
   对标 Claude Code 工具结果折叠卡：概览 + 子任务分配 + 内置执行 + 外部求解。
   运行中显示脉冲占位，完成后展开结构化报告（可整体折叠）。
   ════════════════════════════════════════════ */

interface Props {
  running: boolean
  report: HarnessRunResponse | null
  onClose: () => void
}

export function HarnessReportCard(props: Props) {
  const [open, setOpen] = createSignal(true)

  return (
    <div class="harness-report rounded-xl border border-nt-io-500/20 bg-white/55 backdrop-blur-sm shadow-sm overflow-hidden">
      {/* 头部 */}
      <div class="flex items-center gap-2 px-3 py-2">
        <span class="w-1.5 h-1.5 rounded-full bg-nt-io-500 flex-shrink-0" />
        <span class="text-12px font-semibold text-text-primary flex-shrink-0">Harness 执行报告</span>
        <Show when={props.running}>
          <span class="inline-flex items-center gap-1 text-11px text-nt-io-600" role="status" aria-label="执行中">
            <span class="w-3 h-3 rounded-full border-2 border-nt-io-500/30 border-t-nt-io-500 animate-spin" />
            执行中…
          </span>
        </Show>
        <Show when={!props.running && props.report}>
          <span class="text-11px text-text-muted ml-1 truncate">
            拆解 {props.report!.allocations.length} · 内置 {props.report!.internal_results.filter((x) => x.executed).length}/{props.report!.internal_count} · 外部 {props.report!.external_closures.filter((x) => x.solved).length}/{props.report!.external_gap_count}
          </span>
        </Show>
        <button
          class="ml-auto p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70 focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
          onClick={() => setOpen(!open())}
          aria-label={open() ? '折叠报告' : '展开报告'}
          title={open() ? '折叠' : '展开'}
        >
          <Show when={open()} fallback={<span class="text-10px">▸</span>}>
            <span class="text-10px">▾</span>
          </Show>
        </button>
        <button
          class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70 focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
          onClick={props.onClose}
          aria-label="关闭报告"
          title="关闭"
        >
          <span class="text-11px">✕</span>
        </button>
      </div>

      {/* 运行中占位 */}
      <Show when={props.running && !props.report}>
        <div class="px-3 pb-3">
          <div class="h-2 rounded-full bg-nt-io-500/10 overflow-hidden">
            <div class="h-full w-1/3 bg-nt-io-500/60 rounded-full animate-[harnessPulse_1.1s_ease-in-out_infinite]" />
          </div>
        </div>
      </Show>

      {/* 报告主体 */}
      <Show when={open() && props.report}>
        <div class="px-3 pb-3 space-y-2 text-11px">
          {/* 概览 */}
          <div class="flex flex-wrap gap-1.5">
            <span class="px-2 py-0.5 rounded-full bg-nt-io-500/10 text-nt-io-600">子任务 {props.report!.allocations.length}</span>
            <span class="px-2 py-0.5 rounded-full bg-emerald-500/12 text-emerald-600">内置执行 {props.report!.internal_results.filter((x) => x.executed).length}/{props.report!.internal_count}</span>
            <span class="px-2 py-0.5 rounded-full bg-amber-500/12 text-amber-600">外部求解 {props.report!.external_closures.filter((x) => x.solved).length}/{props.report!.external_gap_count}</span>
            <span class="px-2 py-0.5 rounded-full bg-nt-io-500/10 text-nt-io-600">反思补齐 {props.report!.strengthening_actions}</span>
            <Show when={props.report!.external_gaps.length > 0}>
              <span class="px-2 py-0.5 rounded-full bg-red-500/12 text-red-500">未解缺口 {props.report!.external_gaps.length}</span>
            </Show>
          </div>

          {/* 子任务分配 */}
          <Show when={props.report!.allocations.length > 0}>
            <div>
              <div class="text-10px font-medium text-text-muted uppercase tracking-wider mb-1">子任务分配</div>
              <For each={props.report!.allocations}>
                {(a) => (
                  <div class="flex items-center gap-1.5 py-0.5 text-text-secondary">
                    <span class="w-1 h-1 rounded-full bg-text-muted flex-shrink-0" />
                    <span class="font-mono text-nt-io-700">{a.task.capability_tag}</span>
                    <span class="text-text-muted">→ {a.task.domain} · {a.task.specialist}</span>
                  </div>
                )}
              </For>
            </div>
          </Show>

          {/* 内置执行结果 */}
          <Show when={props.report!.internal_results.length > 0}>
            <div>
              <div class="text-10px font-medium text-text-muted uppercase tracking-wider mb-1">内置执行</div>
              <For each={props.report!.internal_results}>
                {(r) => (
                  <div class="py-0.5">
                    <div class="flex items-center gap-1.5">
                      <span class={clsx('w-1.5 h-1.5 rounded-full flex-shrink-0', r.executed ? 'bg-emerald-500' : 'bg-red-500')} />
                      <span class="text-text-primary truncate">{r.summary}</span>
                    </div>
                    <Show when={r.output}>
                      <pre class="mt-1 text-[10px] font-mono text-text-secondary bg-white/60 rounded p-1.5 max-h-24 overflow-auto border border-border-primary/50 whitespace-pre-wrap">{r.output}</pre>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </Show>

          {/* 外部求解闭环 */}
          <Show when={props.report!.external_closures.length > 0}>
            <div>
              <div class="text-10px font-medium text-text-muted uppercase tracking-wider mb-1">外部求解</div>
              <For each={props.report!.external_closures}>
                {(c) => (
                  <div class="flex items-center gap-1.5 py-0.5 text-text-secondary">
                    <span class={clsx('w-1.5 h-1.5 rounded-full flex-shrink-0', c.solved ? 'bg-emerald-500' : 'bg-amber-500')} />
                    <span>{c.solved ? '已解决' : '未解决'}{c.knowledge_acquired ? ' · 已吸收经验' : ''}</span>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  )
}
