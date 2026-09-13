import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import * as domain from './domain'
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
   会话和消息已迁移到 session.ts / chat.ts（domain plugin）
   ════════════════════════════════════════════ */

/* ── 会话（domain plugin） ── */
export function listSessions(projectPath?: string | null): Promise<NeoCodexSessionInfo[]> {
  return domain.session.list() as Promise<NeoCodexSessionInfo[]>
}

export function createSession(name?: string): Promise<NeoCodexSessionInfo> {
  return domain.session.create(name) as Promise<NeoCodexSessionInfo>
}

export function deleteSession(sessionId: string): Promise<void> {
  return domain.session.delete(sessionId)
}

export function switchSession(sessionId: string): Promise<void> {
  return domain.session.switch(sessionId)
}

export function renameSession(sessionId: string, name: string): Promise<NeoCodexSessionInfo> {
  return domain.session.rename(sessionId, name) as Promise<NeoCodexSessionInfo>
}

export function tagSession(sessionId: string, tag: string): Promise<NeoCodexSessionInfo> {
  return domain.session.tag(sessionId, tag) as Promise<NeoCodexSessionInfo>
}

export function untagSession(sessionId: string, tag: string): Promise<NeoCodexSessionInfo> {
  return domain.session.untag(sessionId, tag) as Promise<NeoCodexSessionInfo>
}

export function archiveSession(sessionId: string): Promise<void> {
  return domain.session.archive(sessionId)
}

export function restoreSession(sessionId: string): Promise<void> {
  return domain.session.restore(sessionId)
}

export function listArchived(): Promise<NeoCodexSessionInfo[]> {
  return domain.session.listArchived() as Promise<NeoCodexSessionInfo[]>
}

export function searchSessions(query: string): Promise<NeoCodexSearchHit[]> {
  return domain.session.search(query) as Promise<NeoCodexSearchHit[]>
}

export function clearSession(sessionId: string): Promise<void> {
  return domain.session.delete(sessionId)
}

export function exportSession(sessionId: string, format?: string): Promise<string> {
  return domain.chat.export(sessionId, format) as Promise<string>
}

/* ── 消息 / 流式（domain plugin） ── */
export function getSessionMessages(sessionId: string): Promise<NeoCodexMessageItem[]> {
  return domain.chat.history(sessionId) as Promise<NeoCodexMessageItem[]>
}

export function sendMessageStream(params: {
  content: string
  attachments?: unknown[]
  regenerate?: boolean
  permission_mode?: string
  temperature?: number
  max_tokens?: number
}): Promise<string> {
  return domain.chat.send(params.content)
}

export function stopStream(): Promise<void> {
  return domain.chat.stop()
}

/**
 * 订阅流式响应事件
 * 返回取消订阅函数
 */
export async function subscribeStream(callbacks: {
  onToken?: (delta: string) => void
  onDone?: () => void
  onError?: (payload: { message?: string }) => void
}): Promise<UnlistenFn> {
  const unlistenFns: UnlistenFn[] = []

  if (callbacks.onToken) {
    unlistenFns.push(
      await listen<string>('neocodex-stream-token', (event) => {
        callbacks.onToken!(event.payload)
      })
    )
  }

  if (callbacks.onDone) {
    unlistenFns.push(
      await listen('neocodex-stream-done', () => {
        callbacks.onDone!()
      })
    )
  }

  if (callbacks.onError) {
    unlistenFns.push(
      await listen<{ message?: string }>('neocodex-stream-error', (event) => {
        callbacks.onError!(event.payload)
      })
    )
  }

  return Promise.resolve(() => {
    unlistenFns.forEach((unlisten) => unlisten())
  })
}

export function editMessage(sessionId: string, index: number, content: string): Promise<NeoCodexMessageItem[]> {
  return domain.chat.editMessage(sessionId, index, content) as Promise<NeoCodexMessageItem[]>
}

export function deleteMessage(sessionId: string, index: number): Promise<NeoCodexMessageItem[]> {
  return domain.chat.deleteMessage(sessionId, index) as Promise<NeoCodexMessageItem[]>
}

export function regenerate(sessionId: string, index: number): Promise<NeoCodexMessageItem[]> {
  return domain.chat.regenerate(sessionId, index) as Promise<NeoCodexMessageItem[]>
}

export function compactSession(sessionId: string, keepMessages?: number): Promise<NeoCodexMessageItem[]> {
  return domain.chat.compact(sessionId) as Promise<NeoCodexMessageItem[]>
}

/* ── 侧聊（domain plugin） ── */
export function getSideChat(sessionId: string): Promise<NeoCodexMessageItem[]> {
  return domain.chat.sideChat.get(sessionId) as Promise<NeoCodexMessageItem[]>
}

export function sendSideChat(sessionId: string, content: string): Promise<NeoCodexMessageItem[]> {
  return domain.chat.sideChat.send(sessionId, content) as Promise<NeoCodexMessageItem[]>
}

/* ── 提供商 / 模式 ── */
export function providerConfig(): Promise<ProviderConfig> {
  return invoke('neocodex_provider_config', {})
}

export function setProvider(name: string): Promise<void> {
  return invoke('neocodex_set_provider', { name })
}

/** 连接测试：验证指定提供商是否可达（需后端 neocodex_test_provider 命令）。 */
export function testProvider(name: string): Promise<boolean> {
  return invoke('neocodex_test_provider', { name })
}

/* ── 代理池健康度 ── */
export interface ProviderHealthStatus {
  name: string
  available: boolean
  circuit_state: string
  success_rate: string
  total_calls: number
  total_errors: number
  is_free: boolean
  composite_score: string
  category: string
  latency_p95_ms: string
  latency_avg_ms: string
  latency_samples: number
  total_tokens: number
  health_penalty: string
  model_locked_count: number
}

export interface PoolSufficiencyReport {
  total_providers: number
  free_total: number
  free_available: number
  locked_models: number
  sufficient: boolean
}

export interface ProbeResult {
  name: string
  reachable: boolean
  status_code: number
  latency_ms: number
  error: string | null
}

export interface DiscoveryResult {
  discovered_count: number
  registered_total: number
  models: { provider: string; model_id: string; base_url: string; is_free: boolean; tier: string }[]
}

/** 获取所有 provider 的健康状态（电路/成功率/调用统计） */
export function providerStatus(): Promise<ProviderHealthStatus[]> {
  return domain.call<ProviderHealthStatus[]>('llamacpp', 'provider_status')
}

/** 获取池子充足度报告 */
export function poolSufficiency(minFree?: number): Promise<PoolSufficiencyReport> {
  return domain.call<PoolSufficiencyReport>('agent', 'pool_sufficiency', { min: minFree ?? 3 })
}

/** 手动触发免费模型发现（刷新 FreeModelCatalog + 注册到 GatewayV2） */
export function discoverModels(force?: boolean): Promise<DiscoveryResult> {
  return domain.call<DiscoveryResult>('llamacpp', 'discover_models', { force: force ?? false })
}

/** 批量探测所有已注册 provider 的网络可达性 */
export function probeAllProviders(): Promise<ProbeResult[]> {
  return domain.call<ProbeResult[]>('llamacpp', 'probe_all_providers')
}

/**
 * 端点连通性探测（兼容 ChatShellProto / ModelsSection 旧调用：providerTest(baseUrl)）。
 * 优先走后端 neocodex_provider_test（若存在），否则前端直连 /models 探测并计时。
 * 返回与旧契约一致的 { ok, status_code, latency_ms }，确保自主架构骨架完整。
 */
export async function providerTest(baseUrl: string): Promise<{ ok: boolean; status_code: number; latency_ms: number; latencyMs?: number }> {
  const t0 = Date.now()
  try {
    // 尝试后端真实探测（Rust 侧若未注册该命令会抛错，自动回退到前端探测）
    const r = await invoke<{ ok: boolean; status_code: number; latency_ms: number }>('neocodex_provider_test', { base_url: baseUrl }).catch(() => null)
    if (r) return { ...r, latencyMs: r.latency_ms }
  } catch { /* 回退 */ }
  // 前端回退：GET {baseUrl}/models 计时
  try {
    const url = baseUrl.trim().replace(/\/+$/, '') + '/models'
    const res = await fetch(url, { method: 'GET' })
    const latency = Date.now() - t0
    return { ok: res.ok, status_code: res.status, latency_ms: latency, latencyMs: latency }
  } catch {
    const latency = Date.now() - t0
    return { ok: false, status_code: 0, latency_ms: latency, latencyMs: latency }
  }
}

/** 外部第三方模型 API 智能配置：新增一个自定义提供商（OpenAI 兼容 / 自定义网关）。 */
export function addCustomProvider(req: CustomProviderReq): Promise<void> {
  return invoke('neocodex_add_custom_provider', { req })
}

/**
 * 智能检测：从 base_url 拉取可用模型列表（GET {base_url}/models，Bearer 鉴权）。
 * 桌面端走 Tauri 命令；浏览器预览无 Tauri 时直连 /models 作为回退（受 CORS 限制）。
 */
export async function fetchProviderModels(baseUrl: string, apiKey: string): Promise<string[]> {
  try {
    return await invoke<string[]>('neocodex_fetch_provider_models', { base_url: baseUrl, api_key: apiKey })
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
  return invoke('neocodex_set_mode', { mode })
}

/* ── 项目 / 文件 ── */
export function setProject(path: string): Promise<void> {
  return invoke('neocodex_set_project', { path })
}

export function getProject(): Promise<string | null> {
  return invoke('neocodex_get_project', {})
}

export function initProject(sessionId: string): Promise<void> {
  return invoke('neocodex_init_project', { session_id: sessionId })
}

export function searchFiles(query: string): Promise<string[]> {
  return invoke('neocodex_search_files', { query })
}

export function projectTree(): Promise<ProjectView> {
  return invoke('neocodex_project_tree', {})
}

export function openFile(path: string): Promise<void> {
  return invoke('neocodex_open_file', { path })
}

export function openExternal(path: string): Promise<void> {
  return invoke('neocodex_open_external', { path })
}

export function fileOperation(op: string, path: string, newName?: string): Promise<void> {
  return invoke('neocodex_file_operation', { op, path, new_name: newName ?? null })
}

/* ── Git ── */
export function gitStatus(): Promise<GitStatus | null> {
  return invoke('neocodex_git_status', {})
}

export function getDiff(): Promise<GitDiffResponse> {
  return invoke('neocodex_get_diff', {})
}

export interface GitDiffFile {
  path: string
  hunks: { lines: { t: 'add' | 'del' | 'ctx'; o: number | null; n: number | null; s: string }[] }[]
}

export interface GitDiffResponse {
  files: GitDiffFile[]
}

export function applyDiff(path: string, action: string): Promise<void> {
  return invoke('neocodex_apply_diff', { path, action })
}

/** 提交已暂存内容（对应面板 accept = git add 后的 commit）。 */
export function gitCommit(message: string): Promise<void> {
  return invoke('neocodex_git_commit', { message })
}

/** 推送当前分支到远程，返回远程输出摘要（无上游时错误内含提示）。 */
export function gitPush(): Promise<string> {
  return invoke('neocodex_git_push', {})
}

/** 列出本地分支（short ref names，如 main）。 */
export function listBranches(): Promise<string[]> {
  return invoke('neocodex_git_branch', {})
}

/** 返回当前已暂存文件列表（git diff --cached --name-only）。 */
export function gitStagedFiles(): Promise<string[]> {
  return invoke('neocodex_git_staged_files', {})
}

/** 切换分支（git checkout），返回切换后的分支名。 */
export function gitCheckout(branch: string): Promise<string> {
  return invoke('neocodex_git_checkout', { branch })
}

/* ── 检查点 ── */
export function checkpointList(sessionId: string): Promise<Checkpoint[]> {
  return invoke('neocodex_checkpoint_list', { session_id: sessionId })
}

export function checkpointRestore(sessionId: string, checkpointId: string): Promise<NeoCodexMessageItem[]> {
  return invoke('neocodex_checkpoint_restore', { session_id: sessionId, checkpoint_id: checkpointId })
}

/* ── 健康 / 状态 / 版本 ── */
export function healthReport(): Promise<HealthReport> {
  return invoke('neocodex_health_report', {})
}

export function agentStatus(): Promise<AgentStatus> {
  return invoke('neocodex_agent_status', {})
}

export function appVersion(): Promise<string> {
  return invoke('neocodex_app_version', {})
}

/* ── 更新（热更新；进度经 Tauri event 推送，见 api/system.ts listenUpdateEvents） ── */
export function checkUpdate(): Promise<UpdateCheckResult> {
  return invoke('neocodex_check_update', {})
}

export function downloadUpdate(): Promise<void> {
  return invoke('neocodex_download_update', {})
}

export function restartApp(): Promise<void> {
  return invoke('neocodex_restart_app', {})
}

/* ── MCP ── */
export function mcpList(): Promise<McpServerInfo[]> {
  return invoke('neocodex_mcp_list', {})
}

export function mcpTools(): Promise<McpToolInfo[]> {
  return invoke('neocodex_mcp_tools', {})
}

/** 注册本地 stdio MCP 服务器（name/command/args），返回注册后的服务器列表。 */
export function mcpRegister(name: string, command: string, args?: string[]): Promise<McpServerInfo[]> {
  return invoke('neocodex_mcp_register', { name, command, args: args ?? null })
}

/* ── 反馈 ── */
export function feedback(sessionId: string, text: string): Promise<void> {
  return invoke('neocodex_feedback', { session_id: sessionId, text })
}

/* ── 画板能力网 → NeoTrix 能力树（直接 invoke，非 domain plugin） ── */
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
  return invoke('canvas_sync_capabilities', { caps })
}

/** 画板覆盖层手动触发 Dark Forest 回收：把 canvas::<kind> 标记废弃 (画板 → 树 写回)。 */
export function canvasPruneCapability(kind: string): Promise<{ kind: string; pruned: boolean; constellation: string }> {
  return invoke('canvas_prune_capability', { kind })
}

/** 画板覆盖层把「期望成熟度」推回能力树：写入/清除 canvas::<kind> 的 canvas_desired 元数据。 */
export function canvasSetDesired(kind: string, stage: number | null): Promise<{ kind: string; pruned: boolean; constellation: string }> {
  return invoke('canvas_set_desired', { kind, stage })
}

/** 画板按自身能力树 SEAL 进化路线自动进化：执行所有作用于 canvas::* 的计划 (Mature/Prune)。 */
export function canvasApplyEvolutionRoute(): Promise<{ matured: number; pruned: number; applied: string[] }> {
  return invoke('canvas_apply_evolution_route', {})
}

/* ── 画板 KV（domain plugin） ── */

export function kbKvSet(namespace: string, key: string, value: string): Promise<void> {
  return domain.kb.kvSet(namespace, key, value)
}
export function kbKvGet(namespace: string, key: string): Promise<string | null> {
  return domain.kb.kvGet(namespace, key)
}
export function kbKvList(namespace: string): Promise<[string, string][]> {
  return domain.kb.kvList(namespace)
}
