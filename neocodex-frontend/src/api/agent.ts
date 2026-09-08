/**
 * Agent API — Agent 状态与配置
 * 
 * 通过 domain_call('agent', action, args) 调用后端 AgentPlugin。
 */
import { domainCall } from './domain-client'

export interface AgentStatus {
  running: boolean
  provider: string
  model: string
  uptime_secs: number
  tasks_completed: number
}

export interface ProviderConfig {
  name: string
  api_key?: string
  base_url?: string
  models: string[]
}

/**
 * 获取 Agent 状态
 */
export async function getStatus(): Promise<AgentStatus> {
  return domainCall<AgentStatus>('agent', 'status')
}

/**
 * 启动 Agent
 */
export async function start(): Promise<void> {
  await domainCall('agent', 'start')
}

/**
 * 停止 Agent
 */
export async function stop(): Promise<void> {
  await domainCall('agent', 'stop')
}

/**
 * 设置 Provider
 */
export async function setProvider(name: string): Promise<void> {
  await domainCall('agent', 'set_provider', { name })
}

/**
 * 测试 Provider
 */
export async function testProvider(config: ProviderConfig): Promise<boolean> {
  return domainCall<boolean>('agent', 'test_provider', config)
}

/**
 * 获取 Provider 配置
 */
export async function getConfig(): Promise<ProviderConfig> {
  return domainCall<ProviderConfig>('agent', 'config')
}

/**
 * 获取健康报告
 */
export async function getHealth(): Promise<any> {
  return domainCall('agent', 'health')
}

/**
 * 设置项目
 */
export async function setProject(path: string): Promise<void> {
  await domainCall('agent', 'set_project', { path })
}

/**
 * 获取当前项目
 */
export async function getProject(): Promise<string> {
  return domainCall<string>('agent', 'get_project')
}
