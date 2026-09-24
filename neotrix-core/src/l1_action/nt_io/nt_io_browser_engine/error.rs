//! error — 从 `nt_io_browser_engine.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。



/// Browser errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum BrowserError {
    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("action failed: {0}")]
    ActionFailed(String),

    #[error("navigation failed: {0}")]
    NavigationFailed(String),

    #[error("auth rejected (http {0})")]
    AuthRejected(u16),

    #[error("auth expired: {hint}")]
    AuthExpired { hint: String },

    #[error("domain denied by allowlist: {0}")]
    DomainDenied(String),

    #[error("SSRF refused: {0}")]
    SsrfRefused(String),

    #[error("rate limited, retry after {0}s")]
    RateLimited(u64),

    #[error("domain cooling down until {0}ms")]
    CoolingDown(u64),

    #[error("session budget exhausted: {0}")]
    BudgetExhausted(String),

    #[error("session expired (ttl)")]
    SessionTtlExpired,

    #[error("backend unavailable: {0}")]
    BackendUnavailable(String),

    #[error("timeout")]
    Timeout,
}
