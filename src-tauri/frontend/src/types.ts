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

export interface BrainHealth {
  health_score: number;
  degradation?: string;
  cognitive_load?: string;
  iteration?: number;
  curiosity_bonus?: number;
}

export interface BrainEvent {
  kind: "stage" | "knowledge";
  status: string;
  name?: string;
  duration_ms?: number;
  concept_count?: number;
  domain?: string;
  iteration?: number;
}

// ========== Consciousness: E8 Engine State ==========
export interface E8State {
  hexagram: number;
  hexagramName: string;
  confidence: number;
  lines: E8Line[];
  transitioning: boolean;
}

export interface E8Line {
  value: 0 | 1;
  changing: boolean;
}

export interface GWTResonance {
  activeCount: number;
  totalCount: number;
  entropy: number;
  experts: GWTExpert[];
}

export interface GWTExpert {
  id: string;
  shortName: string;
  icon: string;
  resonance: number;
  hue: number;
  weight: number;
}

export interface SEALStatus {
  maturityLevel: 1 | 2 | 3 | 4 | 5 | 6;
  currentEpoch: number;
  stageName: string;
  healthScore: number;
}

// ========== Agent Governance (Omnigent-inspired) ==========
export interface AgentPolicy {
  id: string;
  name: string;
  description: string;
  type: "shell" | "file_write" | "network" | "budget" | "custom";
  enabled: boolean;
  config?: Record<string, string | number | boolean>;
}

export interface AgentRuntimeInfo {
  id: string;
  name: string;
  harness: string;
  status: "running" | "idle" | "error";
  model: string;
  provider: string;
  policies: AgentPolicy[];
}

// ========== Privacy Filter (Osaurus-inspired) ==========
export interface PrivacyFilterConfig {
  enabled: boolean;
  mode: "auto" | "strict" | "custom";
  redactMode: "placeholder" | "mask" | "drop";
  unscrubOnReply: boolean;
  customPatterns: string[];
}

export interface PiiDetection {
  type: string;
  value: string;
  placeholder: string;
  confidence: number;
}

// ========== Sandbox (Osaurus-inspired) ==========
export type SandboxProvider = "local-docker" | "apple-container" | "modal" | "e2b";
export type SandboxRuntime = "python" | "node" | "rust" | "linux";
export type SandboxStatus = "running" | "stopped" | "error";
export type SandboxNetwork = "isolated" | "bridge" | "host";

export interface SandboxInstance {
  id: string;
  name: string;
  runtime: SandboxRuntime;
  status: SandboxStatus;
  memory: string;
  cpu: string;
  uptime: string;
  network: SandboxNetwork;
}

// ========== Identity & Keys (Osaurus-inspired) ==========
export interface AccessKey {
  id: string;
  label: string;
  prefix: string;
  scope: "full" | "agent-only" | "read-only";
  status: "active" | "revoked" | "expired";
  createdAt: string;
  lastUsed: string;
}

export interface IdentityInfo {
  name: string;
  address: string;
  edition: number;
  verified: boolean;
}

// ========== Projects & Chats (Codex-style) ==========
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
