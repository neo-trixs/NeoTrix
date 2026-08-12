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
