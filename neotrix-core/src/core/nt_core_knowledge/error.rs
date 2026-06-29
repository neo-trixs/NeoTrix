use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeError {
    EntryNotFound(String),
    InvalidGraph(String),
    ProvenanceError(String),
    Internal(String),
}

impl fmt::Display for KnowledgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KnowledgeError::EntryNotFound(s) => write!(f, "Knowledge entry not found: {}", s),
            KnowledgeError::InvalidGraph(s) => write!(f, "Invalid knowledge graph: {}", s),
            KnowledgeError::ProvenanceError(s) => write!(f, "Knowledge provenance error: {}", s),
            KnowledgeError::Internal(s) => write!(f, "Internal knowledge error: {}", s),
        }
    }
}

impl std::error::Error for KnowledgeError {}

impl From<String> for KnowledgeError {
    fn from(s: String) -> Self {
        KnowledgeError::Internal(s)
    }
}
impl From<&str> for KnowledgeError {
    fn from(s: &str) -> Self {
        KnowledgeError::Internal(s.to_string())
    }
}

impl From<KnowledgeError> for crate::core::CoreError {
    fn from(e: KnowledgeError) -> Self {
        match e {
            KnowledgeError::EntryNotFound(s) => {
                crate::core::CoreError::NotFound(format!("knowledge entry: {}", s))
            }
            KnowledgeError::InvalidGraph(s) => {
                crate::core::CoreError::Knowledge(format!("invalid graph: {}", s))
            }
            KnowledgeError::ProvenanceError(s) => {
                crate::core::CoreError::Knowledge(format!("provenance: {}", s))
            }
            KnowledgeError::Internal(s) => crate::core::CoreError::Knowledge(s),
        }
    }
}
