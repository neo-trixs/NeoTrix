import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { getCurrent } from "@tauri-apps/plugin-deep-link";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import type { ProviderConfig, PermissionRequest, FileNode, DiffBlock, PluginInfo, FileEntry, ImAdapterStatus, ImAdapterConfig, ModelInfo, FeedItem, FeedState, ProxySourceInfo, ProxyConnectivity, RemoteSessionInfo, SandboxStatus, SandboxResult } from "../types";

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
    const result = await invoke<boolean>("test_provider", { payload: config });
    return result;
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
  return invoke("save_provider_config", { payload: config });
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

export async function readConsciousnessResponse(): Promise<string[]> {
  return invoke<string[]>("read_consciousness_response");
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

// ========== Plugin API ==========

export async function pluginList(): Promise<PluginInfo[]> {
  return invoke<PluginInfo[]>("plugin_list");
}

export async function pluginLoad(name: string): Promise<void> {
  await invoke("plugin_load", { name });
}

export async function pluginUnload(name: string): Promise<void> {
  await invoke("plugin_unload", { name });
}

export async function pluginUninstall(name: string): Promise<void> {
  await invoke("plugin_uninstall", { name });
}

export async function pluginInstallFromZip(zipPath: string): Promise<PluginInfo> {
  return invoke<PluginInfo>("plugin_install_from_zip", { zipPath });
}

export async function pluginGetInfo(name: string): Promise<PluginInfo | null> {
  return invoke<PluginInfo | null>("plugin_get_info", { name });
}

export async function pluginWriteData(name: string, key: string, value: string): Promise<void> {
  await invoke("plugin_write_data", { name, key, value });
}

export async function pluginReadData(name: string, key: string): Promise<string | null> {
  return invoke<string | null>("plugin_read_data", { name, key });
}

export async function getCurrentProvider(): Promise<ProviderConfig> {
  return invoke<ProviderConfig>("get_current_provider");
}

// ========== Browser API ==========

export interface BrowserState {
  url: string;
  title: string;
  is_open: boolean;
}

export interface SearchResultItem {
  title: string;
  url: string;
  snippet?: string;
}

export interface CredentialInfo {
  id: string;
  domain: string;
  username: string;
  notes: string;
  created_at: number;
}

export interface WebAppAgentInfo {
  id: string;
  name: string;
  url_pattern: string;
  actions: { id: string; label: string }[];
  is_active: boolean;
}

export async function browserOpen(url: string): Promise<BrowserState> {
  return invoke<BrowserState>("browser_open", { url });
}

export async function browserClose(): Promise<void> {
  await invoke("browser_close");
}

export async function browserBack(): Promise<void> {
  await invoke("browser_back");
}

export async function browserForward(): Promise<void> {
  await invoke("browser_forward");
}

export async function browserReload(): Promise<void> {
  await invoke("browser_reload");
}

export async function browserExtractContent(url: string): Promise<{ title: string; summary: string }> {
  return invoke("browser_extract_content", { args: { url } });
}

export async function browserAgentDetect(url: string, title: string): Promise<WebAppAgentInfo | null> {
  try {
    return await invoke<WebAppAgentInfo | null>("browser_agent_detect", { url, title });
  } catch {
    return null;
  }
}

export async function browserAgentList(): Promise<WebAppAgentInfo[]> {
  return invoke<WebAppAgentInfo[]>("browser_agent_list");
}

export async function browserAgentExecute(agentId: string, actionId: string): Promise<string> {
  return invoke<string>("browser_agent_execute", { agentId, actionId });
}

export async function browserCredentialList(): Promise<CredentialInfo[]> {
  return invoke<CredentialInfo[]>("browser_credential_list");
}

export async function browserCredentialStore(domain: string, username: string, password: string, notes?: string): Promise<CredentialInfo> {
  return invoke<CredentialInfo>("browser_credential_store", { domain, username, password, notes });
}

export async function browserCredentialRemove(id: string): Promise<void> {
  await invoke("browser_credential_remove", { id });
}

export async function browserCredentialAutofill(domain: string): Promise<string> {
  return invoke<string>("browser_credential_autofill", { domain });
}

export async function toolSearch(query: string, count?: number): Promise<SearchResultItem[]> {
  return invoke<SearchResultItem[]>("tool_search", { query, count: count ?? 8 });
}

export async function toolExecute(tool: string, args: Record<string, unknown>): Promise<{ success: boolean; output: string; duration_ms: number }> {
  return invoke("tool_execute", { tool, args });
}

// ========== X Auto-Scroll API ==========

export interface XAutoScrollStatus {
  running: boolean;
  tweet_count: number;
  current_url: string;
  session_active: boolean;
  absorbed: number;
  negentropy_avg: number;
}

export interface XAbsorptionEvent {
  count: number;
  total_negentropy: number;
  avg_negentropy: number;
  tweets_seen: number;
}

export interface XHumanProfile {
  scroll_speed: number;
  pause_range: [number, number];
  scroll_variance: number;
  mouse_trail: boolean;
  interaction_rate: number;
  user_agent: string;
}

export async function browserXStartSession(): Promise<string> {
  return invoke<string>("browser_x_start_session");
}

export async function browserXLogin(username: string, password: string): Promise<string> {
  return invoke<string>("browser_x_login", { username, password });
}

export async function browserXHumanScroll(): Promise<string> {
  return invoke<string>("browser_x_human_scroll");
}

export async function browserXStopSession(): Promise<string> {
  return invoke<string>("browser_x_stop_session");
}

export async function browserXStatus(): Promise<XAutoScrollStatus> {
  return invoke<XAutoScrollStatus>("browser_x_status");
}

export async function browserXHumanProfile(): Promise<XHumanProfile> {
  return invoke<XHumanProfile>("browser_x_human_profile");
}

// ========== Consciousness API ==========

export async function getConsciousnessFull(): Promise<Record<string, unknown>> {
  try {
    return await invoke<Record<string, unknown>>("get_consciousness_full");
  } catch {
    return {};
  }
}

export async function getE8Attention(): Promise<Record<string, unknown>> {
  try {
    return await invoke<Record<string, unknown>>("get_e8_attention");
  } catch {
    return {};
  }
}

export type { FileEntry, ModelInfo };

export async function scanProjectFiles(_path?: string): Promise<FileEntry[]> {
  return [];
}

// ========== Stub API – IM Adapters ==========

export async function imListAdapters(): Promise<ImAdapterStatus[]> {
  return [];
}

export async function imGetAdapter(_id: string): Promise<ImAdapterConfig | null> {
  return null;
}

export async function imSaveAdapter(_name: string, _displayName: string, _botToken?: string | null, _phoneNumberId?: string | null, _accessToken?: string | null, _webhookUrl?: string | null, _verifyToken?: string | null): Promise<void> {
}

export async function imConnectAdapter(_id: string): Promise<void> {
}

export async function imDisconnectAdapter(_id: string): Promise<void> {
}

// ========== Stub API – Models ==========

export async function listModels(): Promise<ModelInfo[]> {
  return [];
}

// ========== Stub API – Moments / Feed ==========

export async function feedInsight(_id: string): Promise<string> {
  return "";
}

export async function feedRefresh(_tag?: string, _search?: string): Promise<FeedState> {
  return { items: [], tags: [], timelines: [] };
}

// ========== Stub API – Proxy Source ==========

export async function proxySourceStatus(): Promise<ProxySourceInfo[]> {
  return [];
}

export async function proxyConnectivity(): Promise<ProxyConnectivity | null> {
  return null;
}

export async function proxyTriggerFetch(_count?: number): Promise<number> {
  return 0;
}

// ========== Stub API – Remote Control ==========

export async function remoteStart(_port?: number): Promise<RemoteSessionInfo> {
  return { id: "", name: "", connected: false };
}

export async function remoteStop(_sessionId?: string): Promise<void> {
}

// ========== Sandbox API ==========

export interface SandboxJobRequest {
  command: string;
  args: string[];
  timeout_secs?: number;
  writable_paths?: string[];
  allow_network?: boolean;
  working_dir?: string;
}

export interface SandboxJobResponse {
  exit_code: number | null;
  stdout: string;
  stderr: string;
  duration_ms: number;
  timed_out: boolean;
  violation: string | null;
}

export async function sandboxStatus(): Promise<Record<string, unknown>> {
  try {
    return await invoke<Record<string, unknown>>("sandbox_status");
  } catch {
    return {};
  }
}

export async function sandboxExecute(req: SandboxJobRequest): Promise<SandboxJobResponse> {
  try {
    return await invoke<SandboxJobResponse>("sandbox_execute", { req });
  } catch (e) {
    return { exit_code: null, stdout: "", stderr: String(e), duration_ms: 0, timed_out: false, violation: String(e) };
  }
}

export async function sandboxKillAll(): Promise<void> {
  try {
    await invoke("sandbox_kill_all");
  } catch { /* command may not exist */ }
}

// ========== Stub API – Workspace ==========

export async function execCommand(_cmd: string): Promise<string> {
  return "";
}

export async function searchFiles(_query: string): Promise<FileEntry[]> {
  return [];
}
