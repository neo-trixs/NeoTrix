//! 统一 Tauri 命令层
//!
//! 只暴露极少数高级命令，所有复杂逻辑委托给核心的 UnifiedApi

use serde::{Deserialize, Serialize};
use tauri::{command, State};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::stub::{
    UnifiedApi, UnifiedApiImpl, UnifiedRequest, UnifiedResponse, UnifiedError,
    RequestContext, ResponseMode, SessionInfo, MessageType, ResponseMetadata, ConsciousnessState,
    ResponsePayload,
};

// 统一 API 状态
pub type UnifiedApiState = Arc<RwLock<UnifiedApiImpl>>;

#[command]
pub async fn unified_init(
    state: State<'_, UnifiedApiState>,
) -> Result<(), String> {
    let mut api = state.write().await;
    *api = UnifiedApiImpl::new();
    Ok(())
}

#[command]
pub async fn unified_chat(
    state: State<'_, UnifiedApiState>,
    request: UnifiedChatRequest,
) -> Result<UnifiedChatResponse, String> {
    let api = state.read().await;

    let unified_request = UnifiedRequest {
        session_id: request.session_id,
        input: request.input,
        context: request.context.map(|c| RequestContext {
            project_path: c.project_path,
            selected_files: c.selected_files,
            selected_code: c.selected_code,
            open_file: c.open_file,
            git_status: c.git_status,
            metadata: c.metadata.unwrap_or_default(),
        }),
        mode: match request.mode.as_deref() {
            Some("code") => ResponseMode::Code,
            Some("design") => ResponseMode::Design,
            Some("diagnose") => ResponseMode::Diagnose,
            Some("explain") => ResponseMode::Explain,
            Some("execute") => ResponseMode::Execute,
            Some("knowledge") => ResponseMode::Knowledge,
            Some("introspect") => ResponseMode::Introspect,
            _ => ResponseMode::Chat,
        },
        stream: request.stream.unwrap_or(false),
    };

    let response = api.handle(unified_request).await
        .map_err(|e| format!("Unified API error: {} - {}", e.code, e.message))?;

    Ok(UnifiedChatResponse::from_response(response))
}

#[command]
pub async fn unified_chat_stream(
    state: State<'_, UnifiedApiState>,
    request: UnifiedChatRequest,
) -> Result<String, String> {
    let api = state.read().await;

    let unified_request = UnifiedRequest {
        session_id: request.session_id,
        input: request.input,
        context: request.context.map(|c| RequestContext {
            project_path: c.project_path,
            selected_files: c.selected_files,
            selected_code: c.selected_code,
            open_file: c.open_file,
            git_status: c.git_status,
            metadata: c.metadata.unwrap_or_default(),
        }),
        mode: match request.mode.as_deref() {
            Some("code") => ResponseMode::Code,
            Some("design") => ResponseMode::Design,
            Some("diagnose") => ResponseMode::Diagnose,
            Some("explain") => ResponseMode::Explain,
            Some("execute") => ResponseMode::Execute,
            Some("knowledge") => ResponseMode::Knowledge,
            Some("introspect") => ResponseMode::Introspect,
            _ => ResponseMode::Chat,
        },
        stream: true,
    };

    let stream_id = uuid::Uuid::new_v4().to_string();
    tokio::spawn(async move {
        let _ = api.handle_stream(unified_request).await;
    });

    Ok(stream_id)
}

#[command]
pub async fn unified_system_state(
    state: State<'_, UnifiedApiState>,
) -> Result<UnifiedChatResponse, String> {
    let api = state.read().await;
    let response = api.get_system_state().await
        .map_err(|e| format!("Unified API error: {} - {}", e.code, e.message))?;
    Ok(UnifiedChatResponse::from_response(response))
}

#[command]
pub async fn unified_create_session(
    state: State<'_, UnifiedApiState>,
    project_path: Option<String>,
) -> Result<SessionInfo, String> {
    let api = state.read().await;
    api.create_session(project_path).await
        .map_err(|e| format!("Unified API error: {} - {}", e.code, e.message))
}

#[command]
pub async fn unified_list_sessions(
    state: State<'_, UnifiedApiState>,
) -> Result<Vec<SessionInfo>, String> {
    let api = state.read().await;
    api.list_sessions().await
        .map_err(|e| format!("Unified API error: {} - {}", e.code, e.message))
}

#[command]
pub async fn unified_delete_session(
    state: State<'_, UnifiedApiState>,
    session_id: String,
) -> Result<(), String> {
    let api = state.read().await;
    api.delete_session(&session_id).await
        .map_err(|e| format!("Unified API error: {} - {}", e.code, e.message))
}

// ========== CLI stubs ==========

#[command]
pub async fn unified_exec_cli(command: String, args: Option<Vec<String>>) -> Result<serde_json::Value, String> {
    let _ = args;
    Ok(serde_json::json!({
        "success": true,
        "message": format!("CLI stub: {} (未接入真实后端)", command),
        "exit_code": 0,
        "json": null
    }))
}

#[command]
pub async fn unified_cli_list() -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![
        serde_json::json!({"name": "/help", "description": "显示帮助", "aliases": ["h"]}),
        serde_json::json!({"name": "/config", "description": "配置管理", "aliases": ["c"]}),
        serde_json::json!({"name": "/stats", "description": "系统统计", "aliases": ["s"]}),
    ])
}

// ========== 请求/响应 DTO ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedChatRequest {
    pub session_id: Option<String>,
    pub input: String,
    pub context: Option<ChatContext>,
    pub mode: Option<String>,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatContext {
    pub project_path: Option<String>,
    pub selected_files: Vec<String>,
    pub selected_code: Option<String>,
    pub open_file: Option<String>,
    pub git_status: Option<String>,
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedChatResponse {
    pub response_id: String,
    pub session_id: String,
    pub content: String,
    pub payload: Option<serde_json::Value>,
    pub message_type: String,
    pub metadata: ResponseMetadataDto,
    pub is_stream_chunk: bool,
    pub stream_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadataDto {
    pub duration_ms: u64,
    pub layers_involved: Vec<String>,
    pub capabilities_used: Vec<String>,
    pub consciousness_state: ConsciousnessStateDto,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessStateDto {
    pub phi: f64,
    pub coherence: f64,
    pub gwt_resonance: f64,
    pub emotion: String,
    pub attention_focus: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfoDto {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub project_path: Option<String>,
}

impl UnifiedChatResponse {
    fn from_response(response: UnifiedResponse) -> Self {
        let payload = response.payload.map(|p| serde_json::to_value(p).unwrap_or(serde_json::Value::Null));
        Self {
            response_id: response.response_id,
            session_id: response.session_id,
            content: response.content,
            payload,
            message_type: format!("{:?}", response.message_type).to_lowercase(),
            metadata: ResponseMetadataDto {
                duration_ms: response.metadata.duration_ms,
                layers_involved: response.metadata.layers_involved,
                capabilities_used: response.metadata.capabilities_used,
                consciousness_state: ConsciousnessStateDto {
                    phi: response.metadata.consciousness_state.phi,
                    coherence: response.metadata.consciousness_state.coherence,
                    gwt_resonance: response.metadata.consciousness_state.gwt_resonance,
                    emotion: response.metadata.consciousness_state.emotion,
                    attention_focus: response.metadata.consciousness_state.attention_focus,
                },
                confidence: response.metadata.confidence,
            },
            is_stream_chunk: response.is_stream_chunk,
            stream_done: response.stream_done,
        }
    }
}
