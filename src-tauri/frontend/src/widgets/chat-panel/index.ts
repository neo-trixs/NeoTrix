/**
 * widgets/chat-panel/ — 对话面板组件
 *
 * 职责：消息列表、输入区域、滚动行为
 * 依赖：features/send-message, features/stream-response, entities/message
 */
export { useChatState, useChatActions, useStreamHandlers } from './model'
export type { ChatStateReturn, ChatActionsReturn } from './model'
