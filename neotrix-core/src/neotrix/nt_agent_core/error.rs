use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentError {
    NotFound(String),
    ToolExecutionFailed(String),
    CommunicationFailed(String),
    InvalidState(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::NotFound(s) => write!(f, "Agent not found: {}", s),
            AgentError::ToolExecutionFailed(s) => write!(f, "Agent tool execution failed: {}", s),
            AgentError::CommunicationFailed(s) => write!(f, "Agent communication failed: {}", s),
            AgentError::InvalidState(s) => write!(f, "Agent invalid state: {}", s),
        }
    }
}

impl std::error::Error for AgentError {}

impl AgentError {
    pub fn contains(&self, pattern: &str) -> bool {
        match self {
            AgentError::NotFound(s)
            | AgentError::ToolExecutionFailed(s)
            | AgentError::CommunicationFailed(s)
            | AgentError::InvalidState(s) => s.contains(pattern),
        }
    }
}

impl From<String> for AgentError {
    fn from(s: String) -> Self {
        AgentError::InvalidState(s)
    }
}
impl From<&str> for AgentError {
    fn from(s: &str) -> Self {
        AgentError::InvalidState(s.to_string())
    }
}

impl From<AgentError> for crate::core::CoreError {
    fn from(e: AgentError) -> Self {
        match e {
            AgentError::NotFound(s) => crate::core::CoreError::AgentNotFound(s),
            AgentError::ToolExecutionFailed(s) => crate::core::CoreError::AgentToolFailed(s),
            AgentError::CommunicationFailed(s) => crate::core::CoreError::AgentCommsFailed(s),
            AgentError::InvalidState(s) => {
                crate::core::CoreError::Agent(format!("invalid state: {}", s))
            }
        }
    }
}
