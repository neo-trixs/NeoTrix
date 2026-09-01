//! Stub types for Tauri compilation without neotrix crate dependency.
//!
//! 当 neotrix crate 有编译错误时，Tauri 使用这些 stub 类型独立编译。
//! 后续 neotrix crate 修复后，可替换为真实实现。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ========== Unified API Types ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRequest {
    pub session_id: Option<String>,
    pub input: String,
    pub context: Option<RequestContext>,
    pub mode: ResponseMode,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub project_path: Option<String>,
    pub selected_files: Vec<String>,
    pub selected_code: Option<String>,
    pub open_file: Option<String>,
    pub git_status: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseMode {
    Chat,
    Code,
    Design,
    Diagnose,
    Explain,
    Execute,
    Knowledge,
    Introspect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResponse {
    pub response_id: String,
    pub session_id: String,
    pub content: String,
    pub payload: Option<ResponsePayload>,
    pub message_type: MessageType,
    pub metadata: ResponseMetadata,
    pub is_stream_chunk: bool,
    pub stream_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponsePayload {
    CodeChanges { changes: Vec<CodeChange> },
    FileOps { operations: Vec<FileOperation> },
    TerminalCommands { commands: Vec<TerminalCommand> },
    KnowledgeGraph { nodes: Vec<GraphNode>, edges: Vec<GraphEdge> },
    CapabilityTree { domains: Vec<DomainCapability> },
    HealthSnapshot { phi: f64, coherence: f64, gwt_resonance: f64, modules: Vec<ModuleHealth> },
    EvolutionPlan { actions: Vec<EvolutionAction> },
    TaskResult { task_id: String, success: bool, output: String, artifacts: Vec<Artifact> },
    Sessions { sessions: Vec<SessionInfo> },
    Providers { providers: Vec<ProviderStatus> },
    KeyValue { data: HashMap<String, serde_json::Value> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: String,
    pub change_type: CodeChangeType,
    pub old_content: Option<String>,
    pub new_content: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeChangeType { Create, Modify, Delete, Move }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub op: FileOpType,
    pub path: String,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileOpType { Read, Write, Delete, List, Search }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCommand {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
    pub description: String,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCapability {
    pub domain: String,
    pub capabilities: Vec<CapabilityInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    pub id: String,
    pub name: String,
    pub maturity: String,
    pub provides: Vec<String>,
    pub health: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHealth {
    pub name: String,
    pub layer: String,
    pub healthy: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionAction {
    pub action_type: String,
    pub node_id: String,
    pub domain: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub artifact_type: String,
    pub path: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub project_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub available: bool,
    pub models: Vec<String>,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Text, Data, Progress, Error, ApprovalRequired, SystemEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub duration_ms: u64,
    pub layers_involved: Vec<String>,
    pub capabilities_used: Vec<String>,
    pub consciousness_state: ConsciousnessState,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    pub phi: f64,
    pub coherence: f64,
    pub gwt_resonance: f64,
    pub emotion: String,
    pub attention_focus: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub recoverable: bool,
    pub suggested_actions: Vec<String>,
}

// ========== UnifiedApi Trait + Impl ==========

#[async_trait::async_trait]
pub trait UnifiedApi: Send + Sync {
    async fn handle(&self, request: UnifiedRequest) -> Result<UnifiedResponse, UnifiedError>;
    async fn handle_stream(
        &self,
        request: UnifiedRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<UnifiedResponse, UnifiedError>>, UnifiedError>;
    async fn get_system_state(&self) -> Result<UnifiedResponse, UnifiedError>;
    async fn create_session(&self, project_path: Option<String>) -> Result<SessionInfo, UnifiedError>;
    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, UnifiedError>;
    async fn delete_session(&self, session_id: &str) -> Result<(), UnifiedError>;
}

pub struct UnifiedApiImpl;

impl UnifiedApiImpl {
    pub fn new() -> Self { Self }
}

#[async_trait::async_trait]
impl UnifiedApi for UnifiedApiImpl {
    async fn handle(&self, request: UnifiedRequest) -> Result<UnifiedResponse, UnifiedError> {
        Ok(UnifiedResponse {
            response_id: uuid::Uuid::new_v4().to_string(),
            session_id: request.session_id.unwrap_or_default(),
            content: "统一 API stub 待接入真实后端".to_string(),
            payload: None,
            message_type: MessageType::Text,
            metadata: ResponseMetadata {
                duration_ms: 0,
                layers_involved: vec![],
                capabilities_used: vec![],
                consciousness_state: ConsciousnessState {
                    phi: 0.0, coherence: 0.0, gwt_resonance: 0.0,
                    emotion: "neutral".to_string(), attention_focus: vec![],
                },
                confidence: 0.0,
            },
            is_stream_chunk: false,
            stream_done: true,
        })
    }

    async fn handle_stream(
        &self, _request: UnifiedRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<UnifiedResponse, UnifiedError>>, UnifiedError> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            let _ = tx.send(Ok(UnifiedResponse {
                response_id: uuid::Uuid::new_v4().to_string(),
                session_id: "".to_string(),
                content: "流式 stub 待实现".to_string(),
                payload: None,
                message_type: MessageType::Text,
                metadata: ResponseMetadata {
                    duration_ms: 0, layers_involved: vec![], capabilities_used: vec![],
                    consciousness_state: ConsciousnessState {
                        phi: 0.0, coherence: 0.0, gwt_resonance: 0.0,
                        emotion: "neutral".to_string(), attention_focus: vec![],
                    },
                    confidence: 0.0,
                },
                is_stream_chunk: true, stream_done: true,
            })).await;
        });
        Ok(rx)
    }

    async fn get_system_state(&self) -> Result<UnifiedResponse, UnifiedError> {
        Ok(UnifiedResponse {
            response_id: uuid::Uuid::new_v4().to_string(),
            session_id: "system".to_string(),
            content: "系统状态 stub".to_string(),
            payload: Some(ResponsePayload::HealthSnapshot {
                phi: 0.0, coherence: 0.0, gwt_resonance: 0.0, modules: vec![],
            }),
            message_type: MessageType::Data,
            metadata: ResponseMetadata {
                duration_ms: 0, layers_involved: vec![], capabilities_used: vec![],
                consciousness_state: ConsciousnessState {
                    phi: 0.0, coherence: 0.0, gwt_resonance: 0.0,
                    emotion: "neutral".to_string(), attention_focus: vec![],
                },
                confidence: 1.0,
            },
            is_stream_chunk: false, stream_done: true,
        })
    }

    async fn create_session(&self, project_path: Option<String>) -> Result<SessionInfo, UnifiedError> {
        Ok(SessionInfo {
            id: uuid::Uuid::new_v4().to_string(),
            title: "新会话".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            message_count: 0, project_path,
        })
    }

    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, UnifiedError> { Ok(vec![]) }
    async fn delete_session(&self, _: &str) -> Result<(), UnifiedError> { Ok(()) }
}

impl UnifiedRequest {
    pub fn chat(input: impl Into<String>) -> Self {
        Self { session_id: None, input: input.into(), context: None, mode: ResponseMode::Chat, stream: false }
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self { project_path: None, selected_files: vec![], selected_code: None, open_file: None, git_status: None, metadata: HashMap::new() }
    }
}

// ========== Sentry Stub ==========

pub fn init_sentry() -> Option<sentry::ClientInitGuard> {
    // Stub: 不初始化 sentry，避免外部依赖
    None
}
