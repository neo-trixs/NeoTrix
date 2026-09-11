/**
 * Domain API — 前端统一域调用客户端
 *
 * 基于 DeepSeek Harness 架构：一切皆插件，能力缝可替换。
 * 前端通过 domain_call(domain, action, args) 与所有 12 个功能域交互。
 */

import { invoke } from '@tauri-apps/api/core'

// ========== Types ==========

export interface DomainCall {
  domain: string
  action: string
  args?: Record<string, unknown>
}

export interface DomainResponse<T = unknown> {
  ok: boolean
  data: T
  error?: {
    code: string
    message: string
    recoverable: boolean
  }
}

export interface DomainInfo {
  name: string
  description: string
  actions: ActionSpec[]
}

export interface ActionSpec {
  name: string
  description: string
  params: ParamSpec[]
  returns: string
}

export interface ParamSpec {
  name: string
  type: string
  description: string
  optional: boolean
}

// ========== Core API ==========

/**
 * 域调用 — 前端唯一入口
 *
 * @example
 * const sessions = await domain.call<Session[]>('session', 'list')
 * const result = await domain.call<KbSearchResult>('kb', 'search', { query: 'rust' })
 */
export async function call<T = unknown>(
  domain: string,
  action: string,
  args: Record<string, unknown> = {}
): Promise<T> {
  const response = await invoke<DomainResponse<T>>('domain_call', {
    domain,
    action,
    args,
  })

  if (!response.ok) {
    const error = response.error || { code: 'UNKNOWN', message: 'Unknown error', recoverable: true }
    throw new DomainError(error.code, error.message, error.recoverable)
  }

  return response.data
}

/**
 * 列出所有已注册域
 */
export async function list(): Promise<DomainInfo[]> {
  return await invoke<DomainInfo[]>('domain_list')
}

/**
 * 检查域是否存在
 */
export async function has(domain: string): Promise<boolean> {
  return await invoke<boolean>('domain_has', { domain })
}

/**
 * 获取域 action 数量
 */
export async function actionCount(domain: string): Promise<number> {
  return await invoke<number>('domain_action_count', { domain })
}

// ========== Domain Error ==========

export class DomainError extends Error {
  constructor(
    public code: string,
    message: string,
    public recoverable: boolean = true,
  ) {
    super(`[${code}] ${message}`)
    this.name = 'DomainError'
  }
}

// ========== Typed Domain Helpers ==========

/**
 * 会话域操作
 */
export const session = {
  list: () => call<Session[]>('session', 'list'),
  create: (name?: string) => call<Session>('session', 'create', { name }),
  delete: (id: string) => call<void>('session', 'delete', { id }),
  switch: (id: string) => call<void>('session', 'switch', { id }),
  archive: (id: string) => call<void>('session', 'archive', { id }),
  restore: (id: string) => call<void>('session', 'restore', { id }),
  search: (query: string) => call<Session[]>('session', 'search', { query }),
  reorder: (ids: string[]) => call<void>('session', 'reorder', { ids }),
  setProject: (id: string, project: string) => call<void>('session', 'set_project', { id, project }),
  fork: (fromId: string, upTo?: number) => call<{ id: string }>('session', 'fork', { from_id: fromId, up_to: upTo }),
  rename: (id: string, name: string) => call<void>('session', 'rename', { id, name }),
  listArchived: () => call<Session[]>('session', 'list_archived'),
  clear: (id: string) => call<void>('session', 'clear', { id }),
  tag: (id: string, tag: string) => call<string[]>('session', 'tag', { id, tag }),
  untag: (id: string, tag: string) => call<string[]>('session', 'untag', { id, tag }),
}

export interface Session {
  id: string
  name: string
  created_at: string
  message_count: number
  project?: string
  sort_order?: number
  tags?: string[]
}

/**
 * 对话域操作
 */
export const chat = {
  send: (content: string, sessionId?: string) =>
    call<ChatResult>('chat', 'send', { content, session_id: sessionId }),
  stop: () => call<void>('chat', 'stop'),
  history: (sessionId: string) => call<ChatMessage[]>('chat', 'history', { session_id: sessionId }),
  compact: (sessionId: string) => call<void>('chat', 'compact', { session_id: sessionId }),
  export: (sessionId: string, format?: string) =>
    call<string>('chat', 'export', { session_id: sessionId, format }),
  clear: (sessionId: string) => call<void>('chat', 'clear', { session_id: sessionId }),
  regenerate: (sessionId: string, messageIndex: number) =>
    call<void>('chat', 'regenerate', { session_id: sessionId, message_index: messageIndex }),
  editMessage: (sessionId: string, index: number, content: string) =>
    call<void>('chat', 'edit_message', { session_id: sessionId, index, content }),
  deleteMessage: (sessionId: string, index: number) =>
    call<void>('chat', 'delete_message', { session_id: sessionId, index }),
  sideChat: {
    get: (sessionId: string) => call<SideChatMessage[]>('chat', 'side_chat_get', { session_id: sessionId }),
    send: (sessionId: string, content: string) =>
      call<ChatResult>('chat', 'side_chat_send', { session_id: sessionId, content }),
  },
}

export interface ChatResult {
  id: string
  content: string
  role: 'user' | 'assistant'
}

export interface ChatMessage {
  id: string
  content: string
  role: 'user' | 'assistant' | 'system'
  timestamp: string
}

export interface SideChatMessage {
  id: string
  content: string
  role: 'user' | 'assistant'
  timestamp: string
}

/**
 * Agent 域操作
 */
export const agent = {
  status: () => call<AgentStatus>('agent', 'status'),
  start: (task: string) => call<void>('agent', 'start', { task }),
  stop: () => call<void>('agent', 'stop'),
  setProvider: (name: string) => call<void>('agent', 'set_provider', { name }),
  testProvider: (name: string) => call<boolean>('agent', 'test_provider', { name }),
  fetchModels: (baseUrl: string, apiKey: string) =>
    call<string[]>('agent', 'fetch_models', { base_url: baseUrl, api_key: apiKey }),
  providerConfig: () => call<ProviderConfig[]>('agent', 'provider_config'),
  addCustomProvider: (config: Record<string, unknown>) =>
    call<void>('agent', 'add_custom_provider', config),
  appVersion: () => call<string>('agent', 'app_version'),
  health: () => call<AgentHealth>('agent', 'health'),
}

export interface AgentStatus {
  running: boolean
  current_task?: string
}

export interface ProviderConfig {
  id: string
  name: string
  model: string
  api_key?: string
  base_url?: string
  learning_rate?: number
  enabled?: boolean
  catalog?: Record<string, unknown>
}

export interface AgentHealth {
  turn_count: number
  tokens_used: number
  cost: number
  provider: string
  model: string
}

/**
 * 知识库域操作
 */
export const kb = {
  search: (query: string, limit?: number) =>
    call<KbResult[]>('kb', 'search', { query, limit }),
  get: (id: string) => call<KbNode | null>('kb', 'get', { id }),
  graph: () => call<KbGraph>('kb', 'graph'),
  stats: () => call<KbStats>('kb', 'stats'),
  kvSet: (namespace: string, key: string, value: string) =>
    call<void>('kb', 'kv_set', { namespace, key, value }),
  kvGet: (namespace: string, key: string) =>
    call<string | null>('kb', 'kv_get', { namespace, key }),
  kvList: (namespace: string) =>
    call<[string, string][]>('kb', 'kv_list', { namespace }),
  docIngest: (id: string, label: string, kind: string, data: string, library?: string) =>
    call<{ id: string }>('kb', 'doc_ingest', { id, label, kind, data, library }),
  docList: (limit?: number) =>
    call<KbDoc[]>('kb', 'doc_list', { limit }),
  docDelete: (id: string) => call<void>('kb', 'doc_delete', { id }),
  docReindex: () => call<{ reindexed: number }>('kb', 'doc_reindex'),
  libraryCreate: (name: string, description: string) =>
    call<{ id: string }>('kb', 'library_create', { name, description }),
  libraryList: () => call<KbLibrary[]>('kb', 'library_list'),
  libraryRename: (id: string, name: string) =>
    call<void>('kb', 'library_rename', { id, name }),
  libraryDelete: (id: string) => call<void>('kb', 'library_delete', { id }),
}

export interface KbResult { id: string; label: string; score: number }
export interface KbNode { id: string; label: string; kind: string }
export interface KbGraph { nodes: KbNode[]; edges: unknown[] }
export interface KbStats { node_count: number; edge_count: number }
export interface KbDoc { id: string; label: string; kind: string; created_at: number }
export interface KbLibrary { id: string; name: string; description: string; doc_count: number; chunk_count: number; created_at: number; updated_at: number }

/**
 * 文件域操作
 */
export const file = {
  read: (path: string) => call<string>('file', 'read', { path }),
  write: (path: string, content: string) => call<void>('file', 'write', { path, content }),
  tree: (path?: string) => call<FileNode[]>('file', 'tree', { path }),
  diff: (path: string) => call<string>('file', 'diff', { path }),
  search: (query: string) => call<string[]>('file', 'search', { query }),
}

export interface FileNode {
  name: string
  path: string
  kind: 'file' | 'directory'
  children?: FileNode[]
}

/**
 * 插件域操作
 */
export const plugin = {
  list: () => call<PluginInfo[]>('plugin', 'list'),
  install: (path: string) => call<void>('plugin', 'install', { path }),
  uninstall: (id: string) => call<void>('plugin', 'uninstall', { id }),
  enable: (id: string) => call<void>('plugin', 'enable', { id }),
  disable: (id: string) => call<void>('plugin', 'disable', { id }),
}

export interface PluginInfo {
  id: string
  name: string
  enabled: boolean
}

/**
 * 工作流域操作
 */
export const workflow = {
  list: () => call<WorkflowInfo[]>('workflow', 'list'),
  create: (name: string, steps: unknown[]) => call<string>('workflow', 'create', { name, steps }),
  delete: (id: string) => call<void>('workflow', 'delete', { id }),
  run: (id: string) => call<string>('workflow', 'run', { id }),
  status: (id: string) => call<WorkflowStatus>('workflow', 'status', { id }),
}

export interface WorkflowInfo { id: string; name: string }
export interface WorkflowStatus { id: string; running: boolean; progress: number }

/**
 * 工具域操作
 */
export const tool = {
  mcpList: () => call<McpServer[]>('tool', 'mcp_list'),
  mcpRegister: (name: string, command: string, args?: string[]) =>
    call<void>('tool', 'mcp_register', { name, command, args }),
  harnessExecute: (instruction: string) =>
    call<unknown>('tool', 'harness_execute', { instruction }),
  computerCapture: () => call<string>('tool', 'computer_capture'),
  voiceSynthesize: (text: string, voice?: string) =>
    call<string>('tool', 'voice_synthesize', { text, voice }),
}

export interface McpServer { name: string; status: string }

/**
 * 系统域操作
 */
export const system = {
  ptySpawn: (sessionId: string, cols: number, rows: number) =>
    call<void>('system', 'pty_spawn', { session_id: sessionId, cols, rows }),
  ptyWrite: (sessionId: string, data: string) =>
    call<void>('system', 'pty_write', { session_id: sessionId, data }),
  ptyResize: (sessionId: string, cols: number, rows: number) =>
    call<void>('system', 'pty_resize', { session_id: sessionId, cols, rows }),
  ptyClose: (sessionId: string) =>
    call<void>('system', 'pty_close', { session_id: sessionId }),
  updateCheck: () => call<UpdateInfo>('system', 'update_check'),
  updateDownload: () => call<void>('system', 'update_download'),
  restartApp: () => call<void>('system', 'restart_app'),
  windowMinimize: () => call<void>('system', 'window_minimize'),
  windowMaximize: () => call<void>('system', 'window_maximize'),
  windowClose: () => call<void>('system', 'window_close'),
}

export interface UpdateInfo { available: boolean; version?: string }

/**
 * 安全域操作
 */
export const security = {
  scan: (path: string) => call<ScanResult>('security', 'scan', { path }),
  permissionRequest: (action: string, target: string) =>
    call<PermissionResult>('security', 'permission_request', { action, target }),
  permissionRespond: (id: string, approved: boolean) =>
    call<void>('security', 'permission_respond', { id, approved }),
}

export interface ScanResult { id: string; findings: number }
export interface PermissionResult { id: string; status: string }

/**
 * 记忆域操作
 */
export const memory = {
  list: (kind?: string) => call<MemoryEntry[]>('memory', 'list', { kind }),
  search: (query: string) => call<MemoryEntry[]>('memory', 'search', { query }),
  clear: (kind?: string) => call<number>('memory', 'clear', { kind }),
  stats: () => call<MemoryStats>('memory', 'stats'),
  timeline: (days?: number) => call<TimelineEntry[]>('memory', 'timeline', { days }),
  export: (format?: string) => call<{ memories: MemoryEntry[]; exported_at: string; count: number }>('memory', 'export', { format }),
  import: (content: string) => call<{ imported: number }>('memory', 'import', { content }),
}

export interface MemoryEntry { id: string; kind: string; content: string }
export interface MemoryStats { total: number; by_kind: Record<string, number> }
export interface TimelineEntry { date: string; count: number }

/**
 * 扩展域操作
 */
export const ext = {
  remoteConnect: (deviceName: string, code: string) =>
    call<string>('ext', 'remote_connect', { device_name: deviceName, code }),
  channelSend: (channelId: string, content: string) =>
    call<void>('ext', 'channel_send', { channel_id: channelId, content }),
  coworkStart: (path: string, description?: string) =>
    call<string>('ext', 'cowork_start', { workspace_path: path, description }),
}

// ========== Git 域 ==========

/**
 * Git 域操作
 */
export const git = {
  status: () => call<GitStatus>('git', 'status'),
  diff: (path?: string) => call<GitDiff>('git', 'diff', { path }),
  stagedFiles: () => call<string[]>('git', 'staged_files'),
  branches: () => call<string[]>('git', 'branches'),
  checkout: (branch: string) => call<void>('git', 'checkout', { branch }),
  commit: (message: string) => call<void>('git', 'commit', { message }),
  push: () => call<void>('git', 'push'),
  applyDiff: (path: string, apply: boolean) => call<void>('git', 'apply_diff', { path, apply }),
}

export interface GitStatus {
  branch: string
  modified: string[]
  staged: string[]
  untracked: string[]
}

export interface GitDiff {
  hunks: GitHunk[]
}

export interface GitHunk {
  file: string
  additions: number
  deletions: number
  content: string
}

// ========== CLI 域 ==========

/**
 * CLI 域操作
 */
export const cli = {
  exec: (command: string) => call<CliResult>('cli', 'exec', { command }),
  list: () => call<CliCommand[]>('cli', 'list'),
}

export interface CliResult {
  message: string
  success: boolean
}

export interface CliCommand {
  name: string
  description: string
  aliases: string[]
}

// ========== Domain Proxy (动态调用) ==========

// ========== Llamacpp 域 ==========

export interface LlamacppModel {
  name: string
  path: string
  size: number
}

export const llamacpp = {
  health: () => call<{ status: string; pid?: number; uptime_secs?: number }>('llamacpp', 'health'),
  models: () => call<LlamacppModel[]>('llamacpp', 'models'),
  start: (model?: string) => call<{ port: number }>('llamacpp', 'start', { model }),
  stop: () => call<void>('llamacpp', 'stop'),
  swap: (model: string) => call<void>('llamacpp', 'swap', { model }),
  send: (prompt: string, opts?: { temperature?: number; max_tokens?: number }) =>
    call<string>('llamacpp', 'send', { prompt, ...opts }),
}

// ========== Domain Proxy (动态调用) ==========

/**
 * 动态域代理 — 用于未定义 typed helper 的域
 *
 * @example
 * const result = await domain.proxy('workflow', 'list')
 * const result2 = await domain.proxy('ext', 'remote_connect', { device_name: 'phone', code: '123' })
 */
export const proxy = call
