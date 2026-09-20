use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Unified application error type.
///
/// Every domain-specific error converts into `AppError` via `From` impls.
/// Tauri commands return `IpcResponse<T>` which wraps `IpcError` — use
/// `From<AppError> for IpcError` to bridge the two.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum AppError {
    /// I/O error (file read/write, network, etc.)
    #[error("IO error: {message}")]
    Io {
        code: String,
        message: String,
        recoverable: bool,
    },

    /// Serialization/deserialization error
    #[error("Serialization error: {message}")]
    Serde { code: String, message: String },

    /// Database error
    #[error("Database error: {message}")]
    Db { code: String, message: String },

    /// Authentication / authorization error
    #[error("Auth error: {message}")]
    Auth { code: String, message: String },

    /// Resource not found
    #[error("Not found: {message}")]
    NotFound { code: String, message: String },

    /// Invalid input / validation error
    #[error("Invalid input: {message}")]
    InvalidInput { code: String, message: String },

    /// Duplicate resource
    #[error("Duplicate: {message}")]
    Duplicate { code: String, message: String },

    /// Rate limited
    #[error("Rate limited: {message}")]
    RateLimited {
        code: String,
        message: String,
        retry_after_secs: Option<u64>,
    },

    /// Network error
    #[error("Network error: {message}")]
    Network { code: String, message: String },

    /// Timeout
    #[error("Timeout: {message}")]
    Timeout { code: String, message: String },

    /// Platform-specific error
    #[error("Platform error: {message}")]
    Platform { code: String, message: String },

    /// Permission denied
    #[error("Permission denied: {message}")]
    PermissionDenied { code: String, message: String },

    /// Encryption / decryption error
    #[error("Crypto error: {message}")]
    Crypto { code: String, message: String },

    /// Engine / model routing error
    #[error("Engine error: {message}")]
    Engine { code: String, message: String },

    /// Configuration error
    #[error("Config error: {message}")]
    Config { code: String, message: String },

    /// Generic catch-all
    #[error("{message}")]
    Other {
        code: String,
        message: String,
        recoverable: bool,
    },
}

impl AppError {
    pub fn code(&self) -> &str {
        match self {
            Self::Io { code, .. }
            | Self::Serde { code, .. }
            | Self::Db { code, .. }
            | Self::Auth { code, .. }
            | Self::NotFound { code, .. }
            | Self::InvalidInput { code, .. }
            | Self::Duplicate { code, .. }
            | Self::RateLimited { code, .. }
            | Self::Network { code, .. }
            | Self::Timeout { code, .. }
            | Self::Platform { code, .. }
            | Self::PermissionDenied { code, .. }
            | Self::Crypto { code, .. }
            | Self::Engine { code, .. }
            | Self::Config { code, .. }
            | Self::Other { code, .. } => code,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Io { message, .. }
            | Self::Serde { message, .. }
            | Self::Db { message, .. }
            | Self::Auth { message, .. }
            | Self::NotFound { message, .. }
            | Self::InvalidInput { message, .. }
            | Self::Duplicate { message, .. }
            | Self::RateLimited { message, .. }
            | Self::Network { message, .. }
            | Self::Timeout { message, .. }
            | Self::Platform { message, .. }
            | Self::PermissionDenied { message, .. }
            | Self::Crypto { message, .. }
            | Self::Engine { message, .. }
            | Self::Config { message, .. }
            | Self::Other { message, .. } => message,
        }
    }

    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Io { recoverable, .. } | Self::Other { recoverable, .. } => *recoverable,
            Self::RateLimited { .. } => true,
            Self::Timeout { .. } => true,
            Self::Network { .. } => true,
            _ => false,
        }
    }
}

// ── From impls: every domain error → AppError ────────────────────────

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::Io {
            code: "IO_ERROR".into(),
            message: e.to_string(),
            recoverable: false,
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde {
            code: "SERDE_ERROR".into(),
            message: e.to_string(),
        }
    }
}

impl From<toml::de::Error> for AppError {
    fn from(e: toml::de::Error) -> Self {
        Self::Serde {
            code: "TOML_PARSE_ERROR".into(),
            message: e.to_string(),
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Db {
            code: "DB_ERROR".into(),
            message: e.to_string(),
        }
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self::Other {
            code: "DOMAIN_ERROR".into(),
            message: s,
            recoverable: true,
        }
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        Self::Other {
            code: "DOMAIN_ERROR".into(),
            message: s.to_string(),
            recoverable: true,
        }
    }
}

// ── AppError → IpcError (for Tauri command boundary) ─────────────────

impl From<AppError> for crate::ipc::IpcError {
    fn from(e: AppError) -> Self {
        crate::ipc::IpcError {
            code: e.code().to_string(),
            message: e.message().to_string(),
            recoverable: e.is_recoverable(),
        }
    }
}

// ── DomainError → AppError ───────────────────────────────────────────

impl From<crate::domain::DomainError> for AppError {
    fn from(e: crate::domain::DomainError) -> Self {
        Self::Other {
            code: e.code,
            message: e.message,
            recoverable: e.recoverable,
        }
    }
}

// ── EngineError → AppError ───────────────────────────────────────────

impl From<crate::engine::EngineError> for AppError {
    fn from(e: crate::engine::EngineError) -> Self {
        use crate::engine::EngineError;
        match e {
            EngineError::AuthFailed(msg) => Self::Auth {
                code: "ENGINE_AUTH_FAILED".into(),
                message: msg,
            },
            EngineError::RateLimited { retry_after_secs } => Self::RateLimited {
                code: "ENGINE_RATE_LIMITED".into(),
                message: format!("Rate limited, retry in {retry_after_secs}s"),
                retry_after_secs: Some(retry_after_secs),
            },
            EngineError::ProviderError(msg) => Self::Engine {
                code: "ENGINE_PROVIDER_ERROR".into(),
                message: msg,
            },
            EngineError::NetworkError(msg) => Self::Network {
                code: "ENGINE_NETWORK_ERROR".into(),
                message: msg,
            },
            EngineError::ContextTooLong { max, actual } => Self::InvalidInput {
                code: "ENGINE_CONTEXT_TOO_LONG".into(),
                message: format!("Context {actual} exceeds max {max}"),
            },
            EngineError::ContentFiltered(msg) => Self::Engine {
                code: "ENGINE_CONTENT_FILTERED".into(),
                message: msg,
            },
            EngineError::Timeout(ms) => Self::Timeout {
                code: "ENGINE_TIMEOUT".into(),
                message: format!("Engine timed out after {ms}ms"),
            },
            EngineError::Unknown(msg) => Self::Engine {
                code: "ENGINE_UNKNOWN".into(),
                message: msg,
            },
        }
    }
}

// ── VaultError → AppError ────────────────────────────────────────────

impl From<crate::vault::VaultError> for AppError {
    fn from(e: crate::vault::VaultError) -> Self {
        use crate::vault::VaultError;
        match e {
            VaultError::TokenExpired => Self::Auth {
                code: "TOKEN_EXPIRED".into(),
                message: "Token expired".into(),
            },
            VaultError::NotFound(name) => Self::NotFound {
                code: "CREDENTIAL_NOT_FOUND".into(),
                message: format!("No credential for {name}"),
            },
            VaultError::EncryptionError(msg) => Self::Crypto {
                code: "ENCRYPTION_ERROR".into(),
                message: msg,
            },
        }
    }
}

// ── BotError → AppError ──────────────────────────────────────────────

impl From<crate::bot::BotError> for AppError {
    fn from(e: crate::bot::BotError) -> Self {
        use crate::bot::BotError;
        match e {
            BotError::NotFound(id) => Self::NotFound {
                code: "BOT_NOT_FOUND".into(),
                message: format!("Bot {id} not found"),
            },
            BotError::Duplicate(id) => Self::Duplicate {
                code: "BOT_DUPLICATE".into(),
                message: format!("Bot {id} already exists"),
            },
            BotError::InvalidInput(msg) => Self::InvalidInput {
                code: "BOT_INVALID_INPUT".into(),
                message: msg,
            },
        }
    }
}

// ── NotificationError → AppError ─────────────────────────────────────

impl From<crate::notifications::NotificationError> for AppError {
    fn from(e: crate::notifications::NotificationError) -> Self {
        use crate::notifications::NotificationError;
        match e {
            NotificationError::Platform(msg) => Self::Platform {
                code: "NOTIFICATION_PLATFORM".into(),
                message: msg,
            },
            NotificationError::PermissionDenied => Self::PermissionDenied {
                code: "NOTIFICATION_PERMISSION".into(),
                message: "Notification permission denied".into(),
            },
        }
    }
}

// ── BrowserError → AppError ──────────────────────────────────────────

impl From<crate::browser_host::BrowserError> for AppError {
    fn from(e: crate::browser_host::BrowserError) -> Self {
        use crate::browser_host::BrowserError;
        match e {
            BrowserError::InvalidUrl(msg) => Self::InvalidInput {
                code: "BROWSER_INVALID_URL".into(),
                message: msg,
            },
            BrowserError::WindowNotFound => Self::NotFound {
                code: "BROWSER_WINDOW_NOT_FOUND".into(),
                message: "Browser window not found".into(),
            },
            BrowserError::WindowCreation(msg) => Self::Platform {
                code: "BROWSER_WINDOW_CREATE".into(),
                message: msg,
            },
            BrowserError::JsEval(msg) => Self::Engine {
                code: "BROWSER_JS_EVAL".into(),
                message: msg,
            },
            BrowserError::Navigation(msg) => Self::Network {
                code: "BROWSER_NAVIGATION".into(),
                message: msg,
            },
            BrowserError::Network(msg) => Self::Network {
                code: "BROWSER_NETWORK".into(),
                message: msg,
            },
        }
    }
}

// ── ConfigError → AppError ───────────────────────────────────────────

impl From<crate::config::ConfigError> for AppError {
    fn from(e: crate::config::ConfigError) -> Self {
        use crate::config::ConfigError;
        match e {
            ConfigError::FileRead(path, io) => Self::Io {
                code: "CONFIG_FILE_READ".into(),
                message: format!("Failed to read {path}: {io}"),
                recoverable: false,
            },
            ConfigError::Parse(path, toml_err) => Self::Serde {
                code: "CONFIG_PARSE".into(),
                message: format!("Failed to parse {path}: {toml_err}"),
            },
            ConfigError::Validation(msg) => Self::InvalidInput {
                code: "CONFIG_VALIDATION".into(),
                message: msg,
            },
        }
    }
}

// ── AtomicWriteError → AppError ──────────────────────────────────────

impl From<crate::atomic_io::AtomicWriteError> for AppError {
    fn from(e: crate::atomic_io::AtomicWriteError) -> Self {
        use crate::atomic_io::AtomicWriteError;
        match e {
            AtomicWriteError::Write(path, io) => Self::Io {
                code: "ATOMIC_WRITE".into(),
                message: format!("Write to {path}: {io}"),
                recoverable: false,
            },
            AtomicWriteError::Sync(path, io) => Self::Io {
                code: "ATOMIC_SYNC".into(),
                message: format!("Fsync {path}: {io}"),
                recoverable: false,
            },
            AtomicWriteError::Rename(tmp, dest, io) => Self::Io {
                code: "ATOMIC_RENAME".into(),
                message: format!("Rename {tmp} → {dest}: {io}"),
                recoverable: false,
            },
            AtomicWriteError::Serialize(msg) => Self::Serde {
                code: "ATOMIC_SERIALIZE".into(),
                message: msg,
            },
            AtomicWriteError::Backup(path, io) => Self::Io {
                code: "ATOMIC_BACKUP".into(),
                message: format!("Backup {path}: {io}"),
                recoverable: false,
            },
        }
    }
}

impl From<crate::atomic_io::AtomicReadError> for AppError {
    fn from(e: crate::atomic_io::AtomicReadError) -> Self {
        use crate::atomic_io::AtomicReadError;
        match e {
            AtomicReadError::NotFound(main, backup, io) => Self::NotFound {
                code: "ATOMIC_READ_NOT_FOUND".into(),
                message: format!("Not found: {main} (backup: {backup}): {io}"),
            },
            AtomicReadError::Deserialize(path, serde_err) => Self::Serde {
                code: "ATOMIC_DESERIALIZE".into(),
                message: format!("Deserialize {path}: {serde_err}"),
            },
        }
    }
}

/// Convenience type alias.
pub type AppResult<T> = Result<T, AppError>;
