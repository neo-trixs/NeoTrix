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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_error_display() {
        let err = PlatformError::Agent("init failed".into());
        assert!(format!("{}", err).contains("init failed"));
    }

    #[test]
    fn platform_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err: PlatformError = io_err.into();
        assert!(matches!(err, PlatformError::Io(_)));
    }

    #[test]
    fn platform_error_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("bad json");
        let err: PlatformError = json_err.unwrap_err().into();
        assert!(matches!(err, PlatformError::Serialization(_)));
    }

    #[test]
    fn platform_error_variants() {
        assert!(format!("{}", PlatformError::Pipeline("x".into())).contains("Pipeline"));
        assert!(format!("{}", PlatformError::Orchestrator("x".into())).contains("Orchestrator"));
        assert!(format!("{}", PlatformError::Registry("x".into())).contains("Registry"));
        assert!(format!("{}", PlatformError::Config("x".into())).contains("Config"));
        assert!(format!("{}", PlatformError::Internal("x".into())).contains("内部"));
    }
}
