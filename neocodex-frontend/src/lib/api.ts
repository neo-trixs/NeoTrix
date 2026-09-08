import { invoke } from "@tauri-apps/api/core";

export interface DomainHealth {
  name: string;
  status: string;
  constellation: string;
}

export interface ConsciousnessStatus {
  cycle: number;
  phi: number;
  coherence: number;
  gwt_resonance: number;
  mars_dual_process: boolean;
  governance_compliant: boolean;
  fog_level: number;
  domains: DomainHealth[];
}

export interface SystemInfo {
  platform: string;
  arch: string;
  neotrix_version: string;
  uptime_seconds: number;
  hostname: string;
  memory_used_mb: number;
  memory_total_mb: number;
  cpu_count: number;
}

export interface KbEntry {
  key: string;
  namespace: string;
  value: string;
  updated_at: string | null;
}

export interface ExperienceEntry {
  cycle_id: string;
  summary: string;
  tags: string[];
  created_at: string;
}

export interface AppConfig {
  theme: string;
  auto_start: boolean;
  daemon_port: number;
  log_level: string;
}

export interface LlmProvider {
  name: string;
  provider_type: string;
  model: string;
  api_base: string | null;
  is_active: boolean;
}

export interface ChatMessage {
  role: string;
  content: string;
}

export interface ChatResponse {
  content: string;
  model: string;
  tokens_used: number | null;
}

// Default values
export const defaultConsciousnessStatus: ConsciousnessStatus = {
  cycle: 0,
  phi: 0,
  coherence: 0,
  gwt_resonance: 0,
  mars_dual_process: false,
  governance_compliant: false,
  fog_level: 0,
  domains: [],
};

export const defaultAppConfig: AppConfig = {
  theme: "dark",
  auto_start: false,
  daemon_port: 9527,
  log_level: "info",
};

// API functions
export async function getConsciousnessStatus(): Promise<ConsciousnessStatus> {
  try {
    return await invoke<ConsciousnessStatus>("get_status");
  } catch {
    return defaultConsciousnessStatus;
  }
}

export async function tickGrowth(cycles?: number): Promise<{ phases: unknown[]; total_cycles: number }> {
  return invoke("tick_growth", { cycles });
}

export async function runTask(instruction: string): Promise<string> {
  return invoke("run_task", { instruction });
}

export async function getSystemInfo(): Promise<SystemInfo> {
  try {
    return await invoke<SystemInfo>("get_system_info");
  } catch {
    return {
      platform: "unknown",
      arch: "unknown",
      neotrix_version: "unknown",
      uptime_seconds: 0,
      hostname: "unknown",
      memory_used_mb: 0,
      memory_total_mb: 0,
      cpu_count: 1,
    };
  }
}

export async function queryKb(namespace: string, keyword?: string): Promise<{ entries: KbEntry[]; total: number }> {
  return invoke("query_kb", { namespace, keyword: keyword || null });
}

export async function listExperiences(): Promise<ExperienceEntry[]> {
  return invoke("list_experiences");
}

export async function getConfig(): Promise<AppConfig> {
  try {
    return await invoke<AppConfig>("get_config");
  } catch {
    return defaultAppConfig;
  }
}

export async function setConfig(config: AppConfig): Promise<void> {
  return invoke("set_config", { config });
}

export async function listProviders(): Promise<LlmProvider[]> {
  try {
    return await invoke<LlmProvider[]>("list_providers");
  } catch {
    return [];
  }
}

export async function sendMessage(messages: ChatMessage[], model?: string): Promise<ChatResponse> {
  return invoke("send_message", { request: { messages, model } });
}

export async function getNeotrixVersion(): Promise<string> {
  try {
    return await invoke<string>("get_version");
  } catch {
    return "unknown";
  }
}
