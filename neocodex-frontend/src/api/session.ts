/**
 * Session API — 会话管理
 * 所有调用走 domain.ts typed helpers
 */

import * as domain from './domain'

export async function listSessions() {
  return domain.session.list()
}

export async function createSession(name?: string) {
  return domain.session.create(name)
}

export async function deleteSession(id: string) {
  return domain.session.delete(id)
}

export async function switchSession(id: string) {
  return domain.session.switch(id)
}

export async function renameSession(id: string, name: string) {
  return domain.session.rename(id, name)
}

export async function archiveSession(id: string) {
  return domain.session.archive(id)
}

export async function restoreSession(id: string) {
  return domain.session.restore(id)
}

export async function searchSessions(query: string) {
  return domain.session.search(query)
}

export async function setSessionProject(id: string, project: string) {
  return domain.session.setProject(id, project)
}

export async function exportSession(id: string, format?: string) {
  return domain.chat.export(id, format)
}

export async function tagSession(id: string, tag: string) {
  return domain.session.tag(id, tag)
}

export async function untagSession(id: string, tag: string) {
  return domain.session.untag(id, tag)
}
