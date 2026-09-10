/**
 * API Health Check Utilities — 后端 API 连通性测试
 * 
 * 测试每个域插件的 API 可用性和响应时间
 */
import { domain } from '../api'
import * as modelPool from '../api/model-pool'
import * as proxyPool from '../api/proxy-pool'
import * as im from '../api/im'
import * as market from '../api/market'

export interface HealthCheckResult {
  name: string
  status: 'ok' | 'error' | 'timeout'
  latencyMs: number
  error?: string
  data?: unknown
}

export interface ApiHealthReport {
  timestamp: string
  results: HealthCheckResult[]
  summary: {
    total: number
    ok: number
    error: number
    avgLatencyMs: number
  }
}

async function withTimeout<T>(promise: Promise<T>, timeoutMs: number): Promise<T> {
  return Promise.race([
    promise,
    new Promise<never>((_, reject) => 
      setTimeout(() => reject(new Error('timeout')), timeoutMs)
    )
  ])
}

async function checkApi(
  name: string,
  fn: () => Promise<unknown>,
  timeoutMs = 5000
): Promise<HealthCheckResult> {
  const start = Date.now()
  try {
    const data = await withTimeout(fn(), timeoutMs)
    return {
      name,
      status: 'ok',
      latencyMs: Date.now() - start,
      data,
    }
  } catch (e) {
    const isTimeout = e instanceof Error && e.message === 'timeout'
    return {
      name,
      status: isTimeout ? 'timeout' : 'error',
      latencyMs: Date.now() - start,
      error: e instanceof Error ? e.message : String(e),
    }
  }
}

/**
 * 测试所有后端 API 连通性
 */
export async function checkAllApis(): Promise<ApiHealthReport> {
  const results = await Promise.all([
    // 域系统
    checkApi('domain.list', () => domain.list()),
    
    // 模型池
    checkApi('model_pool.status', () => modelPool.getModelPoolStatus()),
    
    // 代理池
    checkApi('proxy_pool.status', () => proxyPool.getProxyPoolStatus()),
    
    // IM
    checkApi('im.status', () => im.getImStatus()),
    checkApi('im.channels', () => im.listChannels()),
    
    // 市场
    checkApi('market.status', () => market.marketStatus()),
  ])

  const ok = results.filter(r => r.status === 'ok').length
  const error = results.filter(r => r.status !== 'ok').length
  const avgLatency = results.reduce((sum, r) => sum + r.latencyMs, 0) / results.length

  return {
    timestamp: new Date().toISOString(),
    results,
    summary: {
      total: results.length,
      ok,
      error,
      avgLatencyMs: Math.round(avgLatency),
    },
  }
}

/**
 * 测试单个域的 API
 */
export async function checkDomain(domainName: string): Promise<HealthCheckResult[]> {
  const actions = await domain.list()
    .then(domains => domains.find(d => d.name === domainName)?.actions ?? [])
    .catch(() => [])

  const results: HealthCheckResult[] = []
  for (const action of actions) {
    const result = await checkApi(
      `${domainName}.${action.name}`,
      () => domain.call(domainName, action.name, {}),
      3000
    )
    results.push(result)
  }
  return results
}
