/**
 * Session API — 会话管理
 * 
 * 通过 domain_call('session', action, args) 调用后端 SessionPlugin。
 */
import { domainCall } from './domain-client'

export interface SessionInfo {
  id: string
  name: string
  message_count: number
  created_at: number
  updated_at: number
  project: string
  sort_order: number
}

/**
 * 列出所有会话
 */
export async function listSessions(): Promise<SessionInfo[]> {
  return domainCall<SessionInfo[]>('session', 'list')
}

/**
 * 创建新会话
 */
export async function createSession(name?: string): Promise<{ id: string; name: string }> {
  return domainCall('session', 'create', { name: name || '新会话' })
}

/**
 * 删除会话
 */
export async function deleteSession(id: string): Promise<void> {
  await domainCall('session', 'delete', { id })
}

/**
 * 切换当前会话
 */
export async function switchSession(id: string): Promise<void> {
  await domainCall('session', 'switch', { id })
}

/**
 * 拖拽排序
 */
export async function reorderSessions(ids: string[]): Promise<void> {
  await domainCall('session', 'reorder', { ids })
}

/**
 * 设置会话所属项目
 */
export async function setSessionProject(id: string, project: string): Promise<void> {
  await domainCall('session', 'set_project', { id, project })
}

/**
 * 分支新话题
 */
export async function forkSession(fromId: string, upTo?: number): Promise<string> {
  const result = await domainCall<{ id: string }>('session', 'fork', { from_id: fromId, up_to: upTo })
  return result.id
}

/**
 * 搜索会话
 */
export async function searchSessions(query: string): Promise<SessionInfo[]> {
  return domainCall<SessionInfo[]>('session', 'search', { query })
}
