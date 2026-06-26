use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CoreError {
    NotFound(String),
    Knowledge(String),
    Consciousness(String),
    AgentNotFound(String),
    AgentToolFailed(String),
    AgentCommsFailed(String),
    Agent(String),
    HcubeDimMismatch { expected: usize, got: usize },
    HyperCube(String),
    #[serde(skip)]
    Io(std::sync::Arc<std::io::Error>),
    Serde(String),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::NotFound(s) => write!(f, "not found: {}", s),
            CoreError::Knowledge(s) => write!(f, "knowledge error: {}", s),
            CoreError::Consciousness(s) => write!(f, "consciousness error: {}", s),
            CoreError::AgentNotFound(s) => write!(f, "agent not found: {}", s),
            CoreError::AgentToolFailed(s) => write!(f, "agent tool failed: {}", s),
            CoreError::AgentCommsFailed(s) => write!(f, "agent comms failed: {}", s),
            CoreError::Agent(s) => write!(f, "agent error: {}", s),
            CoreError::HcubeDimMismatch { expected, got } => {
                write!(f, "hypercube dimension mismatch: expected={}, got={}", expected, got)
            }
            CoreError::HyperCube(s) => write!(f, "hypercube error: {}", s),
            CoreError::Io(e) => write!(f, "io error: {}", e.as_ref()),
            CoreError::Serde(s) => write!(f, "serde error: {}", s),
        }
    }
}

impl std::error::Error for CoreError {}

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Io(std::sync::Arc::new(e))
    }
}

impl From<String> for CoreError {
    fn from(s: String) -> Self {
        CoreError::Serde(s)
    }
}

impl From<&str> for CoreError {
    fn from(s: &str) -> Self {
        CoreError::Serde(s.to_string())
    }
}

pub type CoreResult<T> = Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_error_display() {
        let e = CoreError::NotFound("file".into());
        assert_eq!(e.to_string(), "not found: file");
        let e2 = CoreError::AgentToolFailed("crash".into());
        assert_eq!(e2.to_string(), "agent tool failed: crash");
    }

    #[test]
    fn test_core_error_from_string() {
        let e: CoreError = "bad things".into();
        assert!(matches!(e, CoreError::Serde(_)));
    }

    #[test]
    fn test_core_error_dim_mismatch() {
        let e = CoreError::HcubeDimMismatch { expected: 4096, got: 512 };
        assert_eq!(e.to_string(), "hypercube dimension mismatch: expected=4096, got=512");
    }

    #[test]
    fn test_core_result_alias() {
        let ok: CoreResult<i32> = Ok(42);
        assert!(ok.is_ok());
    }
}
