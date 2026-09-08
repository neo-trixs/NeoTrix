/**
 * Domain Client — 统一 API 入口
 * 
 * 所有前端调用通过 domain_call(domain, action, args) 路由到后端 Domain Plugin。
 * 替代散落的 invoke() 调用，确保单一事实源。
 */
import { invoke } from '@tauri-apps/api/core'

export interface DomainResponse<T = any> {
  ok: boolean
  data: T
  error?: {
    code: string
    message: string
    recoverable: boolean
  }
}

/**
 * 统一域调用
 * @param domain - 域名称 (session, chat, llamacpp, kb, memory, etc.)
 * @param action - 操作名称 (list, create, send, health, etc.)
 * @param args - 参数对象
 */
export async function domainCall<T = any>(
  domain: string,
  action: string,
  args: Record<string, any> = {}
): Promise<T> {
  const result = await invoke<DomainResponse<T>>('domain_call', {
    domain,
    action,
    args,
  })
  
  if (!result.ok) {
    const error = result.error || { code: 'UNKNOWN', message: 'Unknown error', recoverable: false }
    throw new Error(`[${error.code}] ${error.message}`)
  }
  
  return result.data
}

/**
 * 列出所有已注册域
 */
export async function domainList() {
  return domainCall<Array<{ name: string; description: string; actions: any[] }>>(
    'session', // dummy domain, will be ignored
    'list',
    {}
  ).catch(() => {
    // domain_list is a separate command
    return invoke<Array<{ name: string; description: string; actions: any[] }>>('domain_list')
  })
}

/**
 * 检查域是否已注册
 */
export async function domainHas(domain: string): Promise<boolean> {
  return invoke<boolean>('domain_has', { domain })
}
