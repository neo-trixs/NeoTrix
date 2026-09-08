/**
 * Model Pool API — 模型池管理
 * 
 * 通过 Tauri invoke 调用后端 model_pool 命令。
 */
import { invoke } from '@tauri-apps/api/core'

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
  return invoke<ModelPoolStatus>('model_pool_status')
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
  return invoke<ModelPoolEntry>('model_pool_add', {
    label: params.label,
    provider: params.provider,
    api_key: params.api_key,
    model: params.model,
    tags: params.tags || [],
    base_url: params.base_url || null,
  })
}

/**
 * 删除模型提供者
 */
export async function removeModelProvider(label: string): Promise<boolean> {
  return invoke<boolean>('model_pool_remove', { label })
}

/**
 * 更新模型提供者的 API Key
 */
export async function updateModelProviderKey(
  label: string,
  new_api_key: string
): Promise<boolean> {
  return invoke<boolean>('model_pool_update_key', { label, new_api_key })
}

/**
 * 检查模型提供者 API 连通性
 */
export async function checkModelProvider(label: string): Promise<string> {
  return invoke<string>('model_pool_check', { label })
}
