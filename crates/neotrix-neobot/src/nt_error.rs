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
