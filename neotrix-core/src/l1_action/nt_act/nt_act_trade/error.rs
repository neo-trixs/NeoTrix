#![forbid(unsafe_code)]

use thiserror::Error;

/// Trade 模块统一错误类型
#[derive(Error, Debug)]
pub enum TradeError {
    // ── 数据提取错误 ──
    #[error("Chrome decrypt failed: {0}")]
    ChromeDecrypt(String),

    #[error("Selenium automation failed: {0}")]
    Selenium(String),

    #[error("API request failed: {0}")]
    ApiRequest(String),

    #[error("Platform not found: {0}")]
    PlatformNotFound(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    // ── 数据处理错误 ──
    #[error("Data normalization failed: {0}")]
    DataNormalization(String),

    #[error("Serialization failed: {0}")]
    Serialization(String),

    #[error("Deserialization failed: {0}")]
    Deserialization(String),

    // ── CRM 错误 ──
    #[error("Customer not found: {0}")]
    CustomerNotFound(String),

    #[error("Customer already exists: {0}")]
    CustomerAlreadyExists(String),

    // ── 邮件错误 ──
    #[error("Email send failed: {0}")]
    EmailSend(String),

    #[error("Email template error: {0}")]
    EmailTemplate(String),

    // ── 编排错误 ──
    #[error("Task execution failed: {0}")]
    TaskExecution(String),

    #[error("Task timeout after {0}s")]
    TaskTimeout(u64),

    #[error("Worker not available for task type: {0}")]
    WorkerNotAvailable(String),

    // ── 知识库错误 ──
    #[error("Knowledge base error: {0}")]
    KnowledgeBase(String),

    // ── 流程引擎错误 ──
    #[error("Process definition error: {0}")]
    ProcessDefinition(String),

    #[error("Step execution failed: {0}")]
    StepExecution(String),

    // ── 通用错误 ──
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<String> for TradeError {
    fn from(s: String) -> Self {
        Self::Internal(s)
    }
}

impl From<&str> for TradeError {
    fn from(s: &str) -> Self {
        Self::Internal(s.to_string())
    }
}

impl From<crate::l0_substrate::nt_core_error::NeoTrixError> for TradeError {
    fn from(err: crate::l0_substrate::nt_core_error::NeoTrixError) -> Self {
        use crate::l0_substrate::nt_core_error::NeoTrixError;
        match err {
            NeoTrixError::Io(msg) => TradeError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                msg,
            )),
            NeoTrixError::Serde(msg) => TradeError::Deserialization(msg),
            NeoTrixError::Network(msg) => TradeError::ApiRequest(msg),
            NeoTrixError::Config(msg) => TradeError::Config(msg),
            NeoTrixError::NotFound(msg) => TradeError::PlatformNotFound(msg),
            NeoTrixError::InvalidInput(msg) => TradeError::DataNormalization(msg),
            NeoTrixError::InvalidState(msg) => TradeError::Internal(msg),
            NeoTrixError::NotImplemented(msg) => TradeError::Internal(format!("not implemented: {}", msg)),
            NeoTrixError::OperationFailed(msg) => TradeError::TaskExecution(msg),
            NeoTrixError::SafetyViolation(msg) => TradeError::Authentication(msg),
            NeoTrixError::Brain(msg) => TradeError::Internal(msg),
            NeoTrixError::Memory(msg) => TradeError::KnowledgeBase(msg),
            NeoTrixError::Steer(msg) => TradeError::Internal(msg),
            NeoTrixError::Shield(msg) => TradeError::Internal(msg),
            NeoTrixError::Mcp(msg) => TradeError::Internal(msg),
            NeoTrixError::Command { cmd, exit_code, stderr } => TradeError::Internal(
                format!("command '{}' failed (exit {:?}): {}", cmd, exit_code, stderr),
            ),
            NeoTrixError::Path { path, detail } => TradeError::Internal(
                format!("path {:?}: {}", path, detail),
            ),
            NeoTrixError::Unimplemented(msg) => TradeError::Internal(msg),
            NeoTrixError::Wasm(msg) => TradeError::Internal(msg),
            NeoTrixError::Crypto(msg) => TradeError::Internal(msg),
            NeoTrixError::Keyring(msg) => TradeError::Internal(msg),
        }
    }
}
