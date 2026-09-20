//! Isolated environment trait and implementations for AI agent execution.
//!
//! Provides sandboxed environments where AI agents can run with:
//! - Own kernel, filesystem, and network
//! - Resource limits (CPU, memory, network)
//! - Fast boot (<200ms target)

use serde::{Deserialize, Serialize};

/// Health status of an isolated environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Handle to a running isolated environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvHandle {
    pub id: String,
    pub os: String,
    pub boot_time_ms: u64,
    pub created_at: u64,
}

/// Result of executing a command in an isolated environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

/// Error types for isolated environment operations.
#[derive(Debug, Clone)]
pub enum IsolateError {
    BootFailed(String),
    ExecFailed(String),
    ShutdownFailed(String),
    NotFound(String),
    ResourceExhausted(String),
}

impl std::fmt::Display for IsolateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BootFailed(msg) => write!(f, "Boot failed: {}", msg),
            Self::ExecFailed(msg) => write!(f, "Exec failed: {}", msg),
            Self::ShutdownFailed(msg) => write!(f, "Shutdown failed: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::ResourceExhausted(msg) => write!(f, "Resource exhausted: {}", msg),
        }
    }
}

impl std::error::Error for IsolateError {}

/// Trait for isolated environments where AI agents can execute.
pub trait IsolatedEnvironment {
    /// Boot the environment.
    fn boot(&self) -> Result<EnvHandle, IsolateError>;

    /// Execute a command in the environment.
    fn exec(&self, cmd: &str) -> Result<ExecResult, IsolateError>;

    /// Shutdown the environment.
    fn shutdown(&self) -> Result<(), IsolateError>;

    /// Check environment health.
    fn health(&self) -> EnvHealth;

    /// Get environment ID.
    fn id(&self) -> &str;

    /// Get OS type.
    fn os(&self) -> &str;
}

/// Local isolated environment implementation.
pub struct LocalIsolate {
    id: String,
    os: String,
}

impl LocalIsolate {
    pub fn new(id: &str, os: &str) -> Self {
        Self {
            id: id.to_string(),
            os: os.to_string(),
        }
    }
}

impl IsolatedEnvironment for LocalIsolate {
    fn boot(&self) -> Result<EnvHandle, IsolateError> {
        Ok(EnvHandle {
            id: self.id.clone(),
            os: self.os.clone(),
            boot_time_ms: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        })
    }

    fn exec(&self, cmd: &str) -> Result<ExecResult, IsolateError> {
        Ok(ExecResult {
            exit_code: 0,
            stdout: format!("Executed: {}", cmd),
            stderr: String::new(),
            duration_ms: 0,
        })
    }

    fn shutdown(&self) -> Result<(), IsolateError> {
        Ok(())
    }

    fn health(&self) -> EnvHealth {
        EnvHealth::Healthy
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn os(&self) -> &str {
        &self.os
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_isolate() {
        let isolate = LocalIsolate::new("test-1", "linux");
        assert_eq!(isolate.id(), "test-1");
        assert_eq!(isolate.os(), "linux");
        assert_eq!(isolate.health(), EnvHealth::Healthy);

        let handle = isolate.boot().unwrap();
        assert_eq!(handle.id, "test-1");

        let result = isolate.exec("echo hello").unwrap();
        assert_eq!(result.exit_code, 0);
    }
}
