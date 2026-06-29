export interface Session {
  id: string;
  name: string;
  messages: Message[];
}

export interface ToolCall {
  id: string;
  tool: string;
  args: Record<string, string>;
  result?: string;
  status: "pending" | "running" | "success" | "error";
  duration_ms?: number;
}

export interface Message {
  role: "user" | "assistant" | "system" | "error" | "tool_call" | "tool_result";
  content: string;
  contentType?: "markdown" | "html" | "text";
  timestamp?: number;
  toolCall?: ToolCall;
  /** Sub‑steps rendered as an ordered list inside the message block */
  steps?: { label: string; status: "done" | "running" | "pending" }[];
  /** Code‑edit diff blocks (green/red diff‑style) */
  diffs?: { file: string; added: number; removed: number; diff: string }[];
  /** Collapsed by default when true */
  collapsible?: boolean;
}

export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileNode[];
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

export interface VirtualApp {
  id: string;
  name: string;
  icon: string;
  description: string;
  action: "terminal" | "files" | "settings" | "splitview" | "agentmaker" | "agentflow";
}

export interface DesktopWindow {
  id: string;
  title: string;
  appId: string;
  x: number;
  y: number;
  width: number;
  height: number;
  zIndex: number;
  minimized: boolean;
}

export interface SplitViewSession {
  id: string;
  leftModel: string;
  rightModel: string;
  prompt: string;
  leftResponse: string;
  rightResponse: string;
  timestamp: number;
}

export type ModelTier = "free" | "low" | "medium" | "high" | "custom";

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

export interface AgentMakerPreset {
  id: string;
  name: string;
  description: string;
  category: string;
  icon: string;
  systemPrompt: string;
  defaultModel: string;
  defaultTier: ModelTier;
  defaultTemperature: number;
  defaultMaxTokens: number;
  suggestedTools: string[];
  suggestedKnowledge: string[];
}

export type AgentStatus = "running" | "completed" | "failed" | "pending" | "idle";
export type AgentNodeType = "orchestrator" | "planner" | "sub-agent" | "critic" | "aggregator" | "input" | "output";

export interface AgentFlowNodeData extends Record<string, unknown> {
  label: string;
  agentType: AgentNodeType;
  status: AgentStatus;
  description: string;
  progress?: number;
  duration?: string;
  steps?: { done: number; total: number };
}

export interface EvolutionState {
  iteration: number;
  strategy: string;
  contextUsage: number;
  intrinsicReward: number;
  confidence: number;
  errorRate: number;
  noveltyScore: number;
  shouldExplore: boolean;
  stabilityScore: number;
  flagsCount: number;
  repairsCount: number;
  archiveSnapshots: number;
  selfRepairs: number;
}

export interface EditorState {
  open: boolean;
  filePath: string;
  initialContent: string;
  language: string;
}

export interface ContextMenuItem {
  label: string;
  icon?: string;
  shortcut?: string;
  action: () => void;
  disabled?: boolean;
  divider?: boolean;
}

export interface AppSettings {
  theme: "light" | "dark" | "system";
  fontSize: number;
  autoSave: boolean;
  language: "zh-CN" | "en-US";
  terminalPath: string;
  maxSessions: number;
}

export interface PluginInfo {
  name: string;
  version: string;
  description: string;
  author: string;
  abi_version: number;
  installed_at: string;
  status: "Installed" | "Loaded" | "Error";
  permissions: string[];
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

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size?: number;
}

export interface BrainHealth {
  health_score: number;
  degradation?: string;
  cognitive_load?: string;
  iteration?: number;
  curiosity_bonus?: number;
}

export interface ImAdapterStatus {
  running: boolean;
  error: boolean;
  enabled: boolean;
  adapter_type?: string;
  name?: string;
}

export interface ImAdapterConfig {
  id: string;
  name: string;
  type: string;
  status: string;
  bot_token?: string;
  phone_number_id?: string;
  access_token?: string;
  webhook_url?: string;
  verify_token?: string;
  config?: Record<string, string>;
}

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  capabilities: string[];
  context_length: number;
  tier?: string;
}

export interface FeedItem {
  id: string;
  type: string;
  content: string;
  timestamp: number;
  tags?: string[];
  metadata?: Record<string, unknown>;
  neotrix_insight?: string;
  source_url?: string;
  content_type?: string;
  source_name?: string;
  published_at?: number;
  title?: string;
  description?: string;
  image_url?: string;
  score?: number;
}

export interface FeedTag {
  name: string;
  count: number;
  is_active?: boolean;
}

export interface TimelineEntry {
  id: string;
  title: string;
  start_time: number;
  end_time: number;
  item_ids: string[];
  key_events: string[];
  neotrix_summary?: string;
}

export interface FeedState {
  items: FeedItem[];
  tags: FeedTag[];
  total_count: number;
  last_refresh?: number;
  timelines: TimelineEntry[];
}

export interface ProxySourceInfo {
  id: string;
  name: string;
  url: string;
  status: string;
  last_fetch?: number;
  on_cooldown?: boolean;
  total_successes?: number;
  total_failures?: number;
  consecutive_failures?: number;
}

export interface ProxyConnectivity {
  online: boolean;
  latency_ms?: number;
  last_checked?: number;
  error?: string;
  active_mode?: string;
  direct_reachable?: boolean;
  direct_latency_ms?: number;
  proxy_healthy_count?: number;
  proxy_total_count?: number;
  proxy_avg_latency_ms?: number;
}

export interface RemoteSessionInfo {
  id: string;
  name: string;
  connected: boolean;
  peer_address?: string;
  started_at?: number;
  qr_svg?: string;
  relay_url?: string;
  qr_url?: string;
  state?: string;
}

export interface BrainEvent {
  id: string;
  type: string;
  label: string;
  description: string;
  timestamp: number;
  phi?: number;
  kind?: string;
  status?: string;
  name?: string;
  duration_ms?: number;
  concept_count?: number;
  domain?: string;
}

export interface CanvasNode {
  id: string;
  label: string;
  node_type: string;
  x: number;
  y: number;
  width?: number;
  height?: number;
  color: string;
  content?: string;
  metadata?: Record<string, string>;
}

export interface CanvasEdge {
  id: string;
  source_id: string;
  target_id: string;
  label?: string;
  edge_type: string;
}

export interface CanvasUpdate {
  nodes: CanvasNode[];
  edges: CanvasEdge[];
  node_count: number;
  edge_count: number;
}

export type AgentMode = "chat" | "plan" | "agent";

export interface ContextChip {
  id: string;
  label: string;
  type: "file" | "folder" | "codebase" | "docs" | "web";
}

export interface SandboxStatus {
  running: boolean;
  active_count: number;
  mode: string;
  uptime_secs: number;
  default_timeout_ms: number;
  allow_network: boolean;
  recent_executions: number;
  failed_count: number;
}

export interface SandboxResult {
  stdout: string;
  stderr: string;
  exit_code: number;
  duration_ms: number;
  timed_out: boolean;
}
