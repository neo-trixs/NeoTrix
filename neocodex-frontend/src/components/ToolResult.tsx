// ══════════════════════════════════════════════════════════════════════════
//  ToolResult — 工具执行结果展示
//  展示 agent 工具调用的输入/输出/状态/耗时
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, Show } from 'solid-js'
import { clsx } from 'clsx'

interface ToolCall {
  id: string
  name: string
  arguments?: string
  result?: string
  success?: boolean
  duration_ms?: number
  domain?: string
}

interface ToolResultProps {
  tools: ToolCall[]
}

export function ToolResult(props: ToolResultProps) {
  const [expanded, setExpanded] = createSignal<string | null>(null)

  return (
    <div class="tool-result">
      <div class="tool-header">
        <span class="tool-icon">🔧</span>
        <span class="tool-title">工具调用 ({props.tools.length})</span>
      </div>
      <div class="tool-list">
        {props.tools.map(tc => (
          <div class={clsx('tool-item', tc.success ? 'tool-ok' : 'tool-fail')}>
            <div class="tool-item-header" onClick={() => setExpanded(expanded() === tc.id ? null : tc.id)}>
              <span class={clsx('tool-status-dot', tc.success ? 'dot-ok' : 'dot-fail')} />
              <span class="tool-name">{tc.name}</span>
              <Show when={tc.domain}>
                <span class="tool-domain">{tc.domain}</span>
              </Show>
              <Show when={tc.duration_ms !== undefined}>
                <span class="tool-time">{tc.duration_ms}ms</span>
              </Show>
              <span class="tool-expand">{expanded() === tc.id ? '▼' : '▶'}</span>
            </div>
            <Show when={expanded() === tc.id}>
              <div class="tool-detail">
                <Show when={tc.arguments}>
                  <div class="tool-section">
                    <div class="tool-section-label">输入</div>
                    <pre class="tool-code">{tc.arguments}</pre>
                  </div>
                </Show>
                <Show when={tc.result}>
                  <div class="tool-section">
                    <div class="tool-section-label">输出</div>
                    <pre class="tool-code">{tc.result}</pre>
                  </div>
                </Show>
              </div>
            </Show>
          </div>
        ))}
      </div>
    </div>
  )
}
