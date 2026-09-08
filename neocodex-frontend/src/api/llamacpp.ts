/**
 * Llamacpp API — 本地模型管理
 * 
 * 通过 domain_call('llamacpp', action, args) 调用后端 LlamacppPlugin。
 */
import { domainCall } from './domain-client'

export interface LlamacppHealth {
  running: boolean
  port: number
  pid: number | null
  uptime_secs: number
  model_loaded: string | null
  binary_found: boolean
}

export interface LocalModel {
  name: string
  path: string
  size_bytes: number
  quantization: string
}

/**
 * 获取健康状态
 */
export async function getHealth(): Promise<LlamacppHealth> {
  return domainCall<LlamacppHealth>('llamacpp', 'health')
}

/**
 * 获取模型列表
 */
export async function getModels(): Promise<LocalModel[]> {
  return domainCall<LocalModel[]>('llamacpp', 'models')
}

/**
 * 启动服务
 */
export async function startServer(modelPath?: string): Promise<void> {
  await domainCall('llamacpp', 'start', { model: modelPath })
}

/**
 * 停止服务
 */
export async function stopServer(): Promise<void> {
  await domainCall('llamacpp', 'stop')
}

/**
 * 切换模型
 */
export async function swapModel(modelPath: string): Promise<void> {
  await domainCall('llamacpp', 'swap', { model: modelPath })
}

/**
 * 发送消息
 */
export async function sendChat(
  messages: Array<{ role: string; content: string }>,
  options?: { temperature?: number; max_tokens?: number }
): Promise<string> {
  const result = await domainCall<{ choices: Array<{ message: { content: string } }> }>(
    'llamacpp',
    'send',
    {
      messages,
      temperature: options?.temperature ?? 0.7,
      max_tokens: options?.max_tokens ?? 2048,
    }
  )
  return result.choices?.[0]?.message?.content || 'No response'
}
