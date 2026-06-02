use axum::{
    extract::{Path, Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        Json,
    },
};
use futures::stream::{self, Stream};
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::atomic::Ordering;

use super::{AgentStatus, BrainStats, DiffBlock, FileNode, PermissionRequest, ProjectInfo, ProviderConfigPayload, SessionInfo};

// ─── Helpers ────────────────────────────────────────────────

fn json_ok<T: serde::Serialize>(v: T) -> Json<serde_json::Value> {
    Json(serde_json::json!(v))
}

fn json_err(msg: &str) -> (axum::http::StatusCode, Json<serde_json::Value>) {
    (
        axum::http::StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": msg})),
    )
}

fn provider_config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("neotrix")
        .join("provider.json")
}

fn read_provider_config() -> Result<serde_json::Value, String> {
    let path = provider_config_path();
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read provider config: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))
}

fn payload_to_provider_config(payload: &ProviderConfigPayload) -> crate::neotrix::provider::ProviderConfig {
    let provider_type = match payload.id.to_lowercase().as_str() {
        "openai" => crate::neotrix::provider::LlmProviderType::OpenAI,
        "anthropic" => crate::neotrix::provider::LlmProviderType::Anthropic,
        "gemini" => crate::neotrix::provider::LlmProviderType::Gemini,
        "ollama" => crate::neotrix::provider::LlmProviderType::Ollama,
        _ => crate::neotrix::provider::LlmProviderType::OpenAI,
    };
    crate::neotrix::provider::ProviderConfig {
        provider_type,
        api_key: Some(payload.api_key.clone()),
        base_url: payload.base_url.clone(),
        model: Some(payload.model.clone()),
        timeout_secs: 120,
    }
}

fn run_git_cmd(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("Git failed: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git error: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn parse_git_diff(diff_str: &str) -> Vec<DiffBlock> {
    let mut blocks = Vec::new();
    for line in diff_str.lines() {
        if let Some(stripped) = line.strip_prefix("+") {
            if !stripped.starts_with("+") {
                blocks.push(DiffBlock { r#type: "added".into(), content: stripped.to_string(), line_start: 0 });
                continue;
            }
        }
        if let Some(stripped) = line.strip_prefix("-") {
            if !stripped.starts_with("-") {
                blocks.push(DiffBlock { r#type: "removed".into(), content: stripped.to_string(), line_start: 0 });
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
            blocks.push(DiffBlock { r#type: "unchanged".into(), content: line.to_string(), line_start: 0 });
        }
    }
    blocks
}

// ─── Brain Handlers ─────────────────────────────────────────

type AppState = super::AppState;

pub async fn brain_stats_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let brain = state.brain.lock().expect("brain lock");
    let bank = state.bank.lock().expect("bank lock");
    let stats = brain.get_statistics();
    json_ok(BrainStats {
        iteration: 0,
        absorb_count: brain.total_absorb_count,
        capability_sum: stats.capability_sum,
        memory_count: bank.memories().len(),
        engine_active: false,
        capability_vector: brain.capability.arr().to_vec(),
        dimension_names: (0..brain.capability.total_dim())
            .map(|i| format!("dim_{}", i))
            .collect(),
    })
}

#[derive(Deserialize)]
pub struct AbsorbBody {
    source: String,
}

pub async fn absorb_source_handler(
    State(state): State<AppState>,
    Json(body): Json<AbsorbBody>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let source = match body.source.to_lowercase().as_str() {
        "heroui" => crate::neotrix::nt_mind::KnowledgeSource::HeroUI,
        "baseui" => crate::neotrix::nt_mind::KnowledgeSource::BaseUI,
        "arcui" => crate::neotrix::nt_mind::KnowledgeSource::ArcUI,
        "cortexui" => crate::neotrix::nt_mind::KnowledgeSource::CortexUI,
        "agenticds" => crate::neotrix::nt_mind::KnowledgeSource::AgenticDS,
        "designphilosophy" => crate::neotrix::nt_mind::KnowledgeSource::DesignPhilosophy,
        _ => {
            return Err(json_err(
                "Unknown source. Options: HeroUI, BaseUI, ArcUI, CortexUI, AgenticDS, DesignPhilosophy",
            ))
        }
    };
    let mut brain = state.brain.lock().expect("brain lock");
    brain.absorb(source);
    Ok(json_ok(serde_json::json!({
        "absorbed": brain.total_absorb_count
    })))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

pub async fn search_knowledge_handler(
    Query(query): Query<SearchQuery>,
) -> Json<serde_json::Value> {
    let path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("neotrix")
        .join("knowledge.json");
    let results: Vec<serde_json::Value> = if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let entries: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap_or_default();
        let q = query.q.to_lowercase();
        entries
            .into_iter()
            .filter(|e| {
                let title = e["title"].as_str().unwrap_or("").to_lowercase();
                let content = e["content"].as_str().unwrap_or("").to_lowercase();
                title.contains(&q) || content.contains(&q)
            })
            .take(10)
            .map(|e| {
                serde_json::json!({
                    "id": e["id"],
                    "title": e["title"],
                    "content": e["content"].as_str().unwrap_or("").chars().take(200).collect::<String>(),
                    "relevance": 1.0
                })
            })
            .collect()
    } else {
        Vec::new()
    };
    json_ok(results)
}

#[derive(Deserialize)]
pub struct ReasonBody {
    prompt: String,
}

pub async fn reason_handler(
    Json(body): Json<ReasonBody>,
) -> Json<serde_json::Value> {
    let config = match read_provider_config() {
        Ok(cfg) => cfg,
        Err(e) => return json_ok(serde_json::json!({
            "success": false, "output": format!("Config error: {}", e)
        })),
    };
    let payload: ProviderConfigPayload = match serde_json::from_value(config) {
        Ok(p) => p,
        Err(e) => return json_ok(serde_json::json!({
            "success": false, "output": format!("Parse error: {}", e)
        })),
    };
    let provider_config = payload_to_provider_config(&payload);
    let provider = crate::neotrix::provider::create_provider(provider_config);
    let request = crate::neotrix::provider::LlmRequest::new(&payload.model, &body.prompt);
    match provider.complete(&request).await {
        Ok(response) => json_ok(serde_json::json!({
            "output": response.content, "success": true
        })),
        Err(e) => json_ok(serde_json::json!({
            "success": false, "output": format!("LLM error: {}", e)
        })),
    }
}

// ─── Session Handlers ───────────────────────────────────────

pub async fn session_list_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let sessions = state.sessions.lock().expect("sessions lock");
    json_ok(sessions.clone())
}

#[derive(Deserialize)]
pub struct CreateSessionBody {
    name: String,
}

pub async fn session_create_handler(
    State(state): State<AppState>,
    Json(body): Json<CreateSessionBody>,
) -> Json<serde_json::Value> {
    let mut sessions = state.sessions.lock().expect("sessions lock");
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let info = SessionInfo {
        id: id.clone(),
        name: body.name,
        message_count: 0,
        created: now,
    };
    sessions.push(info.clone());
    json_ok(info)
}

pub async fn session_switch_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let sessions = state.sessions.lock().expect("sessions lock");
    if sessions.iter().any(|s| s.id == id) {
        Ok(json_ok(serde_json::json!({"switched": true})))
    } else {
        Err(json_err("Session not found"))
    }
}

pub async fn session_delete_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let mut sessions = state.sessions.lock().expect("sessions lock");
    sessions.retain(|s| s.id != id);
    json_ok(serde_json::json!({"deleted": true}))
}

pub async fn session_fork_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let mut sessions = state.sessions.lock().expect("sessions lock");
    let src = sessions
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| json_err("Session not found"))?;
    let new_id = uuid::Uuid::new_v4().to_string();
    let forked = SessionInfo {
        id: new_id.clone(),
        name: format!("{} (副本)", src.name),
        message_count: src.message_count,
        created: chrono::Utc::now().timestamp(),
    };
    sessions.push(forked.clone());
    Ok(json_ok(forked))
}

pub async fn session_export_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let sessions = state.sessions.lock().expect("sessions lock");
    let src = sessions
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| json_err("Session not found"))?;
    let export = serde_json::json!({
        "format_version": 1,
        "sessions": [{
            "id": src.id,
            "name": src.name,
            "message_count": src.message_count,
            "created": src.created,
        }],
    });
    Ok(json_ok(export))
}

#[derive(Deserialize)]
pub struct ImportBody {
    json: String,
}

pub async fn session_import_handler(
    State(state): State<AppState>,
    Json(body): Json<ImportBody>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let value: serde_json::Value =
        serde_json::from_str(&body.json).map_err(|e| json_err(&format!("Parse error: {}", e)))?;
    let version = value
        .get("format_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if version != 1 {
        return Err(json_err(&format!("Unsupported format version: {}", version)));
    }
    let sessions_arr = value
        .get("sessions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| json_err("Missing sessions field"))?;
    let mut lock = state.sessions.lock().expect("sessions lock");
    let mut imported_ids = Vec::new();
    for item in sessions_arr {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("imported");
        let msg_count = item
            .get("message_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let created = item
            .get("created")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| chrono::Utc::now().timestamp());
        let final_name = if lock.iter().any(|s| s.name == name) {
            format!("{} (导入)", name)
        } else {
            name.to_string()
        };
        let new_id = uuid::Uuid::new_v4().to_string();
        lock.push(SessionInfo {
            id: new_id.clone(),
            name: final_name,
            message_count: msg_count,
            created,
        });
        imported_ids.push(new_id);
    }
    Ok(json_ok(serde_json::json!({"imported": imported_ids})))
}

// ─── Agent Handlers ─────────────────────────────────────────

pub async fn agent_status_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let mut status = state
        .agent_running
        .lock()
        .expect("agent_running lock")
        .clone();
    if status.running {
        if let Ok(start) = state.agent_start_time.lock() {
            if let Some(t) = *start {
                status.uptime_secs = t.elapsed().as_secs();
            }
        }
    }
    json_ok(status)
}

#[derive(Deserialize)]
pub struct AgentStartBody {
    prompt: String,
}

pub async fn agent_start_handler(
    State(state): State<AppState>,
    Json(body): Json<AgentStartBody>,
) -> Json<serde_json::Value> {
    let mut running = state.agent_running.lock().expect("agent_running lock");
    *running = AgentStatus {
        running: true,
        current_task: Some(body.prompt),
        uptime_secs: 0,
    };
    if let Ok(mut start) = state.agent_start_time.lock() {
        *start = Some(std::time::Instant::now());
    }
    json_ok(serde_json::json!({"started": true}))
}

pub async fn agent_stop_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let mut running = state.agent_running.lock().expect("agent_running lock");
    *running = AgentStatus {
        running: false,
        current_task: None,
        uptime_secs: 0,
    };
    if let Ok(mut start) = state.agent_start_time.lock() {
        *start = None;
    }
    json_ok(serde_json::json!({"stopped": true}))
}

#[derive(Deserialize)]
pub struct AgentReasonBody {
    prompt: String,
}

pub async fn agent_reason_stream_handler(
    State(_state): State<AppState>,
    Json(body): Json<AgentReasonBody>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();

    tokio::spawn(async move {
        let config = match read_provider_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                let _ = tx.send(serde_json::json!({"error": e}).to_string());
                return;
            }
        };

        let payload: ProviderConfigPayload = match serde_json::from_value(config) {
            Ok(p) => p,
            Err(e) => {
                let _ = tx.send(serde_json::json!({"error": format!("Config parse: {}", e)}).to_string());
                return;
            }
        };

        let provider_config = payload_to_provider_config(&payload);
        let provider = crate::neotrix::provider::create_provider(provider_config);
        let request = crate::neotrix::provider::LlmRequest::new(&payload.model, &body.prompt);

        match provider.complete(&request).await {
            Ok(response) => {
                let _ = tx.send(serde_json::json!({"token": response.content, "done": true}).to_string());
            }
            Err(e) => {
                let _ = tx.send(serde_json::json!({"error": format!("{}", e)}).to_string());
            }
        }
    });

    let stream = stream::once(async {
        let data = rx.await.unwrap_or_else(|_| "{}".to_string());
        Ok::<_, Infallible>(Event::default().data(data))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// ─── Project / File Handlers ───────────────────────────────

#[derive(Deserialize)]
pub struct TreeQuery {
    path: String,
    #[serde(default = "default_depth")]
    depth: u32,
}

fn default_depth() -> u32 {
    3
}

fn read_dir_inner(dir: &std::path::Path, depth: u32, max_depth: u32) -> Result<Vec<FileNode>, String> {
    if depth > max_depth {
        return Ok(Vec::new());
    }
    let mut nodes = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        let is_dir = path.is_dir();
        let children = if is_dir {
            Some(read_dir_inner(&path, depth + 1, max_depth)?)
        } else {
            None
        };
        nodes.push(FileNode {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            children,
        });
    }
    nodes.sort_by(|a, b| b.is_dir.cmp(&a.is_dir));
    Ok(nodes)
}

pub async fn file_tree_handler(
    Query(query): Query<TreeQuery>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let dir = std::path::Path::new(&query.path);
    if !dir.is_dir() {
        return Err(json_err("Not a directory"));
    }
    let tree = read_dir_inner(dir, 0, query.depth).map_err(|e| json_err(&e))?;
    Ok(json_ok(tree))
}

#[derive(Deserialize)]
pub struct FileQuery {
    path: String,
}

pub async fn read_file_handler(
    Query(query): Query<FileQuery>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let content = std::fs::read_to_string(&query.path).map_err(|e| json_err(&e.to_string()))?;
    Ok(json_ok(serde_json::json!({"content": content})))
}

#[derive(Deserialize)]
pub struct WriteFileBody {
    path: String,
    content: String,
}

pub async fn write_file_handler(
    Json(body): Json<WriteFileBody>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    std::fs::write(&body.path, &body.content).map_err(|e| json_err(&e.to_string()))?;
    Ok(json_ok(serde_json::json!({"written": true})))
}

#[derive(Deserialize)]
pub struct DetectQuery {
    path: String,
}

pub async fn detect_project_handler(
    Query(query): Query<DetectQuery>,
) -> Json<serde_json::Value> {
    let dir = std::path::Path::new(&query.path);
    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let language = if dir.join("Cargo.toml").exists() {
        "Rust"
    } else if dir.join("package.json").exists() {
        "JavaScript/TypeScript"
    } else if dir.join("pyproject.toml").exists() || dir.join("setup.py").exists() {
        "Python"
    } else if dir.join("go.mod").exists() {
        "Go"
    } else {
        "Unknown"
    };
    json_ok(ProjectInfo {
        name,
        path: query.path,
        language: language.into(),
        file_count: 0,
    })
}

// ─── Diff Handlers ──────────────────────────────────────────

pub async fn diff_staged_handler() -> Json<serde_json::Value> {
    match run_git_cmd(&["diff", "--cached"]) {
        Ok(s) => json_ok(parse_git_diff(&s)),
        Err(e) => json_ok(serde_json::json!({"error": e})),
    }
}

pub async fn diff_unstaged_handler() -> Json<serde_json::Value> {
    match run_git_cmd(&["diff"]) {
        Ok(s) => json_ok(parse_git_diff(&s)),
        Err(e) => json_ok(serde_json::json!({"error": e})),
    }
}

#[derive(Deserialize)]
pub struct DiffFileQuery {
    path: String,
}

pub async fn diff_file_handler(
    Query(query): Query<DiffFileQuery>,
) -> Json<serde_json::Value> {
    match run_git_cmd(&["diff", "HEAD", "--", &query.path]) {
        Ok(s) => json_ok(parse_git_diff(&s)),
        Err(e) => json_ok(serde_json::json!({"error": e})),
    }
}

// ─── Permission Handlers ────────────────────────────────────

pub async fn pending_permissions_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let perms = state
        .pending_permissions
        .lock()
        .expect("pending_permissions lock");
    json_ok(perms.clone())
}

#[derive(Deserialize)]
pub struct PermissionRequestAction {
    pub action: String,
    pub target: String,
}

pub async fn permission_request_handler(
    State(state): State<AppState>,
    Json(body): Json<PermissionRequestAction>,
) -> Json<serde_json::Value> {
    let id = format!(
        "perm-{}",
        state
            .permission_counter
            .fetch_add(1, Ordering::SeqCst)
    );
    let req = PermissionRequest {
        id: id.clone(),
        action: body.action,
        target: body.target,
        timestamp: chrono::Utc::now().timestamp(),
    };
    state
        .pending_permissions
        .lock()
        .expect("pending_permissions lock")
        .push(req.clone());
    json_ok(req)
}

pub async fn permission_approve_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let mut perms = state
        .pending_permissions
        .lock()
        .expect("pending_permissions lock");
    let len = perms.len();
    perms.retain(|p| p.id != id);
    if perms.len() == len {
        return Err(json_err("Permission request not found"));
    }
    Ok(json_ok(serde_json::json!({"approved": true})))
}

pub async fn permission_deny_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let mut perms = state
        .pending_permissions
        .lock()
        .expect("pending_permissions lock");
    let len = perms.len();
    perms.retain(|p| p.id != id);
    if perms.len() == len {
        return Err(json_err("Permission request not found"));
    }
    Ok(json_ok(serde_json::json!({"denied": true})))
}

// ─── MCP / Provider Handlers ────────────────────────────────

pub async fn test_provider_handler(
    Json(body): Json<ProviderConfigPayload>,
) -> Json<serde_json::Value> {
    if body.api_key.is_empty() || body.model.is_empty() {
        return json_ok(serde_json::json!({
            "success": false,
            "error": "API Key and model cannot be empty"
        }));
    }
    let provider_config = payload_to_provider_config(&body);
    let provider = crate::neotrix::provider::create_provider(provider_config);
    let request = crate::neotrix::provider::LlmRequest::new(&body.model, "Hello");
    match provider.complete(&request).await {
        Ok(_) => json_ok(serde_json::json!({"success": true, "message": "ok"})),
        Err(e) => json_ok(serde_json::json!({
            "success": false,
            "error": format!("Test failed: {}", e)
        })),
    }
}

pub async fn save_provider_handler(
    Json(body): Json<ProviderConfigPayload>,
) -> Json<serde_json::Value> {
    let path = provider_config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(&body) {
        Ok(json) => match std::fs::write(&path, json) {
            Ok(_) => json_ok(serde_json::json!({"saved": true})),
            Err(e) => json_ok(serde_json::json!({"saved": false, "error": e.to_string()})),
        },
        Err(e) => json_ok(serde_json::json!({"saved": false, "error": e.to_string()})),
    }
}

#[derive(Deserialize)]
pub struct CliCommandBody {
    command: String,
}

pub async fn cli_command_handler(
    Json(body): Json<CliCommandBody>,
) -> Json<serde_json::Value> {
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(&body.command)
        .output()
        .await;
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let combined = if stderr.is_empty() {
                stdout
            } else {
                format!("{}\n{}", stdout, stderr)
            };
            json_ok(serde_json::json!({"output": combined}))
        }
        Err(e) => json_ok(serde_json::json!({"error": e.to_string()})),
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
