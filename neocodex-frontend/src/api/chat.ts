/**
 * Chat API — 对话管理
 * 所有调用走 domain.ts typed helpers
 */

import * as domain from './domain'

export async function sendMessage(content: string, sessionId?: string) {
  return domain.chat.send(content, sessionId)
}

export async function sendMessageStream(content: string, sessionId?: string) {
  return domain.chat.send(content, sessionId)
}

export async function stopStream() {
  return domain.chat.stop()
}

export async function getHistory(sessionId: string) {
  return domain.chat.history(sessionId)
}

export async function compactSession(sessionId: string) {
  return domain.chat.compact(sessionId)
}

export async function exportSession(sessionId: string, format?: string) {
  return domain.chat.export(sessionId, format)
}

export async function clearSession(sessionId: string) {
  return domain.chat.clear(sessionId)
}

export async function regenerate(sessionId: string, messageIndex: number) {
  return domain.chat.regenerate(sessionId, messageIndex)
}

export async function editMessage(sessionId: string, index: number, content: string) {
  return domain.chat.editMessage(sessionId, index, content)
}

export async function deleteMessage(sessionId: string, index: number) {
  return domain.chat.deleteMessage(sessionId, index)
}

export async function getSideChat(sessionId: string) {
  return domain.chat.sideChat.get(sessionId)
}

export async function sendSideChat(sessionId: string, content: string) {
  return domain.chat.sideChat.send(sessionId, content)
}
