export interface Session {
  id: string;
  name: string;
  messages: Message[];
}

export interface Message {
  role: "user" | "assistant" | "system" | "error";
  content: string;
  contentType?: "markdown" | "html" | "text";
  timestamp?: number;
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

export interface ProxyStatus {
  running: boolean;
  mode: string;
  pid: number;
  port: number;
  uptime_secs: number;
  active_count: number;
  idle_secs: number;
}
