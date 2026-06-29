use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryKbError {
    ConnectionFailed(String),
    QueryFailed(String),
    EntryNotFound(String),
    Internal(String),
}

impl fmt::Display for MemoryKbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryKbError::ConnectionFailed(s) => {
                write!(f, "Knowledge base connection failed: {}", s)
            }
            MemoryKbError::QueryFailed(s) => write!(f, "Knowledge base query failed: {}", s),
            MemoryKbError::EntryNotFound(s) => write!(f, "Knowledge base entry not found: {}", s),
            MemoryKbError::Internal(s) => write!(f, "Internal knowledge base error: {}", s),
        }
    }
}

impl std::error::Error for MemoryKbError {}

impl From<String> for MemoryKbError {
    fn from(s: String) -> Self {
        MemoryKbError::Internal(s)
    }
}
impl From<&str> for MemoryKbError {
    fn from(s: &str) -> Self {
        MemoryKbError::Internal(s.to_string())
    }
}
