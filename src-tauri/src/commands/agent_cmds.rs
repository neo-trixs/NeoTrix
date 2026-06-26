use std::sync::Mutex;
use tauri::{command, Emitter};
use neotrix::neotrix::nt_io_provider::{ProviderConfig, LlmProviderType, create_provider, LlmRequest};
use super::{AgentStatus, ProviderConfigPayload, ReasonRequest, ReasonResponse};

// ===== Agent state statics =====

static AGENT_RUNNING: std::sync::LazyLock<Mutex<AgentStatus>> =
    std::sync::LazyLock::new(|| Mutex::new(AgentStatus {
        running: false, current_task: None, uptime_secs: 0,
    }));
static AGENT_START_TIME: std::sync::LazyLock<Mutex<Option<std::time::Instant>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

// ===== Provider config helpers (pub, used by mcp_cmds) =====

pub fn read_provider_config() -> Result<ProviderConfigPayload, String> {
    let path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("neotrix")
        .join("provider.json");
    let content = std::fs::read_to_string(&path).map_err(|e| format!("无法读取 Provider 配置: {}", e))?;
    let payload: ProviderConfigPayload = serde_json::from_str(&content).map_err(|e| format!("解析 Provider 配置失败: {}", e))?;
    Ok(payload)
}

pub fn payload_to_provider_config(payload: &ProviderConfigPayload) -> ProviderConfig {
    let provider_type = match payload.id.to_lowercase().as_str() {
        "openai" => LlmProviderType::OpenAI,
        "anthropic" => LlmProviderType::Anthropic,
        "gemini" => LlmProviderType::Gemini,
        "ollama" => LlmProviderType::Ollama,
        _ => LlmProviderType::OpenAI,
    };
    ProviderConfig {
        provider_type,
        api_key: Some(payload.api_key.clone()),
        base_url: payload.base_url.clone(),
        model: Some(payload.model.clone()),
        timeout_secs: 120,
    }
}

// ===== Agent commands =====

#[command]
pub async fn agent_reason(app: tauri::AppHandle, req: ReasonRequest) -> ReasonResponse {
    let payload = match read_provider_config() {
        Ok(p) => p,
        Err(e) => return ReasonResponse { output: e, success: false },
    };
    let config = payload_to_provider_config(&payload);
    let provider = create_provider(config);
    let request = LlmRequest::new(&payload.model, &req.prompt);

    if let Ok(mut rx) = provider.stream_complete(&request).await {
        let mut full_output = String::new();
        while let Some(chunk_result) = rx.recv().await {
            match chunk_result {
                Ok(chunk) => {
                    full_output.push_str(&chunk.content);
                    let _ = app.emit("streaming-token", serde_json::json!({
                        "token": chunk.content,
                        "full": full_output,
                    }));
                }
                Err(e) => {
                    let _ = app.emit("streaming-token", serde_json::json!({
                        "token": "",
                        "error": format!("{}", e),
                        "full": full_output,
                    }));
                    return ReasonResponse { output: format!("LLM 流错误: {}", e), success: false };
                }
            }
        }
        let _ = app.emit("streaming-done", serde_json::json!({ "full": full_output }));
        return ReasonResponse { output: full_output, success: true };
    }

    match provider.complete(&request).await {
        Ok(response) => ReasonResponse {
            output: response.content,
            success: true,
        },
        Err(e) => ReasonResponse {
            output: format!("LLM 错误: {}", e),
            success: false,
        },
    }
}

#[command]
pub fn cmd_agent_start(prompt: String) -> Result<String, String> {
    let mut state = AGENT_RUNNING.lock().map_err(|e| e.to_string())?;
    *state = AgentStatus { running: true, current_task: Some(prompt.clone()), uptime_secs: 0 };
    if let Ok(mut start) = AGENT_START_TIME.lock() { *start = Some(std::time::Instant::now()); }
    Ok("Agent started".to_string())
}

#[command]
pub fn cmd_agent_stop() -> Result<(), String> {
    let mut state = AGENT_RUNNING.lock().map_err(|e| e.to_string())?;
    *state = AgentStatus { running: false, current_task: None, uptime_secs: 0 };
    if let Ok(mut start) = AGENT_START_TIME.lock() { *start = None; }
    Ok(())
}

#[command]
pub fn cmd_agent_status() -> Result<AgentStatus, String> {
    let state = AGENT_RUNNING.lock().map_err(|e| e.to_string())?;
    let mut status = state.clone();
    if status.running {
        if let Ok(start) = AGENT_START_TIME.lock() {
            if let Some(t) = *start { status.uptime_secs = t.elapsed().as_secs(); }
        }
    }
    Ok(status)
}
