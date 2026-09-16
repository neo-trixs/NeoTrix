//! # NT-UNIVERSAL-PROVIDER: 通用模型适配层 (UMA)
//!
//! 所有外部模型的统一接口抽象。
//! 覆盖商业API (Anthropic/OpenAI/GoogleGemini/OpenRouter/DeepSeek/Mistral)、
//! 本地/自托管 (Ollama/LlamaCpp/LMStudio/oMLX/VLLM)、
//! Agent工具 (Goose/Codex/ClaudeCode/Cursor/OpenCode)、
//! Buzz/OpenResearch原生 (BuzzRelay/OpenResearch/NostrRelay)。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// 所有外部模型的统一接口
#[async_trait::async_trait]
pub trait UniversalProvider: Send + Sync {
    /// 模型标识
    fn model_id(&self) -> &str;

    /// 提供商类型
    fn provider_type(&self) -> ProviderType;

    /// 模型能力集
    fn capabilities(&self) -> ModelCapabilities;

    /// 统一消息格式完成 (NIP-01 event / ACP message)
    async fn complete(&self, request: UnifiedRequest) -> Result<UnifiedResponse, ProviderError>;

    /// 流式响应
    async fn stream(&self, request: UnifiedRequest)
        -> Result<mpsc::Receiver<UnifiedResponse>, ProviderError>;

    /// 模型元数据
    fn metadata(&self) -> ModelMetadata;

    /// 下载模型权重
    async fn download(&self, url: &str) -> Result<PathBuf, ProviderError>;
}

/// 提供商类型 — 覆盖所有外部模型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    // 商业 API
    Anthropic,
    OpenAI,
    GoogleGemini,
    OpenRouter,
    DeepSeek,
    Mistral,
    Groq,
    TogetherAI,
    // 本地/自托管
    Ollama,
    LlamaCpp,
    LMStudio,
    oMLX,
    LocalGGUF,
    VLLM,
    TGI,
    // Agent 工具
    Goose,
    Codex,
    ClaudeCode,
    Cursor,
    OpenCode,
    // Buzz/OpenResearch 原生
    BuzzRelay,
    OpenResearch,
    NostrRelay,
}

impl ProviderType {
    pub fn label(&self) -> &'static str {
        match self {
            ProviderType::Anthropic => "Anthropic",
            ProviderType::OpenAI => "OpenAI",
            ProviderType::GoogleGemini => "Google Gemini",
            ProviderType::OpenRouter => "OpenRouter",
            ProviderType::DeepSeek => "DeepSeek",
            ProviderType::Mistral => "Mistral",
            ProviderType::Groq => "Groq",
            ProviderType::TogetherAI => "TogetherAI",
            ProviderType::Ollama => "Ollama",
            ProviderType::LlamaCpp => "Llama.cpp",
            ProviderType::LMStudio => "LM Studio",
            ProviderType::oMLX => "oMLX",
            ProviderType::LocalGGUF => "Local GGUF",
            ProviderType::VLLM => "vLLM",
            ProviderType::TGI => "TGI",
            ProviderType::Goose => "Goose",
            ProviderType::Codex => "Codex",
            ProviderType::ClaudeCode => "Claude Code",
            ProviderType::Cursor => "Cursor",
            ProviderType::OpenCode => "OpenCode",
            ProviderType::BuzzRelay => "Buzz Relay",
            ProviderType::OpenResearch => "OpenResearch",
            ProviderType::NostrRelay => "Nostr Relay",
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(
            self,
            ProviderType::Ollama
                | ProviderType::LlamaCpp
                | ProviderType::LMStudio
                | ProviderType::oMLX
                | ProviderType::LocalGGUF
                | ProviderType::VLLM
                | ProviderType::TGI
        )
    }

    pub fn is_agent_tool(&self) -> bool {
        matches!(
            self,
            ProviderType::Goose
                | ProviderType::Codex
                | ProviderType::ClaudeCode
                | ProviderType::Cursor
                | ProviderType::OpenCode
        )
    }
}

/// 模型能力集
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub text_generation: bool,
    pub streaming: bool,
    pub function_calling: bool,
    pub vision: bool,
    pub audio: bool,
    pub code_generation: bool,
    pub reasoning: bool,
    pub fine_tuning: bool,
    pub max_context_tokens: usize,
    pub max_output_tokens: usize,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            text_generation: true,
            streaming: false,
            function_calling: false,
            vision: false,
            audio: false,
            code_generation: false,
            reasoning: false,
            fine_tuning: false,
            max_context_tokens: 4096,
            max_output_tokens: 2048,
        }
    }
}

/// 统一请求 (与 nt_unified_api::UnifiedRequest 对齐)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRequest {
    pub session_id: Option<String>,
    pub input: String,
    pub context: Option<RequestContext>,
    pub mode: ResponseMode,
    pub stream: bool,
}

/// 请求上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub project_path: Option<String>,
    pub selected_files: Vec<String>,
    pub selected_code: Option<String>,
    pub open_file: Option<String>,
    pub git_status: Option<String>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 响应模式
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

/// 统一响应
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

/// 响应载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponsePayload {
    Text { content: String },
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
    KeyValue { data: std::collections::HashMap<String, serde_json::Value> },
}

/// 代码变更
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
pub enum CodeChangeType {
    Create,
    Modify,
    Delete,
    Move,
}

/// 文件操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub op: FileOpType,
    pub path: String,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileOpType {
    Read,
    Write,
    Delete,
    List,
    Search,
}

/// 终端命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCommand {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
    pub description: String,
    pub requires_approval: bool,
}

/// 图节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// 图边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub weight: f64,
}

/// 域能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCapability {
    pub domain: String,
    pub capabilities: Vec<CapabilityInfo>,
}

/// 能力信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    pub id: String,
    pub name: String,
    pub maturity: String,
    pub provides: Vec<String>,
    pub health: f64,
}

/// 模块健康
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHealth {
    pub name: String,
    pub layer: String,
    pub healthy: bool,
    pub message: Option<String>,
}

/// 进化动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionAction {
    pub action_type: String,
    pub node_id: String,
    pub domain: String,
    pub rationale: String,
}

/// 产物
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub artifact_type: String,
    pub path: Option<String>,
    pub content: Option<String>,
}

/// 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub project_path: Option<String>,
}

/// 提供商状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub available: bool,
    pub models: Vec<String>,
    pub last_used: Option<String>,
}

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Text,
    Data,
    Progress,
    Error,
    ApprovalRequired,
    SystemEvent,
}

/// 响应元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub duration_ms: u64,
    pub layers_involved: Vec<String>,
    pub capabilities_used: Vec<String>,
    pub consciousness_state: ConsciousnessState,
    pub confidence: f64,
}

/// 意识状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    pub phi: f64,
    pub coherence: f64,
    pub gwt_resonance: f64,
    pub emotion: String,
    pub attention_focus: Vec<String>,
}

/// 模型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_id: String,
    pub provider_type: ProviderType,
    pub name: String,
    pub version: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub capabilities: ModelCapabilities,
}

/// 提供商错误
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Provider not found: {0}")]
    NotFound(String),
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("Rate limited: {0}")]
    RateLimited(String),
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Timeout after {0}ms")]
    Timeout(u64),
    #[error("Model download failed: {0}")]
    DownloadFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Provider temporarily unavailable: {0}")]
    Unavailable(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

/// 统一 API 错误 (与 nt_unified_api::UnifiedError 对齐)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub recoverable: bool,
    pub suggested_actions: Vec<String>,
}

impl From<ProviderError> for UnifiedError {
    fn from(e: ProviderError) -> Self {
        let (code, message, recoverable) = match &e {
            ProviderError::NotFound(_) => ("PROVIDER_NOT_FOUND", e.to_string(), true),
            ProviderError::AuthFailed(_) => ("AUTH_FAILED", e.to_string(), false),
            ProviderError::RateLimited(_) => ("RATE_LIMITED", e.to_string(), true),
            ProviderError::RequestFailed(_) => ("REQUEST_FAILED", e.to_string(), true),
            ProviderError::Timeout(_) => ("TIMEOUT", e.to_string(), true),
            ProviderError::DownloadFailed(_) => ("DOWNLOAD_FAILED", e.to_string(), false),
            ProviderError::InvalidConfig(_) => ("INVALID_CONFIG", e.to_string(), false),
            ProviderError::Unavailable(_) => ("UNAVAILABLE", e.to_string(), true),
            ProviderError::Internal(_) => ("INTERNAL_ERROR", e.to_string(), false),
        };
        UnifiedError {
            code: code.into(),
            message: message.to_string(),
            details: None,
            recoverable,
            suggested_actions: vec![format!("Check provider configuration and retry")],
        }
    }
}

/// 提供商注册表
#[derive(Debug, Default)]
pub struct ProviderRegistry {
    providers: std::collections::HashMap<String, Box<dyn UniversalProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn UniversalProvider>) -> Result<(), ProviderError> {
        let id = provider.model_id().to_string();
        self.providers.insert(id, provider);
        Ok(())
    }

    pub fn get(&self, model_id: &str) -> Result<&dyn UniversalProvider, ProviderError> {
        self.providers
            .get(model_id)
            .ok_or_else(|| ProviderError::NotFound(model_id.to_string()))
    }

    pub fn get_mut(&mut self, model_id: &str) -> Result<&mut dyn UniversalProvider, ProviderError> {
        self.providers
            .get_mut(model_id)
            .ok_or_else(|| ProviderError::NotFound(model_id.to_string()))
    }

    pub fn remove(&mut self, model_id: &str) -> Result<(), ProviderError> {
        self.providers
            .remove(model_id)
            .ok_or_else(|| ProviderError::NotFound(model_id.to_string()))?;
        Ok(())
    }

    pub fn list(&self) -> Vec<&dyn UniversalProvider> {
        self.providers.values().collect()
    }

    pub fn by_type(&self, provider_type: ProviderType) -> Vec<&dyn UniversalProvider> {
        self.providers
            .values()
            .filter(|p| p.provider_type() == provider_type)
            .collect()
    }

    pub fn contains(&self, model_id: &str) -> bool {
        self.providers.contains_key(model_id)
    }

    pub fn len(&self) -> usize {
        self.providers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_labels() {
        assert_eq!(ProviderType::Anthropic.label(), "Anthropic");
        assert_eq!(ProviderType::Ollama.label(), "Ollama");
        assert_eq!(ProviderType::NostrRelay.label(), "Nostr Relay");
    }

    #[test]
    fn test_provider_type_classification() {
        assert!(ProviderType::Ollama.is_local());
        assert!(!ProviderType::OpenAI.is_local());
        assert!(ProviderType::ClaudeCode.is_agent_tool());
        assert!(!ProviderType::Anthropic.is_agent_tool());
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = ProviderRegistry::new();
        assert!(registry.is_empty());
        // Registry is empty, get should fail
        assert!(registry.get("test").is_err());
    }
}