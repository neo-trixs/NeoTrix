use neotrix::neotrix::l1_body_impl::nt_io_neocodex::{
    NeoCodexAgent, NeoCodexHealthReport, NeoCodexMode, EvolutionLoop, WireSession, WireEvent,
};
use serde::Serialize;
use tauri::Emitter;

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

#[tauri::command]
pub async fn neocodex_send_message_stream(app: tauri::AppHandle, content: String) -> Result<String, String> {
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

    // Emit start event
    let _ = app.emit("neocodex_stream_start", content.clone());

    let result = agent.react_loop_stream(&content, 4, |token| {
        let _ = app.emit("neocodex_stream_token", token);
    }).await;

    let _ = app.emit("neocodex_stream_end", result.clone().unwrap_or_default());
    Ok(result.unwrap_or_else(|| "[no response]".to_string()))
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

#[tauri::command]
pub async fn neocodex_list_sessions() -> Result<Vec<NeoCodexSessionInfo>, String> {
    let dir = std::env::temp_dir().join("neotrix-sessions");
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
                        WireEvent::UserMessage { content, timestamp } => {
                            if message_count == 0 && content.len() > 20 {
                                name = format!("{}...", &content[..20]);
                            }
                            message_count += 1;
                            updated_at = timestamp.max(0) as u64;
                        }
                        WireEvent::ModeChange { to, .. } => {
                            mode = format!("{:?}", to);
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
pub async fn neocodex_switch_session(session_id: String) -> Result<String, String> {
    let path = std::env::temp_dir().join("neotrix-sessions").join(format!("{}.jsonl", session_id));
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
    let path = std::env::temp_dir().join("neotrix-sessions").join(format!("{}.jsonl", session_id));
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
