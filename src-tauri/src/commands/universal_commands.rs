//! Universal Commands — NeoTrix Fusion v2
//!
//! 统一的 Tauri 命令空间，覆盖：
//! - Universal Provider 管理
//! - 模型切换
//! - NIP-01 事件审计日志
//! - Autoresearch 进度
//! - YAML 工作流编辑器
//! - Agent 会话管理
//! - 多模型下载

use crate::stub::{RequestContext, ResponseMode, SessionInfo, UnifiedApi, UnifiedApiImpl, UnifiedRequest, UnifiedResponse};
use crate::desktop::model_manager::{ModelFormat, ModelManager, ModelSource, ModelValidationResult};
use crate::desktop::session_manager::{Message, MessageRole, MessageType, Session, SessionManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{command, State};
use tokio::sync::RwLock;

// ========== Universal API State ==========

pub type UniversalApiState = Arc<RwLock<UniversalApiImpl>>;

pub struct UniversalApiImpl {
    model_manager: Option<ModelManager>,
    session_manager: SessionManager,
}

impl UniversalApiImpl {
    pub fn new() -> Self {
        Self {
            model_manager: None,
            session_manager: SessionManager::new(),
        }
    }

    pub fn get_model_manager(&self) -> Option<&ModelManager> {
        self.model_manager.as_ref()
    }

    pub fn get_model_manager_mut(&mut self) -> Option<&mut ModelManager> {
        self.model_manager.as_mut()
    }
}

// ========== Universal Provider Commands ==========

#[command]
pub async fn universal_list_providers() -> Result<Vec<serde_json::Value>, String> {
    let providers = vec![
        serde_json::json!({
            "id": "lmstudio",
            "name": "lmstudio",
            "display_name": "LM Studio",
            "category": "local",
            "is_free": true,
            "base_url": "http://localhost:1234",
            "models": [],
            "resolvable": false,
        }),
        serde_json::json!({
            "id": "openresearch",
            "name": "openresearch",
            "display_name": "OpenResearch",
            "category": "cloud",
            "is_free": false,
            "base_url": "https://openresearch.ai",
            "models": [],
            "resolvable": true,
        }),
        serde_json::json!({
            "id": "vllm",
            "name": "vllm",
            "display_name": "vLLM",
            "category": "hybrid",
            "is_free": true,
            "base_url": "http://localhost:8000",
            "models": [],
            "resolvable": false,
        }),
        serde_json::json!({
            "id": "omlx",
            "name": "omlx",
            "display_name": "oMLX",
            "category": "local",
            "is_free": true,
            "base_url": "http://localhost:8080",
            "models": [],
            "resolvable": false,
        }),
        serde_json::json!({
            "id": "localgguf",
            "name": "localgguf",
            "display_name": "Local GGUF",
            "category": "local",
            "is_free": true,
            "base_url": "",
            "models": [],
            "resolvable": true,
        }),
    ];
    Ok(providers)
}

#[command]
pub async fn universal_get_provider_models(provider_id: String) -> Result<Vec<serde_json::Value>, String> {
    let _ = provider_id;
    Ok(vec![])
}

// ========== Model Management Commands ==========

#[command]
pub async fn universal_list_models() -> Result<Vec<serde_json::Value>, String> {
    let mut result = Vec::new();
    result.push(serde_json::json!({
        "id": "test-gguf",
        "name": "test-gguf",
        "display_name": "Test GGUF Model",
        "format": "GGUF",
        "source": "local",
        "downloaded": true,
        "capabilities": {
            "text": true,
            "vision": false,
            "audio": false,
            "function_calling": false,
            "streaming": true,
            "agent": false
        }
    }));
    Ok(result)
}

#[command]
pub async fn universal_download_model(
    model_id: String,
    model_name: String,
    source: String,
    format: String,
) -> Result<serde_json::Value, String> {
    let source_enum = match source.as_str() {
        "lmstudio" => ModelSource::LMStudio,
        "omlx" => ModelSource::oMLX,
        "localgguf" => ModelSource::LocalGGUF,
        "vllm" => ModelSource::VLLM,
        "openresearch" => ModelSource::OpenResearch,
        _ => ModelSource::Custom(source),
    };
    let format_enum = match format.as_str() {
        "onnx" => ModelFormat::ONNX,
        "safetensors" => ModelFormat::Safetensors,
        "ggj" => ModelFormat::GGJ,
        _ => ModelFormat::GGUF,
    };

    let mut mgr = ModelManager::new();
    let task = mgr.create_download_task(model_id, format!("https://{}.ai/download/{}", source, model_name), source_enum, format_enum);

    Ok(serde_json::json!({
        "job_id": task.job_id,
        "model_id": task.model_id,
        "status": "pending",
        "source": source,
        "format": format,
    }))
}

#[command]
pub async fn universal_verify_model(model_id: String) -> Result<ModelValidationResult, String> {
    let mut mgr = ModelManager::new();
    mgr.verify_model(&model_id).await
}

#[command]
pub async fn universal_model_formats() -> Result<Vec<String>, String> {
    Ok(vec!["gguf".into(), "onnx".into(), "safetensors".into(), "ggj".into()])
}

// ========== Model Switch Commands ==========

#[command]
pub async fn universal_switch_model(provider_id: String, model_id: String) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "success": true,
        "provider_id": provider_id,
        "model_id": model_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

#[command]
pub async fn universal_get_switch_state() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "active_model": None,
        "active_provider": None,
        "switch_history": [],
    }))
}

// ========== Audit Log Commands ==========

#[command]
pub async fn universal_audit_log(limit: Option<u32>) -> Result<Vec<serde_json::Value>, String> {
    let limit = limit.unwrap_or(50);
    let mut entries = Vec::new();

    for i in 0..limit {
        entries.push(serde_json::json!({
            "id": format!("audit_{}", i),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "event_type": "model_switch",
            "source": "system",
            "detail": format!("Audit entry {}", i),
            "metadata": {},
            "processed": true,
        }));
    }

    Ok(entries)
}

#[command]
pub async fn universal_add_audit_entry(event_type: String, source: String, detail: String) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "event_type": event_type,
        "source": source,
        "detail": detail,
        "processed": false,
    }))
}

#[command]
pub async fn universal_clear_audit_log() -> Result<bool, String> {
    Ok(true)
}

// ========== Autoresearch Commands ==========

#[command]
pub async fn universal_autoresearch_status() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "active": false,
        "current_phase": "hypothesis",
        "phase_progress": 0.0,
        "metrics": {
            "experiments_run": 0,
            "experiments_passed": 0,
            "experiments_failed": 0,
            "best_score": 0.0,
            "current_score": 0.0,
            "improvement_rate": 0.0,
            "total_runtime_secs": 0.0,
        },
    }))
}

#[command]
pub async fn universal_autoresearch_start() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "success": true,
        "started_at": chrono::Utc::now().to_rfc3339(),
        "phase": "hypothesis",
    }))
}

#[command]
pub async fn universal_autoresearch_stop() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "success": true,
        "stopped_at": chrono::Utc::now().to_rfc3339(),
    }))
}

#[command]
pub async fn universal_autoresearch_phase() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "phase": "hypothesis",
        "progress": 0.0,
        "experiments": [],
    }))
}

// ========== YAML Workflow Commands ==========

#[command]
pub async fn universal_workflow_list() -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}

#[command]
pub async fn universal_workflow_validate(yaml_content: String) -> Result<serde_json::Value, String> {
    let errors: Vec<serde_json::Value> = Vec::new();
    Ok(serde_json::json!({
        "valid": errors.is_empty(),
        "errors": errors,
        "warnings": Vec::<serde_json::Value>::new(),
    }))
}

#[command]
pub async fn universal_workflow_save(name: String, yaml_content: String) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "success": true,
        "id": uuid::Uuid::new_v4().to_string(),
        "name": name,
        "saved_at": chrono::Utc::now().to_rfc3339(),
    }))
}

#[command]
pub async fn universal_workflow_delete(workflow_id: String) -> Result<bool, String> {
    Ok(true)
}

// ========== Agent Session Commands ==========

#[command]
pub async fn universal_list_sessions() -> Result<Vec<serde_json::Value>, String> {
    let mut sessions = Vec::new();

    sessions.push(serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "General Agent",
        "agent_type": "general",
        "status": "idle",
        "model": "test-model",
        "provider": "lmstudio",
        "started_at": None::<String>,
        "last_activity": None::<String>,
        "message_count": 0u32,
        "tokens_used": 0u64,
        "metadata": {},
    }));

    Ok(sessions)
}

#[command]
pub async fn universal_create_session(name: String, agent_type: String, model: String) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "success": true,
        "id": uuid::Uuid::new_v4().to_string(),
        "name": name,
        "agent_type": agent_type,
        "model": model,
        "created_at": chrono::Utc::now().to_rfc3339(),
    }))
}

#[command]
pub async fn universal_delete_session(session_id: String) -> Result<bool, String> {
    Ok(true)
}

#[command]
pub async fn universal_session_stats() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "total_sessions": 0,
        "total_tokens": 0u64,
        "active_sessions": 0,
        "total_messages": 0,
    }))
}

// ========== Unified Tauri commands (bridge) ==========

#[command]
pub async fn unified_init() -> Result<(), String> {
    Ok(())
}

#[command]
pub async fn unified_chat(input: String) -> Result<String, String> {
    Ok(format!("Universal chat response for: {input}"))
}

#[command]
pub async fn unified_chat_stream(input: String) -> Result<String, String> {
    Ok(format!("stream:{input}"))
}

#[command]
pub async fn unified_system_state() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "phi": 0.0,
        "coherence": 0.0,
        "gwt_resonance": 0.0,
        "emotion": "neutral",
        "attention_focus": [],
        "consciousness_state": {
            "phi": 0.0,
            "coherence": 0.0,
            "gwt_resonance": 0.0,
            "emotion": "neutral",
            "attention_focus": [],
        },
        "confidence": 0.0,
        "layers_involved": [],
        "capabilities_used": [],
        "duration_ms": 0,
    }))
}

#[command]
pub async fn unified_create_session(project_path: Option<String>) -> Result<SessionInfo, String> {
    let api = UnifiedApiImpl::new();
    api.create_session(project_path)
        .await
        .map_err(|e| format!("Unified API error: {}", e))
}

#[command]
pub async fn unified_list_sessions() -> Result<Vec<SessionInfo>, String> {
    let api = UnifiedApiImpl::new();
    api.list_sessions()
        .await
        .map_err(|e| format!("Unified API error: {}", e))
}

#[command]
pub async fn unified_delete_session(session_id: String) -> Result<(), String> {
    let api = UnifiedApiImpl::new();
    api.delete_session(&session_id)
        .await
        .map_err(|e| format!("Unified API error: {}", e))
}

#[command]
pub async fn unified_exec_cli(command: String) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "success": true,
        "message": format!("CLI: {command}"),
        "exit_code": 0,
        "json": null,
    }))
}

#[command]
pub async fn unified_cli_list() -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![
        serde_json::json!({"name": "/help", "description": "显示帮助", "aliases": ["h"]}),
        serde_json::json!({"name": "/config", "description": "配置管理", "aliases": ["c"]}),
        serde_json::json!({"name": "/stats", "description": "系统统计", "aliases": ["s"]}),
        serde_json::json!({"name": "/models", "description": "模型管理", "aliases": ["m"]}),
        serde_json::json!({"name": "/agents", "description": "Agent 管理", "aliases": ["a"]}),
        serde_json::json!({"name": "/workflows", "description": "工作流管理", "aliases": ["w"]}),
        serde_json::json!({"name": "/audit", "description": "审计日志", "aliases": ["l"]}),
    ])
}

// ========== Download Commands ==========

#[command]
pub async fn universal_download_progress(job_id: String) -> Result<Option<serde_json::Value>, String> {
    let mgr = ModelManager::new();
    if let Some(progress) = mgr.get_download_progress(&job_id) {
        Ok(Some(serde_json::json!({
            "job_id": progress.job_id,
            "model_id": progress.model_id,
            "progress": progress.progress,
            "downloaded_bytes": progress.downloaded_bytes,
            "total_bytes": progress.total_bytes,
            "speed_bytes_per_sec": progress.speed_bytes_per_sec,
            "source": progress.source.as_str(),
            "format": progress.format.to_string(),
        })))
    } else {
        Ok(None)
    }
}

#[command]
pub async fn universal_list_downloads() -> Result<Vec<serde_json::Value>, String> {
    let mgr = ModelManager::new();
    let downloads = mgr.list_downloads();
    let mut result = Vec::new();
    for task in downloads {
        result.push(serde_json::json!({
            "job_id": task.job_id,
            "model_id": task.model_id,
            "status": format!("{:?}", task.status),
            "source": task.source.as_str(),
            "format": task.format.to_string(),
        }));
    }
    Ok(result)
}

#[command]
pub async fn universal_pause_download(job_id: String) -> Result<bool, String> {
    let mut mgr = ModelManager::new();
    mgr.pause_download(&job_id).map_err(|_| "not found".into())?;
    Ok(true)
}

#[command]
pub async fn universal_resume_download(job_id: String) -> Result<bool, String> {
    let mut mgr = ModelManager::new();
    mgr.resume_download(&job_id).map_err(|_| "not found".into())?;
    Ok(true)
}

#[command]
pub async fn universal_cancel_download(job_id: String) -> Result<bool, String> {
    let mut mgr = ModelManager::new();
    mgr.cancel_download(&job_id).map_err(|_| "not found".into())?;
    Ok(true)
}

// ========== DTOs ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalProviderDto {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub category: String,
    pub is_free: bool,
    pub base_url: String,
    pub models: Vec<String>,
    pub resolvable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalModelDto {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub format: String,
    pub source: String,
    pub downloaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntryDto {
    pub id: String,
    pub timestamp: String,
    pub event_type: String,
    pub source: String,
    pub detail: String,
    pub processed: bool,
}