//! `nt_computer` — 受控 computer 动作.
//!
//! 语义移植 openbot `server/src/computer/schema.ts`
//! (`COMPUTER_TOOLS` 11 / `COMPUTER_ACTING_TOOLS` 8) + 网关
//! (resolve → policy → 先写 audit → 再执行).
//! 本地后端 trait 化: 当前仅 `NoopBackend` (诚实失败,
//! 不伪造截图/点击); 真浏览器后端 (Playwright/chromiumoxide)
//! 按 `ComputerBackend` 实现即插.

use serde::{Deserialize, Serialize};

use crate::nt_error::NtBotError;

/// 受控动作 (openbot `COMPUTER_ACTING_TOOLS` 本地子集).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputerAction {
    Navigate,
    Click,
    Type,
    Key,
    Scroll,
    Screenshot,
    ReadFile,
    WriteFile,
    ListFiles,
}

impl ComputerAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Navigate => "navigate",
            Self::Click => "click",
            Self::Type => "type",
            Self::Key => "key",
            Self::Scroll => "scroll",
            Self::Screenshot => "screenshot",
            Self::ReadFile => "read_file",
            Self::WriteFile => "write_file",
            Self::ListFiles => "list_files",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "navigate" => Some(Self::Navigate),
            "click" => Some(Self::Click),
            "type" => Some(Self::Type),
            "key" => Some(Self::Key),
            "scroll" => Some(Self::Scroll),
            "screenshot" => Some(Self::Screenshot),
            "read_file" => Some(Self::ReadFile),
            "write_file" => Some(Self::WriteFile),
            "list_files" => Some(Self::ListFiles),
            _ => None,
        }
    }
}

/// 一次 computer 调用 (网关已审, 后端执行).
#[derive(Debug, Clone)]
pub struct ComputerCall {
    pub action: ComputerAction,
    pub target: String,
    pub text: String,
}

/// 解析 `computer_act` 参数.
pub fn parse_computer_call(args: &serde_json::Value) -> Result<ComputerCall, NtBotError> {
    let action_raw = args
        .get("action")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let Some(action) = ComputerAction::parse(action_raw) else {
        return Err(NtBotError::Invalid(format!("unknown computer action '{action_raw}'")));
    };
    Ok(ComputerCall {
        action,
        target: args
            .get("target")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_owned(),
        text: args
            .get("text")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_owned(),
    })
}

/// computer 后端接口 — 真实现 (浏览器/CDP) 按此 trait 接入.
pub trait ComputerBackend {
    fn backend_id(&self) -> &str;
    fn execute(&self, call: &ComputerCall) -> Result<String, NtBotError>;
}

/// 空后端 — 永远诚实失败, 不伪造任何屏幕/点击结果.
#[derive(Debug, Default)]
pub struct NoopBackend;

impl ComputerBackend for NoopBackend {
    fn backend_id(&self) -> &str {
        "noop"
    }

    fn execute(&self, call: &ComputerCall) -> Result<String, NtBotError> {
        Err(NtBotError::Engine {
            engine: "computer".to_owned(),
            reason: format!(
                "computer backend not configured; refused {} (target hidden)",
                call.action.as_str()
            ),
        })
    }
}

/// 从 URL/类 URL 目标中提取 host (无 `url` 依赖的最小实现).
pub fn host_of(target: &str) -> Option<String> {
    let after_scheme = target.split_once("://").map(|(_, rest)| rest).unwrap_or(target);
    let host = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .trim()
        .trim_end_matches('.');
    if host.is_empty() {
        return None;
    }
    Some(host.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::{NoopBackend, ComputerBackend, host_of, parse_computer_call};

    #[test]
    fn parses_action() {
        let call = parse_computer_call(&serde_json::json!({
            "action": "navigate", "target": "https://example.com/a?b=1",
        }))
        .expect("parse");
        assert_eq!(call.action, super::ComputerAction::Navigate);
        assert_eq!(host_of(&call.target).as_deref(), Some("example.com"));
    }

    #[test]
    fn noop_never_executes() {
        let call = parse_computer_call(&serde_json::json!({"action": "click"})).expect("parse");
        assert!(NoopBackend.execute(&call).is_err());
    }

    #[test]
    fn unknown_action_rejected() {
        assert!(parse_computer_call(&serde_json::json!({"action": "rm"})).is_err());
    }
}
