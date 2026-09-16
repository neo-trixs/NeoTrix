//! Universal Provider UI Panel — NeoTrix Fusion v2
//!
//! 统一外部模型管理面板，包含：
//! - Universal Provider 模型列表
//! - 模型切换 UI
//! - NIP-01 事件审计日志
//! - Autoresearch 进度面板
//! - YAML 工作流编辑器
//! - Agent 会话管理面板

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

// ========== Universal Provider 模型列表 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalProvider {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub category: ProviderCategory,
    pub is_free: bool,
    pub base_url: String,
    pub models: Vec<UniversalModel>,
    pub resolvable: bool,
    pub health_status: ProviderHealth,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderCategory {
    Local,
    Cloud,
    Proxy,
    Hybrid,
}

impl std::fmt::Display for ProviderCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderCategory::Local => write!(f, "local"),
            ProviderCategory::Cloud => write!(f, "cloud"),
            ProviderCategory::Proxy => write!(f, "proxy"),
            ProviderCategory::Hybrid => write!(f, "hybrid"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalModel {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub format: ModelFormatDisplay,
    pub context_length: Option<u32>,
    pub parameter_count: Option<String>,
    pub capabilities: ModelCapabilityFlags,
    pub is_active: bool,
    pub downloaded: bool,
    pub download_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelFormatDisplay {
    GGUF,
    ONNX,
    Safetensors,
    GGJ,
}

impl std::fmt::Display for ModelFormatDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelFormatDisplay::GGUF => write!(f, "GGUF"),
            ModelFormatDisplay::ONNX => write!(f, "ONNX"),
            ModelFormatDisplay::Safetensors => write!(f, "Safetensors"),
            ModelFormatDisplay::GGJ => write!(f, "GGJ"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilityFlags {
    pub text: bool,
    pub vision: bool,
    pub audio: bool,
    pub function_calling: bool,
    pub streaming: bool,
    pub agent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub available: bool,
    pub latency_ms: u64,
    pub last_check: String,
    pub circuit_state: CircuitState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

// ========== 模型切换 UI ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSwitchState {
    pub active_model: Option<String>,
    pub active_provider: Option<String>,
    pub available_models: Vec<UniversalModel>,
    pub switch_history: Vec<SwitchRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchRecord {
    pub timestamp: String,
    pub from_model: Option<String>,
    pub to_model: String,
    pub provider: String,
    pub success: bool,
}

// ========== NIP-01 事件审计日志 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: String,
    pub event_type: AuditEventType,
    pub source: String,
    pub detail: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub processed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    ModelSwitch,
    ModelDownload,
    ModelVerify,
    ProviderConnect,
    ProviderDisconnect,
    SessionCreate,
    SessionDelete,
    CommandExecuted,
    ErrorOccurred,
    ConfigChange,
    AgentSpawn,
    AgentComplete,
    WorkflowRun,
    SecurityAlert,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::ModelSwitch => write!(f, "model_switch"),
            AuditEventType::ModelDownload => write!(f, "model_download"),
            AuditEventType::ModelVerify => write!(f, "model_verify"),
            AuditEventType::ProviderConnect => write!(f, "provider_connect"),
            AuditEventType::ProviderDisconnect => write!(f, "provider_disconnect"),
            AuditEventType::SessionCreate => write!(f, "session_create"),
            AuditEventType::SessionDelete => write!(f, "session_delete"),
            AuditEventType::CommandExecuted => write!(f, "command_executed"),
            AuditEventType::ErrorOccurred => write!(f, "error_occurred"),
            AuditEventType::ConfigChange => write!(f, "config_change"),
            AuditEventType::AgentSpawn => write!(f, "agent_spawn"),
            AuditEventType::AgentComplete => write!(f, "agent_complete"),
            AuditEventType::WorkflowRun => write!(f, "workflow_run"),
            AuditEventType::SecurityAlert => write!(f, "security_alert"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogPanel {
    pub entries: Vec<AuditLogEntry>,
    pub total_count: usize,
    pub unread_count: usize,
    pub filter: Option<AuditEventType>,
    pub auto_scroll: bool,
}

// ========== Autoresearch 进度面板 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoresearchProgress {
    pub active: bool,
    pub current_phase: ResearchPhase,
    pub phase_progress: f32,
    pub experiments: Vec<ExperimentRecord>,
    pub metrics: ResearchMetrics,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResearchPhase {
    Hypothesis,
    Experiment,
    Measurement,
    Analysis,
    Synthesis,
    Complete,
}

impl std::fmt::Display for ResearchPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResearchPhase::Hypothesis => write!(f, "hypothesis"),
            ResearchPhase::Experiment => write!(f, "experiment"),
            ResearchPhase::Measurement => write!(f, "measurement"),
            ResearchPhase::Analysis => write!(f, "analysis"),
            ResearchPhase::Synthesis => write!(f, "synthesis"),
            ResearchPhase::Complete => write!(f, "complete"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentRecord {
    pub id: String,
    pub name: String,
    pub status: ExperimentStatus,
    pub progress: f32,
    pub result: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExperimentStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchMetrics {
    pub experiments_run: u32,
    pub experiments_passed: u32,
    pub experiments_failed: u32,
    pub best_score: f64,
    pub current_score: f64,
    pub improvement_rate: f64,
    pub total_runtime_secs: f64,
}

// ========== YAML 工作流编辑器 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub yaml_content: String,
    pub validated: bool,
    pub validation_errors: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEditorState {
    pub workflows: Vec<Workflow>,
    pub active_workflow: Option<String>,
    pub yaml_errors: Vec<WorkflowError>,
    pub auto_validate: bool,
    pub syntax_highlight: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowError {
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

// ========== Agent 会话管理面板 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub name: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub model: String,
    pub provider: String,
    pub started_at: Option<String>,
    pub last_activity: Option<String>,
    pub message_count: u32,
    pub tokens_used: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentType {
    General,
    Coding,
    Design,
    Research,
    Debug,
    System,
    Custom(String),
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::General => write!(f, "general"),
            AgentType::Coding => write!(f, "coding"),
            AgentType::Design => write!(f, "design"),
            AgentType::Research => write!(f, "research"),
            AgentType::Debug => write!(f, "debug"),
            AgentType::System => write!(f, "system"),
            AgentType::Custom(name) => write!(f, "custom:{name}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Running,
    Paused,
    Completed,
    Error,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Idle => write!(f, "idle"),
            AgentStatus::Running => write!(f, "running"),
            AgentStatus::Paused => write!(f, "paused"),
            AgentStatus::Completed => write!(f, "completed"),
            AgentStatus::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManagementPanel {
    pub sessions: Vec<AgentSession>,
    pub active_session: Option<String>,
    pub total_sessions: usize,
    pub total_tokens: u64,
    pub filter_status: Option<AgentStatus>,
    pub filter_type: Option<AgentType>,
}

// ========== Universal UI State ==========

pub struct UniversalUIState {
    pub providers: Vec<UniversalProvider>,
    pub model_switch: ModelSwitchState,
    pub audit_log: AuditLogPanel,
    pub autoresearch: AutoresearchProgress,
    pub workflow_editor: WorkflowEditorState,
    pub session_manager: SessionManagementPanel,
}

impl UniversalUIState {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            model_switch: ModelSwitchState {
                active_model: None,
                active_provider: None,
                available_models: Vec::new(),
                switch_history: Vec::new(),
            },
            audit_log: AuditLogPanel {
                entries: Vec::new(),
                total_count: 0,
                unread_count: 0,
                filter: None,
                auto_scroll: true,
            },
            autoresearch: AutoresearchProgress {
                active: false,
                current_phase: ResearchPhase::Hypothesis,
                phase_progress: 0.0,
                experiments: Vec::new(),
                metrics: ResearchMetrics {
                    experiments_run: 0,
                    experiments_passed: 0,
                    experiments_failed: 0,
                    best_score: 0.0,
                    current_score: 0.0,
                    improvement_rate: 0.0,
                    total_runtime_secs: 0.0,
                },
                started_at: None,
                completed_at: None,
            },
            workflow_editor: WorkflowEditorState {
                workflows: Vec::new(),
                active_workflow: None,
                yaml_errors: Vec::new(),
                auto_validate: true,
                syntax_highlight: true,
            },
            session_manager: SessionManagementPanel {
                sessions: Vec::new(),
                active_session: None,
                total_sessions: 0,
                total_tokens: 0,
                filter_status: None,
                filter_type: None,
            },
        }
    }

    /// 刷新所有提供者
    pub async fn refresh_providers(&mut self) -> Result<(), String> {
        self.providers = Self::discover_providers().await?;
        Ok(())
    }

    async fn discover_providers() -> Result<Vec<UniversalProvider>, String> {
        let mut providers = Vec::new();

        // LM Studio 本地提供者
        providers.push(UniversalProvider {
            id: "lmstudio".into(),
            name: "lmstudio".into(),
            display_name: "LM Studio".into(),
            category: ProviderCategory::Local,
            is_free: true,
            base_url: "http://localhost:1234".into(),
            models: Vec::new(),
            resolvable: false,
            health_status: ProviderHealth {
                available: false,
                latency_ms: 0,
                last_check: chrono::Utc::now().to_rfc3339(),
                circuit_state: CircuitState::Closed,
            },
            icon: None,
        });

        // OpenResearch 提供者
        providers.push(UniversalProvider {
            id: "openresearch".into(),
            name: "openresearch".into(),
            display_name: "OpenResearch".into(),
            category: ProviderCategory::Cloud,
            is_free: false,
            base_url: "https://openresearch.ai".into(),
            models: Vec::new(),
            resolvable: false,
            health_status: ProviderHealth {
                available: false,
                latency_ms: 0,
                last_check: chrono::Utc::now().to_rfc3339(),
                circuit_state: CircuitState::Closed,
            },
            icon: None,
        });

        // vLLM 提供者
        providers.push(UniversalProvider {
            id: "vllm".into(),
            name: "vllm".into(),
            display_name: "vLLM".into(),
            category: ProviderCategory::Hybrid,
            is_free: true,
            base_url: "http://localhost:8000".into(),
            models: Vec::new(),
            resolvable: false,
            health_status: ProviderHealth {
                available: false,
                latency_ms: 0,
                last_check: chrono::Utc::now().to_rfc3339(),
                circuit_state: CircuitState::Closed,
            },
            icon: None,
        });

        Ok(providers)
    }

    /// 添加审计日志条目
    pub fn add_audit_entry(&mut self, entry: AuditLogEntry) {
        self.audit_log.entries.insert(0, entry);
        self.audit_log.total_count += 1;
        self.audit_log.unread_count += 1;
    }

    /// 标记所有审计条目为已读
    pub fn mark_all_read(&mut self) {
        for entry in &mut self.audit_log.entries {
            entry.processed = true;
        }
        self.audit_log.unread_count = 0;
    }
}

// ========== Universal Model Manager (UI 层) ==========

pub struct UniversalModelManager {
    pub all_models: Vec<UniversalModel>,
    pub downloaded_models: Vec<UniversalModel>,
    pub pending_downloads: Vec<UniversalModel>,
    pub source_index: HashMap<String, UniversalProvider>,
}

impl UniversalModelManager {
    pub fn new() -> Self {
        Self {
            all_models: Vec::new(),
            downloaded_models: Vec::new(),
            pending_downloads: Vec::new(),
            source_index: HashMap::new(),
        }
    }

    /// 扫描所有可用模型
    pub async fn scan_all_models(&mut self) -> Result<(), String> {
        let _local_models = self.scan_local_gguf().await?;
        self.all_models.extend(_local_models);
        Ok(())
    }

    async fn scan_local_gguf(&self) -> Result<Vec<UniversalModel>, String> {
        let cache_dir = dirs::home_dir()
            .unwrap_or_default()
            .join(".neotrix")
            .join("models");

        let mut models = Vec::new();
        if !cache_dir.exists() {
            return Ok(models);
        }

        let entries = tokio::fs::read_dir(&cache_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());
            if ext == Some("gguf") || ext == Some("onnx") || ext == Some("safetensors") {
                let file_name = path.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                models.push(UniversalModel {
                    id: file_name.clone(),
                    name: file_name.clone(),
                    display_name: file_name.replace('_', " "),
                    format: detect_display_format(&path),
                    context_length: None,
                    parameter_count: None,
                    capabilities: ModelCapabilityFlags {
                        text: true,
                        vision: false,
                        audio: false,
                        function_calling: false,
                        streaming: true,
                        agent: false,
                    },
                    is_active: false,
                    downloaded: true,
                    download_path: Some(path),
                });
            }
        }

        Ok(models)
    }
}

fn detect_display_format(path: &std::path::Path) -> ModelFormatDisplay {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext.to_lowercase().as_str() {
        "gguf" => ModelFormatDisplay::GGUF,
        "onnx" => ModelFormatDisplay::ONNX,
        "safetensors" => ModelFormatDisplay::Safetensors,
        "ggj" => ModelFormatDisplay::GGJ,
        _ => {
            let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
            if name.contains("safetensors") {
                ModelFormatDisplay::Safetensors
            } else {
                ModelFormatDisplay::GGUF
            }
        }
    }
}