/**
 * Model Pool API — 模型池管理
 *
 * 迁移到新架构：状态查询走 chatSend。
 */
import { chatSend, extractResult } from './chat'

/** 模型池条目 */
export interface ModelPoolEntry {
  label: string
  provider: string
  api_key_masked: string
  model: string
  tags: string[]
  base_url: string | null
  created_ts: number
}

/** 模型池状态 */
export interface ModelPoolStatus {
  total: number
  active: number
  providers: ModelPoolEntry[]
  config_path: string
}

/**
 * 获取模型池状态
 */
export async function getModelPoolStatus(): Promise<ModelPoolStatus> {
  const response = await chatSend("查看模型列表")
  return extractResult<ModelPoolStatus>(response, { total: 0, active: 0, providers: [], config_path: '' })
}

/**
 * 添加模型提供者
 */
export async function addModelProvider(params: {
  label: string
  provider: string
  api_key: string
  model: string
  tags?: string[]
  base_url?: string
}): Promise<ModelPoolEntry> {
  const response = await chatSend(`添加模型提供者 ${params.label} ${params.provider} ${params.model}`)
  return extractResult<ModelPoolEntry>(response, {
    label: params.label,
    provider: params.provider,
    api_key_masked: '****',
    model: params.model,
    tags: params.tags || [],
    base_url: params.base_url || null,
    created_ts: Date.now(),
  })
}

/**
 * 删除模型提供者
 */
export async function removeModelProvider(label: string): Promise<boolean> {
  const response = await chatSend(`删除模型提供者 ${label}`)
  return response.actions.length > 0
}

/**
 * 更新模型提供者的 API Key
 */
export async function updateModelProviderKey(
  label: string,
  new_api_key: string
): Promise<boolean> {
  const response = await chatSend(`更新模型提供者 ${label} 的API密钥`)
  return response.actions.length > 0
}

/**
 * 检查模型提供者 API 连通性
 */
export async function checkModelProvider(label: string): Promise<string> {
  const response = await chatSend(`检查模型提供者 ${label} 连通性`)
  return response.message
}
