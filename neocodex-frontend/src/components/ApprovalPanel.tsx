import { createSignal, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import type { HarnessApproval } from '../api/harness'

/* ════════════════════════════════════════════
   ApprovalPanel — 左栏 bot「审批流」可见性面板
   对标 Claude Code 的权限审批队列：展示 Harness 外部动作待审批项。
   当前后端仅暴露只读 harness_approval_list（无 approve/reject 端点），
   故本面板为只读展示；待后端补齐审批动作端点后再接交互。
   ════════════════════════════════════════════ */

interface Props {
  approvals: HarnessApproval[]
  onClose: () => void
}

export function ApprovalPanel(props: Props) {
  const [open, setOpen] = createSignal(true)
  const pending = () => props.approvals.filter((a) => a.state === 'pending' || a.state === 'awaiting')

  return (
    <div class="approval-panel rounded-xl border border-amber-500/25 bg-white/55 backdrop-blur-sm shadow-sm overflow-hidden">
      <div class="flex items-center gap-2 px-3 py-2">
        <span class="w-1.5 h-1.5 rounded-full bg-amber-500 flex-shrink-0" />
        <span class="text-12px font-semibold text-text-primary flex-shrink-0">待审批动作</span>
        <Show when={pending().length > 0}>
          <span class="px-1.5 py-0.5 rounded-full bg-amber-500/15 text-amber-600 text-10px font-medium">{pending().length} 待处理</span>
        </Show>
        <button
          class="ml-auto p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70 focus-visible:ring-2 focus-visible:ring-amber-500 focus-visible:outline-none"
          onClick={() => setOpen(!open())}
          aria-label={open() ? '折叠审批面板' : '展开审批面板'}
          title={open() ? '折叠' : '展开'}
        >
          <span class="text-10px">{open() ? '▾' : '▸'}</span>
        </button>
        <button
          class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70 focus-visible:ring-2 focus-visible:ring-amber-500 focus-visible:outline-none"
          onClick={props.onClose}
          aria-label="关闭审批面板"
          title="关闭"
        >
          <span class="text-11px">✕</span>
        </button>
      </div>

      <Show when={open()}>
        <div class="px-3 pb-3 space-y-1.5 text-11px">
          <Show
            when={props.approvals.length > 0}
            fallback={<div class="text-text-muted py-1">无待审批动作（Harness 外部操作无需人工确认时为空）</div>}
          >
            <For each={props.approvals}>
              {(a) => (
                <div class="flex items-center gap-2 py-1 px-2 rounded bg-white/50 border border-border-primary/40">
                  <span class={clsx('w-1.5 h-1.5 rounded-full flex-shrink-0', a.state === 'pending' || a.state === 'awaiting' ? 'bg-amber-500' : 'bg-emerald-500')} />
                  <span class="font-mono text-nt-io-700 truncate">{a.action}</span>
                  <span class="text-text-muted ml-auto truncate">{a.id}</span>
                  <span class={clsx('px-1.5 py-0.5 rounded-full text-10px', a.state === 'pending' || a.state === 'awaiting' ? 'bg-amber-500/15 text-amber-600' : 'bg-emerald-500/12 text-emerald-600')}>
                    {a.state}
                  </span>
                </div>
              )}
            </For>
          </Show>
        </div>
      </Show>
    </div>
  )
}
