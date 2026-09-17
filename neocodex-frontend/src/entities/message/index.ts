/**
 * entities/message/ — 消息实体
 *
 * 职责：消息类型定义、消息状态管理、消息 UI 组件
 * 依赖：shared/api, shared/lib
 */
export type { Message, Session, ChatState } from './model/types'
export { MessageBubble, MessageContent, StreamingText, CodeBlock } from './ui'
