import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { getCurrent } from "@tauri-apps/plugin-deep-link";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import type { ProviderConfig, PermissionRequest, FileNode, DiffBlock } from "../types";

export interface BrainStats {
  iteration: number;
  absorb_count: number;
  capability_sum: number;
  memory_count: number;
  engine_active: boolean;
  capability_vector: number[];
  dimension_names: string[];
}

export interface SessionInfo {
  id: string;
  name: string;
  message_count: number;
}

export interface ProjectInfo {
  name: string;
  path: string;
  language: string;
  file_count: number;
}

export async function getBrainStats(): Promise<BrainStats> {
  return invoke<BrainStats>("brain_stats");
}

export async function getBrainStatsV2(): Promise<BrainStats> {
  return invoke<BrainStats>("get_brain_stats");
}

export async function agentReason(prompt: string): Promise<{ output: string; success: boolean }> {
  return invoke("agent_reason", { req: { prompt } });
}

export async function testProviderConnection(config: ProviderConfig): Promise<boolean> {
  try {
    const result = await invoke<string>("test_provider", { config });
    return result === "ok";
  } catch {
    return false;
  }
}

export async function loadSessions(): Promise<SessionInfo[]> {
  return invoke<SessionInfo[]>("session_list");
}

export async function createSession(name: string): Promise<SessionInfo> {
  return invoke<SessionInfo>("session_create", { name });
}

export async function readDirRecursive(path: string, maxDepth?: number): Promise<FileNode[]> {
  return invoke<FileNode[]>("read_dir_recursive", { path, maxDepth: maxDepth ?? 3 });
}

export async function detectProject(path: string): Promise<ProjectInfo> {
  return invoke<ProjectInfo>("detect_project", { path });
}

export async function readFile(path: string): Promise<string> {
  return invoke<string>("read_file", { path });
}

export async function getPendingPermissions(): Promise<PermissionRequest[]> {
  return invoke<PermissionRequest[]>("get_pending_permissions");
}

export async function respondPermission(requestId: string, approved: boolean): Promise<void> {
  return invoke("respond_permission", { requestId, approved });
}

export async function requestPermission(req: PermissionRequest): Promise<PermissionRequest> {
  return invoke<PermissionRequest>("request_permission", { req });
}

export async function getDiffStaged(): Promise<DiffBlock[]> {
  return invoke<DiffBlock[]>("cmd_diff_staged");
}

export async function getDiffUnstaged(): Promise<DiffBlock[]> {
  return invoke<DiffBlock[]>("cmd_diff_unstaged");
}

export async function getDiffFile(filePath: string): Promise<DiffBlock[]> {
  return invoke<DiffBlock[]>("cmd_diff_file", { filePath });
}

export async function searchKnowledge(query: string): Promise<{ id: string; title: string; content: string; relevance: number }[]> {
  try {
    const results = await invoke<string>("search_knowledge", { query });
    return JSON.parse(results);
  } catch {
    return [];
  }
}

export async function saveProviderConfig(config: ProviderConfig): Promise<void> {
  return invoke("save_provider_config", { config });
}

// ========== User Avatar API ==========

export interface UserAvatar {
  edition: number;
  confidence: number;
  language_preference: number;
  communication_style: number;
  reasoning_depth: number;
  technical_depth: number;
  domain_scores: Record<string, number>;
  task_affinity: Record<string, number>;
  knowledge_affinity: Record<string, number>;
  tags: string[];
  summary: string;
  total_messages_processed: number;
}

export interface DistillationNode {
  id: string;
  label: string;
  status: string;
  description: string;
  type: string;
  progress: number;
  ttl_seconds: number;
}

export interface DistillationEdge {
  source: string;
  target: string;
}

export interface DistillationFlowEvent {
  nodes: DistillationNode[];
  edges: DistillationEdge[];
  avatar_summary: string;
  avatar_confidence: number;
}

export async function getUserAvatar(): Promise<UserAvatar> {
  return invoke<UserAvatar>("get_user_avatar");
}

export async function getDistillationFlow(): Promise<DistillationFlowEvent> {
  return invoke<DistillationFlowEvent>("get_distillation_flow");
}

export async function distillMessage(text: string): Promise<DistillationFlowEvent> {
  return invoke<DistillationFlowEvent>("distill_message", { text });
}

// ========== Identity API ==========

export interface AvatarIdentity {
  name: string;
  identity_key_hmac: string;
  created_at: number;
  updated_at: number;
  edition: number;
}

export interface ChainStats {
  total_entries: number;
  outbound_count: number;
  inbound_count: number;
  genesis_hash: string;
  chain_valid: boolean;
  identity_name: string;
  identity_edition: number;
}

export async function setUserIdentity(name: string): Promise<UserAvatar> {
  return invoke<UserAvatar>("set_user_identity", { name });
}

export async function getIdentity(): Promise<AvatarIdentity | null> {
  return invoke<AvatarIdentity | null>("get_identity");
}

export async function getChainStats(): Promise<ChainStats> {
  return invoke<ChainStats>("get_chain_stats");
}

export async function brainWriteBack(text: string): Promise<number> {
  return invoke<number>("brain_write_back", { text });
}

export interface ConsciousnessMetrics {
  phi: number;
  fcs: number;
  usk: number;
}

export async function getConsciousnessMetrics(): Promise<ConsciousnessMetrics> {
  const stats = await getBrainStatsV2();
  return {
    phi: stats.capability_sum,
    fcs: stats.capability_vector.length,
    usk: stats.memory_count,
  };
}

// ========== File Dialog API ==========

export async function openFileDialog(): Promise<string | null> {
  try {
    const result = await open({
      multiple: false,
      directories: false,
    });
    return result ?? null;
  } catch {
    return null;
  }
}

export async function saveFileDialog(data: string, filename: string): Promise<void> {
  try {
    const path = await save({
      defaultPath: filename,
    });
    if (path) {
      await writeTextFile(path, data);
    }
  } catch (e) {
    console.error("Save file dialog error:", e);
  }
}

// ========== Deep Link API ==========

export async function getDeepLinkUrl(): Promise<string | null> {
  try {
    const urls = await getCurrent();
    return urls ? urls.join(",") : null;
  } catch {
    return null;
  }
}

// ========== Proxy API ==========

import type { ProxyStatus } from "../types";

export async function proxyStatus(): Promise<ProxyStatus> {
  try {
    return await invoke<ProxyStatus>("proxy_status");
  } catch {
    return { running: false, mode: "off", pid: 0, port: 11080, uptime_secs: 0, active_count: 0, idle_secs: 0 };
  }
}

export async function proxySetMode(mode: string): Promise<string> {
  return invoke<string>("proxy_set_mode", { mode });
}

export async function proxyStartDaemon(): Promise<string> {
  return invoke<string>("proxy_start_daemon");
}

export async function proxyStopDaemon(): Promise<string> {
  return invoke<string>("proxy_stop_daemon");
}
