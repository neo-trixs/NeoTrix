/**
 * entities/message/ui/ — 消息相关 UI 组件
 *
 * 从 components/ 重导出，建立 FSD 归属
 * 原始实现保留在 components/，后续逐步迁移
 *
 * 已清理：MessageBubble / MessageContent 为死代码（无实际导入者），
 * 消息渲染内联在 routes/Chat.tsx 中。
 */
export { StreamingText } from '../../../components/StreamingText'
export { CodeBlock } from '../../../components/CodeBlock'
