//! Unified NtError hierarchy — consolidates NeoTrixError, L1Error, FFI errors.
//!
//! Design:
//! - `NtError` is the root enum for all NeoTrix errors
//! - `ErrorKind` groups errors by domain (io, config, network, brain, etc.)
//! - `From` implementations enable `?` operator across the codebase
//! - `Display` produces human-readable messages
//! - `Error` trait is implemented for `anyhow`/`thiserror` interop

use std::fmt;
use std::path::PathBuf;

// ════════════════════════════════════════════════════════════════
// NtError — unified root error type
// ════════════════════════════════════════════════════════════════

/// Unified NeoTrix error type.
///
/// Every subsystem returns `Result<T, NtError>`. Domain-specific error types
/// convert into `NtError` via `From` impls.
#[derive(Debug)]
pub enum NtError {
    // ── Infrastructure ──────────────────────────────────────────
    /// Configuration error (missing key, invalid value).
    Config(String),
    /// I/O error (file, pipe, process).
    Io(String),
    /// Serialization / deserialization error.
    Serde(String),
    /// Network error (DNS, TLS, timeout).
    Network(String),
    /// WASM runtime error.
    Wasm(String),
    /// Cryptographic error.
    Crypto(String),
    /// OS keyring / credential store error.
    Keyring(String),

    // ── Subsystem ───────────────────────────────────────────────
    /// MCP (Model Context Protocol) error.
    Mcp(String),
    /// Brain / LLM inference error.
    Brain(String),
    /// Memory / KB storage error.
    Memory(String),
    /// Shield (security) interception.
    Shield(String),
    /// Safety policy violation (NT-SHIELD).
    SafetyViolation(String),

    // ── Process ─────────────────────────────────────────────────
    /// External command execution failure.
    Command {
        cmd: String,
        exit_code: Option<i32>,
        stderr: String,
    },
    /// Path-related error.
    Path {
        path: PathBuf,
        detail: String,
    },

    // ── State ───────────────────────────────────────────────────
    /// Resource / entity not found.
    NotFound(String),
    /// Input parameter is invalid.
    InvalidInput(String),
    /// System state is invalid (precondition not met).
    InvalidState(String),

    // ── Capability ──────────────────────────────────────────────
    /// Operation not yet implemented.
    NotImplemented(String),
    /// Alias for NotImplemented (backward compat).
    Unimplemented(String),
    /// Generic operation failure.
    OperationFailed(String),

    // ── Control flow ────────────────────────────────────────────
    /// pi-agent steer: slow process suggests re-route (keeps progress).
    Steer(String),

    // ── Nested ──────────────────────────────────────────────────
    /// Wraps an arbitrary boxed error (for interop with external crates).
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for NtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(msg) => write!(f, "Config error: {}", msg),
            Self::Io(msg) => write!(f, "IO error: {}", msg),
            Self::Serde(msg) => write!(f, "Serde error: {}", msg),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Wasm(msg) => write!(f, "WASM error: {}", msg),
            Self::Crypto(msg) => write!(f, "Crypto error: {}", msg),
            Self::Keyring(msg) => write!(f, "Keyring error: {}", msg),
            Self::Mcp(msg) => write!(f, "MCP error: {}", msg),
            Self::Brain(msg) => write!(f, "Brain error: {}", msg),
            Self::Memory(msg) => write!(f, "Memory error: {}", msg),
            Self::Shield(msg) => write!(f, "Shield intercepted: {}", msg),
            Self::SafetyViolation(msg) => write!(f, "Safety violation: {}", msg),
            Self::Command { cmd, exit_code, stderr } => {
                write!(f, "Command '{}' failed (exit={:?}): {}", cmd, exit_code, stderr)
            }
            Self::Path { path, detail } => write!(f, "Path error {:?}: {}", path, detail),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            Self::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            Self::Unimplemented(msg) => write!(f, "Unimplemented: {}", msg),
            Self::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
            Self::Steer(msg) => write!(f, "Steer redirect: {}", msg),
            Self::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for NtError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Internal(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

// ════════════════════════════════════════════════════════════════
// From implementations — enable ? operator across codebase
// ════════════════════════════════════════════════════════════════

// ── Standard library ────────────────────────────────────────────

impl From<std::io::Error> for NtError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<String> for NtError {
    fn from(msg: String) -> Self {
        Self::Brain(msg)
    }
}

impl From<&str> for NtError {
    fn from(msg: &str) -> Self {
        Self::Brain(msg.to_string())
    }
}

impl From<serde_json::Error> for NtError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serde(err.to_string())
    }
}

// ── L1Error (neotrix-types crate) ──────────────────────────────

impl From<crate::l1_error::L1Error> for NtError {
    fn from(err: crate::l1_error::L1Error) -> Self {
        match err {
            crate::l1_error::L1Error::Config(msg) => Self::Config(msg),
            crate::l1_error::L1Error::Io(msg) => Self::Io(msg),
            crate::l1_error::L1Error::Serde(msg) => Self::Serde(msg),
            crate::l1_error::L1Error::Network(msg) => Self::Network(msg),
            crate::l1_error::L1Error::Command { cmd, exit_code, stderr } => {
                Self::Command { cmd, exit_code, stderr }
            }
            crate::l1_error::L1Error::Path { path, detail } => Self::Path { path, detail },
            crate::l1_error::L1Error::Wasm(msg) => Self::Wasm(msg),
            crate::l1_error::L1Error::Crypto(msg) => Self::Crypto(msg),
            crate::l1_error::L1Error::Keyring(msg) => Self::Keyring(msg),
            crate::l1_error::L1Error::Brain(msg) => Self::Brain(msg),
        }
    }
}

// ── LlmError ───────────────────────────────────────────────────

impl From<crate::llm_types::LlmError> for NtError {
    fn from(err: crate::llm_types::LlmError) -> Self {
        Self::Brain(err.to_string())
    }
}

// ── SandboxError ────────────────────────────────────────────────

impl From<crate::core::context::sandbox::SandboxError> for NtError {
    fn from(err: crate::core::context::sandbox::SandboxError) -> Self {
        Self::OperationFailed(err.to_string())
    }
}

// ════════════════════════════════════════════════════════════════
// Convenience constructors
// ════════════════════════════════════════════════════════════════

impl NtError {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }

    pub fn serde(msg: impl Into<String>) -> Self {
        Self::Serde(msg.into())
    }

    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    pub fn brain(msg: impl Into<String>) -> Self {
        Self::Brain(msg.into())
    }

    pub fn memory(msg: impl Into<String>) -> Self {
        Self::Memory(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Self::InvalidState(msg.into())
    }

    pub fn not_implemented(msg: impl Into<String>) -> Self {
        Self::NotImplemented(msg.into())
    }

    pub fn operation_failed(msg: impl Into<String>) -> Self {
        Self::OperationFailed(msg.into())
    }

    pub fn safety_violation(msg: impl Into<String>) -> Self {
        Self::SafetyViolation(msg.into())
    }

    pub fn steer(msg: impl Into<String>) -> Self {
        Self::Steer(msg.into())
    }

    /// Wrap an arbitrary error into NtError::Internal.
    pub fn internal(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Internal(Box::new(err))
    }
}

// ════════════════════════════════════════════════════════════════
// NtResult type alias
// ════════════════════════════════════════════════════════════════

pub type NtResult<T> = Result<T, NtError>;

// ════════════════════════════════════════════════════════════════
// Backward-compatible NeoTrixResult alias
// ════════════════════════════════════════════════════════════════

/// Backward-compatible alias — existing code uses `NeoTrixResult<T>`.
pub type NeoTrixResult<T> = NtResult<T>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_nt_error_display() {
        let e = NtError::Config("missing key".into());
        assert_eq!(format!("{}", e), "Config error: missing key");
    }

    #[test]
    fn test_nt_error_from_string() {
        let e: NtError = "something went wrong".into();
        assert!(format!("{}", e).contains("something went wrong"));
    }

    #[test]
    fn test_nt_error_from_io() {
        let io = std::io::Error::new(std::io::ErrorKind::Other, "io error");
        let e: NtError = io.into();
        match e {
            NtError::Io(_) => {}
            _ => panic!("expected Io variant"),
        }
    }

    #[test]
    fn test_nt_error_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid");
        let e: NtError = json_err.unwrap_err().into();
        match e {
            NtError::Serde(_) => {}
            _ => panic!("expected Serde variant"),
        }
    }

    #[test]
    fn test_nt_error_convenience_constructors() {
        let _ = NtError::config("test");
        let _ = NtError::io("test");
        let _ = NtError::not_found("test");
        let _ = NtError::invalid_input("test");
        let _ = NtError::not_implemented("test");
        let _ = NtError::operation_failed("test");
        let _ = NtError::safety_violation("test");
        let _ = NtError::steer("test");
    }

    #[test]
    fn test_nt_error_source() {
        let io = std::io::Error::new(std::io::ErrorKind::Other, "io error");
        let e = NtError::internal(io);
        assert!(e.source().is_some());
    }

    #[test]
    fn test_nt_result_type_alias() {
        let r: NtResult<i32> = Ok(42);
        assert_eq!(r.unwrap(), 42);
    }
}
