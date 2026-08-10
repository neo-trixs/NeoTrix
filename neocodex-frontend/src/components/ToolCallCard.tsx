import { createSignal, Show } from 'solid-js'
import { ChevronDown, ChevronRight, Check, X, Wrench, Copy } from 'lucide-solid'
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

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`
    return `${(ms / 1000).toFixed(1)}s`
  }

  const copyText = async (key: string, text: string) => {
    try {
      await navigator.clipboard.writeText(text)
      setCopiedKey(key)
      setTimeout(() => setCopiedKey(null), 1500)
    } catch {
      /* ignore */
    }
  }

  return (
    <div class={clsx('tool-inline overflow-hidden my-2', props.call.success ? '' : 'border-red-600/30')}>
      {/* 单行头部 */}
      <button
        class={clsx(
          'w-full flex items-center gap-2 px-2 py-2 text-left hover:bg-white/60 transition-colors',
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
        <Wrench class={clsx('w-3.5 h-3.5 flex-shrink-0', props.call.success ? 'text-nt-io-600' : 'text-red-600')} />
        <span class="text-xs font-medium text-text-primary font-mono truncate">{props.call.name}</span>

        {props.call.success ? (
          <span class="inline-flex items-center gap-1 text-emerald-600 flex-shrink-0">
            <Check class="w-3.5 h-3.5" />
          </span>
        ) : (
          <span class="inline-flex items-center gap-1 text-red-600 flex-shrink-0">
            <X class="w-3.5 h-3.5" />
          </span>
        )}

        <span class="ml-auto text-[11px] text-text-muted flex-shrink-0 tabular-nums">
          {formatDuration(props.call.duration_ms)}
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
                  class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70"
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
                  class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70"
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
