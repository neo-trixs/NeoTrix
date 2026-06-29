// Re-export from core/ for backward compat
pub use crate::core::nt_core_error::*;

use std::fmt;

#[derive(Debug, Clone)]
pub enum NeoTrixError {
    Brain(String),
    General {
        msg: String,
        backtrace: Option<String>,
    },
    Io(std::sync::Arc<std::io::Error>),
    Config(String),
    Serde(String),
    Network(String),
    Internal(String),
    Command {
        cmd: String,
        exit_code: Option<i32>,
        stderr: String,
    },
    Memory(String),
    Path {
        path: String,
        detail: String,
    },
}

impl fmt::Display for NeoTrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NeoTrixError::Brain(s) => write!(f, "brain error: {}", s),
            NeoTrixError::General { msg, .. } => write!(f, "general error: {}", msg),
            NeoTrixError::Io(e) => write!(f, "io error: {}", e.as_ref()),
            NeoTrixError::Config(s) => write!(f, "config error: {}", s),
            NeoTrixError::Serde(s) => write!(f, "serde error: {}", s),
            NeoTrixError::Network(s) => write!(f, "network error: {}", s),
            NeoTrixError::Internal(s) => write!(f, "internal error: {}", s),
            NeoTrixError::Command { cmd, .. } => write!(f, "command failed: {}", cmd),
            NeoTrixError::Memory(s) => write!(f, "memory error: {}", s),
            NeoTrixError::Path { path, .. } => write!(f, "path error: {}", path),
        }
    }
}

impl std::error::Error for NeoTrixError {}

impl From<String> for NeoTrixError {
    fn from(s: String) -> Self {
        NeoTrixError::Internal(s)
    }
}
impl From<&str> for NeoTrixError {
    fn from(s: &str) -> Self {
        NeoTrixError::Internal(s.to_string())
    }
}

impl From<std::io::Error> for NeoTrixError {
    fn from(e: std::io::Error) -> Self {
        NeoTrixError::Io(std::sync::Arc::new(e))
    }
}

pub type NeoTrixResult<T> = Result<T, NeoTrixError>;
