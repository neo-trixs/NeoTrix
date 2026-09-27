//! `nt_error` — neobot 统一错误 (无 `unwrap`/`panic`, 全经 `?` 传播).

use thiserror::Error;

/// neobot 全域错误.
#[derive(Debug, Error)]
pub enum NtBotError {
    /// SQLite 错误.
    #[error("store: {0}")]
    Store(String),
    /// 策略拒绝 (fail-closed, 非异常, 调用方按正常分支处理).
    #[error("denied by policy rule '{rule}': {reason}")]
    Denied { rule: String, reason: String },
    /// 引擎错误 (本地 CLI 缺失/执行失败等).
    #[error("engine '{engine}': {reason}")]
    Engine { engine: String, reason: String },
    /// 参数/协议错误.
    #[error("invalid: {0}")]
    Invalid(String),
    /// IO 错误.
    #[error("io: {0}")]
    Io(String),
    /// 序列化错误.
    #[error("codec: {0}")]
    Codec(String),
}

impl From<rusqlite::Error> for NtBotError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Store(value.to_string())
    }
}

impl From<serde_json::Error> for NtBotError {
    fn from(value: serde_json::Error) -> Self {
        Self::Codec(value.to_string())
    }
}

impl From<std::io::Error> for NtBotError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

/// 引擎失败机器可读分类（failover/重试决策只看 kind，不解析展示串）。
/// 顺序即优先级：resume → overflow → rate-limit → auth → transport → unknown。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineFailureKind {
    ResumeNotFound,
    ContextOverflow,
    RateLimit,
    Authentication,
    Transport,
    Unknown,
}

impl EngineFailureKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ResumeNotFound => "resume-not-found",
            Self::ContextOverflow => "context-overflow",
            Self::RateLimit => "rate-limit",
            Self::Authentication => "authentication",
            Self::Transport => "transport",
            Self::Unknown => "unknown",
        }
    }

    /// 可重试（transport/rate-limit 值得等一会儿再试；其余重试无意义）。
    pub fn is_retryable(self) -> bool {
        matches!(self, Self::Transport | Self::RateLimit)
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

/// 分类引擎错误文本（大小写不敏感；空串 → Unknown）。
pub fn classify_engine_failure(diagnostic: &str) -> EngineFailureKind {
    let text = diagnostic.to_ascii_lowercase();
    if text.contains("no such session")
        || text.contains("session not found")
        || text.contains("conversation not found")
        || text.contains("cannot resume")
        || text.contains("failed to resume")
    {
        return EngineFailureKind::ResumeNotFound;
    }
    if contains_any(
        &text,
        &[
            "context window",
            "context length",
            "context_length_exceeded",
            "maximum context",
            "prompt is too long",
            "input is too long",
            "too many tokens",
        ],
    ) {
        return EngineFailureKind::ContextOverflow;
    }
    if contains_any(
        &text,
        &[
            "rate limit",
            "rate_limit",
            "ratelimit",
            "too many requests",
            "quota",
            "overloaded",
            "over capacity",
            "usage limit",
        ],
    ) || text.contains("429")
    {
        return EngineFailureKind::RateLimit;
    }
    if contains_any(
        &text,
        &[
            "unauthorized",
            "unauthorised",
            "forbidden",
            "invalid api key",
            "invalid api_key",
            "authentication failed",
            "not logged in",
            "please sign in",
            "401",
        ],
    ) {
        return EngineFailureKind::Authentication;
    }
    if contains_any(
        &text,
        &[
            "econnrefused",
            "econnreset",
            "epipe",
            "socket hang up",
            "connection refused",
            "connection reset",
            "connection timed out",
            "connection closed",
            "connection lost",
            "network",
            "transport",
            "timed out",
            "timeout",
            "failed to spawn",
            "spawn failed",
        ],
    ) {
        return EngineFailureKind::Transport;
    }
    EngineFailureKind::Unknown
}

#[cfg(test)]
mod tests {
    use super::{EngineFailureKind, classify_engine_failure};

    #[test]
    fn failure_kinds_classified_in_priority_order() {
        assert_eq!(
            classify_engine_failure("Error 429: rate limit exceeded"),
            EngineFailureKind::RateLimit
        );
        assert_eq!(
            classify_engine_failure("401 unauthorized: invalid api key"),
            EngineFailureKind::Authentication
        );
        assert_eq!(
            classify_engine_failure("transport: Connection refused (os error 61)"),
            EngineFailureKind::Transport
        );
        assert_eq!(
            classify_engine_failure("spawn failed: No such file or directory"),
            EngineFailureKind::Transport
        );
        assert_eq!(
            classify_engine_failure("context_length_exceeded: prompt is too long"),
            EngineFailureKind::ContextOverflow
        );
        assert_eq!(
            classify_engine_failure("cannot resume: no such session"),
            EngineFailureKind::ResumeNotFound
        );
        assert_eq!(classify_engine_failure("weird new thing"), EngineFailureKind::Unknown);
        assert_eq!(classify_engine_failure(""), EngineFailureKind::Unknown);
        assert!(EngineFailureKind::Transport.is_retryable());
        assert!(EngineFailureKind::RateLimit.is_retryable());
        assert!(!EngineFailureKind::Authentication.is_retryable());
    }
}
