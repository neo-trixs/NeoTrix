/**
 * features/send-message/ui/SendButton.tsx — 发送/停止按钮
 *
 * 从 Chat.tsx 提取的发送按钮组件
 */
import { Show } from 'solid-js'
import { Square } from 'lucide-solid'
import { NeoSend } from '../../../components/neo-icons'

interface SendButtonProps {
  isGenerating: () => boolean
  disabled: boolean
  onClick: () => void
}

export function SendButton(props: SendButtonProps) {
  return (
    <button
      class="vc-btn vc-send"
      disabled={props.disabled}
      onClick={props.onClick}
      aria-label={props.isGenerating() ? '停止生成' : '发送消息'}
      title={props.isGenerating() ? '停止生成' : '发送消息'}
    >
      {props.isGenerating() ? <Square class="w-4 h-4" /> : <NeoSend class="w-4 h-4" />}
    </button>
  )
}
