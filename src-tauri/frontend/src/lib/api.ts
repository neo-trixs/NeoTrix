import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { getCurrent } from "@tauri-apps/plugin-deep-link";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import type { ProviderConfig, PermissionRequest, FileNode, DiffBlock, ProxySourceInfo, ProxyConnectivity, BrowserSession, ImageGenerationResult, ImageGenOptions, ScreenCapture, WindowInfo, FrontmostApp, RemoteHostConfig, RemoteExecutionResult, BackgroundTask, TaskRun, ReviewConfig, ReviewResult, ReviewIssue } from "../types";

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
  return invoke<FileNode[]>("read_dir_recursive", { path, max_depth: maxDepth ?? 3 });
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
  return invoke("respond_permission", { request_id: requestId, approved });
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

export interface KbGraphNode {
  id: string;
  node_type: string;
  title: string;
  summary: string | null;
  domain: string | null;
  confidence: number;
  importance: number;
}

export interface KbGraphEdge {
  id: string;
  source_id: string;
  target_id: string;
  relation_type: string;
  weight: number;
}

export interface KbGraphResponse {
  nodes: KbGraphNode[];
  edges: KbGraphEdge[];
}

export interface KbStatsResponse {
  total_nodes: number;
  total_edges: number;
  by_type: [string, number][];
}

export async function getKnowledgeGraph(): Promise<KbGraphResponse> {
  return invoke<KbGraphResponse>("get_knowledge_graph");
}

export async function getKnowledgeStats(): Promise<KbStatsResponse> {
  return invoke<KbStatsResponse>("get_knowledge_stats");
}

export interface KbNode {
  id: string;
  node_type: string;
  title: string;
  summary: string | null;
  content: string | null;
  url: string | null;
  domain: string | null;
  confidence: number;
  importance: number;
  metadata?: Record<string, unknown> | null;
}

export interface KbSearchResult {
  id: string;
  node_type: string;
  title: string;
  summary: string | null;
  content: string | null;
  url: string | null;
  domain: string | null;
  confidence: number;
  importance: number;
  created_at: number;
}

export async function kbSearch(query: string, limit?: number): Promise<KbSearchResult[]> {
  return invoke<KbSearchResult[]>("kb_search", { query, limit: limit ?? 10 });
}

export async function kbGetNode(id: string): Promise<KbNode | null> {
  return invoke<KbNode | null>("kb_get_node", { id });
}

export async function kbGetRelated(id: string, relationType?: string, limit?: number): Promise<KbSearchResult[]> {
  return invoke<KbSearchResult[]>("kb_get_related", { id, relationType: relationType ?? null, limit: limit ?? 10 });
}

export async function kbFeed(limit?: number, offset?: number, sort?: string): Promise<KbSearchResult[]> {
  return invoke<KbSearchResult[]>("kb_feed", { limit: limit ?? 50, offset: offset ?? 0, sort: sort ?? "recent" });
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

// ========== Project API ==========

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

export async function projectList(): Promise<Project[]> {
  return invoke<Project[]>("project_list");
}

export async function projectCreate(
  name: string,
  path: string,
  projectType?: string,
  description?: string,
  color?: string,
  icon?: string
): Promise<Project> {
  return invoke<Project>("project_create", { name, path, projectType, description, color, icon });
}

export async function projectGet(id: string): Promise<Project | null> {
  return invoke<Project | null>("project_get", { id });
}

export async function projectUpdate(
  id: string,
  updates: Partial<Pick<Project, "name" | "description" | "color" | "icon" | "pinned" | "archived">>
): Promise<Project> {
  return invoke<Project>("project_update", { id, ...updates });
}

export async function projectDelete(id: string): Promise<void> {
  return invoke("project_delete", { id });
}

export async function projectScanDirectory(path: string): Promise<Project> {
  return invoke<Project>("project_scan_directory", { path });
}

export async function projectChatList(projectId: string): Promise<ProjectChat[]> {
  return invoke<ProjectChat[]>("project_chat_list", { projectId });
}

export async function projectChatCreate(
  projectId: string,
  name: string,
  sessionId?: string
): Promise<ProjectChat> {
  return invoke<ProjectChat>("project_chat_create", { projectId, name, sessionId });
}

export async function projectChatUpdate(
  chatId: string,
  updates: Partial<Pick<ProjectChat, "name" | "pinned" | "archived" | "message_count">>
): Promise<ProjectChat> {
  return invoke<ProjectChat>("project_chat_update", { chatId, ...updates });
}

export async function projectChatDelete(chatId: string): Promise<void> {
  return invoke("project_chat_delete", { chatId });
}

export async function projectSourceList(projectId: string): Promise<ProjectSource[]> {
  return invoke<ProjectSource[]>("project_source_list", { projectId });
}

export async function projectSourceAdd(
  projectId: string,
  sourceType: string,
  path: string | undefined,
  url: string | undefined,
  name: string
): Promise<ProjectSource> {
  return invoke<ProjectSource>("project_source_add", { projectId, sourceType, path, url, name });
}

export async function projectSourceUpdate(
  sourceId: string,
  updates: Partial<Pick<ProjectSource, "enabled" | "name">>
): Promise<ProjectSource> {
  return invoke<ProjectSource>("project_source_update", { sourceId, ...updates });
}

export async function projectSourceDelete(sourceId: string): Promise<void> {
  return invoke("project_source_delete", { sourceId });
}

export async function projectInstructionList(projectId: string): Promise<ProjectInstruction[]> {
  return invoke<ProjectInstruction[]>("project_instruction_list", { projectId });
}

export async function projectInstructionAdd(projectId: string, content: string): Promise<ProjectInstruction> {
  return invoke<ProjectInstruction>("project_instruction_add", { projectId, content });
}

export async function projectInstructionUpdate(
  instructionId: string,
  updates: Partial<Pick<ProjectInstruction, "content" | "enabled">>
): Promise<ProjectInstruction> {
  return invoke<ProjectInstruction>("project_instruction_update", { instructionId, ...updates });
}

export async function projectInstructionDelete(instructionId: string): Promise<void> {
  return invoke("project_instruction_delete", { instructionId });
}

// ========== Provider Status API ==========

export interface ProviderStateInfo {
  name: string;
  available: boolean;
  circuit_state: string;
  success_rate: string;
  total_calls: number;
  total_errors: number;
  is_free: boolean;
  composite_score: string;
}

export async function getProviderStatus(): Promise<ProviderStateInfo[]> {
  try {
    return await invoke<ProviderStateInfo[]>("provider_status");
  } catch {
    return [];
  }
}

// ========== Proxy API ==========

import type { ProxyStatus, ProxyNodeInfo, ProxyConfigData } from "../types";

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

export async function proxySourceStatus(): Promise<ProxySourceInfo[]> {
  try {
    return await invoke<ProxySourceInfo[]>("proxy_source_status");
  } catch {
    return [];
  }
}

export async function proxyConnectivity(): Promise<ProxyConnectivity> {
  try {
    return await invoke<ProxyConnectivity>("proxy_connectivity");
  } catch {
    return { active_mode: "auto", direct_reachable: true, direct_latency_ms: null, proxy_healthy_count: 0, proxy_total_count: 0, proxy_avg_latency_ms: null };
  }
}

export async function proxyTriggerFetch(max_count?: number): Promise<number> {
  try {
    return await invoke<number>("proxy_trigger_fetch", { maxCount: max_count ?? 200 });
  } catch {
    return 0;
  }
}

export async function proxySubList(): Promise<string[]> {
  try {
    return await invoke<string[]>("proxy_sub_list");
  } catch {
    return [];
  }
}

export async function proxySubAdd(url: string): Promise<string> {
  return invoke<string>("proxy_sub_add", { url });
}

export async function proxySubRemove(url: string): Promise<string> {
  return invoke<string>("proxy_sub_remove", { url });
}

export async function proxyPoolNodes(): Promise<ProxyNodeInfo[]> {
  try {
    return await invoke<ProxyNodeInfo[]>("proxy_pool_nodes");
  } catch {
    return [];
  }
}

export async function proxyConfigGet(): Promise<ProxyConfigData> {
  try {
    return await invoke<ProxyConfigData>("proxy_config_get");
  } catch {
    return {
      local_port: 11080, socks_port: 9050, min_nodes: 5,
      health_check_interval_secs: 60, selection_strategy: "auto",
      system_proxy_enabled: true, direct_timeout_secs: 3,
    };
  }
}

export async function proxyConfigSet(config: Partial<ProxyConfigData>): Promise<string> {
  try {
    return await invoke<string>("proxy_config_set", { config });
  } catch {
    return "error";
  }
}

// ========== Browser / X Feed API ==========

export interface XAutoScrollStatus {
  session_active: boolean;
  running: boolean;
  tweet_count: number;
  absorbed: number;
  negentropy_avg: number;
  current_url: string;
}

export interface XHumanProfile {
  scroll_speed: number;
  pause_range: [number, number];
  scroll_variance: number;
  mouse_trail: boolean;
  interaction_rate: number;
  user_agent: string;
}

export interface XAbsorptionEvent {
  count: number;
  avg_negentropy: number;
}

export interface BrowserState {
  is_open: boolean;
  url: string;
  title: string;
}

export interface SearchResultItem {
  url: string;
  title: string;
  snippet: string;
}

export interface WebAppAgentInfo {
  id: string;
  name: string;
  url_pattern: string;
  is_active: boolean;
  actions: { id: string; label: string }[];
}

export interface ToolResult {
  success: boolean;
  output: string;
  duration_ms: number;
}

export async function browserXStatus(): Promise<XAutoScrollStatus> {
  return invoke<XAutoScrollStatus>("browser_x_status");
}

export async function browserXStartSession(): Promise<string> {
  return invoke<string>("browser_x_start_session");
}

export async function browserXHumanProfile(): Promise<XHumanProfile> {
  return invoke<XHumanProfile>("browser_x_human_profile");
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

export async function browserOpen(url: string): Promise<BrowserState> {
  return invoke<BrowserState>("browser_open", { url });
}

export async function browserClose(): Promise<void> {
  return invoke<void>("browser_close");
}

export async function browserBack(): Promise<void> {
  return invoke<void>("browser_back");
}

export async function browserForward(): Promise<void> {
  return invoke<void>("browser_forward");
}

export async function browserReload(): Promise<void> {
  return invoke<void>("browser_reload");
}

export async function browserAgentDetect(url: string, title: string): Promise<WebAppAgentInfo | null> {
  try {
    return await invoke<WebAppAgentInfo | null>("browser_agent_detect", { url, title });
  } catch {
    return null;
  }
}

export async function browserExtractContent(url: string): Promise<{ title: string; summary: string }> {
  return invoke<{ title: string; summary: string }>("browser_extract_content", { url });
}

export async function browserAgentList(): Promise<WebAppAgentInfo[]> {
  try {
    return await invoke<WebAppAgentInfo[]>("browser_agent_list");
  } catch {
    return [];
  }
}

export async function browserAgentExecute(agentId: string, actionId: string): Promise<string> {
  return invoke<string>("browser_agent_execute", { agentId, actionId });
}

export async function toolSearch(query: string, count?: number): Promise<SearchResultItem[]> {
  try {
    return await invoke<SearchResultItem[]>("tool_search", { query, count: count ?? 8 });
  } catch {
    return [];
  }
}

export async function toolExecute(tool: string, args: Record<string, unknown>): Promise<ToolResult> {
  return invoke<ToolResult>("tool_execute", { tool, args });
}

// ========== Chat API ==========

export async function saveApiKey(key: string): Promise<void> {
  await invoke("save_api_key", { key });
}

export async function hasApiKey(): Promise<boolean> {
  return invoke<boolean>("has_api_key");
}

export async function deleteApiKey(): Promise<void> {
  await invoke("delete_api_key");
}

export async function sendMessage(
  conversationId: string,
  content: string,
  model?: string
): Promise<string> {
  return invoke<string>("send_message", { conversationId, content, model });
}

export async function stopGeneration(): Promise<void> {
  await invoke("stop_generation");
}

// ========== Image Generation API ==========
export function imageGenerate(prompt: string, options: ImageGenOptions): Promise<{success: boolean; result_url?: string}> {
  return invoke("image_generate", { prompt, options });
}

// ========== Computer Use API ==========
export function computerCaptureScreen(): Promise<ScreenCapture> {
  return invoke("capture_screen");
}

export function computerGetWindowList(): Promise<WindowInfo[]> {
  return invoke("get_window_list");
}

export function computerGetFrontmostApp(): Promise<FrontmostApp> {
  return invoke("get_frontmost_app");
}

export function computerSwitchApp(appName: string): Promise<{ok: boolean}> {
  return invoke("switch_app", { app_name: appName });
}

export function computerExecuteRemote(appName: string, command: string): Promise<string> {
  return invoke("execute_remote", { app_name: appName, command });
}

export function computerReadClipboard(): Promise<{text: string}> {
  return invoke("read_clipboard");
}

export function computerWriteClipboard(text: string): Promise<{ok: boolean}> {
  return invoke("write_clipboard", { text });
}

// ========== Remote Devbox API ==========
export function remoteListHosts(): Promise<RemoteHostConfig[]> {
  return invoke("list_remote_hosts");
}

export function remoteAddHost(config: RemoteHostConfig): Promise<RemoteHostConfig> {
  return invoke("add_remote_host", { name: config.name, host: config.host, port: config.port, user: config.user, auth_method: config.auth_method, key_path: config.key_path });
}

export function remoteRemoveHost(id: string): Promise<void> {
  return invoke("remove_remote_host", { id });
}

export function remoteTestConnection(id: string): Promise<string> {
  return invoke("test_remote_connection", { id });
}

export function remoteExec(id: string, command: string): Promise<string> {
  return invoke("execute_remote", { id, command });
}

// ========== Background Routines API ==========
export function backgroundListTasks(): Promise<BackgroundTask[]> {
  return invoke("list_background_tasks");
}

export function backgroundCreateTask(name: string, prompt: string, schedule: string): Promise<BackgroundTask> {
  return invoke("create_background_task", { name, prompt, schedule });
}

export function backgroundPauseTask(id: string): Promise<void> {
  return invoke("pause_background_task", { id });
}

export function backgroundResumeTask(id: string): Promise<void> {
  return invoke("resume_background_task", { id });
}

export function backgroundDeleteTask(id: string): Promise<void> {
  return invoke("delete_background_task", { id });
}

export function backgroundRunNow(id: string): Promise<string> {
  return invoke("run_background_task_now", { id });
}

export function backgroundGetLog(id: string): Promise<TaskRun[]> {
  return invoke("get_background_task_log", { id });
}

// ========== Ultra Review API ==========
export function ultraReview(config: ReviewConfig): Promise<ReviewResult> {
  return invoke("ultra_review", { config });
}

