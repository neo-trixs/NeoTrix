export interface Session {
  id: string;
  name: string;
  messages: Message[];
  pinned?: boolean;
  lastActive?: number;
}

export interface Message {
  role: "user" | "assistant" | "system" | "error";
  content: string;
  contentType?: "markdown" | "html" | "text";
  timestamp?: number;
  attachments?: Attachment[];
}

export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  depth: number;
  size?: number;
}

export type ProviderId = "anthropic" | "openai" | "gemini" | "ollama";

export interface ProviderConfig {
  id: ProviderId;
  name: string;
  model: string;
  apiKey: string;
  baseUrl?: string;
  learningRate: number;
}

export interface DiffBlock {
  type: "added" | "removed" | "unchanged";
  content: string;
  lineStart: number;
}

export interface PermissionRequest {
  id: string;
  action: string;
  target: string;
  details: string;
  timestamp: number;
}

export interface KnowledgeEntry {
  id: string;
  title: string;
  source: string;
  category: string;
  tags: string[];
  content: string;
  created: number;
  updated: number;
}

export interface AgentPreset {
  id: string;
  name: string;
  description: string;
  systemPrompt: string;
  model: string;
  modelTier: ModelTier;
  temperature: number;
  tools: string[];
  knowledgeSources: string[];
  maxTokens: number;
  isBuiltin: boolean;
}

export interface Attachment {
  id: string;
  name: string;
  size: number;
  mimeType: string;
  data: string; // base64
}

export interface AppSettings {
  theme: "light" | "dark" | "system";
  fontSize: number;
  autoSave: boolean;
  language: "zh-CN" | "en-US";
  terminalPath: string;
  maxSessions: number;
  // ── Enhanced settings ──
  voiceInput: boolean;
  voiceLang: string;
  voiceAutoSend: boolean;
  privacyStoreMessages: boolean;
  privacyTelemetry: boolean;
  privacyLocalFirst: boolean;
  privacyPreflightCheck: boolean;
  defaultModel: string;
  temperature: number;
  maxTokens: number;
}

export interface ProxyStatus {
  running: boolean;
  mode: string;
  pid: number;
  port: number;
  uptime_secs: number;
  active_count: number;
  idle_secs: number;
}

export interface ProxySourceInfo {
  name: string;
  total_successes: number;
  total_failures: number;
  consecutive_failures: number;
  on_cooldown: boolean;
}

export interface ProxyConnectivity {
  active_mode: string;
  direct_reachable: boolean;
  direct_latency_ms: number | null;
  proxy_healthy_count: number;
  proxy_total_count: number;
  proxy_avg_latency_ms: number | null;
}

export interface ProxyNodeInfo {
  url: string;
  tag: string;
  latency_ms: number | null;
  fail_count: number;
  success_count: number;
  from_subscription: boolean;
  geo_tag: string | null;
  ip_addr: string | null;
  speed_tier: "Fast" | "Medium" | "Slow" | "Unknown";
  score: number;
  healthy: boolean;
}

export interface ProxyConfigData {
  local_port: number;
  socks_port: number;
  min_nodes: number;
  health_check_interval_secs: number;
  selection_strategy: string;
  system_proxy_enabled: boolean;
  direct_timeout_secs: number;
}

export interface Project {
  id: string;
  name: string;
  path: string;
  project_type: string;
  description: string | null;
  created_at: number;
  updated_at: number;
  pinned: boolean;
  archived: boolean;
  color: string | null;
  icon: string | null;
}

export interface ProjectChat {
  id: string;
  project_id: string;
  name: string;
  session_id: string | null;
  message_count: number;
  created_at: number;
  updated_at: number;
  pinned: boolean;
  archived: boolean;
}

export interface ProjectSource {
  id: string;
  project_id: string;
  source_type: string;
  path: string | null;
  url: string | null;
  name: string;
  enabled: boolean;
  created_at: number;
}

export interface ProjectInstruction {
  id: string;
  project_id: string;
  content: string;
  enabled: boolean;
  created_at: number;
  updated_at: number;
}

// ========== Browser Pane ==========
export interface ImageGenOptions {
  width: number;
  height: number;
  style: string;
  quality: string;
}

// ========== Computer Use ==========
export interface ScreenCapture {
  image_base64: string;
  width: number;
  height: number;
}

export interface WindowInfo {
  title: string;
  pid: number;
  app_name: string;
}

export interface FrontmostApp {
  app_name: string;
  title: string;
}

// ========== Remote Devbox ==========
export type NeoCodexMode = "Agent" | "Shell" | "Plan";

export interface NeoCodexHealthReport {
  mode: NeoCodexMode;
  turn_count: number;
  tool_call_count: number;
  tokens_used: number;
  context_usage: number;
  context_turns: number;
  provider_count: number;
  provider_resolvable: boolean;
  provider_model: string;
  session_writable: boolean;
  goals_active: boolean;
  cost_spent: number;
  cost_budget: number;
  subagent_results: number;
  consciousness_attached: boolean;
  brain_attached: boolean;
  event_bus_attached: boolean;
  evolution_iterations: number;
  tool_grounding_degraded: boolean;
}

export interface NeoCodexProviderConfig {
  provider_count: number;
  resolvable: boolean;
  active_model: string;
  providers: NeoCodexProviderEntry[];
}

export interface NeoCodexEvolutionState {
  iteration: number;
  fixes_applied: number;
  gaps_found: number;
  history: string[];
}

export interface NeoCodexSession {
  id: string;
  name: string;
  mode: NeoCodexMode;
  messages: Message[];
  wire_path: string;
  created_at: number;
  updated_at: number;
}

// ========== API 支撑类型 (lib/api.ts) ==========
export type ModelTier = "free" | "low" | "medium" | "high" | "custom";

export interface BrowserSession {
  id: string;
  url: string;
  title: string;
  created_at: number;
  is_active: boolean;
}

export interface ImageGenerationResult {
  id: string;
  prompt: string;
  image_url: string;
  width: number;
  height: number;
  style: string;
  quality: string;
  created_at: number;
}

export interface RemoteHostConfig {
  id: string;
  name: string;
  host: string;
  port: number;
  user: string;
  auth_method: string;
  key_path: string | null;
  last_connected: number | null;
}

export interface RemoteExecutionResult {
  success: boolean;
  output: string;
  error: string | null;
  exit_code: number;
  duration_ms: number;
}

export interface BackgroundTask {
  id: string;
  name: string;
  prompt: string;
  schedule: string;
  last_run: number | null;
  next_run: number | null;
  status: "idle" | "running" | "paused" | "error";
  runs: TaskRun[];
}

export interface TaskRun {
  timestamp: number;
  summary: string;
  result?: string;
}

export interface ReviewConfig {
  scope: "changed" | "staged" | "all";
  depth: "standard" | "deep" | "exhaustive";
  dimensions: string[];
  auto_fix: boolean;
}

export interface ReviewResult {
  run_id: string;
  config: ReviewConfig;
  issues_found: number;
  issues: ReviewIssue[];
  duration_ms: number;
}

export interface ReviewIssue {
  id: string;
  dimension: string;
  severity: "info" | "warning" | "error";
  file: string;
  line: number;
  message: string;
  suggestion?: string;
}

export interface NeoCodexProviderEntry {
  name: string;
  model: string;
  resolvable: boolean;
}


