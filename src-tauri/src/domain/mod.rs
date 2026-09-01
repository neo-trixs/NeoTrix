//! # Domain Plugin System
//!
//! 基于 DeepSeek Harness 架构理念：一切皆插件，能力缝可替换。
//!
//! 每个功能域（session, chat, kb, ...）是一个 DomainPlugin，
//! 注册到 DomainRegistry，通过统一的 domain_call 入口调用。

pub mod registry;
pub mod plugins;

// Re-export serde_json for plugin convenience
pub use serde_json;
pub use registry::DomainRegistry;

use serde::{Deserialize, Serialize};
use std::fmt;

// ========== Core Types ==========

/// Action 规格 — 描述一个可调用操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSpec {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub params: Vec<ParamSpec>,
    #[serde(default = "default_returns")]
    pub returns: String,
}

fn default_returns() -> String { "Value".into() }

/// 参数规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSpec {
    pub name: String,
    pub r#type: String,
    pub description: String,
    #[serde(default)]
    pub optional: bool,
}

/// 统一请求 — 前端调用入口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCall {
    pub domain: String,
    pub action: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

/// 统一响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResponse {
    pub ok: bool,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<DomainError>,
}

/// 域错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainError {
    pub code: String,
    pub message: String,
    #[serde(default = "default_true")]
    pub recoverable: bool,
}

fn default_true() -> bool { true }

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl From<String> for DomainError {
    fn from(s: String) -> Self {
        Self { code: "DOMAIN_ERROR".into(), message: s, recoverable: true }
    }
}

impl From<&str> for DomainError {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

/// 域信息 — 用于 domain_list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    pub name: String,
    pub description: String,
    pub actions: Vec<ActionSpec>,
}

/// 域事件 — 跨插件通信
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub domain: String,
    pub event: String,
    pub data: serde_json::Value,
    pub timestamp: String,
}

// ========== Plugin Trait ==========

/// 域插件 trait — 每个功能域实现此 trait
///
/// 设计参考 DeepSeek Harness 的 Capability Seam 模式：
/// - Service Definition: `name()` + `actions()`
/// - Service Provider: `call()` + `init()`
/// - Consumer: 前端通过 `domain_call` 调用
pub trait DomainPlugin: Send + Sync {
    /// 域名称 (如 "session", "chat", "kb")
    fn name(&self) -> &str;

    /// 域描述
    fn description(&self) -> &str { "" }

    /// 该域支持的 action 列表
    fn actions(&self) -> Vec<ActionSpec>;

    /// 处理 action 调用
    fn call(&self, action: &str, args: serde_json::Value)
        -> Result<serde_json::Value, DomainError>;

    /// 初始化 (插件注册后调用)
    fn init(&mut self) -> Result<(), DomainError> { Ok(()) }

    /// 关闭 (应用退出前调用)
    fn shutdown(&mut self) -> Result<(), DomainError> { Ok(()) }
}

// ========== Helper ==========

/// 构造成功响应
pub fn ok(data: serde_json::Value) -> DomainResponse {
    DomainResponse { ok: true, data, error: None }
}

/// 构造错误响应
pub fn err(error: DomainError) -> DomainResponse {
    DomainResponse { ok: false, data: serde_json::Value::Null, error: Some(error) }
}

/// 构造错误响应 (from string)
pub fn err_msg(msg: impl Into<String>) -> DomainResponse {
    err(DomainError::from(msg.into()))
}
