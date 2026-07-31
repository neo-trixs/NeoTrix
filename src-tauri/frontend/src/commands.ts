import { invoke } from "@tauri-apps/api/core";

// ══════════════════════════════════════════════════════════════════════════════
// Buddy
// ══════════════════════════════════════════════════════════════════════════════

export interface BuddyState {
  mood: string;
  energy: number;
  xp: number;
  level: number;
  name: string;
  last_interaction: string;
  active: boolean;
}

export interface BuddyAction {
  action: string;
  timestamp: string;
  description: string;
}

export interface Achievement {
  id: string;
  name: string;
  description: string;
  unlocked_at: string;
}

export function buddyStatus(): Promise<BuddyState> {
  return invoke("buddy_status");
}

export function buddyPet(name: string): Promise<string> {
  return invoke("buddy_pet", { name });
}

export function buddyFeed(food: string): Promise<string> {
  return invoke("buddy_feed", { food });
}

export function buddyTrain(task: string): Promise<string> {
  return invoke("buddy_train", { task });
}

export function buddyRest(): Promise<string> {
  return invoke("buddy_rest");
}

export function buddyAchievements(): Promise<Achievement[]> {
  return invoke("buddy_achievements");
}

export function buddyIdleTick(): Promise<BuddyState> {
  return invoke("buddy_idle_tick");
}

export function buddyLog(count: number): Promise<BuddyAction[]> {
  return invoke("buddy_log", { count });
}

// ══════════════════════════════════════════════════════════════════════════════
// Daemon
// ══════════════════════════════════════════════════════════════════════════════

export interface DaemonConfig {
  enabled: boolean;
  watch_path: string;
  interval_secs: number;
  auto_fix: boolean;
  max_workers: number;
}

export interface DaemonStatus {
  running: boolean;
  pid: number;
  uptime_secs: number;
  files_watched: number;
  auto_fixes_applied: number;
  last_cycle: string;
}

export interface DaemonEvent {
  timestamp: string;
  kind: string;
  path: string;
  message: string;
}

export function daemonStart(config: DaemonConfig): Promise<string> {
  return invoke("daemon_start", { config });
}

export function daemonStop(): Promise<void> {
  return invoke("daemon_stop");
}

export function daemonStatus(): Promise<DaemonStatus> {
  return invoke("daemon_status");
}

export function daemonLog(count: number): Promise<DaemonEvent[]> {
  return invoke("daemon_log", { count });
}

export function daemonAutoFix(path: string): Promise<string> {
  return invoke("daemon_auto_fix", { path });
}

// ══════════════════════════════════════════════════════════════════════════════
// Dream
// ══════════════════════════════════════════════════════════════════════════════

export interface DreamConfig {
  enabled: boolean;
  interval_minutes: number;
  auto_run: boolean;
}

export interface DreamStatus {
  phase: string;
  progress: number;
  last_run: string;
  total_consolidations: number;
  memories_harvested: number;
  contradictions_removed: number;
}

export interface DreamMemoryEntry {
  id: string;
  content: string;
  source: string;
  timestamp: string;
  confidence: number;
  tags: string[];
}

export function dreamStart(config: DreamConfig): Promise<string> {
  return invoke("dream_start", { config });
}

export function dreamStop(): Promise<void> {
  return invoke("dream_stop");
}

export function dreamStatus(): Promise<DreamStatus> {
  return invoke("dream_status");
}

export function dreamEntries(): Promise<DreamMemoryEntry[]> {
  return invoke("dream_entries");
}

export function dreamConsolidateNow(): Promise<string> {
  return invoke("dream_consolidate_now");
}

// ══════════════════════════════════════════════════════════════════════════════
// Gate
// ══════════════════════════════════════════════════════════════════════════════

export interface GateConfig {
  auto_check: boolean;
  strict_mode: boolean;
  checks_enabled: string[];
}

export interface GateCheck {
  name: string;
  status: string;
  message: string;
  details: string;
}

export interface GateResult {
  overall: string;
  checks: GateCheck[];
  score: number;
}

export interface GatePolicy {
  block_on_fail: boolean;
  require_review: boolean;
  auto_fix_trivial: boolean;
}

export function gateRunCheck(path: string): Promise<GateResult> {
  return invoke("gate_run_check", { path });
}

export function gateSetConfig(config: GateConfig): Promise<void> {
  return invoke("gate_set_config", { config });
}

export function gateGetConfig(): Promise<GateConfig> {
  return invoke("gate_get_config");
}

export function gateSetPolicy(policy: GatePolicy): Promise<void> {
  return invoke("gate_set_policy", { policy });
}

export function gateGetPolicy(): Promise<GatePolicy> {
  return invoke("gate_get_policy");
}

export function gateApprove(reason: string): Promise<string> {
  return invoke("gate_approve", { reason });
}

export function gateAuditLog(count: number): Promise<Record<string, unknown>[]> {
  return invoke("gate_audit_log", { count });
}

// ══════════════════════════════════════════════════════════════════════════════
// Undercover
// ══════════════════════════════════════════════════════════════════════════════

export interface IdentityProfile {
  name: string;
  email: string;
  use_for_commits: boolean;
  strip_co_auth: boolean;
  custom_prefix: string;
}

export interface CommitTrace {
  commit_hash: string;
  repo: string;
  original_author: string;
  committed_as: string;
  timestamp: string;
  trace_hash: string;
}

export function undercoverStatus(): Promise<Record<string, unknown>> {
  return invoke("undercover_status");
}

export function undercoverSetProfile(name: string, email: string, stripCoAuth: boolean): Promise<void> {
  return invoke("undercover_set_profile", { name, email, stripCoAuth });
}

export function undercoverGetProfiles(): Promise<IdentityProfile[]> {
  return invoke("undercover_get_profiles");
}

export function undercoverActivateProfile(name: string): Promise<void> {
  return invoke("undercover_activate_profile", { name });
}

export function undercoverStripMetadata(path: string): Promise<string> {
  return invoke("undercover_strip_metadata", { path });
}

export function undercoverCommitLog(count: number): Promise<CommitTrace[]> {
  return invoke("undercover_commit_log", { count });
}

export function undercoverVerifyAnonymity(path: string): Promise<Record<string, unknown>> {
  return invoke("undercover_verify_anonymity", { path });
}

// ══════════════════════════════════════════════════════════════════════════════
// Summary
// ══════════════════════════════════════════════════════════════════════════════

export interface SummarySession {
  id: string;
  start_time: string;
  status: string;
  task_count: number;
  plan_count: number;
  artifact_count: number;
  source_count: number;
}

export interface SessionPlan {
  id: string;
  title: string;
  status: string;
  progress_pct: number;
  steps: string[];
}

export interface SessionArtifact {
  id: string;
  name: string;
  kind: string;
  path: string;
  size: number;
}

export interface SessionSource {
  id: string;
  title: string;
  url: string;
  relevance: number;
  accessed_at: string;
}

export function summaryActive(): Promise<SummarySession> {
  return invoke("summary_active");
}

export function summaryStart(): Promise<string> {
  return invoke("summary_start");
}

export function summaryPause(): Promise<void> {
  return invoke("summary_pause");
}

export function summaryResume(): Promise<void> {
  return invoke("summary_resume");
}

export function summaryPlans(): Promise<SessionPlan[]> {
  return invoke("summary_plans");
}

export function summaryArtifacts(): Promise<SessionArtifact[]> {
  return invoke("summary_artifacts");
}

export function summarySources(): Promise<SessionSource[]> {
  return invoke("summary_sources");
}

export function summaryAddArtifact(name: string, kind: string, path: string): Promise<void> {
  return invoke("summary_add_artifact", { name, kind, path });
}

export function summaryAddSource(title: string, url: string, relevance: number): Promise<void> {
  return invoke("summary_add_source", { title, url, relevance });
}

export function summaryAddPlan(title: string, steps: string[]): Promise<string> {
  return invoke("summary_add_plan", { title, steps });
}

// ══════════════════════════════════════════════════════════════════════════════
// MCP Host
// ══════════════════════════════════════════════════════════════════════════════

export interface McpHostConfig {
  enabled: boolean;
  host: string;
  port: number;
  max_connections: number;
  auth_token?: string;
}

export interface McpHostEndpoint {
  id: string;
  name: string;
  description: string;
  endpoint_type: string;
  parameters: string[];
  enabled: boolean;
}

export interface McpHostSession {
  client_id: string;
  connected_at: string;
  tool_calls: number;
  status: string;
}

export interface McpHostStatus {
  running: boolean;
  port: number;
  uptime_secs: number;
  active_sessions: number;
  total_endpoints: number;
  total_calls: number;
}

export function mcpHostStart(config: McpHostConfig): Promise<string> {
  return invoke("mcp_host_start", { config });
}

export function mcpHostStop(): Promise<void> {
  return invoke("mcp_host_stop");
}

export function mcpHostStatus(): Promise<McpHostStatus> {
  return invoke("mcp_host_status");
}

export function mcpHostListEndpoints(): Promise<McpHostEndpoint[]> {
  return invoke("mcp_host_list_endpoints");
}

export function mcpHostRegisterEndpoint(name: string, description: string, params: string[]): Promise<void> {
  return invoke("mcp_host_register_endpoint", { name, description, params });
}

export function mcpHostUnregisterEndpoint(name: string): Promise<void> {
  return invoke("mcp_host_unregister_endpoint", { name });
}

export function mcpHostSessions(): Promise<McpHostSession[]> {
  return invoke("mcp_host_sessions");
}

export function mcpHostLog(count: number): Promise<Record<string, unknown>[]> {
  return invoke("mcp_host_log", { count });
}

// ══════════════════════════════════════════════════════════════════════════════
// Plugin
// ══════════════════════════════════════════════════════════════════════════════

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  homepage: string;
  entry_points: string[];
  requires: string[];
  permissions: string[];
}

export interface PluginStatus {
  id: string;
  name: string;
  version: string;
  enabled: boolean;
  loaded: boolean;
  load_time_ms: number;
  error?: string;
}

export interface PluginEvent {
  timestamp: string;
  kind: string;
  plugin_id: string;
  message: string;
}

export interface PluginConfig {
  plugins_dir: string;
  auto_load: boolean;
  allow_unverified: boolean;
  max_plugins: number;
}

export function pluginList(): Promise<PluginStatus[]> {
  return invoke("plugin_list");
}

export function pluginInstall(path: string): Promise<PluginStatus> {
  return invoke("plugin_install", { path });
}

export function pluginUninstall(id: string): Promise<void> {
  return invoke("plugin_uninstall", { id });
}

export function pluginEnable(id: string): Promise<void> {
  return invoke("plugin_enable", { id });
}

export function pluginDisable(id: string): Promise<void> {
  return invoke("plugin_disable", { id });
}

export function pluginGet(id: string): Promise<PluginStatus> {
  return invoke("plugin_get", { id });
}

export function pluginConfig(): Promise<PluginConfig> {
  return invoke("plugin_config");
}

export function pluginSetConfig(config: PluginConfig): Promise<void> {
  return invoke("plugin_set_config", { config });
}

export function pluginEventLog(count: number): Promise<PluginEvent[]> {
  return invoke("plugin_event_log", { count });
}

export function pluginRun(id: string, entryPoint: string, args?: string[]): Promise<string> {
  return invoke("plugin_run", { id, entryPoint, args });
}

// ══════════════════════════════════════════════════════════════════════════════
// Web Search + Agent SDK
// ══════════════════════════════════════════════════════════════════════════════

export interface WebSearchResult {
  title: string;
  url: string;
  snippet: string;
  relevance: number;
}

export interface SearchConfig {
  max_results: number;
  timeout_secs: number;
  safe_search: boolean;
}

export interface AgentSdkBlueprint {
  id: string;
  name: string;
  description: string;
  tools_allowed: string[];
  max_steps: number;
  model: string;
  system_prompt: string;
}

export interface AgentSdkInstance {
  id: string;
  blueprint_id: string;
  status: string;
  progress_pct: number;
  current_step: string;
  started_at: string;
}

export interface AgentSdkResult {
  instance_id: string;
  status: string;
  output: string;
  steps_taken: number;
  duration_ms: number;
  error?: string;
}

export function webSearch(query: string, maxResults?: number): Promise<WebSearchResult[]> {
  return invoke<WebSearchResult[]>("web_search", { query, maxResults }).then((r) => {
    invoke("insights_record_event", {
      eventType: "search_performed",
      details: `web_search: ${query.slice(0, 80)}`,
      sessionId: null,
      project: null,
      durationMs: null,
    }).catch(() => {});
    return r;
  });
}

export function webSearchConfig(): Promise<SearchConfig> {
  return invoke("web_search_config");
}

export function webSearchSetConfig(config: SearchConfig): Promise<void> {
  return invoke("web_search_set_config", { config });
}

export function agentSdkCreateBlueprint(name: string, description: string, tools: string[], maxSteps: number, systemPrompt: string): Promise<string> {
  return invoke("agent_sdk_create_blueprint", { name, description, tools, maxSteps, systemPrompt });
}

export function agentSdkListBlueprints(): Promise<AgentSdkBlueprint[]> {
  return invoke("agent_sdk_list_blueprints");
}

export function agentSdkGetBlueprint(id: string): Promise<AgentSdkBlueprint> {
  return invoke("agent_sdk_get_blueprint", { id });
}

export function agentSdkDeleteBlueprint(id: string): Promise<void> {
  return invoke("agent_sdk_delete_blueprint", { id });
}

export function agentSdkRun(blueprintId: string, input: string): Promise<string> {
  return invoke("agent_sdk_run", { blueprintId, input });
}

export function agentSdkListInstances(): Promise<AgentSdkInstance[]> {
  return invoke("agent_sdk_list_instances");
}

export function agentSdkGetResult(instanceId: string): Promise<AgentSdkResult> {
  return invoke("agent_sdk_get_result", { instanceId });
}

// ══════════════════════════════════════════════════════════════════════════════
// Channels + Slack
// ══════════════════════════════════════════════════════════════════════════════

export interface ChannelConfig {
  id: string;
  name: string;
  channel_type: string;
  webhook_url: string;
  enabled: boolean;
  secret?: string;
}

export interface ChannelMessage {
  id: string;
  channel_id: string;
  from: string;
  content: string;
  timestamp: string;
  kind: string;
}

export interface ChannelSession {
  channel_id: string;
  session_id: string;
  linked_at: string;
  active: boolean;
}

export function channelsList(): Promise<ChannelConfig[]> {
  return invoke("channels_list");
}

export function channelsAdd(type_: string, name: string, webhookUrl: string): Promise<string> {
  return invoke("channels_add", { type: type_, name, webhookUrl });
}

export function channelsRemove(id: string): Promise<void> {
  return invoke("channels_remove", { id });
}

export function channelsEnable(id: string): Promise<void> {
  return invoke("channels_enable", { id });
}

export function channelsDisable(id: string): Promise<void> {
  return invoke("channels_disable", { id });
}

export function channelsSend(channelId: string, content: string): Promise<void> {
  return invoke("channels_send", { channelId, content });
}

export function channelsReceive(channelId: string): Promise<ChannelMessage[]> {
  return invoke("channels_receive", { channelId });
}

export function channelsLinkSession(channelId: string, sessionId: string): Promise<void> {
  return invoke("channels_link_session", { channelId, sessionId });
}

export function channelsUnlinkSession(channelId: string): Promise<void> {
  return invoke("channels_unlink_session", { channelId });
}

export function channelsHistory(count: number): Promise<ChannelMessage[]> {
  return invoke("channels_history", { count });
}

export interface SlackConfig {
  workspace: string;
  channel: string;
  token: string;
}

export function slackConfig(): Promise<Record<string, unknown>> {
  return invoke("slack_config");
}

export function slackConfigure(workspace: string, channel: string, token: string): Promise<void> {
  return invoke("slack_configure", { workspace, channel, token });
}

export function slackSend(message: string): Promise<void> {
  return invoke("slack_send", { message });
}

export function slackStatus(): Promise<Record<string, unknown>> {
  return invoke("slack_status");
}

// ══════════════════════════════════════════════════════════════════════════════
// Preview + Chrome Debug
// ══════════════════════════════════════════════════════════════════════════════

export interface PreviewConfig {
  enabled: boolean;
  port: number;
  default_width: number;
  default_height: number;
  allow_navigation: boolean;
}

export interface PreviewSession {
  id: string;
  url: string;
  title: string;
  width: number;
  height: number;
  status: string;
  started_at: string;
}

export interface PreviewScreenshot {
  session_id: string;
  path: string;
  width: number;
  height: number;
  taken_at: string;
}

export interface ChromeDebugTarget {
  id: string;
  title: string;
  url: string;
  description: string;
  favicon_url: string;
  debug_url: string;
}

export interface ChromeDebugConsoleEntry {
  level: string;
  message: string;
  timestamp: string;
  source: string;
}

export interface ChromeDebugConfig {
  enabled: boolean;
  host: string;
  port: number;
  auto_connect: boolean;
}

export function previewStart(url: string, width?: number, height?: number): Promise<string> {
  return invoke("preview_start", { url, width, height });
}

export function previewStop(sessionId: string): Promise<void> {
  return invoke("preview_stop", { sessionId });
}

export function previewList(): Promise<PreviewSession[]> {
  return invoke("preview_list");
}

export function previewNavigate(sessionId: string, url: string): Promise<void> {
  return invoke("preview_navigate", { sessionId, url });
}

export function previewReload(sessionId: string): Promise<void> {
  return invoke("preview_reload", { sessionId });
}

export function previewScreenshot(sessionId: string): Promise<PreviewScreenshot> {
  return invoke("preview_screenshot", { sessionId });
}

export function previewConfig(): Promise<PreviewConfig> {
  return invoke("preview_config");
}

export function previewSetConfig(config: PreviewConfig): Promise<void> {
  return invoke("preview_set_config", { config });
}

export function chromeDebugConnect(host?: string, port?: number): Promise<string> {
  return invoke("chrome_debug_connect", { host, port });
}

export function chromeDebugDisconnect(): Promise<void> {
  return invoke("chrome_debug_disconnect");
}

export function chromeDebugStatus(): Promise<ChromeDebugConfig> {
  return invoke("chrome_debug_status");
}

export function chromeDebugListTargets(): Promise<ChromeDebugTarget[]> {
  return invoke("chrome_debug_list_targets");
}

export function chromeDebugNavigate(url: string): Promise<void> {
  return invoke("chrome_debug_navigate", { url });
}

export function chromeDebugReload(): Promise<void> {
  return invoke("chrome_debug_reload");
}

export function chromeDebugEvaluate(expression: string): Promise<string> {
  return invoke("chrome_debug_evaluate", { expression });
}

export function chromeDebugGetConsoleLogs(): Promise<ChromeDebugConsoleEntry[]> {
  return invoke("chrome_debug_get_console_logs");
}

export function chromeDebugClearConsoleLogs(): Promise<void> {
  return invoke("chrome_debug_clear_console_logs");
}

export function chromeDebugCaptureScreenshot(): Promise<string> {
  return invoke("chrome_debug_capture_screenshot");
}

// ══════════════════════════════════════════════════════════════════════════════
// Teleport + Agent Teams
// ══════════════════════════════════════════════════════════════════════════════

export interface TeleportSession {
  id: string;
  source: string;
  destination: string;
  session_data: string;
  created_at: string;
  expires_at: string;
  claimed: boolean;
}

export interface TeleportCode {
  code: string;
  session_id: string;
  expires_at: string;
  used: boolean;
}

export interface TeleportConfig {
  enabled: boolean;
  max_sessions: number;
  default_ttl_secs: number;
}

export interface AgentTeam {
  id: string;
  name: string;
  description: string;
  strategy: string;
  created_at: string;
  status: string;
}

export interface AgentTeamMember {
  id: string;
  team_id: string;
  role: string;
  task: string;
  status: string;
  result?: string;
}

export interface AgentTeamMessage {
  from: string;
  to: string;
  content: string;
  timestamp: string;
  kind: string;
}

export interface AgentTeamResult {
  team_id: string;
  overall_status: string;
  member_count: number;
  completed_count: number;
  failed_count: number;
  duration_ms: number;
}

export function teleportCreate(source: string, sessionData: string): Promise<TeleportCode> {
  return invoke("teleport_create", { source, sessionData });
}

export function teleportClaim(code: string, destination: string): Promise<TeleportSession> {
  return invoke("teleport_claim", { code, destination });
}

export function teleportList(): Promise<TeleportSession[]> {
  return invoke("teleport_list");
}

export function teleportConfig(): Promise<TeleportConfig> {
  return invoke("teleport_config");
}

export function teleportSetConfig(config: TeleportConfig): Promise<void> {
  return invoke("teleport_set_config", { config });
}

export function teleportRevoke(sessionId: string): Promise<void> {
  return invoke("teleport_revoke", { sessionId });
}

export function agentTeamCreate(name: string, description: string, strategy: string): Promise<string> {
  return invoke("agent_team_create", { name, description, strategy });
}

export function agentTeamList(): Promise<AgentTeam[]> {
  return invoke("agent_team_list");
}

export function agentTeamGet(id: string): Promise<Record<string, unknown>> {
  return invoke("agent_team_get", { id });
}

export function agentTeamAddMember(teamId: string, role: string, task: string): Promise<string> {
  return invoke("agent_team_add_member", { teamId, role, task });
}

export function agentTeamRemoveMember(teamId: string, memberId: string): Promise<void> {
  return invoke("agent_team_remove_member", { teamId, memberId });
}

export function agentTeamStart(teamId: string): Promise<void> {
  return invoke("agent_team_start", { teamId });
}

export function agentTeamCompleteMember(memberId: string, result: string): Promise<void> {
  return invoke("agent_team_complete_member", { memberId, result });
}

export function agentTeamFailMember(memberId: string, error: string): Promise<void> {
  return invoke("agent_team_fail_member", { memberId, error });
}

export function agentTeamStatus(teamId: string): Promise<Record<string, unknown>> {
  return invoke("agent_team_status", { teamId });
}

export function agentTeamMessages(teamId: string): Promise<AgentTeamMessage[]> {
  return invoke("agent_team_messages", { teamId });
}

export function agentTeamSendMessage(teamId: string, to: string, content: string, kind: string): Promise<void> {
  return invoke("agent_team_send_message", { teamId, to, content, kind });
}

export function agentTeamResult(teamId: string): Promise<AgentTeamResult> {
  return invoke("agent_team_result", { teamId });
}

// ══════════════════════════════════════════════════════════════════════════════
// Workflow
// ══════════════════════════════════════════════════════════════════════════════

export interface Workflow {
  id: string;
  name: string;
  description: string;
  version: number;
  steps: WorkflowStep[];
  created_at: string;
  updated_at: string;
  tags: string[];
}

export interface WorkflowStep {
  id: string;
  kind: string;
  name: string;
  params: Record<string, unknown>;
  depends_on: string[];
  timeout_secs: number;
  retry_count: number;
}

export interface WorkflowRun {
  id: string;
  workflow_id: string;
  status: string;
  current_step: string;
  progress_pct: number;
  started_at: string;
  completed_at: string | null;
  error?: string;
}

export interface WorkflowRunStep {
  run_id: string;
  step_id: string;
  status: string;
  started_at: string;
  duration_ms: number;
  output?: string;
  error?: string;
}

export interface WorkflowSchedule {
  id: string;
  workflow_id: string;
  trigger: string;
  cron_expr?: string;
  enabled: boolean;
}

export function workflowCreate(name: string, description: string, stepsJson: string): Promise<string> {
  return invoke("workflow_create", { name, description, stepsJson });
}

export function workflowList(): Promise<Workflow[]> {
  return invoke("workflow_list");
}

export function workflowGet(id: string): Promise<Workflow> {
  return invoke("workflow_get", { id });
}

export function workflowUpdate(id: string, name?: string, description?: string, stepsJson?: string): Promise<void> {
  return invoke("workflow_update", { id, name, description, stepsJson });
}

export function workflowDelete(id: string): Promise<void> {
  return invoke("workflow_delete", { id });
}

export function workflowRun(workflowId: string): Promise<string> {
  return invoke("workflow_run", { workflowId });
}

export function workflowRunStatus(runId: string): Promise<WorkflowRun> {
  return invoke("workflow_run_status", { runId });
}

export function workflowRunList(workflowId: string): Promise<WorkflowRun[]> {
  return invoke("workflow_run_list", { workflowId });
}

export function workflowRunSteps(runId: string): Promise<WorkflowRunStep[]> {
  return invoke("workflow_run_steps", { runId });
}

export function workflowRunCancel(runId: string): Promise<void> {
  return invoke("workflow_run_cancel", { runId });
}

export function workflowScheduleCreate(workflowId: string, trigger: string, cronExpr?: string): Promise<string> {
  return invoke("workflow_schedule_create", { workflowId, trigger, cronExpr });
}

export function workflowScheduleList(): Promise<WorkflowSchedule[]> {
  return invoke("workflow_schedule_list");
}

export function workflowScheduleDelete(scheduleId: string): Promise<void> {
  return invoke("workflow_schedule_delete", { scheduleId });
}

export function workflowImportFromJson(json: string): Promise<string> {
  return invoke("workflow_import_from_json", { json });
}

// ══════════════════════════════════════════════════════════════════════════════
// Enterprise + API
// ══════════════════════════════════════════════════════════════════════════════

export interface EnterprisePolicy {
  id: string;
  key: string;
  value: string;
  description: string;
  enforced: boolean;
  scope: string;
}

export interface EnterpriseAuditEntry {
  timestamp: string;
  action: string;
  actor: string;
  detail: string;
}

export interface EnterpriseLicense {
  id: string;
  key: string;
  status: string;
  expires_at: string;
  seats_total: number;
  seats_used: number;
  features: string[];
}

export interface ApiEndpoint {
  id: string;
  name: string;
  url: string;
  method: string;
  headers: Record<string, string>;
  last_call: string | null;
  last_status: number | null;
  enabled: boolean;
}

export interface RealApiConfig {
  endpoints: ApiEndpoint[];
  default_timeout_secs: number;
  retry_count: number;
  verify_ssl: boolean;
}

export function enterpriseStatus(): Promise<Record<string, unknown>> {
  return invoke("enterprise_status");
}

export function enterpriseListPolicies(): Promise<EnterprisePolicy[]> {
  return invoke("enterprise_list_policies");
}

export function enterpriseSetPolicy(key: string, value: string, description: string, enforced: boolean): Promise<void> {
  return invoke("enterprise_set_policy", { key, value, description, enforced });
}

export function enterpriseDeletePolicy(key: string): Promise<void> {
  return invoke("enterprise_delete_policy", { key });
}

export function enterpriseAuditLog(count: number): Promise<EnterpriseAuditEntry[]> {
  return invoke("enterprise_audit_log", { count });
}

export function enterpriseAuditLogAction(action: string, actor: string, detail: string): Promise<void> {
  return invoke("enterprise_audit_log_action", { action, actor, detail });
}

export function enterpriseComplianceCheck(): Promise<Record<string, unknown>> {
  return invoke("enterprise_compliance_check");
}

export function enterpriseLicenseInfo(): Promise<EnterpriseLicense> {
  return invoke("enterprise_license_info");
}

export function apiRegister(name: string, url: string, method: string): Promise<string> {
  return invoke("api_register", { name, url, method });
}

export function apiList(): Promise<ApiEndpoint[]> {
  return invoke("api_list");
}

export function apiTest(id: string): Promise<Record<string, unknown>> {
  return invoke("api_test", { id });
}

export function apiDelete(id: string): Promise<void> {
  return invoke("api_delete", { id });
}

export function apiConfig(): Promise<RealApiConfig> {
  return invoke("api_config");
}

export function apiSetConfig(timeout?: number, retryCount?: number, verifySsl?: boolean): Promise<void> {
  return invoke("api_set_config", { timeout, retryCount, verifySsl });
}

export function apiCall(name: string, body?: string): Promise<Record<string, unknown>> {
  return invoke("api_call", { name, body });
}

// ══════════════════════════════════════════════════════════════════════════════
// Agent View
// ══════════════════════════════════════════════════════════════════════════════

export interface AgentViewSession {
  id: string;
  name: string;
  surface: string;
  status: string;
  current_action: string;
  progress_pct: number;
  started_at: number;
  last_active_at: number;
  cpu_pct: number;
  memory_mb: number;
  tokens_used: number;
  tasks_completed: number;
  error_count: number;
}

export interface AgentViewSummary {
  total_sessions: number;
  active_sessions: number;
  waiting_input: number;
  completed_today: number;
  failed_today: number;
  avg_cpu: number;
  avg_memory: number;
}

export interface AgentViewEvent {
  timestamp: number;
  session_id: string;
  kind: string;
  detail: string;
}

export interface AgentViewConfig {
  enabled: boolean;
  poll_interval_ms: number;
  max_sessions: number;
  show_completed: boolean;
  group_by: string;
}

export function agentViewSummary(): Promise<AgentViewSummary> {
  return invoke("agent_view_summary");
}

export function agentViewList(): Promise<AgentViewSession[]> {
  return invoke("agent_view_list");
}

export function agentViewGet(id: string): Promise<AgentViewSession> {
  return invoke("agent_view_get", { id });
}

export function agentViewEvents(sessionId: string, count?: number): Promise<AgentViewEvent[]> {
  return invoke("agent_view_events", { sessionId, count });
}

export function agentViewPause(sessionId: string): Promise<void> {
  return invoke("agent_view_pause", { sessionId });
}

export function agentViewResume(sessionId: string): Promise<void> {
  return invoke("agent_view_resume", { sessionId });
}

export function agentViewCancel(sessionId: string): Promise<void> {
  return invoke("agent_view_cancel", { sessionId });
}

export function agentViewConfig(): Promise<AgentViewConfig> {
  return invoke("agent_view_config");
}

export function agentViewSetConfig(config: AgentViewConfig): Promise<void> {
  return invoke("agent_view_set_config", { config });
}

export function agentViewTick(): Promise<AgentViewSummary> {
  return invoke("agent_view_tick");
}

// ══════════════════════════════════════════════════════════════════════════════
// Loop (cron-style scheduled tasks)
// ══════════════════════════════════════════════════════════════════════════════

export interface LoopSchedule {
  id: string;
  name: string;
  description: string;
  cron_expr: string;
  task_type: string;
  task_config: Record<string, unknown>;
  enabled: boolean;
  created_at: string;
  last_run_at: string | null;
  next_run_at: string | null;
  run_count: number;
  success_count: number;
  fail_count: number;
}

export interface LoopExecution {
  id: string;
  schedule_id: string;
  status: string;
  started_at: string;
  completed_at: string | null;
  duration_ms: number;
  result_summary: string;
  error: string | null;
  output: string | null;
}

export interface LoopStats {
  total_schedules: number;
  active_schedules: number;
  executed_today: number;
  failed_today: number;
  avg_duration_ms: number;
  success_rate: number;
  next_scheduled_run: string | null;
}

export function loopCreate(name: string, description: string, cronExpr: string, taskType: string, taskConfig?: Record<string, unknown>): Promise<string> {
  return invoke("loop_create", { name, description, cronExpr, taskType, taskConfig });
}
export function loopList(): Promise<LoopSchedule[]> { return invoke("loop_list"); }
export function loopGet(id: string): Promise<LoopSchedule> { return invoke("loop_get", { id }); }
export function loopUpdate(id: string, name?: string, description?: string, cronExpr?: string, enabled?: boolean, taskConfig?: Record<string, unknown>): Promise<void> {
  return invoke("loop_update", { id, name, description, cronExpr, enabled, taskConfig });
}
export function loopDelete(id: string): Promise<void> { return invoke("loop_delete", { id }); }
export function loopEnable(id: string): Promise<void> { return invoke("loop_enable", { id }); }
export function loopDisable(id: string): Promise<void> { return invoke("loop_disable", { id }); }
export function loopExecuteNow(id: string): Promise<string> { return invoke("loop_execute_now", { id }); }
export function loopExecutionHistory(id: string, count?: number): Promise<LoopExecution[]> { return invoke("loop_execution_history", { id, count }); }
export function loopStats(): Promise<LoopStats> { return invoke("loop_stats"); }
export function loopNextScheduled(): Promise<LoopSchedule | null> { return invoke("loop_next_scheduled"); }
export function loopValidateCron(expression: string): Promise<boolean> { return invoke("loop_validate_cron", { expression }); }
export function loopTick(): Promise<string[]> { return invoke("loop_tick"); }

// ══════════════════════════════════════════════════════════════════════════════
// Security Scan (vulnerability detection)
// ══════════════════════════════════════════════════════════════════════════════

export type VulnerabilitySeverity = "Critical" | "High" | "Medium" | "Low" | "Info";
export type VulnerabilityStatus = "Open" | "Verified" | "Fixed" | "WontFix" | "FalsePositive";

export interface VulnerabilityFinding {
  id: string;
  title: string;
  description: string;
  severity: VulnerabilitySeverity;
  file_path: string;
  line_start: number;
  line_end: number;
  cwe_id: string | null;
  cve_id: string | null;
  confidence: number;
  remediation: string;
  patch_suggestion: string | null;
  status: VulnerabilityStatus;
  discovered_at: string;
  verified_at: string | null;
  fixed_at: string | null;
}

export interface ScanResult {
  scan_id: string;
  target_path: string;
  total_files_scanned: number;
  total_findings: number;
  critical: number;
  high: number;
  medium: number;
  low: number;
  info: number;
  duration_ms: number;
  started_at: string;
  completed_at: string;
  overall_score: number;
  by_category: Record<string, number>;
}

export interface SecurityScanConfig {
  enabled: boolean;
  scan_on_save: boolean;
  scan_depth: string;
  include_patterns: string[];
  exclude_patterns: string[];
  max_file_size_kb: number;
  auto_fix_critical: boolean;
  notify_on_critical: boolean;
}

export interface ScanSummary {
  total_scans: number;
  total_findings: number;
  open_critical: number;
  open_high: number;
  open_medium: number;
  fixed_today: number;
  avg_scan_duration_ms: number;
  security_score: number;
}

export function securityScanStart(targetPath: string, depth?: string): Promise<string> { return invoke("security_scan_start", { targetPath, depth }); }
export function securityScanStatus(scanId: string): Promise<ScanResult> { return invoke("security_scan_status", { scanId }); }
export function securityScanList(): Promise<ScanResult[]> { return invoke("security_scan_list"); }
export function securityScanFindings(scanId: string, severity?: string): Promise<VulnerabilityFinding[]> { return invoke("security_scan_findings", { scanId, severity }); }
export function securityScanFindingDetail(findingId: string): Promise<VulnerabilityFinding> { return invoke("security_scan_finding_detail", { findingId }); }
export function securityScanApplyPatch(findingId: string): Promise<string> { return invoke("security_scan_apply_patch", { findingId }); }
export function securityScanMarkStatus(findingId: string, status: string): Promise<void> { return invoke("security_scan_mark_status", { findingId, status }); }
export function securityScanConfig(): Promise<SecurityScanConfig> { return invoke("security_scan_config"); }
export function securityScanSetConfig(config: SecurityScanConfig): Promise<void> { return invoke("security_scan_set_config", { config }); }
export function securityScanSummary(): Promise<ScanSummary> { return invoke("security_scan_summary"); }
export function securityScanQuickCheck(): Promise<{ critical_count: number; has_critical: boolean; summary: string }> { return invoke("security_scan_quick_check"); }
export function securityScanFixAll(findingIds: string[]): Promise<{ fixed: number; failed: number; details: unknown[] }> { return invoke("security_scan_fix_all", { findingIds }); }

// ══════════════════════════════════════════════════════════════════════════════
// Voice Mode (speech-to-text coding)
// ══════════════════════════════════════════════════════════════════════════════

export interface VoiceTranscript {
  id: string;
  text: string;
  language: string;
  confidence: number;
  is_final: boolean;
  timestamp: string;
}

export interface VoiceConfig {
  enabled: boolean;
  language: string;
  auto_submit: boolean;
  wake_word: string;
  push_to_talk: boolean;
  stt_backend: string;
  tts_enabled: boolean;
  tts_voice: string;
}

export interface VoiceSession {
  id: string;
  status: string;
  started_at: string;
  commands_executed: number;
  duration_secs: number;
  language: string;
}

export interface VoiceStats {
  total_sessions: number;
  commands_executed: number;
  avg_confidence: number;
  top_languages: [string, number][];
  top_actions: [string, number][];
}

export function voiceStartSession(language?: string): Promise<string> { return invoke("voice_start_session", { language }); }
export function voiceStopSession(sessionId: string): Promise<VoiceSession> { return invoke("voice_stop_session", { sessionId }); }
export function voiceSessionStatus(sessionId: string): Promise<VoiceSession> { return invoke("voice_session_status", { sessionId }); }
export function voiceSendAudio(sessionId: string, audioData: string): Promise<VoiceTranscript> { return invoke("voice_send_audio", { sessionId, audioData }); }
export function voiceGetTranscription(audioData: string, language?: string): Promise<VoiceTranscript> { return invoke("voice_get_transcription", { audioData, language }); }
export function voiceListSessions(): Promise<VoiceSession[]> { return invoke("voice_list_sessions"); }
export function voiceSessionHistory(sessionId: string): Promise<VoiceTranscript[]> { return invoke("voice_session_history", { sessionId }); }
export function voiceSynthesize(text: string, voice?: string): Promise<string> { return invoke("voice_synthesize", { text, voice }); }
export function voiceConfig(): Promise<VoiceConfig> { return invoke("voice_config"); }
export function voiceSetConfig(config: VoiceConfig): Promise<void> { return invoke("voice_set_config", { config }); }
export function voiceTestMicrophone(): Promise<boolean> { return invoke("voice_test_microphone"); }
export function voiceStats(): Promise<VoiceStats> { return invoke("voice_stats"); }
export function voiceExecuteCommand(transcript: string): Promise<string> { return invoke("voice_execute_command", { transcript }); }

// ══════════════════════════════════════════════════════════════════════════════
// Plugin Marketplace
// ══════════════════════════════════════════════════════════════════════════════

export interface MarketplacePlugin {
  id: string; name: string; version: string; author: string; description: string;
  category: string; tags: string[]; downloads: number; rating: number; rating_count: number;
  is_installed: boolean; has_update: boolean; installed_version: string | null;
  homepage: string | null; repository: string | null; license: string | null; size_kb: number;
  created_at: string; updated_at: string;
}

export interface MarketplaceSearchResult {
  total: number; results: MarketplacePlugin[]; page: number; total_pages: number;
}

export interface MarketplaceReview {
  id: string; plugin_id: string; author: string; rating: number;
  title: string; body: string; created_at: string; helpful_count: number;
}

export interface MarketplaceConfig {
  enabled: boolean; auto_check_updates: boolean; update_channel: string;
  curated_only: boolean; auto_install_security: boolean;
}

export interface MarketplaceStats {
  total_plugins: number; total_downloads: number; total_installed: number;
  updates_available: number; categories: number;
}

export function marketplaceList(category?: string, page?: number, sort?: string): Promise<MarketplaceSearchResult> { return invoke("marketplace_list", { category, page, sort }); }
export function marketplaceSearch(query: string, category?: string, page?: number): Promise<MarketplaceSearchResult> { return invoke("marketplace_search", { query, category, page }); }
export function marketplaceGet(pluginId: string): Promise<MarketplacePlugin> { return invoke("marketplace_get", { pluginId }); }
export function marketplaceInstall(pluginId: string): Promise<string> { return invoke("marketplace_install", { pluginId }); }
export function marketplaceUninstall(pluginId: string): Promise<void> { return invoke("marketplace_uninstall", { pluginId }); }
export function marketplaceUpdate(pluginId: string): Promise<string> { return invoke("marketplace_update", { pluginId }); }
export function marketplaceCheckUpdates(): Promise<MarketplacePlugin[]> { return invoke("marketplace_check_updates"); }
export function marketplaceUpdateAll(): Promise<number> { return invoke("marketplace_update_all"); }
export function marketplaceReviews(pluginId: string): Promise<MarketplaceReview[]> { return invoke("marketplace_reviews", { pluginId }); }
export function marketplaceSubmitReview(pluginId: string, rating: number, title: string, body: string): Promise<void> { return invoke("marketplace_submit_review", { pluginId, rating, title, body }); }
export function marketplaceCategories(): Promise<{ id: string; name: string; description: string; count: number }[]> { return invoke("marketplace_categories"); }
export function marketplaceStats(): Promise<MarketplaceStats> { return invoke("marketplace_stats"); }
export function marketplaceConfig(): Promise<MarketplaceConfig> { return invoke("marketplace_config"); }
export function marketplaceSetConfig(config: MarketplaceConfig): Promise<void> { return invoke("marketplace_set_config", { config }); }
export function marketplaceFeatured(): Promise<MarketplacePlugin[]> { return invoke("marketplace_featured"); }

// ══════════════════════════════════════════════════════════════════════════════
// Memory Manager (KB memory viewer/editor)
// ══════════════════════════════════════════════════════════════════════════════

export interface MemoryEntry {
  id: string; kind: string; content: string; summary: string; source: string;
  confidence: number; created_at: string; last_accessed_at: string;
  access_count: number; tags: string[]; is_pinned: boolean;
}

export interface MemoryStats {
  total_entries: number; total_categories: number; oldest_entry: string;
  newest_entry: string; avg_confidence: number; top_tags: [string, number][];
  memory_usage_bytes: number;
}

export interface MemoryConfig {
  enabled: boolean; auto_consolidate: boolean; consolidation_interval_mins: number;
  max_entries: number; enable_search: boolean; enable_pinning: boolean;
}

export function memoryList(kind?: string, page?: number, sort?: string): Promise<MemoryEntry[]> { return invoke("memory_list", { kind, page, sort }); }
export function memoryGet(id: string): Promise<MemoryEntry> { return invoke("memory_get", { id }); }
export function memorySearch(query: string, kind?: string): Promise<{ total: number; results: MemoryEntry[]; query: string }> { return invoke("memory_search", { query, kind }); }
export function memoryCreate(kind: string, content: string, summary?: string, tags?: string[], source?: string): Promise<string> { return invoke("memory_create", { kind, content, summary, tags, source }); }
export function memoryUpdate(id: string, content?: string, summary?: string, tags?: string[], confidence?: number, isPinned?: boolean): Promise<void> { return invoke("memory_update", { id, content, summary, tags, confidence, isPinned }); }
export function memoryDelete(id: string): Promise<void> { return invoke("memory_delete", { id }); }
export function memoryPin(id: string): Promise<void> { return invoke("memory_pin", { id }); }
export function memoryUnpin(id: string): Promise<void> { return invoke("memory_unpin", { id }); }
export function memoryCategories(): Promise<{ id: string; name: string; description: string; count: number }[]> { return invoke("memory_categories"); }
export function memoryStats(): Promise<MemoryStats> { return invoke("memory_stats"); }
export function memoryTimeline(days?: number): Promise<{ date: string; entries_created: number; entries_accessed: number; top_topic: string }[]> { return invoke("memory_timeline", { days }); }
export function memoryConsolidateNow(): Promise<{ consolidated: number; deleted_duplicates: number; duration_ms: number }> { return invoke("memory_consolidate_now"); }
export function memoryClear(kind?: string): Promise<number> { return invoke("memory_clear", { kind }); }
export function memoryExport(format?: string): Promise<string> { return invoke("memory_export", { format }); }
export function memoryImport(data: string): Promise<number> { return invoke("memory_import", { data }); }
export function memoryConfig(): Promise<MemoryConfig> { return invoke("memory_config"); }
export function memorySetConfig(config: MemoryConfig): Promise<void> { return invoke("memory_set_config", { config }); }

// ══════════════════════════════════════════════════════════════════════════════
// Context Compaction
// ══════════════════════════════════════════════════════════════════════════════

export interface ContextSegment {
  id: string; original_length_chars: number; compacted_length_chars: number;
  ratio: number; summary: string; key_points: string[]; decisions: string[];
  preserved_code: string[]; created_at: string;
}

export interface CompactionResult {
  session_id: string; original_total_chars: number; compacted_total_chars: number;
  reduction_pct: number; segments_compacted: number; strategy_used: string;
  level: string; preserved_decision_count: number; preserved_code_snippets: number;
}

export interface CompactionConfig {
  enabled: boolean; auto_compact: boolean; auto_compact_threshold_chars: number;
  strategy: string; level: string; preserve_decisions: boolean;
  preserve_code_changes: boolean; preserve_user_messages: boolean; show_compaction_notice: boolean;
}

export function contextAnalyze(sessionId: string): Promise<{ session_id: string; current_chars: number; message_count: number; oldest_message_age_mins: number; estimated_value_pct: number; has_been_compacted: boolean; compaction_count: number }> { return invoke("context_analyze", { sessionId }); }
export function contextCompact(sessionId: string, level?: string, strategy?: string): Promise<CompactionResult> { return invoke("context_compact", { sessionId, level, strategy }); }
export function contextGetSegments(sessionId: string): Promise<ContextSegment[]> { return invoke("context_get_segments", { sessionId }); }
export function contextGetSegment(segmentId: string): Promise<ContextSegment> { return invoke("context_get_segment", { segmentId }); }
export function contextExpand(): Promise<void> { return invoke("context_expand"); }
export function contextConfig(): Promise<CompactionConfig> { return invoke("context_config"); }
export function contextSetConfig(config: CompactionConfig): Promise<void> { return invoke("context_set_config", { config }); }
export function contextStats(): Promise<{ total_compactions: number; total_chars_reduced: number; avg_reduction_pct: number; sessions_compacted: number; storage_saved_bytes: number }> { return invoke("context_stats"); }
export function contextSummarize(text: string, maxChars?: number): Promise<string> { return invoke("context_summarize", { text, maxChars }); }
export function contextExtractDecisions(text: string): Promise<string[]> { return invoke("context_extract_decisions", { text }); }
export function contextCheckThreshold(sessionId: string): Promise<boolean> { return invoke("context_check_threshold", { sessionId }); }

// ══════════════════════════════════════════════════════════════════════════════
// Profile System (named config profiles)
// ══════════════════════════════════════════════════════════════════════════════

export interface ProfileConfig {
  model: string; approval_mode: string; sandbox_mode: string;
  web_search_enabled: boolean; context_compaction: string; max_tokens: number;
  temperature: number; theme: string; custom_instructions: string | null;
  mcp_servers: string[]; plugins: string[];
}

export interface ProfileInfo {
  name: string; description: string; is_active: boolean; is_default: boolean;
  created_at: string; updated_at: string; config: ProfileConfig;
}

export function profileCreate(name: string, description: string, config?: ProfileConfig): Promise<string> { return invoke("profile_create", { name, description, config }); }
export function profileList(): Promise<ProfileInfo[]> { return invoke("profile_list"); }
export function profileGet(name: string): Promise<ProfileInfo> { return invoke("profile_get", { name }); }
export function profileUpdate(name: string, description?: string, config?: ProfileConfig): Promise<void> { return invoke("profile_update", { name, description, config }); }
export function profileDelete(name: string): Promise<void> { return invoke("profile_delete", { name }); }
export function profileActivate(name: string): Promise<void> { return invoke("profile_activate", { name }); }
export function profileDuplicate(name: string, newName: string, newDescription?: string): Promise<string> { return invoke("profile_duplicate", { name, newName, newDescription }); }
export function profileReset(name: string): Promise<void> { return invoke("profile_reset", { name }); }
export function profileExport(name: string): Promise<string> { return invoke("profile_export", { name }); }
export function profileImport(jsonData: string): Promise<string> { return invoke("profile_import", { jsonData }); }
export function profileSummary(): Promise<{ total_profiles: number; active_profile: string; default_profile: string; profiles_by_model: Record<string, number> }> { return invoke("profile_summary"); }
export function profileTemplates(): Promise<ProfileInfo[]> { return invoke("profile_templates"); }

// ══════════════════════════════════════════════════════════════════════════════
// Browser Page Annotations
// ══════════════════════════════════════════════════════════════════════════════

export interface PageAnnotation {
  id: string; url: string; page_title: string; selector: string;
  highlighted_text: string; comment: string; annotation_type: string;
  author: string; created_at: string; updated_at: string; resolved: boolean;
  resolved_at: string | null; tags: string[]; screenshot_path: string | null;
}

export function annotationCreate(url: string, pageTitle: string, selector: string, highlightedText: string, comment: string, annotationType?: string, tags?: string[]): Promise<string> { return invoke("annotation_create", { url, pageTitle, selector, highlightedText, comment, annotationType, tags }); }
export function annotationList(url?: string, resolved?: boolean, page?: number): Promise<PageAnnotation[]> { return invoke("annotation_list", { url, resolved, page }); }
export function annotationGet(id: string): Promise<PageAnnotation> { return invoke("annotation_get", { id }); }
export function annotationUpdate(id: string, comment?: string, annotationType?: string, tags?: string[]): Promise<void> { return invoke("annotation_update", { id, comment, annotationType, tags }); }
export function annotationDelete(id: string): Promise<void> { return invoke("annotation_delete", { id }); }
export function annotationResolve(id: string): Promise<void> { return invoke("annotation_resolve", { id }); }
export function annotationUnresolve(id: string): Promise<void> { return invoke("annotation_unresolve", { id }); }
export function annotationGetForUrl(url: string): Promise<PageAnnotation[]> { return invoke("annotation_get_for_url", { url }); }
export function annotationSearch(query: string): Promise<PageAnnotation[]> { return invoke("annotation_search", { query }); }
export function annotationStats(): Promise<{ total_annotations: number; unresolved: number; resolved_today: number; collections: number; urls_tracked: number; top_tags: [string, number][] }> { return invoke("annotation_stats"); }
export function annotationConfig(): Promise<{ enabled: boolean; auto_collect: boolean; collect_on_navigate: boolean; show_on_page_load: boolean; notify_on_unresolved: boolean; max_annotations_per_page: number }> { return invoke("annotation_config"); }
export function annotationSetConfig(config: Record<string, unknown>): Promise<void> { return invoke("annotation_set_config", { config }); }

// ══════════════════════════════════════════════════════════════════════════════
// Activity Insights & Usage Cards
// ══════════════════════════════════════════════════════════════════════════════

export interface DailyActivity {
  date: string; total_events: number; active_minutes: number; sessions_count: number;
  commands_executed: number; files_edited: number; searches_performed: number;
  reviews_done: number; errors_count: number; top_project: string | null;
  categories: Record<string, number>;
}

export interface ActivityInsight {
  insight_type: string; title: string; description: string; value: string;
  change_pct: number | null; is_positive: boolean; emoji: string;
}

export interface UsageCard {
  id: string; title: string; subtitle: string; stats: Record<string, string>;
  period: string; generated_at: string; share_url: string | null; theme: string;
}

export function insightsRecordEvent(eventType: string, details: string, sessionId?: string, project?: string, durationMs?: number): Promise<string> { return invoke("insights_record_event", { eventType, details, sessionId, project, durationMs }); }
export function insightsDaily(date?: string): Promise<DailyActivity> { return invoke("insights_daily", { date }); }
export function insightsWeekly(weekStart?: string): Promise<{ week_start: string; week_end: string; total_active_hours: number; avg_daily_hours: number; most_active_day: string; projects_worked: string[]; top_category: string; insights: ActivityInsight[]; overall_productivity_score: number }> { return invoke("insights_weekly", { weekStart }); }
export function insightsInsights(period?: string): Promise<ActivityInsight[]> { return invoke("insights_insights", { period }); }
export function insightsGenerateCard(period?: string, theme?: string, title?: string): Promise<UsageCard> { return invoke("insights_generate_card", { period, theme, title }); }
export function insightsCardList(): Promise<UsageCard[]> { return invoke("insights_card_list"); }
export function insightsCardGet(id: string): Promise<UsageCard> { return invoke("insights_card_get", { id }); }
export function insightsCardShare(id: string): Promise<string> { return invoke("insights_card_share", { id }); }
export function insightsTrend(days?: number): Promise<{ dates: string[]; active_minutes: number[]; commands: number[]; projects: number[]; trend_direction: string; change_pct: number }> { return invoke("insights_trend", { days }); }
export function insightsConfig(): Promise<{ enabled: boolean; track_activity: boolean; show_notifications: boolean; weekly_summary_enabled: boolean; share_usage_enabled: boolean; retention_days: number }> { return invoke("insights_config"); }
export function insightsSetConfig(config: Record<string, unknown>): Promise<void> { return invoke("insights_set_config", { config }); }
export function insightsStats(): Promise<{ total_events_tracked: number; days_active: number; current_streak_days: number; longest_streak_days: number; avg_daily_active_mins: number; projects_this_month: number; cards_generated: number }> { return invoke("insights_stats"); }
export function insightsReset(): Promise<void> { return invoke("insights_reset"); }

// ══════════════════════════════════════════════════════════════════════════════
// Terminal Tabs (multi-terminal per session)
// ══════════════════════════════════════════════════════════════════════════════

export interface TerminalTab {
  id: string; session_id: string; name: string; index: number; cwd: string;
  shell: string; is_active: boolean; created_at: string; last_used_at: string;
  color: string | null; columns: number; rows: number;
}

export function termTabsCreate(sessionId: string, name?: string, cwd?: string, shell?: string, color?: string): Promise<string> { return invoke("term_tabs_create", { sessionId, name, cwd, shell, color }); }
export function termTabsList(sessionId: string): Promise<TerminalTab[]> { return invoke("term_tabs_list", { sessionId }); }
export function termTabsGet(tabId: string): Promise<TerminalTab> { return invoke("term_tabs_get", { tabId }); }
export function termTabsRename(tabId: string, name: string): Promise<void> { return invoke("term_tabs_rename", { tabId, name }); }
export function termTabsClose(tabId: string): Promise<void> { return invoke("term_tabs_close", { tabId }); }
export function termTabsActivate(tabId: string): Promise<void> { return invoke("term_tabs_activate", { tabId }); }
export function termTabsReorder(sessionId: string, tabIds: string[]): Promise<void> { return invoke("term_tabs_reorder", { sessionId, tabIds }); }
export function termTabsSetColor(tabId: string, color: string): Promise<void> { return invoke("term_tabs_set_color", { tabId, color }); }
export function termTabsLayout(sessionId: string): Promise<{ session_id: string; tabs: TerminalTab[]; active_tab_id: string | null; layout: string; split_pct: number }> { return invoke("term_tabs_layout", { sessionId }); }
export function termTabsSetLayout(sessionId: string, layout: string, splitPct?: number): Promise<void> { return invoke("term_tabs_set_layout", { sessionId, layout, splitPct }); }
export function termTabsGroupCreate(name: string, sessionId: string, tabIds: string[]): Promise<string> { return invoke("term_tabs_group_create", { name, sessionId, tabIds }); }
export function termTabsGroupList(sessionId: string): Promise<{ id: string; name: string; session_id: string; tab_ids: string[]; is_collapsed: boolean }[]> { return invoke("term_tabs_group_list", { sessionId }); }
export function termTabsGroupDelete(groupId: string): Promise<void> { return invoke("term_tabs_group_delete", { groupId }); }
export function termTabsConfig(): Promise<{ default_shell: string; default_columns: number; default_rows: number; max_tabs_per_session: number; enable_colors: boolean; enable_groups: boolean; scrollback_lines: number }> { return invoke("term_tabs_config"); }
export function termTabsSetConfig(config: Record<string, unknown>): Promise<void> { return invoke("term_tabs_set_config", { config }); }
export function termTabsStats(): Promise<{ total_tabs: number; total_groups: number; avg_tabs_per_session: number }> { return invoke("term_tabs_stats"); }

// ══════════════════════════════════════════════════════════════════════════════
// Unified Session Overview (local + remote + teleport)
// ══════════════════════════════════════════════════════════════════════════════

export interface UnifiedSession {
  id: string; name: string; type_: string; surface: string; status: string;
  project: string | null; started_at: string; last_active_at: string;
  active_duration_minutes: number; command_count: number; file_changes: number;
  error_count: number; remote_host: string | null; remote_location: string | null;
  sync_status: string; tags: string[];
}

export function unifiedSessionList(filter?: { types?: string[]; statuses?: string[]; projects?: string[]; surfaces?: string[]; search?: string }): Promise<UnifiedSession[]> { return invoke("unified_session_list", { filter }); }
export function unifiedSessionGet(id: string): Promise<UnifiedSession> { return invoke("unified_session_get", { id }); }
export function unifiedSessionSummary(): Promise<{ total_sessions: number; active_local: number; active_remote: number; active_teleport: number; total_active: number; total_idle: number; total_paused: number; total_errors: number; most_active_project: string | null; avg_duration_minutes: number }> { return invoke("unified_session_summary"); }
export function unifiedSessionGroupBy(field: string): Promise<{ group_by: string; groups: { key: string; sessions: UnifiedSession[]; count: number }[] }> { return invoke("unified_session_group_by", { field }); }
export function unifiedSessionSearch(query: string): Promise<UnifiedSession[]> { return invoke("unified_session_search", { query }); }
export function unifiedSessionStats(): Promise<{ total_sessions_all_time: number; total_active_hours: number; most_used_surface: string; most_used_type: string; sessions_per_day: number; peak_hour: number }> { return invoke("unified_session_stats"); }
export function unifiedSessionConnect(id: string): Promise<void> { return invoke("unified_session_connect", { id }); }
export function unifiedSessionDisconnect(id: string): Promise<void> { return invoke("unified_session_disconnect", { id }); }
export function unifiedSessionTag(id: string, tags: string[]): Promise<void> { return invoke("unified_session_tag", { id, tags }); }
export function unifiedSessionUntag(id: string, tags: string[]): Promise<void> { return invoke("unified_session_untag", { id, tags }); }
export function unifiedSessionExport(): Promise<string> { return invoke("unified_session_export"); }
export function unifiedSessionImport(data: string): Promise<number> { return invoke("unified_session_import", { data }); }
export function unifiedSessionRefresh(): Promise<{ total_sessions: number; active_local: number; active_remote: number; active_teleport: number; total_active: number }> { return invoke("unified_session_refresh"); }

// ══════════════════════════════════════════════════════════════════════════════
// Cowork (file/folder productivity mode)
// ══════════════════════════════════════════════════════════════════════════════

export interface CoworkSession {
  id: string; name: string; workspace_path: string; status: string;
  files_read: number; files_created: number; files_modified: number;
  started_at: string; last_active_at: string; deliverables: string[];
  description: string; tags: string[];
}

export interface CoworkFile {
  path: string; relative_path: string; size_bytes: number; kind: string;
  last_modified: string; content_summary: string; is_deliverable: boolean;
}

export interface CoworkDeliverable {
  id: string; session_id: string; name: string; path: string; kind: string;
  created_at: string; size_bytes: number; description: string; quality_score: number | null;
}

export function coworkStart(workspacePath: string, description: string, name?: string, tags?: string[]): Promise<string> { return invoke("cowork_start", { workspacePath, description, name, tags }); }
export function coworkList(): Promise<CoworkSession[]> { return invoke("cowork_list"); }
export function coworkGet(sessionId: string): Promise<CoworkSession> { return invoke("cowork_get", { sessionId }); }
export function coworkStatus(sessionId: string): Promise<CoworkSession> { return invoke("cowork_status", { sessionId }); }
export function coworkPause(sessionId: string): Promise<void> { return invoke("cowork_pause", { sessionId }); }
export function coworkResume(sessionId: string): Promise<void> { return invoke("cowork_resume", { sessionId }); }
export function coworkStop(sessionId: string): Promise<CoworkSession> { return invoke("cowork_stop", { sessionId }); }
export function coworkScanFiles(sessionId: string, pattern?: string): Promise<CoworkFile[]> { return invoke("cowork_scan_files", { sessionId, pattern }); }
export function coworkReadFile(sessionId: string, path: string): Promise<string> { return invoke("cowork_read_file", { sessionId, path }); }
export function coworkWriteFile(sessionId: string, path: string, content: string): Promise<void> { return invoke("cowork_write_file", { sessionId, path, content }); }
export function coworkDeleteFile(sessionId: string, path: string): Promise<void> { return invoke("cowork_delete_file", { sessionId, path }); }
export function coworkListDeliverables(sessionId: string): Promise<CoworkDeliverable[]> { return invoke("cowork_list_deliverables", { sessionId }); }
export function coworkGetDeliverable(deliverableId: string): Promise<CoworkDeliverable> { return invoke("cowork_get_deliverable", { deliverableId }); }
export function coworkTemplates(category?: string): Promise<{ id: string; name: string; description: string; category: string; steps: { order: number; action: string; description: string }[]; suggested_prompt: string }[]> { return invoke("cowork_templates", { category }); }
export function coworkApplyTemplate(sessionId: string, templateId: string): Promise<{ id: string; session_id: string; action_type: string; target_path: string; status: string; started_at: string; completed_at: string | null; details: string | null; result_summary: string | null }[]> { return invoke("cowork_apply_template", { sessionId, templateId }); }
export function coworkActions(sessionId: string): Promise<{ id: string; session_id: string; action_type: string; target_path: string; status: string; started_at: string; completed_at: string | null; details: string | null; result_summary: string | null }[]> { return invoke("cowork_actions", { sessionId }); }
export function coworkConfig(): Promise<{ enabled: boolean; max_files_per_scan: number; max_file_size_kb: number; auto_save: boolean; deliverable_formats: string[]; allow_file_create: boolean; allow_file_modify: boolean; allow_file_delete: boolean }> { return invoke("cowork_config"); }
export function coworkSetConfig(config: Record<string, unknown>): Promise<void> { return invoke("cowork_set_config", { config }); }
export function coworkStats(): Promise<{ total_sessions: number; total_deliverables: number; files_processed: number; active_sessions: number; avg_files_per_session: number; top_category: string; top_template: string }> { return invoke("cowork_stats"); }
export function coworkExportSession(sessionId: string, format?: string): Promise<string> { return invoke("cowork_export_session", { sessionId, format }); }

// ══════════════════════════════════════════════════════════════════════════════
// Background Computer Use
// ══════════════════════════════════════════════════════════════════════════════

export interface BackgroundTask {
  id: string; name: string; description: string; task_type: string;
  status: string; progress_pct: number; target: string;
  created_at: string; started_at: string | null; completed_at: string | null;
  duration_ms: number | null; result: string | null; error: string | null; retry_count: number;
}

export function computerBgSubmit(type_: string, target: string, params?: Record<string, unknown>): Promise<string> { return invoke("computer_bg_submit", { type_, target, params }); }
export function computerBgList(status?: string): Promise<BackgroundTask[]> { return invoke("computer_bg_list", { status }); }
export function computerBgGet(taskId: string): Promise<BackgroundTask> { return invoke("computer_bg_get", { taskId }); }
export function computerBgCancel(taskId: string): Promise<void> { return invoke("computer_bg_cancel", { taskId }); }
export function computerBgRetry(taskId: string): Promise<void> { return invoke("computer_bg_retry", { taskId }); }
export function computerBgClear(includeRunning?: boolean): Promise<number> { return invoke("computer_bg_clear", { includeRunning }); }
export function computerBgStats(): Promise<{ total_created: number; total_completed: number; total_failed: number; total_cancelled: number; currently_running: number; avg_duration_ms: number; success_rate: number }> { return invoke("computer_bg_stats"); }
export function computerBgConfig(): Promise<{ enabled: boolean; max_concurrent_tasks: number; poll_interval_ms: number; auto_retry: boolean; max_retries: number; notify_on_completion: boolean; log_path: string | null }> { return invoke("computer_bg_config"); }
export function computerBgSetConfig(config: Record<string, unknown>): Promise<void> { return invoke("computer_bg_set_config", { config }); }
export function computerBgRunScript(scriptName: string, args?: string[]): Promise<string> { return invoke("computer_bg_run_script", { scriptName, args }); }
