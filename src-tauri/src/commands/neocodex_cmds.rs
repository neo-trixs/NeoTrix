use neotrix::neotrix::l1_body_impl::nt_io_neocodex::{
    NeoCodexAgent, NeoCodexHealthReport, NeoCodexMode, WireSession, WireEvent,
};
use neotrix::neotrix::l1_body_impl::nt_agent_mcp_registry::McpRegistry;
use serde::Serialize;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

static NEOCODEX_AGENT: std::sync::LazyLock<tokio::sync::Mutex<Option<NeoCodexAgent>>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(None));

static STREAM_CANCELLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_stop_stream() -> Result<(), String> {
    STREAM_CANCELLED.store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

#[derive(serde::Deserialize, Default)]
pub struct NeoCodexAttachmentPayload {
    pub name: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub data: Option<String>,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_send_message_stream(
    app: tauri::AppHandle,
    content: String,
    attachments: Option<Vec<NeoCodexAttachmentPayload>>,
    regenerate: Option<bool>,
    permission_mode: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
) -> Result<String, String> {
    let mut agent_guard = NEOCODEX_AGENT.lock().await;
    let agent = match agent_guard.as_mut() {
        Some(a) => a,
        None => {
            let mut a = NeoCodexAgent::new("neotrix-tauri");
            a.provider.ensure_production_provider();
            *agent_guard = Some(a);
            agent_guard.as_mut().unwrap()
        }
    };

    // P2-1: apply settings-panel generation params to the next request.
    agent.set_generation_params(temperature, max_tokens);

    // Permission mode (Claude Code Manual/AcceptEdits/Plan / Codex approval
    // parity). Plan forces the read-only planning path (exec_plan — real
    // enforcement in the core loop); Manual/Accept are enforced at the UI
    // review layer (diff accept/reject), recorded here for session continuity.
    let mode = permission_mode.unwrap_or_else(|| "auto".to_string());
    agent.set_permission_mode(&mode);

    let is_regenerate = regenerate.unwrap_or(false);
    if !is_regenerate {
        // Persist the user message (fixes streaming-not-persisted gap) with any
        // attachments, keeping the wire + in-memory mirror in sync.
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        // P2-7: server-side cap on attachment payloads (defense-in-depth over
        // the frontend limit). A malicious/oversized IPC payload must not land
        // in the wire file or context.
        const MAX_ATTACHMENT_BYTES: u64 = 10 * 1024 * 1024;
        let core_atts = attachments.map(|v| {
            v.into_iter().filter(|a| !a.name.is_empty() && a.size <= MAX_ATTACHMENT_BYTES).map(|a| {
                neotrix::neotrix::l1_body_impl::nt_io_neocodex::NeoCodexAttachment {
                    name: a.name,
                    size: a.size,
                    mime_type: a.mime_type,
                    data: a.data,
                }
            }).collect::<Vec<_>>()
        });
        agent.wire.record(WireEvent::UserMessage {
            content: content.clone(),
            timestamp: ts,
            attachments: core_atts,
        });
        let est = content.len() / 4;
        agent.context.push("user", content.clone(), est);
        agent.state.tokens_used += est;
        agent.state.turn_count += 1;
    }

    // Emit start event
    let _ = app.emit("neocodex_stream_start", content.clone());

    STREAM_CANCELLED.store(false, std::sync::atomic::Ordering::Relaxed);
    let started = std::time::Instant::now();
    let result = agent.react_loop_stream(&content, 4, |token| {
        let _ = app.emit("neocodex_stream_token", token);
        !STREAM_CANCELLED.load(std::sync::atomic::Ordering::Relaxed)
    }, |name, args, result, duration_ms, success| {
        let _ = app.emit("neocodex_stream_tool", serde_json::json!({
            "name": name,
            "args": args,
            "result": result,
            "duration_ms": duration_ms,
            "success": success,
        }));
        !STREAM_CANCELLED.load(std::sync::atomic::Ordering::Relaxed)
    }).await;

    let answer = result.unwrap_or_else(|| "[no response]".to_string());
    // Persist the assistant response so streamed conversations survive reload.
    let ats = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    agent.wire.record(WireEvent::AgentMessage {
        content: answer.clone(),
        timestamp: ats,
    });
    let est2 = answer.len() / 4;
    agent.context.push("assistant", answer.clone(), est2);
    agent.state.tokens_used += est2;

    // Auto-checkpoint after each complete turn (Claude Desktop checkpoint /
    // Codex revert parity). Skip regenerate turns — those truncate the wire
    // and re-snapshot the same state; snapshot only fresh user turns.
    if !is_regenerate {
        let sess = agent.wire.path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_default();
        if !sess.is_empty() {
            snapshot_checkpoint(&agent.wire.path, &sess);
        }
    }

    let _ = app.emit("neocodex_stream_end", answer.clone());

    // Engineering gap: surface completion (or cancellation) via the OS
    // notification centre when the app window is not focused, and track the
    // cancelled flag so the frontend can label the truncated reply.
    let was_cancelled = STREAM_CANCELLED.load(std::sync::atomic::Ordering::Relaxed);
    STREAM_CANCELLED.store(false, std::sync::atomic::Ordering::Relaxed);
    let _ = app.emit("neocodex_stream_done", serde_json::json!({
        "cancelled": was_cancelled,
        "elapsed_ms": started.elapsed().as_millis() as u64,
        "content": answer,
    }));

    let focused = app.get_webview_window("main")
        .map(|w| w.is_focused().unwrap_or(false))
        .unwrap_or(false);
    if !focused {
        let title = if was_cancelled { "NeoCodex 已停止" } else { "NeoCodex 任务完成" };
        let body = if was_cancelled {
            format!("已停止生成（已累积 {} 字符）", answer.chars().count())
        } else {
            let n = answer.chars().count();
            if n > 80 { format!("{}…", answer.chars().take(80).collect::<String>()) } else { answer.clone() }
        };
        let _ = app.notification()
            .builder()
            .title(title)
            .body(&body)
            .show();
    }

    Ok(answer)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_health_report() -> Result<NeoCodexHealthReport, String> {
    let guard = NEOCODEX_AGENT.lock().await;
    match guard.as_ref() {
        Some(a) => Ok(a.health_report()),
        None => Ok(NeoCodexHealthReport {
            mode: NeoCodexMode::Agent,
            turn_count: 0,
            tool_call_count: 0,
            tokens_used: 0,
            context_usage: 0.0,
            context_turns: 0,
            provider_count: 0,
            provider_resolvable: false,
            provider_model: "none".to_string(),
            session_writable: false,
            goals_active: false,
            cost_spent: 0.0,
            cost_budget: 0.0,
            subagent_results: 0,
            consciousness_attached: false,
            brain_attached: false,
            event_bus_attached: false,
            evolution_iterations: 0,
            tool_grounding_degraded: false,
            node_snapshots: Vec::new(),
        }),
    }
}

#[derive(serde::Serialize)]
pub struct NeoCodexAgentStatus {
    pub running: bool,
    pub current_task: Option<String>,
    pub uptime_secs: u64,
    pub turn_count: u64,
    pub tokens_used: usize,
    pub context_usage: f64,
    pub provider_model: String,
    pub evolution_iterations: u64,
    pub cost_spent: f64,
    pub cost_budget: f64,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_agent_status() -> Result<NeoCodexAgentStatus, String> {
    let guard = NEOCODEX_AGENT.lock().await;
    match guard.as_ref() {
        Some(a) => {
            let hr = a.health_report();
            Ok(NeoCodexAgentStatus {
                running: hr.turn_count > 0 || hr.goals_active,
                current_task: if hr.goals_active { Some("处理任务中".to_string()) } else { None },
                uptime_secs: hr.turn_count, // approximate
                turn_count: hr.turn_count,
                tokens_used: hr.tokens_used,
                context_usage: hr.context_usage,
                provider_model: hr.provider_model,
                evolution_iterations: hr.evolution_iterations,
                cost_spent: hr.cost_spent,
                cost_budget: hr.cost_budget,
            })
        }
        None => Ok(NeoCodexAgentStatus {
            running: false,
            current_task: None,
            uptime_secs: 0,
            turn_count: 0,
            tokens_used: 0,
            context_usage: 0.0,
            provider_model: "none".to_string(),
            evolution_iterations: 0,
            cost_spent: 0.0,
            cost_budget: 0.0,
        }),
    }
}

#[derive(serde::Serialize, Clone)]
pub struct NeoCodexProviderEntry {
    pub name: String,
    pub model: String,
    pub resolvable: bool,
}

#[derive(serde::Serialize, Clone)]
pub struct NeoCodexProviderConfig {
    pub provider_count: usize,
    pub resolvable: bool,
    pub active_model: String,
    pub providers: Vec<NeoCodexProviderEntry>,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_provider_config() -> Result<NeoCodexProviderConfig, String> {
    let guard = NEOCODEX_AGENT.lock().await;
    match guard.as_ref() {
        Some(a) => {
            let providers = a.provider.providers
                .iter()
                .map(|p| NeoCodexProviderEntry {
                    name: p.name.clone(),
                    model: p.model.clone(),
                    resolvable: a.provider.is_resolvable_for(&p.name),
                })
                .collect();
            Ok(NeoCodexProviderConfig {
                provider_count: a.provider.providers.len(),
                resolvable: a.provider.is_resolvable(),
                active_model: a.provider.active_model(),
                providers,
            })
        }
        None => Ok(NeoCodexProviderConfig {
            provider_count: 0,
            resolvable: false,
            active_model: "unknown".to_string(),
            providers: Vec::new(),
        }),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_set_provider(name: String) -> Result<String, String> {
    let mut guard = NEOCODEX_AGENT.lock().await;
    let agent = match guard.as_mut() {
        Some(a) => a,
        None => {
            let mut a = NeoCodexAgent::new("neotrix-tauri");
            a.provider.ensure_production_provider();
            *guard = Some(a);
            guard.as_mut().unwrap()
        }
    };
    if agent.provider.set_active_provider(&name) {
        agent.provider.save_persisted();
        Ok(format!("provider set to {}", name))
    } else {
        Err(format!("provider {} not found", name))
    }
}

fn sessions_dir() -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from(".neocodex"))
        .join("neocodex")
        .join("sessions")
}

static CURRENT_PROJECT: std::sync::LazyLock<tokio::sync::RwLock<Option<std::path::PathBuf>>> =
    std::sync::LazyLock::new(|| tokio::sync::RwLock::new(None));

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_set_project(path: String) -> Result<String, String> {
    let path_buf = std::path::PathBuf::from(&path);
    if !path_buf.exists() {
        return Err("Project path does not exist".to_string());
    }
    let mut guard = CURRENT_PROJECT.write().await;
    *guard = Some(path_buf.clone());
    Ok(format!("Project set to {}", path_buf.display()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_get_project() -> Result<Option<String>, String> {
    let guard = CURRENT_PROJECT.read().await;
    Ok(guard.as_ref().map(|p| p.to_string_lossy().to_string()))
}

/// Sanitize a session_id for use in a filename. `session_path` sanitizes for
/// the active-list path, but several archived-branch callers joined the RAW id
/// directly (P1-3 path-transversal: "../../x" escaped the sessions dir and
/// could remove/move arbitrary .jsonl). Centralize the allowlist here so both
/// active and archived paths sanitize identically.
fn sanitize_session_id(session_id: &str) -> Option<String> {
    let safe: String = session_id.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if safe.is_empty() || safe != session_id {
        None
    } else {
        Some(safe)
    }
}

fn session_path(session_id: &str) -> std::path::PathBuf {
    sanitize_session_id(session_id)
        .map(|s| sessions_dir().join(format!("{}.jsonl", s)))
        .unwrap_or_else(|| sessions_dir().join("__invalid__.jsonl"))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_list_sessions(project_path: Option<String>) -> Result<Vec<NeoCodexSessionInfo>, String> {
    let dir = sessions_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    
    // If project_path is provided, use it; otherwise check global current project
    let filter_path = if let Some(p) = project_path {
        Some(std::path::PathBuf::from(p))
    } else {
        let guard = CURRENT_PROJECT.read().await;
        guard.clone()
    };
    
    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            // If filtering by project, check if session is under the project path
            if let Some(ref project) = filter_path {
                let session_path_str = path.to_string_lossy();
                let project_str = project.to_string_lossy();
                if !session_path_str.starts_with(project_str.as_ref()) {
                    continue;
                }
            }
            
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let lines: Vec<&str> = content.lines().collect();
            let mut name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
            let mut mode = "Agent".to_string();
            let mut message_count = 0;
            let mut tags: Vec<String> = Vec::new();
            let mut updated_at = entry.metadata().map_err(|e| e.to_string())?.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
            
            for line in &lines {
                if let Ok(event) = serde_json::from_str::<WireEvent>(line) {
                    match event {
                        WireEvent::UserMessage { content, timestamp, .. } => {
                            if message_count == 0 && content.len() > 20 {
                                name = format!("{}...", &content[..content.floor_char_boundary(20)]);
                            }
                            message_count += 1;
                            updated_at = timestamp.max(0) as u64;
                        }
                        WireEvent::ModeChange { to, .. } => {
                            mode = format!("{:?}", to);
                        }
                        WireEvent::SessionMeta { name: n, timestamp, tags: t, .. } => {
                            if !n.is_empty() {
                                name = n;
                            }
                            if !t.is_empty() {
                                tags = t;
                            }
                            updated_at = timestamp.max(0) as u64;
                        }
                        _ => { message_count += 1; }
                    }
                }
            }
            sessions.push(NeoCodexSessionInfo {
                id: path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string(),
                name,
                mode,
                message_count,
                wire_path: path.to_string_lossy().to_string(),
                updated_at,
                tags,
            });
        }
    }
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(sessions)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NeoCodexSearchHit {
    pub session_id: String,
    pub session_name: String,
    pub role: String,
    pub snippet: String,
    pub timestamp: i64,
    pub match_count: usize,
}

/// P2-2: full-text search across session message content (Codex ⌘G /
/// Claude find parity). The session sidebar previously filtered only by
/// session name; this command lets the UI search the actual message bodies.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_search_sessions(query: String) -> Result<Vec<NeoCodexSearchHit>, String> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() || needle.len() < 2 {
        return Ok(Vec::new());
    }
    let dir = sessions_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut hits = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }
        let session_id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
        let mut session_name = session_id.clone();
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        for line in content.lines() {
            if let Ok(event) = serde_json::from_str::<WireEvent>(line) {
                match event {
                    WireEvent::SessionMeta { name, .. } => {
                        if !name.is_empty() {
                            session_name = name;
                        }
                    }
                    WireEvent::UserMessage { content: msg, timestamp, .. } => {
                        push_search_hit(&msg, "user", timestamp, &needle, &session_id, &session_name, &mut hits);
                    }
                    WireEvent::AgentMessage { content: msg, timestamp } => {
                        push_search_hit(&msg, "assistant", timestamp, &needle, &session_id, &session_name, &mut hits);
                    }
                    WireEvent::SessionMeta { name, .. } => {
                        if !name.is_empty() {
                            session_name = name;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    hits.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    hits.truncate(50);
    Ok(hits)
}

fn snippet_around(haystack: &str, needle: &str, radius: usize) -> String {
    let lower = haystack.to_lowercase();
    if let Some(pos) = lower.find(needle) {
        let start = pos.saturating_sub(radius);
        let end = (pos + needle.len() + radius).min(haystack.len());
        let mut s = haystack[start..end].to_string();
        if start > 0 {
            s.insert_str(0, "…");
        }
        if end < haystack.len() {
            s.push('…');
        }
        s.replace('\n', " ")
    } else {
        haystack.chars().take(radius * 2).collect()
    }
}

fn push_search_hit(
    msg: &str,
    role: &str,
    timestamp: i64,
    needle: &str,
    session_id: &str,
    session_name: &str,
    hits: &mut Vec<NeoCodexSearchHit>,
) {
    let lower = msg.to_lowercase();
    if !lower.contains(needle) {
        return;
    }
    let snippet = snippet_around(msg, needle, 120);
    let match_count = lower.matches(needle).count();
    hits.push(NeoCodexSearchHit {
        session_id: session_id.to_string(),
        session_name: session_name.to_string(),
        role: role.to_string(),
        snippet,
        timestamp,
        match_count,
    });
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NeoCodexMcpServerInfo {
    pub name: String,
    pub transport: String,
    pub tool_count: usize,
    pub healthy: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NeoCodexMcpToolInfo {
    pub name: String,
    pub description: String,
    pub server: String,
}

/// P2-5: register an MCP stdio server and attach it to the NeoCodex agent so
/// its tools become callable via the `mcp_call` agent tool (Codex/Claude MCP
/// parity). Previously the MCP host existed only for CLI/headless.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_mcp_register(
    name: String,
    command: String,
    args: Option<Vec<String>>,
) -> Result<Vec<NeoCodexMcpServerInfo>, String> {
    let mut agent_guard = NEOCODEX_AGENT.lock().await;
    let agent = match agent_guard.as_mut() {
        Some(a) => a,
        None => {
            let mut a = NeoCodexAgent::new("neotrix-tauri");
            a.provider.ensure_production_provider();
            *agent_guard = Some(a);
            agent_guard.as_mut().unwrap()
        }
    };
    let registry = agent.mcp.get_or_insert_with(McpRegistry::new);
    let arg_owned: Vec<String> = args.unwrap_or_default();
    let arg_refs: Vec<&str> = arg_owned.iter().map(|s| s.as_str()).collect();
    registry.register_stdio(&name, &command, &arg_refs, vec![]);
    Ok(registry.list_servers().iter().map(|s| NeoCodexMcpServerInfo {
        name: s.name.clone(),
        transport: s.transport.transport_type().to_string(),
        tool_count: s.tools.len(),
        healthy: s.healthy,
    }).collect())
}

/// P2-5: list MCP servers and tools currently attached to the NeoCodex agent.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_mcp_list() -> Result<Vec<NeoCodexMcpServerInfo>, String> {
    let agent_guard = NEOCODEX_AGENT.lock().await;
    let Some(agent) = agent_guard.as_ref() else {
        return Ok(Vec::new());
    };
    let Some(registry) = agent.mcp.as_ref() else {
        return Ok(Vec::new());
    };
    Ok(registry.list_servers().iter().map(|s| NeoCodexMcpServerInfo {
        name: s.name.clone(),
        transport: s.transport.transport_type().to_string(),
        tool_count: s.tools.len(),
        healthy: s.healthy,
    }).collect())
}

/// P2-5: list all MCP tools (name + description) exposed by the attached registry.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_mcp_tools() -> Result<Vec<NeoCodexMcpToolInfo>, String> {
    let agent_guard = NEOCODEX_AGENT.lock().await;
    let Some(agent) = agent_guard.as_ref() else {
        return Ok(Vec::new());
    };
    let Some(registry) = agent.mcp.as_ref() else {
        return Ok(Vec::new());
    };
    Ok(registry.list_tools().iter().map(|t| NeoCodexMcpToolInfo {
        name: t.name.clone(),
        description: t.description.clone(),
        server: t.server_name.clone(),
    }).collect())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_create_session(name: Option<String>) -> Result<NeoCodexSessionInfo, String> {
    let session_id = format!("s-{}", chrono::Utc::now().timestamp_millis());
    let path = session_path(&session_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, "").map_err(|e| e.to_string())?;
    Ok(NeoCodexSessionInfo {
        id: session_id,
        name: name.unwrap_or_else(|| "新会话".to_string()),
        mode: "Agent".to_string(),
        message_count: 0,
        wire_path: path.to_string_lossy().to_string(),
        updated_at: chrono::Utc::now().timestamp() as u64,
        tags: Vec::new(),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_get_session_messages(session_id: String) -> Result<Vec<NeoCodexMessageItem>, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for line in content.lines() {
        if let Ok(event) = serde_json::from_str::<WireEvent>(line) {
            match event {
                WireEvent::UserMessage { content, timestamp, attachments } => items.push(NeoCodexMessageItem {
                    id: items.len(),
                    role: "user".to_string(),
                    content,
                    timestamp,
                    attachments: attachments.map(|v| v.into_iter().map(|a| NeoCodexAttachmentDto {
                        name: a.name,
                        size: a.size,
                        mime_type: a.mime_type,
                        data: a.data,
                    }).collect()),
                    tool_call: None,
                }),
                WireEvent::AgentMessage { content, timestamp } => items.push(NeoCodexMessageItem {
                    id: items.len(),
                    role: "assistant".to_string(),
                    content,
                    timestamp,
                    attachments: None,
                    tool_call: None,
                }),
                WireEvent::ToolCall { name, args, result, duration_ms, success } => items.push(NeoCodexMessageItem {
                    id: items.len(),
                    role: "tool".to_string(),
                    content: format!("**{}**{}\n```\n{}\n```", name, if success { "" } else { " (失败)" }, result.chars().take(500).collect::<String>()),
                    timestamp: 0,
                    attachments: None,
                    tool_call: Some(NeoCodexToolCallDto {
                        name,
                        args,
                        result: result.chars().take(5000).collect::<String>(),
                        duration_ms,
                        success,
                    }),
                }),
                WireEvent::SystemEvent { kind, detail, timestamp } => {
                    if !kind.is_empty() {
                        items.push(NeoCodexMessageItem {
                            id: items.len(),
                            role: "system".to_string(),
                            content: format!("**{}**: {}", kind, detail),
                            timestamp,
                            attachments: None,
                            tool_call: None,
                        });
                    }
                }
                WireEvent::SideChatMessage { .. } => { /* side chat stays out of the main thread */ }
                _ => {}
            }
        }
    }
    Ok(items)
}

/// Read all wire events for a session (skipping side-chat lines).
/// P2-4: corrupt lines are surfaced as a SystemEvent instead of being silently
/// dropped — a partial write or bad JSON used to vanish with no trace, leaving
/// the user wondering why earlier messages were gone.
fn read_wire_events(path: &std::path::Path) -> Result<Vec<WireEvent>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut events = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<WireEvent>(line) {
            Ok(event) => events.push(event),
            Err(e) => {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
                events.push(WireEvent::SystemEvent {
                    kind: "wire_corrupt".to_string(),
                    detail: format!("unreadable line dropped: {}", e),
                    timestamp: ts,
                });
            }
        }
    }
    Ok(events)
}

/// Rewrite a session's wire file from an event list (used by edit/delete/regenerate).
/// P2-6: write atomically (tmp + fsync + rename) so a crash mid-rewrite can't
/// truncate the session file into a lost-history state. `std::fs::write` was a
/// non-atomic truncate. Serialization failures are surfaced, not silently dropped.
fn write_wire_events(path: &std::path::Path, events: &[WireEvent]) -> Result<(), String> {
    let mut out = String::new();
    for event in events {
        let line = serde_json::to_string(event).map_err(|e| e.to_string())?;
        out.push_str(&line);
        out.push('\n');
    }
    let tmp = path.with_extension("jsonl.tmp");
    {
        let mut f = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
        use std::io::Write as _;
        f.write_all(out.as_bytes()).map_err(|e| e.to_string())?;
        f.flush().map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
    }
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

/// Map a message index (as shown in the UI thread) to a user/assistant message
/// in the wire event list. Returns (event_index, event_index_in_messages).
fn visible_message_indices(events: &[WireEvent]) -> Vec<usize> {
    events
        .iter()
        .enumerate()
        .filter_map(|(i, e)| match e {
            WireEvent::UserMessage { .. } | WireEvent::AgentMessage { .. } => Some(i),
            _ => None,
        })
        .collect()
}

fn rebuild_agent_for(path: &std::path::Path, guard: &mut tokio::sync::MutexGuard<'_, Option<NeoCodexAgent>>) {
    if let Some(agent) = guard.as_mut() {
        if agent.wire.path == path {
            agent.rebuild_context_from_wire();
        }
    }
}

/// Edit a user/assistant message in place by its index in the visible thread.
/// Rewrites the JSONL and reloads the agent context so the next turn reflects
/// the correction (Claude-style in-place edit).
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_edit_message(session_id: String, index: usize, content: String) -> Result<Vec<NeoCodexMessageItem>, String> {
    if content.trim().is_empty() {
        return Err("empty message".to_string());
    }
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let mut events = read_wire_events(&path)?;
    let visible = visible_message_indices(&events);
    let target = *visible.get(index).ok_or_else(|| "message not found".to_string())?;
    match &mut events[target] {
        WireEvent::UserMessage { content: c, .. } | WireEvent::AgentMessage { content: c, .. } => {
            *c = content.trim().to_string();
        }
        _ => return Err("target is not a message".to_string()),
    }
    write_wire_events(&path, &events)?;
    let mut guard = NEOCODEX_AGENT.lock().await;
    rebuild_agent_for(&path, &mut guard);
    drop(guard);
    neocodex_get_session_messages(session_id).await
}

/// Delete a message by its index in the visible thread, persisting to the wire
/// (no longer a frontend-only filter that resets on reload).
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_delete_message(session_id: String, index: usize) -> Result<Vec<NeoCodexMessageItem>, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let mut events = read_wire_events(&path)?;
    let visible = visible_message_indices(&events);
    let target = *visible.get(index).ok_or_else(|| "message not found".to_string())?;
    events.remove(target);
    write_wire_events(&path, &events)?;
    let mut guard = NEOCODEX_AGENT.lock().await;
    rebuild_agent_for(&path, &mut guard);
    drop(guard);
    neocodex_get_session_messages(session_id).await
}

/// Regenerate the assistant reply following the user message at `index`:
/// removes that assistant turn (and any trailing tool/system events) from the
/// wire so the frontend can re-send the prompt for a fresh answer.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_regenerate(session_id: String, index: usize) -> Result<Vec<NeoCodexMessageItem>, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let mut events = read_wire_events(&path)?;
    let visible = visible_message_indices(&events);
    let target = *visible.get(index).ok_or_else(|| "message not found".to_string())?;
    // Remove from the target onward: an assistant reply (and any tool/system
    // events produced while generating it) is discarded; the user message stays.
    // `truncate(target)` (not target+1) — truncate keeps [0..target), so the
    // assistant message at `target` and everything after it are discarded,
    // otherwise the stale reply persists in the wire and a re-send duplicates it.
    events.truncate(target);
    write_wire_events(&path, &events)?;
    let mut guard = NEOCODEX_AGENT.lock().await;
    rebuild_agent_for(&path, &mut guard);
    drop(guard);
    neocodex_get_session_messages(session_id).await
}

/// Compact a session: keep the most recent `keep_messages` user/assistant
/// turns and drop everything older (Claude /compact parity). Early tool/system
/// context is trimmed; the newest conversation remains fully intact. Returns
/// the refreshed visible thread.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_compact_session(session_id: String, keep_messages: Option<usize>) -> Result<Vec<NeoCodexMessageItem>, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let events = read_wire_events(&path)?;
    let keep = keep_messages.unwrap_or(8).max(2).min(50);
    let visible = visible_message_indices(&events);
    if visible.len() <= keep {
        return neocodex_get_session_messages(session_id).await;
    }
    let first_keep = visible[visible.len() - keep];
    // Drop everything strictly before the oldest kept visible message.
    let mut kept: Vec<WireEvent> = events[first_keep..].to_vec();
    // Preserve the session metadata/name so the sidebar title survives.
    let meta = events.iter().find(|e| matches!(e, WireEvent::SessionMeta { .. })).cloned();
    if let Some(m) = meta {
        kept.insert(0, m);
    }
    kept.insert(0, WireEvent::SystemEvent {
        kind: "compact".to_string(),
        detail: format!("上下文已压缩：保留了最近 {} 轮对话，更早的消息被截断。", keep),
        timestamp: chrono::Utc::now().timestamp_millis(),
    });
    write_wire_events(&path, &kept)?;
    let mut guard = NEOCODEX_AGENT.lock().await;
    rebuild_agent_for(&path, &mut guard);
    drop(guard);
    neocodex_get_session_messages(session_id).await
}

/// Fetch persisted side-chat messages for a session (branched questions that
/// never re-enter the main context).
#[tauri::command(rename_all = "snake_case")]
/// Read side-chat history from a session wire file, honoring each message's
/// role (P1-2: assistant answers are persisted with role="assistant").
fn read_side_chat(path: &std::path::Path) -> Vec<NeoCodexMessageItem> {
    let mut items = Vec::new();
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            if let Ok(WireEvent::SideChatMessage { content, timestamp, role }) =
                serde_json::from_str::<WireEvent>(line)
            {
                let role = if role.is_empty() { "user" } else { &role };
                items.push(NeoCodexMessageItem {
                    id: items.len(),
                    role: role.to_string(),
                    content,
                    timestamp,
                    attachments: None,
                    tool_call: None,
                });
            }
        }
    }
    items
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_get_side_chat(session_id: String) -> Result<Vec<NeoCodexMessageItem>, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Ok(Vec::new());
    }
    Ok(read_side_chat(&path))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_send_side_chat(session_id: String, content: String) -> Result<Vec<NeoCodexMessageItem>, String> {
    if content.trim().is_empty() {
        return Err("empty side chat message".to_string());
    }
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let mut guard = NEOCODEX_AGENT.lock().await;
    let agent = match guard.as_mut() {
        Some(a) => a,
        None => {
            let mut a = NeoCodexAgent::new("neotrix-tauri");
            a.provider.ensure_production_provider();
            *guard = Some(a);
            guard.as_mut().unwrap()
        }
    };
    // Route through the live agent so the wire's in-memory mirror stays in sync
    // when this is the active session; otherwise record to the target file.
    if agent.wire.path == path {
        agent.record_side_chat(&content, "user");
        // P1-2: generate a real answer (isolated from main context) and
        // persist it as an assistant side-chat event so the UI can show it.
        let answer = agent.side_chat_ask(&content).await;
        agent.record_side_chat(&answer, "assistant");
    } else {
        let mut wire = WireSession::new(&session_id);
        wire.path = path.clone();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        wire.record(WireEvent::SideChatMessage {
            content: content.trim().to_string(),
            timestamp: ts,
            role: "user".to_string(),
        });
        // Non-active session: generate a one-shot answer too.
        wire.record(WireEvent::SideChatMessage {
            content: agent.side_chat_ask(&content).await,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
            role: "assistant".to_string(),
        });
    }
    // Return the full side-chat history for immediate re-render.
    Ok(read_side_chat(&path))
}

/// Persist a user-chosen session name into the session's wire stream.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_rename_session(session_id: String, name: String) -> Result<NeoCodexSessionInfo, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let mut guard = NEOCODEX_AGENT.lock().await;
    let agent = match guard.as_mut() {
        Some(a) => a,
        None => {
            let mut a = NeoCodexAgent::new("neotrix-tauri");
            a.provider.ensure_production_provider();
            *guard = Some(a);
            guard.as_mut().unwrap()
        }
    };
    if agent.wire.path == path {
        agent.rename_session(&name);
    } else {
        let mut wire = WireSession::new(&session_id);
        wire.path = path.clone();
        let existing_tags = read_session_tags_from_path(&path)?;
        wire.record(WireEvent::SessionMeta {
            name: name.trim().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
            tags: existing_tags,
        });
    }
    Ok(NeoCodexSessionInfo {
        id: session_id,
        name: name.trim().to_string(),
        mode: "Agent".to_string(),
        message_count: 0,
        wire_path: path.to_string_lossy().to_string(),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as u64,
        tags: read_session_tags_from_path(&path)?,
    })
}

/// Read persisted tags for a session from its JSONL SessionMeta (last wins).
fn read_session_tags_from_path(path: &std::path::Path) -> Result<Vec<String>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    for line in content.lines().rev() {
        if let Ok(ev) = serde_json::from_str::<WireEvent>(line) {
            if let WireEvent::SessionMeta { tags, .. } = ev {
                return Ok(tags);
            }
        }
    }
    Ok(Vec::new())
}

/// Read the persisted session name from JSONL SessionMeta (last wins).
fn read_session_name_from_path(path: &std::path::Path) -> Result<String, String> {
    if !path.exists() {
        return Ok(path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string());
    }
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    for line in content.lines().rev() {
        if let Ok(ev) = serde_json::from_str::<WireEvent>(line) {
            if let WireEvent::SessionMeta { name, .. } = ev {
                if !name.is_empty() {
                    return Ok(name);
                }
            }
        }
    }
    Ok(path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string())
}

/// Persist a tag onto a session (JSONL SessionMeta). Returns updated tags.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_tag_session(session_id: String, tag: String) -> Result<NeoCodexSessionInfo, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let clean = tag.trim().to_lowercase().replace(' ', "-").replace('#', "");
    if clean.is_empty() {
        return Err("Empty tag".to_string());
    }

    {
        let mut guard = NEOCODEX_AGENT.lock().await;
        let agent = match guard.as_mut() {
            Some(a) => a,
            None => {
                let mut a = NeoCodexAgent::new("neotrix-tauri");
                a.provider.ensure_production_provider();
                *guard = Some(a);
                guard.as_mut().unwrap()
            }
        };
        if agent.wire.path == path {
            agent.tag_session(&clean);
        } else {
            let mut existing = read_session_tags_from_path(&path)?;
            if !existing.contains(&clean) {
                existing.push(clean);
            }
            let mut wire = WireSession::new(&session_id);
            wire.path = path.clone();
            wire.record(WireEvent::SessionMeta {
                name: read_session_name_from_path(&path)?,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
                tags: existing,
            });
        }
    }

    let tags = read_session_tags_from_path(&path)?;
    Ok(NeoCodexSessionInfo {
        id: session_id,
        name: read_session_name_from_path(&path)?,
        mode: "Agent".to_string(),
        message_count: 0,
        wire_path: path.to_string_lossy().to_string(),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as u64,
        tags,
    })
}

/// Remove a tag from a session (JSONL SessionMeta). Returns updated tags.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_untag_session(session_id: String, tag: String) -> Result<NeoCodexSessionInfo, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }

    {
        let mut guard = NEOCODEX_AGENT.lock().await;
        let agent = match guard.as_mut() {
            Some(a) => a,
            None => {
                let mut a = NeoCodexAgent::new("neotrix-tauri");
                a.provider.ensure_production_provider();
                *guard = Some(a);
                guard.as_mut().unwrap()
            }
        };
        if agent.wire.path == path {
            agent.untag_session(&tag);
        } else {
            let mut existing = read_session_tags_from_path(&path)?;
            existing.retain(|t| t != &tag);
            let mut wire = WireSession::new(&session_id);
            wire.path = path.clone();
            wire.record(WireEvent::SessionMeta {
                name: read_session_name_from_path(&path)?,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
                tags: existing,
            });
        }
    }

    let tags = read_session_tags_from_path(&path)?;
    Ok(NeoCodexSessionInfo {
        id: session_id,
        name: read_session_name_from_path(&path)?,
        mode: "Agent".to_string(),
        message_count: 0,
        wire_path: path.to_string_lossy().to_string(),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as u64,
        tags,
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_set_mode(mode: String) -> Result<String, String> {
    let parsed = match mode.as_str() {
        "Agent" => NeoCodexMode::Agent,
        "Shell" => NeoCodexMode::Shell,
        "Plan" => NeoCodexMode::Plan,
        _ => return Err(format!("unknown mode {}", mode)),
    };
    let mut guard = NEOCODEX_AGENT.lock().await;
    let agent = match guard.as_mut() {
        Some(a) => a,
        None => {
            let mut a = NeoCodexAgent::new("neotrix-tauri");
            a.provider.ensure_production_provider();
            *guard = Some(a);
            guard.as_mut().unwrap()
        }
    };
    agent.set_mode(parsed);
    Ok(format!("mode set to {}", mode))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_switch_session(session_id: String) -> Result<String, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let mut guard = NEOCODEX_AGENT.lock().await;
    let agent = match guard.as_mut() {
        Some(a) => a,
        None => {
            let mut a = NeoCodexAgent::new("neotrix-tauri");
            a.provider.ensure_production_provider();
            *guard = Some(a);
            guard.as_mut().unwrap()
        }
    };
    agent.wire.path = path.clone();
    // Fix cross-session context bleed: switching must clear the prior
    // session's in-memory context/tokens (rebuild clears) else A's turns leak
    // into B. `resume_session` only appends; `rebuild_context_from_wire`
    // clears then restores.
    let count = agent.rebuild_context_from_wire();
    Ok(format!("Switched to session {} (restored {} events)", session_id, count))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_delete_session(session_id: String) -> Result<String, String> {
    let path = session_path(&session_id);
    let mut deleted = false;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        deleted = true;
    }
    let archived = archived_session_path(&session_id);
    if archived.exists() {
        std::fs::remove_file(&archived).map_err(|e| e.to_string())?;
        deleted = true;
    }
    if !deleted {
        return Err("Session not found".to_string());
    }
    // P2-3: same detach as archive — stale wire.path must not resurrect the
    // deleted file on the next record().
    let mut guard = NEOCODEX_AGENT.lock().await;
    if let Some(agent) = guard.as_mut() {
        if agent.wire.path == path || agent.wire.path == archived {
            agent.detach_wire();
        }
    }
    drop(guard);
    Ok(format!("Deleted session {}", session_id))
}

fn archived_dir() -> std::path::PathBuf {
    sessions_dir().join("archived")
}

/// P1-3: archived paths must sanitize the id exactly like `session_path`;
/// joining the raw id previously allowed LFI path traversal out of the
/// sessions archive (delete/archive/restore could escape to arbitrary files).
fn archived_session_path(session_id: &str) -> std::path::PathBuf {
    sanitize_session_id(session_id)
        .map(|s| archived_dir().join(format!("{}.jsonl", s)))
        .unwrap_or_else(|| archived_dir().join("__invalid__.jsonl"))
}

fn checkpoints_dir() -> std::path::PathBuf {
    sessions_dir().join("checkpoints")
}

/// Non-locking snapshot of an in-memory-only wire path. Used by
/// `neocodex_send_message_stream` (which already holds the agent lock) to
/// auto-create a checkpoint after each non-regenerate turn — this is what
/// makes the timeline's "每次发送消息会自动创建" claim true.
fn snapshot_checkpoint(path: &std::path::Path, session_id: &str) {
    if !path.exists() {
        return;
    }
    let dir = checkpoints_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    let dest = dir.join(format!("{}-{}.jsonl", session_id, ts));
    let _ = std::fs::copy(path, &dest);
}

fn list_checkpoints_inner(session_id: &str) -> Vec<serde_json::Value> {
    let dir = checkpoints_dir();
    let mut list = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        let prefix = format!("{}-", session_id);
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if !fname.starts_with(&prefix) || !fname.ends_with(".jsonl") {
                continue;
            }
            let ts: u64 = fname
                .trim_start_matches(&prefix)
                .trim_end_matches(".jsonl")
                .parse()
                .unwrap_or(0);
            let message_count = std::fs::read_to_string(dir.join(&fname))
                .map(|s| s.lines().count())
                .unwrap_or(0);
            list.push(serde_json::json!({
                "id": fname,
                "created_at": ts,
                "message_count": message_count
            }));
        }
    }
    // P2-6: newest-first. The frontend labels index 0 as "最新" — with an
    // ascending sort that mislabeled the OLDEST checkpoint instead.
    list.sort_by_key(|v| std::cmp::Reverse(v["created_at"].as_u64().unwrap_or(0)));
    list
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_checkpoint_list(session_id: String) -> Result<Vec<serde_json::Value>, String> {
    Ok(list_checkpoints_inner(&session_id))
}

/// Rewind a session to a checkpoint: replaces the active wire file with the
/// snapshot and rebuilds the agent context so the next turn continues from
/// that point (Claude `/rewind` parity, code + conversation).
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_checkpoint_restore(session_id: String, checkpoint_id: String) -> Result<Vec<NeoCodexMessageItem>, String> {
    // Anti-traversal: checkpoint_id must match our generated naming.
    let safe: String = checkpoint_id.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if safe.is_empty() || safe != checkpoint_id || !checkpoint_id.ends_with(".jsonl") {
        return Err("Invalid checkpoint id".to_string());
    }
    let src = checkpoints_dir().join(&checkpoint_id);
    if !src.exists() {
        return Err("Checkpoint not found".to_string());
    }
    let path = session_path(&session_id);
    std::fs::copy(&src, &path).map_err(|e| e.to_string())?;
    let mut guard = NEOCODEX_AGENT.lock().await;
    rebuild_agent_for(&path, &mut guard);
    drop(guard);
    neocodex_get_session_messages(session_id).await
}

/// Move a session's wire file into the archived/ subfolder (Claude-style
/// "Archive" — keeps history without cluttering the active list).
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_archive_session(session_id: String) -> Result<String, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let dir = archived_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = archived_session_path(&session_id);
    std::fs::rename(&path, &dest).map_err(|e| e.to_string())?;
    // P2-3: if the active agent pointed at this wire, detach so the next
    // `record()` cannot resurrect the moved/deleted file (create+append split).
    let mut guard = NEOCODEX_AGENT.lock().await;
    if let Some(agent) = guard.as_mut() {
        if agent.wire.path == path {
            agent.detach_wire();
        }
    }
    drop(guard);
    Ok(format!("Archived session {}", session_id))
}

/// Move a session back from archived/ into the active list.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_restore_session(session_id: String) -> Result<String, String> {
    let src = archived_session_path(&session_id);
    if !src.exists() {
        return Err("Archived session not found".to_string());
    }
    let dest = session_path(&session_id);
    std::fs::rename(&src, &dest).map_err(|e| e.to_string())?;
    Ok(format!("Restored session {}", session_id))
}

/// List archived sessions (same metadata shape as active ones).
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_list_archived() -> Result<Vec<NeoCodexSessionInfo>, String> {
    let dir = archived_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let mut name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
            let mut message_count = 0;
            let mut updated_at = entry.metadata().map_err(|e| e.to_string())?.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
            for line in content.lines() {
                if let Ok(WireEvent::UserMessage { content: c, timestamp, .. }) = serde_json::from_str::<WireEvent>(line) {
                    if message_count == 0 && c.len() > 20 {
                        name = format!("{}...", &c[..c.floor_char_boundary(20)]);
                    }
                    message_count += 1;
                    updated_at = timestamp.max(0) as u64;
                } else if let Ok(WireEvent::AgentMessage { .. }) = serde_json::from_str::<WireEvent>(line) {
                    message_count += 1;
                }
            }
            sessions.push(NeoCodexSessionInfo {
                id: path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string(),
                name,
                mode: "Agent".to_string(),
                message_count,
                wire_path: path.to_string_lossy().to_string(),
                updated_at,
                tags: read_session_tags_from_path(&path).unwrap_or_default(),
            });
        }
    }
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(sessions)
}

#[derive(serde::Serialize)]
pub struct NeoCodexSessionInfo {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub message_count: usize,
    pub wire_path: String,
    pub updated_at: u64,
    /// 会话标签（JSONL SessionMeta 持久化，对标 Obsidian tag）
    pub tags: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct NeoCodexMessageItem {
    pub id: usize,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
    pub attachments: Option<Vec<NeoCodexAttachmentDto>>,
    /// Structured tool call (role == "tool"). Lets the frontend render a real
    /// tool card instead of the legacy markdown blob.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<NeoCodexToolCallDto>,
}

#[derive(serde::Serialize)]
pub struct NeoCodexToolCallDto {
    pub name: String,
    pub args: String,
    pub result: String,
    pub duration_ms: u64,
    pub success: bool,
}

#[derive(serde::Serialize)]
pub struct NeoCodexAttachmentDto {
    pub name: String,
    pub size: u64,
    pub mime_type: String,
    pub data: Option<String>,
}

/// Recursively search the workspace for files/dirs matching `query` (for the
/// @-autocomplete in the composer). Skips heavy/dependency directories and
/// caps results to keep the menu snappy.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_search_files(query: String) -> Result<Vec<String>, String> {
    let q = query.trim().to_lowercase();
    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    let mut stack = vec![root.clone()];
    let mut visited: usize = 0;
    const MAX_DIRS: usize = 600;
    const MAX_RESULTS: usize = 60;
    const SKIP: &[&str] = &["node_modules", ".git", "target", "dist", "build", ".next", ".nuxt", "__pycache__", ".venv", "vendor", ".dart_tool", "Pods"];

    while let Some(dir) = stack.pop() {
        if visited > MAX_DIRS || results.len() >= MAX_RESULTS {
            break;
        }
        visited += 1;
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if SKIP.contains(&name.as_str()) {
                continue;
            }
            let path = entry.path();
            let rel = path.strip_prefix(&root).unwrap_or(&path).to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let matched = q.is_empty() || name.to_lowercase().contains(&q) || rel.to_lowercase().contains(&q);
            if matched && rel.len() <= 200 {
                results.push(if is_dir { format!("{}/", rel) } else { rel });
                if results.len() >= MAX_RESULTS {
                    break;
                }
            }
            if is_dir {
                stack.push(path);
            }
        }
    }
    results.sort();
    Ok(results)
}

#[tauri::command(rename_all = "snake_case")]
pub fn neocodex_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[derive(serde::Serialize)]
pub struct ProjectTreeItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<ProjectTreeItem>>,
}

#[derive(serde::Serialize)]
pub struct ProjectView {
    pub root: String,
    pub tree: Vec<ProjectTreeItem>,
    pub agents_md: Option<String>,
    pub file_count: usize,
}

/// Build a bounded directory tree for the project view panel. Skips heavy
/// dependency/vcs dirs, caps breadth (MAX_ENTRIES_PER_DIR) and depth (MAX_DEPTH)
/// so huge monorepos stay responsive. Also returns AGENTS.md if present.
#[tauri::command(rename_all = "snake_case")]
pub fn neocodex_project_tree() -> Result<ProjectView, String> {
    const MAX_DEPTH: usize = 6;
    const MAX_ENTRIES_PER_DIR: usize = 80;
    const SKIP: &[&str] = &["node_modules", ".git", "target", "dist", "build", ".next", ".nuxt", "__pycache__", ".venv", "vendor", ".dart_tool", "Pods", "Pods.xcworkspace", ".svelte-kit", ".turbo", ".cache"];

    let root = std::env::current_dir().map_err(|e| e.to_string())?;

    fn build_dir(dir: &std::path::Path, depth: usize, cap: &mut usize, files: &mut usize) -> Vec<ProjectTreeItem> {
        if depth > MAX_DEPTH || *cap <= 0 {
            return Vec::new();
        }
        let mut items = Vec::new();
        let mut entries: Vec<_> = match std::fs::read_dir(dir) {
            Ok(e) => e.flatten().collect(),
            Err(_) => return items,
        };
        // Directories first, then files, both sorted.
        entries.sort_by_key(|e| {
            let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
            (!is_dir, e.file_name().to_string_lossy().to_lowercase().to_string())
        });
        for entry in entries.into_iter().take(MAX_ENTRIES_PER_DIR) {
            let name = entry.file_name().to_string_lossy().to_string();
            if SKIP.contains(&name.as_str()) {
                continue;
            }
            let path = entry.path();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let rel = path.strip_prefix(dir).unwrap_or(&path).to_string_lossy().to_string();
            if is_dir {
                if *cap == 0 {
                    // 容量已耗尽: 跳过该子目录 (避免 *cap -= 1 下溢 panic)
                    continue;
                }
                *cap -= 1;
                let children = build_dir(&path, depth + 1, cap, files);
                items.push(ProjectTreeItem {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_dir: true,
                    children: Some(children),
                });
            } else {
                *files += 1;
                items.push(ProjectTreeItem {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_dir: false,
                    children: None,
                });
            }
        }
        items
    }

    let mut cap = 300;
    let mut file_count = 0usize;
    let tree = build_dir(&root, 0, &mut cap, &mut file_count);

    // AGENTS.md (project constitution) content for the panel.
    let agents_md = std::fs::read_to_string(root.join("AGENTS.md")).ok();

    Ok(ProjectView {
        root: root.to_string_lossy().to_string(),
        tree,
        agents_md,
        file_count,
    })
}

#[derive(serde::Serialize)]
pub struct UpdateCheckResult {
    pub current: String,
    pub available: bool,
    pub latest: String,
    pub error: Option<String>,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_check_update(app: tauri::AppHandle) -> Result<UpdateCheckResult, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let updater = app.updater().map_err(|e| e.to_string());
    let updater = match updater {
        Ok(u) => u,
        Err(e) => {
            return Ok(UpdateCheckResult {
                current: current.clone(),
                available: false,
                latest: current.clone(),
                error: Some(e),
            });
        }
    };
    let update = updater.check().await.map_err(|e| e.to_string());
    match update {
        Ok(Some(update)) => {
            let latest = update.version.to_string();
            Ok(UpdateCheckResult {
                current: current.clone(),
                available: true,
                latest,
                error: None,
            })
        }
        Ok(None) => Ok(UpdateCheckResult {
            current: current.clone(),
            available: false,
            latest: current.clone(),
            error: None,
        }),
        Err(e) => Ok(UpdateCheckResult {
            current: current.clone(),
            available: false,
            latest: current.clone(),
            error: Some(e.to_string()),
        }),
    }
}

/// Report the current git branch + dirty state for the status bar (parity
/// with Claude Code / Codex Desktop, both of which surface the branch inline).
#[tauri::command(rename_all = "snake_case")]
pub fn neocodex_git_status() -> Result<Option<GitStatus>, String> {
    fn git(args: &[&str]) -> Option<String> {
        let out = std::process::Command::new("git")
            .args(args)
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }
    let branch = match git(&["rev-parse", "--abbrev-ref", "HEAD"]) {
        Some(b) => b,
        None => return Ok(None),
    };
    let is_dirty = git(&["status", "--porcelain"])
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    Ok(Some(GitStatus { branch, dirty: is_dirty }))
}

#[derive(serde::Serialize)]
pub struct GitStatus {
    pub branch: String,
    pub dirty: bool,
}

/// /init — scaffold AGENTS.md with project structure + conventions.
/// Returns the generated markdown so the frontend can insert it or save it.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_init_project(session_id: String) -> Result<String, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let mut wire = WireSession::new(&session_id);
    wire.path = path;
    let _ = wire.load(); // load events to verify access
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut structure = Vec::new();
    fn walk(dir: &std::path::Path, cwd: &std::path::Path, out: &mut Vec<String>, depth: usize) {
        if depth > 4 { return; }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "target" || name == "node_modules" || name == ".git" || name == "dist" {
                    continue;
                }
                let rel = p.strip_prefix(cwd).unwrap_or(&p).to_string_lossy().to_string();
                if p.is_dir() {
                    out.push(format!("{}/", rel));
                    walk(&p, cwd, out, depth + 1);
                } else {
                    out.push(rel);
                }
            }
        }
    }
    walk(&cwd, &cwd, &mut structure, 0);
    structure.sort();
    let structure_md = structure.join("\n");
    let md = format!(r#"# AGENTS.md

This file guides AI coding agents working in this repository.

## Project Structure
```text
{structure_md}
```

## Conventions
- Use the existing patterns and libraries in this codebase.
- Run `cargo check` / `npm run lint` before committing.
- Tests: `cargo test -p neotrix --lib` and `npm test` in frontend.

## Agent Instructions
- Prefer native file tools (read/write/edit) over shell escapes.
- Ask before running destructive commands.
- Summarize changes in a single commit message.
"#);
    Ok(md)
}

/// /export — export current session as Markdown (reuse SessionSidebar logic).
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_export_session(session_id: String, format: Option<String>) -> Result<String, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut msgs = Vec::new();
    for line in content.lines() {
        if let Ok(ev) = serde_json::from_str::<WireEvent>(line) {
            match ev {
                WireEvent::UserMessage { content, timestamp, attachments: _ } => {
                    msgs.push(format!("[User {}]: {}", timestamp, content));
                }
                WireEvent::AgentMessage { content, timestamp } => {
                    msgs.push(format!("[Assistant {}]: {}", timestamp, content));
                }
                WireEvent::ToolCall { name, args, result, success, .. } => {
                    msgs.push(format!("[Tool {} {}] args={} result={}", name, if success { "✓" } else { "✗" }, args, result));
                }
                WireEvent::SystemEvent { kind, detail, timestamp } => {
                    msgs.push(format!("[System {}] {}: {}", timestamp, kind, detail));
                }
                WireEvent::SideChatMessage { content, timestamp, role } => {
                    msgs.push(format!("[{} {}]: {}", role, timestamp, content));
                }
                _ => {}
            }
        }
    }
    if format.as_deref() == Some("json") {
        return Ok(serde_json::to_string_pretty(&msgs).unwrap_or_default());
    }
    Ok(msgs.join("\n\n"))
}

/// /clear — wipe all messages from the current session wire (keep file for continuity).
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_clear_session(session_id: String) -> Result<String, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    // Keep SessionMeta if present, drop everything else
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut keep = Vec::new();
    for line in content.lines() {
        if let Ok(ev) = serde_json::from_str::<WireEvent>(line) {
            if matches!(ev, WireEvent::SessionMeta { .. }) {
                keep.push(line.to_string());
            }
        }
    }
    let events = keep.iter().map(|l| serde_json::from_str::<WireEvent>(l).unwrap()).collect::<Vec<_>>();
    write_wire_events(&path, &events)?;
    // If active agent points here, rebuild
    let mut guard = NEOCODEX_AGENT.lock().await;
    if let Some(agent) = guard.as_mut() {
        if agent.wire.path == path {
            agent.rebuild_context_from_wire();
        }
    }
    Ok(format!("Cleared session {} (kept metadata)", session_id))
}

/// /feedback — record user feedback as a SystemEvent for telemetry.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_feedback(session_id: String, text: String) -> Result<String, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    let mut wire = WireSession::new(&session_id);
    wire.path = path.clone();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    wire.record(WireEvent::SystemEvent {
        kind: "feedback".to_string(),
        detail: text.trim().to_string(),
        timestamp: ts,
    });
    // If active agent points here, append to its in-memory mirror
    let mut guard = NEOCODEX_AGENT.lock().await;
    if let Some(agent) = guard.as_mut() {
        if agent.wire.path == path {
            agent.wire.events.push(WireEvent::SystemEvent {
                kind: "feedback".to_string(),
                detail: text.trim().to_string(),
                timestamp: ts,
            });
        }
    }
    Ok("Feedback recorded".to_string())
}

/// Download and install a pending update. Emits `neocodex_update_progress`
/// {downloaded_bytes, total_bytes?} as chunks arrive, then triggers the
/// native relaunch once the new bundle is staged.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_download_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;
    let Some(update) = update else {
        return Err("没有可用的更新".into());
    };
    let app_handle = app.clone();
    let app_handle2 = app.clone();
    update
        .download_and_install(
            move |downloaded, total| {
                let _ = app_handle.emit(
                    "neocodex_update_progress",
                    serde_json::json!({
                        "downloaded": downloaded,
                        "total": total,
                    }),
                );
            },
            move || {
                let _ = app_handle2.emit("neocodex_update_downloaded", ());
            },
        )
        .await
        .map_err(|e| format!("更新安装失败: {e}"))?;
    // Relaunch so the new version takes effect (updater staged the bundle).
    app.restart();
    Ok(())
}

/// Get per-file diffs for the active neocodex session's working tree.
/// Returns the unified frontend contract:
/// `{ "files": [ { "path": "...", "hunks": [ { "lines": [
///    { "t": "ctx"|"del"|"add", "o": old_line|0, "n": new_line|0, "s": content } ] } ] } ] }`
/// so the chat's diff panel can render real changes (Claude/Codex parity).
#[tauri::command(rename_all = "snake_case")]
pub fn neocodex_get_diff() -> Result<serde_json::Value, String> {
    use std::process::Command;
    let out = Command::new("git")
        .args(["diff", "--name-only"])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("git diff failed".into());
    }
    let files: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|s| s.to_string())
        .collect();

    let mut result: Vec<serde_json::Value> = Vec::new();
    for file in &files {
        let diff_out = Command::new("git")
            .args(["diff", "HEAD", "--", file])
            .output()
            .map_err(|e| e.to_string())?;
        let diff_text = String::from_utf8_lossy(&diff_out.stdout).to_string();
        let hunks = parse_unified_diff_rich(&diff_text);
        result.push(serde_json::json!({ "path": file, "hunks": hunks }));
    }

    // Also include untracked files as a single all-added hunk
    let untracked_out = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| e.to_string())?;
    let untracked: Vec<String> = String::from_utf8_lossy(&untracked_out.stdout)
        .lines()
        .filter(|l| l.starts_with("??"))
        .map(|l| l[3..].trim().to_string())
        .collect();

    for file in &untracked {
        if let Ok(content) = std::fs::read_to_string(file) {
            let lines: Vec<serde_json::Value> = content
                .lines()
                .enumerate()
                .map(|(i, l)| serde_json::json!({ "t": "add", "o": null, "n": i + 1, "s": l }))
                .collect();
            result.push(serde_json::json!({ "path": file, "hunks": [ { "lines": lines } ] }));
        }
    }

    Ok(serde_json::json!({ "files": result }))
}

/// Parse a unified git diff into frontend hunk/line rows with old & new line
/// numbers so the panel can render +/- badges and per-line comments.
/// Header format: `@@ -old_start[,old_count] +new_start[,new_count] @@`
fn commit_header(num: &str, is_old: bool, old_nr: &mut u64, new_nr: &mut u64) {
    if !num.is_empty() {
        if let Ok(v) = num.parse() {
            if is_old { *old_nr = v; } else { *new_nr = v; }
        }
    }
}

fn parse_unified_diff_rich(diff_text: &str) -> Vec<serde_json::Value> {
    let mut hunks: Vec<serde_json::Value> = Vec::new();
    let mut lines: Vec<serde_json::Value> = Vec::new();
    let mut old_nr: u64 = 0;
    let mut new_nr: u64 = 0;

    for raw in diff_text.lines() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if let Some(rest) = line.strip_prefix("@@") {
            if !lines.is_empty() {
                hunks.push(serde_json::json!({ "lines": std::mem::take(&mut lines) }));
            }
            // extract the two numeric starts: -A[,B] and +C[,D]
            let mut num = String::new();
            let mut is_old = true;
            let mut skip_count = false;
            for ch in rest.chars() {
                match ch {
                    '-' => { commit_header(&num, is_old, &mut old_nr, &mut new_nr); is_old = true; num.clear(); skip_count = false; }
                    '+' => { commit_header(&num, is_old, &mut old_nr, &mut new_nr); is_old = false; num.clear(); skip_count = false; }
                    '0'..='9' if !skip_count => num.push(ch),
                    ',' => { commit_header(&num, is_old, &mut old_nr, &mut new_nr); num.clear(); skip_count = true; }
                    _ => { commit_header(&num, is_old, &mut old_nr, &mut new_nr); num.clear(); }
                }
            }
            commit_header(&num, is_old, &mut old_nr, &mut new_nr);
            old_nr = old_nr.max(1);
            new_nr = new_nr.max(1);
            continue;
        }
        if let Some(s) = line.strip_prefix('+') {
            if !s.starts_with('+') {
                lines.push(serde_json::json!({ "t": "add", "o": null, "n": new_nr, "s": s }));
                new_nr += 1;
                continue;
            }
        }
        if let Some(s) = line.strip_prefix('-') {
            if !s.starts_with('-') {
                lines.push(serde_json::json!({ "t": "del", "o": old_nr, "n": null, "s": s }));
                old_nr += 1;
                continue;
            }
        }
        if line.starts_with("diff") || line.starts_with("index")
            || line.starts_with("---") || line.starts_with("+++") || line.starts_with("\\ ") {
            continue;
        }
        lines.push(serde_json::json!({ "t": "ctx", "o": old_nr, "n": new_nr, "s": line }));
        old_nr += 1;
        new_nr += 1;
    }
    if !lines.is_empty() {
        hunks.push(serde_json::json!({ "lines": lines }));
    }
    hunks
}

/// Apply (accept) or reject a file's diff in the neocodex session.
/// action: "accept" stages the file (git add), "reject" restores it (git restore).
#[tauri::command(rename_all = "snake_case")]
pub fn neocodex_apply_diff(path: String, action: String) -> Result<(), String> {
    use std::process::Command;
    match action.as_str() {
        "accept" => {
            let out = Command::new("git")
                .args(["add", "--", &path])
                .output()
                .map_err(|e| e.to_string())?;
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                return Err(stderr.trim().to_string());
            }
        }
        "reject" => {
            // Check if untracked
            let porcelain = Command::new("git")
                .args(["status", "--porcelain", "--", &path])
                .output()
                .map_err(|e| e.to_string())?;
            let is_untracked = String::from_utf8_lossy(&porcelain.stdout)
                .lines()
                .any(|l| l.starts_with("??"));

            if is_untracked {
                let _ = std::fs::remove_file(&path);
            } else {
                let out = Command::new("git")
                    .args(["restore", "--staged", "--worktree", "--source=HEAD", "--", &path])
                    .output()
                    .map_err(|e| e.to_string())?;
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    return Err(stderr.trim().to_string());
                }
            }
        }
        _ => return Err(format!("Invalid action: {}", action)),
    }
    Ok(())
}

fn parse_neocodex_diff(diff_str: &str) -> Vec<crate::commands::DiffBlock> {
    let mut blocks = Vec::new();
    for line in diff_str.lines() {
        if let Some(stripped) = line.strip_prefix('+') {
            if !stripped.starts_with('+') {
                blocks.push(crate::commands::DiffBlock {
                    r#type: "added".into(),
                    content: stripped.to_string(),
                    line_start: 0,
                });
                continue;
            }
        }
        if let Some(stripped) = line.strip_prefix('-') {
            if !stripped.starts_with('-') {
                blocks.push(crate::commands::DiffBlock {
                    r#type: "removed".into(),
                    content: stripped.to_string(),
                    line_start: 0,
                });
                continue;
            }
        }
        if !line.starts_with("diff")
            && !line.starts_with("index")
            && !line.starts_with("---")
            && !line.starts_with("+++")
            && !line.starts_with("@@")
            && !line.starts_with("\\ ")
        {
            blocks.push(crate::commands::DiffBlock {
                r#type: "unchanged".into(),
                content: line.to_string(),
                line_start: 0,
            });
        }
    }
    blocks
}

/// Open a file in the internal editor by dispatching a frontend event.
#[tauri::command(rename_all = "snake_case")]
pub fn neocodex_open_file(app: tauri::AppHandle, path: String) -> Result<(), String> {
    app.emit("neocodex:open-file", path).map_err(|e| e.to_string())
}

/// Open a file in the external editor (OS default).
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_open_external(path: String) -> Result<(), String> {
    let status = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd").args(["/C", "start", ""]).arg(&path).status()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(&path).status()
    } else {
        std::process::Command::new("xdg-open").arg(&path).status()
    };
    status.map_err(|e| format!("failed to launch external opener: {e}"))?;
    Ok(())
}

/// Get git status for all files in the repo (porcelain format).
#[tauri::command(rename_all = "snake_case")]
pub fn neocodex_git_file_status(cwd: Option<String>) -> Result<Vec<GitFileStatus>, String> {
    use std::process::Command;
    let cwd = cwd.unwrap_or_else(|| std::env::current_dir().unwrap_or_default().to_string_lossy().to_string());
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&cwd)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(stderr.trim().to_string());
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut statuses = Vec::new();
    for line in stdout.lines() {
        if line.len() < 3 {
            continue;
        }
        let status = &line[0..2];
        let path = line[3..].trim().to_string();
        statuses.push(GitFileStatus {
            path,
            status: status.to_string(),
        });
    }
    Ok(statuses)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GitFileStatus {
    pub path: String,
    pub status: String,
}

/// File operations: create file, create folder, delete, rename.
#[tauri::command(rename_all = "snake_case")]
pub async fn neocodex_file_operation(op: String, path: String, new_name: Option<String>) -> Result<(), String> {
    use std::fs;
    use std::path::Path;
    let p = Path::new(&path);
    match op.as_str() {
        "new_file" => {
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::File::create(p).map_err(|e| e.to_string())?;
        }
        "new_folder" => {
            fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        "delete" => {
            if p.is_dir() {
                fs::remove_dir_all(p).map_err(|e| e.to_string())?;
            } else {
                fs::remove_file(p).map_err(|e| e.to_string())?;
            }
        }
        "rename" => {
            let new_name = new_name.ok_or("new_name required for rename")?;
            let new_path = p.with_file_name(new_name);
            fs::rename(p, new_path).map_err(|e| e.to_string())?;
        }
        _ => return Err(format!("Invalid operation: {}", op)),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// neocodex_project_tree 应返回当前目录树，跳过依赖/VCS 目录，且包含 AGENTS.md。
    #[test]
    fn test_project_tree_returns_bounded_tree() {
        let pv = neocodex_project_tree().expect("project tree should build");
        assert!(!pv.root.is_empty(), "root should be set");
        assert!(pv.file_count > 0, "should find at least one file");
        // 顶层不应包含被跳过的目录
        let names: Vec<&str> = pv.tree.iter().map(|i| i.name.as_str()).collect();
        assert!(!names.contains(&"node_modules"), "node_modules must be skipped");
        assert!(!names.contains(&".git"), ".git must be skipped");
        assert!(!names.contains(&"target"), "target must be skipped");
        // 目录项应带 children，文件项不带
        for item in &pv.tree {
            if item.is_dir {
                assert!(item.children.is_some(), "dir {} should have children", item.name);
            } else {
                assert!(item.children.is_none(), "file {} should have no children", item.name);
            }
        }
    }

    /// AGENTS.md 若存在于项目根则应被读取。
    #[test]
    fn project_tree_reads_agents_md() {
        let pv = neocodex_project_tree().expect("project tree should build");
        let root_has_agents = std::path::Path::new(&pv.root).join("AGENTS.md").exists();
        if root_has_agents {
            let agents = pv.agents_md.expect("agents_md should be Some when AGENTS.md exists");
            assert!(!agents.trim().is_empty(), "AGENTS.md content should not be empty");
        } else {
            // 测试 cwd 可能不在仓库根（如 src-tauri/），此时 AGENTS.md 不存在属正常
            assert!(pv.agents_md.is_none(), "no AGENTS.md in cwd -> agents_md should be None");
        }
    }
}
