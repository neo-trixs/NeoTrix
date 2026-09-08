/**
 * API Adapter — 路由旧 neocodex_* 调用到 domain_call
 * 
 * 这个适配器让现有的 neocodex.* API 调用透明地路由到后端 Domain Plugin 系统，
 * 而不需要修改所有调用点。
 */
import { invoke } from '@tauri-apps/api/core'

// 域名称映射：旧命令 → domain name
const DOMAIN_MAP: Record<string, string> = {
  // Session commands
  neocodex_list_sessions: 'session',
  neocodex_create_session: 'session',
  neocodex_delete_session: 'session',
  neocodex_switch_session: 'session',
  neocodex_rename_session: 'session',
  neocodex_tag_session: 'session',
  neocodex_untag_session: 'session',
  neocodex_archive_session: 'session',
  neocodex_restore_session: 'session',
  neocodex_list_archived: 'session',
  neocodex_search_sessions: 'session',
  neocodex_clear_session: 'session',
  neocodex_export_session: 'session',
  // Chat commands
  neocodex_get_session_messages: 'chat',
  neocodex_send_message_stream: 'chat',
  neocodex_stop_stream: 'chat',
  neocodex_edit_message: 'chat',
  neocodex_delete_message: 'chat',
  neocodex_regenerate: 'chat',
  neocodex_compact_session: 'chat',
  neocodex_get_side_chat: 'chat',
  neocodex_send_side_chat: 'chat',
  // Agent commands
  neocodex_provider_config: 'agent',
  neocodex_set_provider: 'agent',
  neocodex_test_provider: 'agent',
  neocodex_provider_test: 'agent',
  neocodex_add_custom_provider: 'agent',
  neocodex_fetch_provider_models: 'agent',
  neocodex_set_mode: 'agent',
  neocodex_set_project: 'agent',
  neocodex_get_project: 'agent',
  neocodex_init_project: 'agent',
  neocodex_search_files: 'agent',
  neocodex_project_tree: 'agent',
  neocodex_open_file: 'system',
  neocodex_open_external: 'system',
  neocodex_file_operation: 'system',
  neocodex_git_status: 'agent',
  neocodex_get_diff: 'agent',
  neocodex_apply_diff: 'agent',
  neocodex_git_commit: 'agent',
  neocodex_git_push: 'agent',
  neocodex_git_branch: 'agent',
  neocodex_git_staged_files: 'agent',
  neocodex_git_checkout: 'agent',
  neocodex_checkpoint_list: 'agent',
  neocodex_checkpoint_restore: 'agent',
  neocodex_health_report: 'agent',
  neocodex_agent_status: 'agent',
  neocodex_app_version: 'system',
  neocodex_check_update: 'system',
  neocodex_download_update: 'system',
  neocodex_restart_app: 'system',
  neocodex_mcp_list: 'tool',
  neocodex_mcp_tools: 'tool',
  neocodex_mcp_register: 'tool',
  neocodex_feedback: 'agent',
  kb_kv_set: 'kb',
  kb_kv_get: 'kb',
  kb_kv_list: 'kb',
  canvas_sync_capabilities: 'agent',
  canvas_prune_capability: 'agent',
  canvas_set_desired: 'agent',
  canvas_apply_evolution_route: 'agent',
  provider_status: 'agent',
  pool_sufficiency: 'agent',
  discover_models: 'agent',
  probe_all_providers: 'agent',
  // Memory commands
  memory_stats: 'memory',
  memory_list: 'memory',
  memory_search: 'memory',
  memory_clear: 'memory',
  memory_timeline: 'memory',
  memory_export: 'memory',
  memory_import: 'memory',
  // KB commands
  kb_doc_ingest: 'kb',
  kb_doc_list: 'kb',
  kb_doc_delete: 'kb',
  kb_doc_reindex: 'kb',
  // System commands
  save_api_key: 'system',
  has_api_key: 'system',
  delete_api_key: 'system',
}

// action 名称映射：去掉前缀，转换为 domain action
function mapAction(cmd: string): { domain: string; action: string } {
  const domain = DOMAIN_MAP[cmd]
  if (domain) {
    // 去掉各种前缀
    const action = cmd
      .replace(/^neocodex_/, '')
      .replace(/^kb_/, '')
      .replace(/^canvas_/, '')
      .replace(/^provider_/, '')
      .replace(/^pool_/, '')
      .replace(/^discover_/, '')
      .replace(/^probe_/, '')
      .replace(/^memory_/, '')
    return { domain, action }
  }
  // 默认返回原始命令
  return { domain: 'system', action: cmd }
}

/**
 * 增强的 invoke 函数，自动路由命令到 domain_call
 */
export async function enhancedInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  // 直接注册的命令（model_pool_*, proxy_pool_*, im_*）直接调用
  if (cmd.startsWith('model_pool_') || cmd.startsWith('proxy_pool_') || cmd.startsWith('im_')) {
    return invoke<T>(cmd, args)
  }

  // 如果是 neocodex_* 或相关命令，路由到 domain_call
  if (cmd.startsWith('neocodex_') || cmd.startsWith('kb_') || cmd.startsWith('canvas_') || 
      cmd.startsWith('provider_') || cmd.startsWith('pool_') || cmd.startsWith('discover_') || 
      cmd.startsWith('probe_') || cmd.startsWith('memory_') || cmd === 'save_api_key' || 
      cmd === 'has_api_key' || cmd === 'delete_api_key') {
    const { domain, action } = mapAction(cmd)
    try {
      const result = await invoke<{ ok: boolean; data: T; error?: any }>('domain_call', {
        domain,
        action,
        args: args || {},
      })
      if (!result.ok) {
        throw new Error(result.error?.message || 'Domain call failed')
      }
      return result.data
    } catch (e) {
      // 如果 domain_call 失败，回退到直接调用（可能后端有直接注册的命令）
      console.warn(`Domain call failed for ${cmd}, falling back to direct invoke:`, e)
      return invoke<T>(cmd, args)
    }
  }
  // 其他命令直接调用
  return invoke<T>(cmd, args)
}

/**
 * 包装 call 函数，使用 enhancedInvoke
 */
export async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return enhancedInvoke<T>(cmd, args)
}
