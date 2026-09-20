/**
 * features/send-message/ui/InputArea.tsx — 输入区域
 *
 * 从 Chat.tsx 提取的输入区域组件（textarea + 附件 + 模型选择 + 发送按钮）
 */
import { Show, type JSX } from 'solid-js'
import { ModelSwitcher } from '../../../components/ModelSwitcher'
import { SendButton } from './SendButton'
import { estimateTokens } from '../../../lib/text'
import type { NeoCodexAttachmentDto } from '../../../stores/chat'

interface InputAreaProps {
  inputValue: () => string
  onInput: (e: Event) => void
  onKeyDown: (e: KeyboardEvent) => void
  onPaste: (e: ClipboardEvent) => void
  textareaRef: (el: HTMLTextAreaElement) => void
  isGenerating: () => boolean
  pendingAttachments: () => NeoCodexAttachmentDto[]
  annotationHint: () => string | null
  onPickAttachment: () => void
  onSend: () => void
  onStop: () => void
}

export function InputArea(props: InputAreaProps) {
  const isDisabled = () =>
    !props.inputValue().trim() &&
    props.pendingAttachments().length === 0 &&
    !props.annotationHint() &&
    !props.isGenerating()

  return (
    <div class="cic">
      <textarea
        ref={props.textareaRef}
        class="flex-1 bg-transparent border-none resize-none min-h-[52px] max-h-[240px] py-2 text-[14px] leading-relaxed text-text-primary placeholder-text-muted/70 focus:outline-none focus:ring-0 focus:border-none"
        placeholder={props.isGenerating() ? '生成中仍可输入，下一条稍后发送…' : '输入消息… (Enter 发送, Shift+Enter 换行)'}
        value={props.inputValue()}
        onInput={props.onInput}
        onKeyDown={props.onKeyDown}
        onPaste={props.onPaste}
        rows={1}
      />
      <div class="cic-actions">
        <div class="cic-left">
          <button class="cic-attach" onClick={props.onPickAttachment} aria-label="附加文件" title="附加文件">
            <svg viewBox="0 0 16 16">
              <line x1="8" y1="3" x2="8" y2="11" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
              <line x1="4" y1="8" x2="12" y2="8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
            </svg>
          </button>
          <ModelSwitcher disabled={props.isGenerating()} />
        </div>
        <div class="cic-right">
          <Show when={props.inputValue().trim() || props.pendingAttachments().length > 0}>
            <span class="text-10px text-text-muted/70 font-mono mr-2">
              ≈{estimateTokens(props.inputValue())} tok
              <Show when={props.pendingAttachments().length > 0}> · {props.pendingAttachments().length} 附件</Show>
            </span>
          </Show>
          <SendButton
            isGenerating={props.isGenerating}
            disabled={isDisabled()}
            onClick={props.isGenerating() ? props.onStop : props.onSend}
          />
        </div>
      </div>
    </div>
  )
}
