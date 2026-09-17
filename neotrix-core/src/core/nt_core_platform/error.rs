#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Agent 错误: {0}")]
    Agent(String),

    #[error("Pipeline 错误: {0}")]
    Pipeline(String),

    #[error("Orchestrator 错误: {0}")]
    Orchestrator(String),

    #[error("Registry 错误: {0}")]
    Registry(String),

    #[error("Config 错误: {0}")]
    Config(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("内部错误: {0}")]
    Internal(String),
}

pub type PlatformResult<T> = Result<T, PlatformError>;
