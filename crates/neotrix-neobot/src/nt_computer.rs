//! `nt_computer` — 受控 computer 动作.
//!
//! 受控动作表（11 声明 / 8 执行）+ 网关（resolve → policy →
//! 先写 audit → 再执行）。本地后端 trait 化：当前仅 `NoopBackend`
//! （诚实失败，不伪造截图/点击）；真浏览器后端按 `ComputerBackend`
//! 实现即插.

use serde::{Deserialize, Serialize};

use crate::nt_error::NtBotError;

/// 受控动作（执行子集）.
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

/// 动作回执三态（`computer-use` P7「防重放」吸收）。
///
/// **为什么要三态**：动作「发出去没有」不是两态。当回执是
/// `OutcomeUnknown`（已派发、效果未知）时，**任何自动重试都可能双击/双下单**
/// ⇒ 该态**必须**由上层决策（人），永不自动重试。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionOutcome {
    /// 已确认生效（后端回读过，或确定性 API 返回成功）。
    Applied,
    /// 已派发但效果未知 ⇒ **禁止自动重试**（P7 possibly_sent）。
    Unknown,
    /// 明确未生效（拒绝 / 参数错 / 连接失败）。
    Failed,
}

/// 上层处置建议（对应「Grok 错误即指令」的四档：换授权/换策略/停止/可重试）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryAdvice {
    /// 可原样重试（幂等失败：网络/连接层）。
    Retry,
    /// 需用户改授权/换目标再试（policy 拒绝、域禁区）。
    ChangeAuth,
    /// 应停止本轮（动作语义不支持、连续失败）。
    Stop,
    /// 效果未知 ⇒ 只能问人，禁止自动重试。
    AskHuman,
}

/// 一次动作的完整回执（动作面 → UI/账本）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ActionReceipt {
    pub action: String,
    /// 是否**已派发**给后端（决定是否可能重复生效）。
    pub sent: bool,
    pub outcome: ActionOutcome,
    pub advice: RetryAdvice,
    /// 人话原因（失败/未知时必填）。
    pub detail: String,
}

/// 把网关/后端错误映射成「三态 + 四档建议」。
///
/// 映射依据（`computer-use` P7 + 我方 `nt_cancel`/`tasks.outcome_unknown` 语义）：
/// - `Denied`（策略拒绝）⇒ **未派发** + `Failed` + `ChangeAuth`；
/// - `Invalid`（参数错）⇒ 未派发 + `Failed` + `Stop`；
/// - `Io`（传输层失败，**动作可能已发出**）⇒ `sent=true` + `Unknown` + `Retry`；
/// - `Engine{reason 含 refused/backend}` ⇒ 未派发 + `Failed` + `Stop`；
/// - 其它 ⇒ `Unknown` + `AskHuman`（**默认最保守**：宁可问人也不双击）。
#[must_use]
pub fn receipt_for_error(action: &str, err: &NtBotError) -> ActionReceipt {
    let (sent, outcome, advice, detail) = match err {
        NtBotError::Denied { rule, reason } => (
            false,
            ActionOutcome::Failed,
            RetryAdvice::ChangeAuth,
            format!("denied by '{rule}': {reason}"),
        ),
        NtBotError::Invalid(detail) => (false, ActionOutcome::Failed, RetryAdvice::Stop, detail.clone()),
        NtBotError::Io(detail) => (true, ActionOutcome::Unknown, RetryAdvice::Retry, detail.clone()),
        NtBotError::Engine { engine, reason } => {
            let refused = reason.contains("refused") || reason.contains("not configured");
            (
                !refused,
                if refused { ActionOutcome::Failed } else { ActionOutcome::Unknown },
                if refused { RetryAdvice::Stop } else { RetryAdvice::AskHuman },
                format!("{engine}: {reason}"),
            )
        }
        other => (
            true,
            ActionOutcome::Unknown,
            RetryAdvice::AskHuman,
            other.to_string(),
        ),
    };
    ActionReceipt {
        action: action.to_owned(),
        sent,
        outcome,
        advice,
        detail,
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

    #[test]
    fn receipt_denial_is_not_sent_and_asks_for_auth() {
        use crate::nt_error::NtBotError;
        use super::{receipt_for_error, ActionOutcome, RetryAdvice};
        let r = receipt_for_error(
            "click",
            &NtBotError::Denied { rule: "domain-ban".into(), reason: "target denied".into() },
        );
        assert!(!r.sent, "策略拒绝发生在派发之前");
        assert_eq!(r.outcome, ActionOutcome::Failed);
        assert_eq!(r.advice, RetryAdvice::ChangeAuth);
    }

    #[test]
    fn receipt_io_is_unknown_but_retryable() {
        use crate::nt_error::NtBotError;
        use super::{receipt_for_error, ActionOutcome, RetryAdvice};
        let r = receipt_for_error("type", &NtBotError::Io("connection reset".into()));
        assert!(r.sent, "传输层失败不能断言未派发");
        assert_eq!(r.outcome, ActionOutcome::Unknown);
        assert_eq!(r.advice, RetryAdvice::Retry, "幂等传输失败才可自动重试");
    }

    #[test]
    fn receipt_backend_refused_is_failed_not_unknown() {
        use crate::nt_error::NtBotError;
        use super::{receipt_for_error, ActionOutcome, RetryAdvice};
        let call = parse_computer_call(&serde_json::json!({"action": "click"})).expect("parse");
        let err = NoopBackend.execute(&call).expect_err("noop refuses");
        let r = receipt_for_error(call.action.as_str(), &err);
        assert!(!r.sent);
        assert_eq!(r.outcome, ActionOutcome::Failed);
        assert_eq!(r.advice, RetryAdvice::Stop);
    }
}
