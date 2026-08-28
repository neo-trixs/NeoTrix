import { call } from './client'
import type {
  AgentStatus,
  Checkpoint,
  GitStatus,
  HealthReport,
  McpServerInfo,
  McpToolInfo,
  NeoCodexMessageItem,
  NeoCodexSearchHit,
  NeoCodexSessionInfo,
  ProviderConfig,
  CustomProviderReq,
  ProjectView,
  UpdateCheckResult,
} from './types'

/* ════════════════════════════════════════════
   api/neocodex.ts — 会话 / 消息 / 提供商 / 项目 / Git / 更新
   前端唯一 neocodex_* 命令入口（契约见 api/types.ts）
   ════════════════════════════════════════════ */

/* ── 会话 ── */
export function listSessions(projectPath?: string | null): Promise<NeoCodexSessionInfo[]> {
  return call('neocodex_list_sessions', { project_path: projectPath ?? null })
}

/* ── 通用 KV 网关（KB kv_store；智能画板等开放 JSON 落盘，对齐吸收纪律） ── */
export function kbKvSet(namespace: string, key: string, value: string): Promise<void> {
  return call('kb_kv_set', { namespace, key, value })
}
export function kbKvGet(namespace: string, key: string): Promise<string | null> {
  return call('kb_kv_get', { namespace, key })
}
export function kbKvList(namespace: string): Promise<[string, string][]> {
  return call('kb_kv_list', { namespace })
}

/* ── 画板能力网 → NeoTrix 能力树（自进化路线融合） ── */
export interface CanvasCapabilityInput {
  kind: string
  label: string
  stage: number // 0..=5 → C0..C5 (NeoTrix Constellations)
  usage: number
  user_added: boolean
}
export interface CanvasCapabilitySyncResult {
  nodes_synced: number
  tree_cycle: string
  deprecated: number
  matured: number
  plans: { action: string; node_id: string; rationale: string }[]
  canonical: { kind: string; label: string; constellation: string; usage: number; deprecated: boolean; desired?: number }[]
}
/** 把画板能力网快照并入 NeoTrix 能力树 (KB kv_store capability_tree)，执行 Budding/Strengthen/Dark-Forest 回收。 */
export function canvasSyncCapabilities(caps: CanvasCapabilityInput[]): Promise<CanvasCapabilitySyncResult> {
  return call('canvas_sync_capabilities', { caps })
}

/** 画板覆盖层手动触发 Dark Forest 回收：把 canvas::<kind> 标记废弃 (画板 → 树 写回)。 */
export function canvasPruneCapability(kind: string): Promise<{ kind: string; pruned: boolean; constellation: string }> {
  return call('canvas_prune_capability', { kind })
}

/** 画板覆盖层把「期望成熟度」推回能力树：写入/清除 canvas::<kind> 的 canvas_desired 元数据。 */
export function canvasSetDesired(kind: string, stage: number | null): Promise<{ kind: string; pruned: boolean; constellation: string }> {
  return call('canvas_set_desired', { kind, stage })
}

export function createSession(name?: string): Promise<NeoCodexSessionInfo> {
  return call('neocodex_create_session', { name: name ?? null })
}

export function deleteSession(sessionId: string): Promise<void> {
  return call('neocodex_delete_session', { session_id: sessionId })
}

export function switchSession(sessionId: string): Promise<void> {
  return call('neocodex_switch_session', { session_id: sessionId })
}

export function renameSession(sessionId: string, name: string): Promise<NeoCodexSessionInfo> {
  return call('neocodex_rename_session', { session_id: sessionId, name })
}

export function tagSession(sessionId: string, tag: string): Promise<NeoCodexSessionInfo> {
  return call('neocodex_tag_session', { session_id: sessionId, tag })
}

export function untagSession(sessionId: string, tag: string): Promise<NeoCodexSessionInfo> {
  return call('neocodex_untag_session', { session_id: sessionId, tag })
}

export function archiveSession(sessionId: string): Promise<void> {
  return call('neocodex_archive_session', { session_id: sessionId })
}

export function restoreSession(sessionId: string): Promise<void> {
  return call('neocodex_restore_session', { session_id: sessionId })
}

export function listArchived(): Promise<NeoCodexSessionInfo[]> {
  return call('neocodex_list_archived', {})
}

export function searchSessions(query: string): Promise<NeoCodexSearchHit[]> {
  return call('neocodex_search_sessions', { query })
}

export function clearSession(sessionId: string): Promise<void> {
  return call('neocodex_clear_session', { session_id: sessionId })
}

export function exportSession(sessionId: string, format?: string): Promise<string> {
  return call('neocodex_export_session', { session_id: sessionId, format: format ?? null })
}

/* ── 消息 / 流式 ── */
export function getSessionMessages(sessionId: string): Promise<NeoCodexMessageItem[]> {
  return call('neocodex_get_session_messages', { session_id: sessionId })
}

export function sendMessageStream(params: {
  content: string
  attachments?: unknown[]
  regenerate?: boolean
  permission_mode?: string
  temperature?: number
  max_tokens?: number
}): Promise<string> {
  return call('neocodex_send_message_stream', {
    content: params.content,
    attachments: params.attachments ?? null,
    regenerate: params.regenerate ?? false,
    permission_mode: params.permission_mode ?? null,
    temperature: params.temperature ?? null,
    max_tokens: params.max_tokens ?? null,
  })
}

export function stopStream(): Promise<void> {
  return call('neocodex_stop_stream', {})
}

export function editMessage(sessionId: string, index: number, content: string): Promise<NeoCodexMessageItem[]> {
  return call('neocodex_edit_message', { session_id: sessionId, index, content })
}

export function deleteMessage(sessionId: string, index: number): Promise<NeoCodexMessageItem[]> {
  return call('neocodex_delete_message', { session_id: sessionId, index })
}

export function regenerate(sessionId: string, index: number): Promise<NeoCodexMessageItem[]> {
  return call('neocodex_regenerate', { session_id: sessionId, index })
}

export function compactSession(sessionId: string, keepMessages?: number): Promise<NeoCodexMessageItem[]> {
  return call('neocodex_compact_session', { session_id: sessionId, keep_messages: keepMessages ?? null })
}

/* ── 侧聊 ── */
export function getSideChat(sessionId: string): Promise<NeoCodexMessageItem[]> {
  return call('neocodex_get_side_chat', { session_id: sessionId })
}

export function sendSideChat(sessionId: string, content: string): Promise<NeoCodexMessageItem[]> {
  return call('neocodex_send_side_chat', { session_id: sessionId, content })
}

/* ── 提供商 / 模式 ── */
export function providerConfig(): Promise<ProviderConfig> {
  return call('neocodex_provider_config', {})
}

export function setProvider(name: string): Promise<void> {
  return call('neocodex_set_provider', { name })
}

/** 连接测试：验证指定提供商是否可达（需后端 neocodex_test_provider 命令）。 */
export function testProvider(name: string): Promise<boolean> {
  return call('neocodex_test_provider', { name })
}

/** 外部第三方模型 API 智能配置：新增一个自定义提供商（OpenAI 兼容 / 自定义网关）。 */
export function addCustomProvider(req: CustomProviderReq): Promise<void> {
  return call('neocodex_add_custom_provider', { req })
}

/**
 * 智能检测：从 base_url 拉取可用模型列表（GET {base_url}/models，Bearer 鉴权）。
 * 桌面端走 Tauri 命令；浏览器预览无 Tauri 时直连 /models 作为回退（受 CORS 限制）。
 */
export async function fetchProviderModels(baseUrl: string, apiKey: string): Promise<string[]> {
  try {
    return await call<string[]>('neocodex_fetch_provider_models', { base_url: baseUrl, api_key: apiKey })
  } catch {
    try {
      const url = baseUrl.trim().replace(/\/+$/, '') + '/models'
      const res = await fetch(url, { headers: apiKey ? { Authorization: `Bearer ${apiKey}` } : {} })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const json = (await res.json()) as { data?: { id?: string }[] }
      return (json.data ?? []).map((m) => m.id).filter((id): id is string => !!id)
    } catch {
      throw new Error('智能检测失败：桌面后端不可用或存在跨域限制，请手动填写模型名')
    }
  }
}

export function setMode(mode: string): Promise<void> {
  return call('neocodex_set_mode', { mode })
}

/* ── 项目 / 文件 ── */
export function setProject(path: string): Promise<void> {
  return call('neocodex_set_project', { path })
}

export function getProject(): Promise<string | null> {
  return call('neocodex_get_project', {})
}

export function initProject(sessionId: string): Promise<void> {
  return call('neocodex_init_project', { session_id: sessionId })
}

export function searchFiles(query: string): Promise<string[]> {
  return call('neocodex_search_files', { query })
}

export function projectTree(): Promise<ProjectView> {
  return call('neocodex_project_tree', {})
}

export function openFile(path: string): Promise<void> {
  return call('neocodex_open_file', { path })
}

export function openExternal(path: string): Promise<void> {
  return call('neocodex_open_external', { path })
}

export function fileOperation(op: string, path: string, newName?: string): Promise<void> {
  return call('neocodex_file_operation', { op, path, new_name: newName ?? null })
}

/* ── Git ── */
export function gitStatus(): Promise<GitStatus | null> {
  return call('neocodex_git_status', {})
}

export function getDiff(): Promise<GitDiffResponse> {
  return call('neocodex_get_diff', {})
}

export interface GitDiffFile {
  path: string
  hunks: { lines: { t: 'add' | 'del' | 'ctx'; o: number | null; n: number | null; s: string }[] }[]
}

export interface GitDiffResponse {
  files: GitDiffFile[]
}

export function applyDiff(path: string, action: string): Promise<void> {
  return call('neocodex_apply_diff', { path, action })
}

/** 提交已暂存内容（对应面板 accept = git add 后的 commit）。 */
export function gitCommit(message: string): Promise<void> {
  return call('neocodex_git_commit', { message })
}

/** 推送当前分支到远程，返回远程输出摘要（无上游时错误内含提示）。 */
export function gitPush(): Promise<string> {
  return call('neocodex_git_push', {})
}

/** 列出本地分支（short ref names，如 main）。 */
export function listBranches(): Promise<string[]> {
  return call('neocodex_git_branch', {})
}

/** 返回当前已暂存文件列表（git diff --cached --name-only）。 */
export function gitStagedFiles(): Promise<string[]> {
  return call('neocodex_git_staged_files', {})
}

/** 切换分支（git checkout），返回切换后的分支名。 */
export function gitCheckout(branch: string): Promise<string> {
  return call('neocodex_git_checkout', { branch })
}

/* ── 检查点 ── */
export function checkpointList(sessionId: string): Promise<Checkpoint[]> {
  return call('neocodex_checkpoint_list', { session_id: sessionId })
}

export function checkpointRestore(sessionId: string, checkpointId: string): Promise<NeoCodexMessageItem[]> {
  return call('neocodex_checkpoint_restore', { session_id: sessionId, checkpoint_id: checkpointId })
}

/* ── 健康 / 状态 / 版本 ── */
export function healthReport(): Promise<HealthReport> {
  return call('neocodex_health_report', {})
}

export function agentStatus(): Promise<AgentStatus> {
  return call('neocodex_agent_status', {})
}

export function appVersion(): Promise<string> {
  return call('neocodex_app_version', {})
}

/* ── 更新（热更新；进度经 Tauri event 推送，见 api/system.ts listenUpdateEvents） ── */
export function checkUpdate(): Promise<UpdateCheckResult> {
  return call('neocodex_check_update', {})
}

export function downloadUpdate(): Promise<void> {
  return call('neocodex_download_update', {})
}

export function restartApp(): Promise<void> {
  return call('neocodex_restart_app', {})
}

/* ── MCP ── */
export function mcpList(): Promise<McpServerInfo[]> {
  return call('neocodex_mcp_list', {})
}

export function mcpTools(): Promise<McpToolInfo[]> {
  return call('neocodex_mcp_tools', {})
}

/** 注册本地 stdio MCP 服务器（name/command/args），返回注册后的服务器列表。 */
export function mcpRegister(name: string, command: string, args?: string[]): Promise<McpServerInfo[]> {
  return call('neocodex_mcp_register', { name, command, args: args ?? null })
}

/* ── 反馈 ── */
export function feedback(sessionId: string, text: string): Promise<void> {
  return call('neocodex_feedback', { session_id: sessionId, text })
}
