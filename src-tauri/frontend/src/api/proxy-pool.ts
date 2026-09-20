/**
 * Proxy Pool API — 代理 IP 池管理
 * 
 * 通过 Tauri invoke 调用后端 proxy_pool 命令。
 */
import { invoke } from '@tauri-apps/api/core'

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
  return invoke<ProxyPoolStatus>('proxy_pool_status')
}

/**
 * 获取代理池快照
 */
export async function getProxyPoolSnapshot(): Promise<ProxyPoolSnapshot> {
  return invoke<ProxyPoolSnapshot>('proxy_pool_snapshot')
}

/**
 * 添加代理节点
 */
export async function addProxyNode(url: string, tag: string): Promise<ProxyPoolEntry> {
  return invoke<ProxyPoolEntry>('proxy_pool_add', { url, tag })
}

/**
 * 删除代理节点
 */
export async function removeProxyNode(url: string): Promise<boolean> {
  return invoke<boolean>('proxy_pool_remove', { url })
}

/**
 * 添加订阅源
 */
export async function addSubscription(url: string): Promise<string[]> {
  return invoke<string[]>('proxy_pool_add_subscription', { url })
}

/**
 * 删除订阅源
 */
export async function removeSubscription(url: string): Promise<string[]> {
  return invoke<string[]>('proxy_pool_remove_subscription', { url })
}

/**
 * 设置选择策略
 */
export async function setProxyStrategy(strategy: string): Promise<string> {
  return invoke<string>('proxy_pool_set_strategy', { strategy })
}

/**
 * 获取可用策略列表
 */
export async function listProxyStrategies(): Promise<string[]> {
  return invoke<string[]>('proxy_pool_list_strategies')
}
