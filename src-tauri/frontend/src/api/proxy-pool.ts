/**
 * Proxy Pool API — 代理 IP 池管理
 * 
 * 迁移到新架构：状态查询走 chatSend，写操作保留 domain call。
 */
import { chatSend, extractResult } from './chat'

/** 代理池条目 */
export interface ProxyPoolEntry {
  url: string
  tag: string
  geo_tag: string | null
  latency_ms: number | null
  success_count: number
  fail_count: number
  speed_tier: string
  from_subscription: boolean
}

/** 代理池状态 */
export interface ProxyPoolStatus {
  total: number
  healthy: number
  unhealthy: number
  strategy: string
  nodes: ProxyPoolEntry[]
  subscriptions: string[]
}

/** 代理池快照 */
export interface ProxyPoolSnapshot {
  total: number
  healthy: number
  avg_latency_ms: number
  strategy: string
  geo_distribution: Record<string, number>
  speed_tiers: Record<string, number>
}

/**
 * 获取代理池状态
 */
export async function getProxyPoolStatus(): Promise<ProxyPoolStatus> {
  const response = await chatSend("查看代理池状态")
  return extractResult<ProxyPoolStatus>(response, { total: 0, healthy: 0, unhealthy: 0, strategy: 'round_robin', nodes: [], subscriptions: [] })
}

/**
 * 获取代理池快照
 */
export async function getProxyPoolSnapshot(): Promise<ProxyPoolSnapshot> {
  const response = await chatSend("查看代理池快照")
  return extractResult<ProxyPoolSnapshot>(response, { total: 0, healthy: 0, avg_latency_ms: 0, strategy: 'round_robin', geo_distribution: {}, speed_tiers: {} })
}

/**
 * 添加代理节点
 */
export async function addProxyNode(url: string, tag: string): Promise<ProxyPoolEntry> {
  const response = await chatSend(`添加代理 ${url} 标签 ${tag}`)
  return extractResult<ProxyPoolEntry>(response, { url, tag, geo_tag: null, latency_ms: null, success_count: 0, fail_count: 0, speed_tier: 'normal', from_subscription: false })
}

/**
 * 删除代理节点
 */
export async function removeProxyNode(url: string): Promise<boolean> {
  const response = await chatSend(`删除代理 ${url}`)
  return response.actions.length > 0
}

/**
 * 添加订阅源
 */
export async function addSubscription(url: string): Promise<string[]> {
  const response = await chatSend(`添加代理订阅 ${url}`)
  return response.actions.length > 0 ? [url] : []
}

/**
 * 删除订阅源
 */
export async function removeSubscription(url: string): Promise<string[]> {
  const response = await chatSend(`删除代理订阅 ${url}`)
  return response.actions.length > 0 ? [] : [url]
}

/**
 * 设置选择策略
 */
export async function setProxyStrategy(strategy: string): Promise<string> {
  const response = await chatSend(`设置代理策略为 ${strategy}`)
  return strategy
}

/**
 * 获取可用策略列表
 */
export async function listProxyStrategies(): Promise<string[]> {
  const response = await chatSend("查看可用代理策略")
  return extractResult<string[]>(response, ['round_robin', 'least_latency', 'random'])
}
