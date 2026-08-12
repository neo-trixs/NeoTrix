import { createSignal, onCleanup, Show } from 'solid-js'
import { ChevronDown, ChevronRight, Check, X, Wrench, Copy, Loader2 } from 'lucide-solid'
import type { ToolCallRecord } from '../stores/chat'
import { clsx } from 'clsx'

/**
 * ToolCallCard — 紧凑内联工具调用块（对标 Claude Code 内联）。
 * 一行：Wrench图标 + tool_name + 成功✓/失败✗ + 耗时，默认折叠。
 * 展开显示 args / result 的 pre 块。
 */
export function ToolCallCard(props: { call: ToolCallRecord }) {
  const [expanded, setExpanded] = createSignal(false)
  const [copiedKey, setCopiedKey] = createSignal<string | null>(null)
  let copyTimer: number | undefined
  // 卸载时清理复制反馈定时器，避免泄漏
  onCleanup(() => window.clearTimeout(copyTimer))

  // 结束判定：所有 neocodex_stream_tool 事件均为工具执行完成事件（后端只在
  // 执行后回调 on_tool，无"开始"事件），因此 duration_ms 存在即视为已结束。
  // 不能再用 duration_ms 是否为 0 推断"执行中"——0 时长可能是成功（exit_code=0）
  // 或失败（审批拒绝 / 参数错误 / TOOL_ERROR），否则会永久误标"执行中…"。
  const isRunning = () => props.call.duration_ms == null

  // 失败原因：result 常以 TOOL_ERROR / 具体错误负载，截取首行在前端标头透出
  const failureReason = () => {
    if (props.call.success || !props.call.result) return null
    return props.call.result.split('\n')[0].slice(0, 120)
  }

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`
    return `${(ms / 1000).toFixed(1)}s`
  }

  const copyText = async (key: string, text: string) => {
    try {
      await navigator.clipboard.writeText(text)
      setCopiedKey(key)
      window.clearTimeout(copyTimer)
      copyTimer = window.setTimeout(() => setCopiedKey(null), 1500)
    } catch {
      /* ignore */
    }
  }

  return (
    <div class={clsx('tool-inline overflow-hidden my-2', props.call.success ? '' : 'border-red-600/30')}>
      {/* 单行头部 */}
      <button
        class={clsx(
          'w-full flex items-center gap-2 px-2 py-2 text-left hover:bg-white/60 transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none',
          expanded() && 'border-b border-border-primary/60'
        )}
        onClick={() => setExpanded(!expanded())}
        aria-expanded={expanded()}
      >
        {expanded() ? (
          <ChevronDown class="w-3.5 h-3.5 text-text-muted flex-shrink-0" />
        ) : (
          <ChevronRight class="w-3.5 h-3.5 text-text-muted flex-shrink-0" />
        )}
        <Wrench class={clsx('w-3.5 h-3.5 flex-shrink-0', isRunning() ? 'text-text-muted' : props.call.success ? 'text-nt-io-600' : 'text-red-600')} />
        <span class="text-xs font-medium text-text-primary font-mono truncate">{props.call.name}</span>

        {isRunning() ? (
          <span class="inline-flex items-center gap-1 text-nt-io-600 flex-shrink-0" role="status" aria-label="工具执行中">
            <Loader2 class="w-3.5 h-3.5 animate-spin" />
            执行中…
          </span>
        ) : props.call.success ? (
          <span class="inline-flex items-center gap-1 text-emerald-600 flex-shrink-0" role="status" aria-label="工具调用成功">
            <Check class="w-3.5 h-3.5" />
          </span>
        ) : (
          <span class="inline-flex items-center gap-1 text-red-600 flex-shrink-0" role="status" aria-label="工具调用失败">
            <X class="w-3.5 h-3.5" />
          </span>
        )}

        {!isRunning() && !props.call.success && failureReason() && (
          <span class="text-[11px] text-red-600/80 flex-shrink-0 min-w-0 truncate max-w-[220px]" title={props.call.result}>
            {failureReason()}
          </span>
        )}

        <span class="ml-auto text-[11px] text-text-muted flex-shrink-0 tabular-nums">
          {isRunning() ? '…' : formatDuration(props.call.duration_ms)}
        </span>
      </button>

      {/* 展开详情 */}
      <Show when={expanded()}>
        <div class="px-2 py-2 space-y-2 bg-white/30">
          <Show when={props.call.args}>
            <div>
              <div class="flex items-center justify-between mb-1">
                <span class="text-[10px] font-medium text-text-muted uppercase tracking-wider">args</span>
                <button
                  class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70 focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
                  onClick={() => copyText('args', props.call.args)}
                  aria-label="复制参数"
                >
                  <Show when={copiedKey() === 'args'} fallback={<Copy class="w-3.5 h-3.5" />}>
                    <Check class="w-3.5 h-3.5 text-emerald-600" />
                  </Show>
                </button>
              </div>
              <pre class="text-[11px] font-mono text-text-secondary bg-white/60 rounded p-2 overflow-x-auto max-h-40 overflow-y-auto border border-border-primary/50">
                {props.call.args}
              </pre>
            </div>
          </Show>
          <Show when={props.call.result}>
            <div>
              <div class="flex items-center justify-between mb-1">
                <span class="text-[10px] font-medium text-text-muted uppercase tracking-wider">result</span>
                <button
                  class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70 focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
                  onClick={() => copyText('result', props.call.result)}
                  aria-label="复制结果"
                >
                  <Show when={copiedKey() === 'result'} fallback={<Copy class="w-3.5 h-3.5" />}>
                    <Check class="w-3.5 h-3.5 text-emerald-600" />
                  </Show>
                </button>
              </div>
              <pre class="text-[11px] font-mono text-text-secondary bg-white/60 rounded p-2 overflow-x-auto max-h-48 overflow-y-auto border border-border-primary/50">
                {props.call.result}
              </pre>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  )
}
