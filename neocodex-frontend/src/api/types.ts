/* ════════════════════════════════════════════
   api/types.ts — 前后端 IPC 统一契约（单一事实源）
   所有 tauri 命令的返回类型集中于此，组件/路由只 import 这里，
   避免命令名散落与类型漂移。字段 snake_case 对齐 Rust serde。
   ════════════════════════════════════════════ */

/* ── neocodex：会话 / 消息 ── */
export interface NeoCodexSessionInfo {
  id: string
  name: string
  mode: string
  message_count: number
  wire_path: string
  updated_at: number
  tags?: string[]
}

export interface NeoCodexAttachmentDto {
  name: string
  size: number
  mime_type: string
  data?: string
}

export interface NeoCodexMessageItem {
  id: number
  role: string
  content: string
  timestamp: number
  attachments?: NeoCodexAttachmentDto[]
  tool_call?: {
    name: string
    args: string
    result: string
    duration_ms: number
    success: boolean
  }
}

export interface ToolCallRecord {
  id: string
  name: string
  args: string
  result: string
  duration_ms: number
  success: boolean
}

export interface NeoCodexSearchHit {
  session_id: string
  session_name: string
  timestamp: number
  role: string
  content: string
  tag?: string
}

/* ── neocodex：提供商 ── */
export interface ProviderMeta {
  name: string
  display_name: string
  category: string // local | proxy | cloud | unknown
  is_free: boolean
  base_url: string
  model: string
  models: string[]
  resolvable: boolean
}

export interface ProviderConfig {
  provider_count: number
  resolvable: boolean
  active_model: string
  providers: ProviderMeta[]
}

/* ── 系统 / 更新 ── */
export interface UpdateCheckResult {
  current: string
  available: boolean
  latest: string
  error: string | null
}

export interface UpdateProgress {
  downloaded: number
  total: number
}

export interface HealthReport {
  healthy: boolean
  uptime_secs: number
  turn_count: number
  tokens_used: number
  context_usage: number
  provider_model: string
  evolution_iterations: number
  cost_spent: number
  cost_budget: number
}

export interface AgentStatus {
  running: boolean
  current_task: string | null
  uptime_secs: number
  turn_count: number
  tokens_used: number
  context_usage: number
  provider_model: string
  evolution_iterations: number
  cost_spent: number
  cost_budget: number
}

/* ── 记忆 / 数据 ── */
export interface MemoryStats {
  total_entries: number
  total_categories: number
  avg_confidence: number
  memory_usage_bytes: number
}

/* ── 项目 ── */
export interface ProjectTreeItem {
  name: string
  path: string
  is_dir: boolean
  children?: ProjectTreeItem[] | null
}

export interface ProjectView {
  root: string
  tree: ProjectTreeItem[]
  agents_md: string | null
  file_count: number
}

/* ── 协同 ── */
export interface CoworkSession {
  id: string
  name: string
  workspace_path: string
  status: string
  files_read: number
  files_created: number
  files_modified: number
  started_at: number
  last_active_at: number
  deliverables: string[]
  description: string
  tags: string[]
}

export interface CoworkAction {
  id: string
  session_id: string
  action_type: string
  target_path: string
  status: string
  started_at: number
  completed_at: number | null
  details: string | null
  result_summary: string | null
}

export interface CoworkDeliverable {
  id: string
  session_id: string
  name: string
  path: string
  kind: string
  created_at: number
  size_bytes: number
  description: string
  quality_score: number | null
}

/* ── 定时任务 ── */
export interface TaskRun {
  timestamp: number
  summary: string
}

export interface BackgroundTask {
  id: string
  name: string
  prompt: string
  schedule: string
  last_run: number | null
  next_run: number | null
  status: string
  runs: TaskRun[]
}

/* ── 插件 ── */
export interface PluginStatus {
  id: string
  name: string
  version: string
  enabled: boolean
  loaded: boolean
  load_time_ms: number
  error: string | null
}

export interface PluginEvent {
  timestamp: number
  kind: string
  plugin_id: string
  message: string
}

/* ── 电脑操作 ── */
export interface ScreenCapture {
  path: string
  width: number
  height: number
  format: string
  timestamp: number
}

export interface WindowInfo {
  title: string
  pid: number
  app_name: string
}

export interface FrontmostApp {
  app_name: string
  title: string
}

export interface MousePosition {
  x: number
  y: number
}

export interface DisplayInfo {
  id: number
  name: string
  width: number
  height: number
  is_primary: boolean
  scale_factor: number
}

/* ── 检查点 / 语音 ── */
export interface Checkpoint {
  id: string
  created_at: number
  message_count: number
}

export interface VoiceTranscript {
  id: string
  text: string
  confidence: number
  duration_ms: number
}

/* ── Git ── */
export interface GitStatus {
  branch: string
  dirty: boolean
}
