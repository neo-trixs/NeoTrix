use neotrix::neotrix::l1_body_impl::nt_io_neocodex::{
    NeoCodexAgent, NeoCodexHealthReport, NeoCodexMode, EvolutionLoop, WireSession, WireEvent,
};
use serde::Serialize;
use tauri::Emitter;
use tauri_plugin_updater::UpdaterExt;

static NEOCODEX_AGENT: std::sync::LazyLock<tokio::sync::Mutex<Option<NeoCodexAgent>>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(None));

#[tauri::command]
pub async fn neocodex_send_message(content: String) -> Result<String, String> {
    let mut agent_guard = NEOCODEX_AGENT.lock().await;
    let agent = match agent_guard.as_mut() {
        Some(a) => a,
        None => {
            let mut a = NeoCodexAgent::new("neotrix-tauri");
            a.provider.sync_from_real();
            *agent_guard = Some(a);
            agent_guard.as_mut().unwrap()
        }
    };

    let response = agent.process(&content).await;
    Ok(response)
}

#[derive(serde::Deserialize, Default)]
pub struct NeoCodexAttachmentPayload {
    pub name: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub mime_type: String,
}

#[tauri::command]
pub async fn neocodex_send_message_stream(
    app: tauri::AppHandle,
    content: String,
    attachments: Option<Vec<NeoCodexAttachmentPayload>>,
) -> Result<String, String> {
    let mut agent_guard = NEOCODEX_AGENT.lock().await;
    let agent = match agent_guard.as_mut() {
        Some(a) => a,
        None => {
            let mut a = NeoCodexAgent::new("neotrix-tauri");
            a.provider.sync_from_real();
            *agent_guard = Some(a);
            agent_guard.as_mut().unwrap()
        }
    };

    // Persist the user message (fixes streaming-not-persisted gap) with any
    // attachments, keeping the wire + in-memory mirror in sync.
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let core_atts = attachments.map(|v| {
        v.into_iter().filter(|a| !a.name.is_empty()).map(|a| {
            neotrix::neotrix::l1_body_impl::nt_io_neocodex::NeoCodexAttachment {
                name: a.name,
                size: a.size,
                mime_type: a.mime_type,
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

    // Emit start event
    let _ = app.emit("neocodex_stream_start", content.clone());

    let result = agent.react_loop_stream(&content, 4, |token| {
        let _ = app.emit("neocodex_stream_token", token);
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

    let _ = app.emit("neocodex_stream_end", answer.clone());
    Ok(answer)
}

#[tauri::command]
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
        }),
    }
}

#[tauri::command]
pub async fn neocodex_evolution_step() -> Result<String, String> {
    let mut guard = NEOCODEX_AGENT.lock().await;
    match guard.as_mut() {
        Some(a) => {
            EvolutionLoop::step(a);
            Ok(format!(
                "iteration {} ({} fixes applied total)",
                a.evolution.iteration, a.evolution.fixes_applied
            ))
        }
        None => Err("agent not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn neocodex_resume() -> Result<usize, String> {
    let mut guard = NEOCODEX_AGENT.lock().await;
    match guard.as_mut() {
        Some(a) => Ok(a.resume_session()),
        None => Err("agent not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn neocodex_mode_toggle() -> Result<String, String> {
    let mut guard = NEOCODEX_AGENT.lock().await;
    match guard.as_mut() {
        Some(a) => {
            let mode = a.toggle_mode();
            Ok(format!("{:?}", mode))
        }
        None => Err("agent not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn neocodex_add_goal(desc: String, max_iter: u64) -> Result<String, String> {
    let mut guard = NEOCODEX_AGENT.lock().await;
    match guard.as_mut() {
        Some(a) => {
            a.add_goal(&desc, max_iter);
            Ok(format!("goal added: {} (max {} iters)", desc, max_iter))
        }
        None => Err("agent not initialized".to_string()),
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

#[tauri::command]
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

#[tauri::command]
pub async fn neocodex_set_provider(name: String) -> Result<String, String> {
    let mut guard = NEOCODEX_AGENT.lock().await;
    let agent = match guard.as_mut() {
        Some(a) => a,
        None => {
            let mut a = NeoCodexAgent::new("neotrix-tauri");
            a.provider.sync_from_real();
            *guard = Some(a);
            guard.as_mut().unwrap()
        }
    };
    if agent.provider.set_active_provider(&name) {
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

fn session_path(session_id: &str) -> std::path::PathBuf {
    sessions_dir().join(format!("{}.jsonl", session_id))
}

#[tauri::command]
pub async fn neocodex_list_sessions() -> Result<Vec<NeoCodexSessionInfo>, String> {
    let dir = sessions_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let lines: Vec<&str> = content.lines().collect();
            let mut name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
            let mut mode = "Agent".to_string();
            let mut message_count = 0;
            let mut updated_at = entry.metadata().map_err(|e| e.to_string())?.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
            
            for line in &lines {
                if let Ok(event) = serde_json::from_str::<WireEvent>(line) {
                    match event {
                        WireEvent::UserMessage { content, timestamp, .. } => {
                            if message_count == 0 && content.len() > 20 {
                                name = format!("{}...", &content[..20]);
                            }
                            message_count += 1;
                            updated_at = timestamp.max(0) as u64;
                        }
                        WireEvent::ModeChange { to, .. } => {
                            mode = format!("{:?}", to);
                        }
                        WireEvent::SessionMeta { name: n, timestamp, .. } => {
                            if !n.is_empty() {
                                name = n;
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
            });
        }
    }
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(sessions)
}

#[tauri::command]
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
    })
}

#[tauri::command]
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
                    role: "user".to_string(),
                    content,
                    timestamp,
                    attachments: attachments.map(|v| v.into_iter().map(|a| NeoCodexAttachmentDto {
                        name: a.name,
                        size: a.size,
                        mime_type: a.mime_type,
                    }).collect()),
                }),
                WireEvent::AgentMessage { content, timestamp } => items.push(NeoCodexMessageItem {
                    role: "assistant".to_string(),
                    content,
                    timestamp,
                    attachments: None,
                }),
                WireEvent::ToolCall { name, args, result, success, .. } => items.push(NeoCodexMessageItem {
                    role: "tool".to_string(),
                    content: format!("**{}**{}\n```\n{}\n```", name, if success { "" } else { " (失败)" }, result.chars().take(500).collect::<String>()),
                    timestamp: 0,
                    attachments: None,
                }),
                WireEvent::SystemEvent { kind, detail, timestamp } => {
                    if !kind.is_empty() {
                        items.push(NeoCodexMessageItem {
                            role: "system".to_string(),
                            content: format!("**{}**: {}", kind, detail),
                            timestamp,
                            attachments: None,
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

/// Fetch persisted side-chat messages for a session (branched questions that
/// never re-enter the main context).
#[tauri::command]
pub async fn neocodex_get_side_chat(session_id: String) -> Result<Vec<NeoCodexMessageItem>, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for line in content.lines() {
        if let Ok(WireEvent::SideChatMessage { content, timestamp }) = serde_json::from_str::<WireEvent>(line) {
            items.push(NeoCodexMessageItem {
                role: "user".to_string(),
                content,
                timestamp,
                attachments: None,
            });
        }
    }
    Ok(items)
}

#[tauri::command]
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
            a.provider.sync_from_real();
            *guard = Some(a);
            guard.as_mut().unwrap()
        }
    };
    // Route through the live agent so the wire's in-memory mirror stays in sync
    // when this is the active session; otherwise record to the target file.
    if agent.wire.path == path {
        agent.record_side_chat(&content);
    } else {
        let mut wire = WireSession::new(&session_id);
        wire.path = path.clone();
        wire.record(WireEvent::SideChatMessage {
            content: content.trim().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
        });
    }
    // Return the full side-chat history for immediate re-render.
    let content2 = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for line in content2.lines() {
        if let Ok(WireEvent::SideChatMessage { content, timestamp }) = serde_json::from_str::<WireEvent>(line) {
            items.push(NeoCodexMessageItem {
                role: "user".to_string(),
                content,
                timestamp,
                attachments: None,
            });
        }
    }
    Ok(items)
}

/// Persist a user-chosen session name into the session's wire stream.
#[tauri::command]
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
            a.provider.sync_from_real();
            *guard = Some(a);
            guard.as_mut().unwrap()
        }
    };
    if agent.wire.path == path {
        agent.rename_session(&name);
    } else {
        let mut wire = WireSession::new(&session_id);
        wire.path = path.clone();
        wire.record(WireEvent::SessionMeta {
            name: name.trim().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
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
    })
}

#[tauri::command]
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
            a.provider.sync_from_real();
            *guard = Some(a);
            guard.as_mut().unwrap()
        }
    };
    agent.set_mode(parsed);
    Ok(format!("mode set to {}", mode))
}

#[tauri::command]
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
            a.provider.sync_from_real();
            *guard = Some(a);
            guard.as_mut().unwrap()
        }
    };
    agent.wire.path = path.clone();
    let count = agent.resume_session();
    Ok(format!("Switched to session {} (restored {} events)", session_id, count))
}

#[tauri::command]
pub async fn neocodex_delete_session(session_id: String) -> Result<String, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err("Session not found".to_string());
    }
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(format!("Deleted session {}", session_id))
}

#[derive(serde::Serialize)]
pub struct NeoCodexSessionInfo {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub message_count: usize,
    pub wire_path: String,
    pub updated_at: u64,
}

#[derive(serde::Serialize)]
pub struct NeoCodexMessageItem {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
    pub attachments: Option<Vec<NeoCodexAttachmentDto>>,
}

#[derive(serde::Serialize)]
pub struct NeoCodexAttachmentDto {
    pub name: String,
    pub size: u64,
    pub mime_type: String,
}

#[tauri::command]
pub fn neocodex_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[derive(serde::Serialize)]
pub struct UpdateCheckResult {
    pub current: String,
    pub available: bool,
    pub latest: String,
    pub error: Option<String>,
}

#[tauri::command]
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
