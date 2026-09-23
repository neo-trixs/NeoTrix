//! `nt_types` — neobot 核心类型.
//!
//! 对标: OpenMuse `packages/domain/src/agent.ts` (`AgentTask`), cumora
//! `tools-shared.ts` (`TurnStatusValue`), openbot `computer/schema.ts`
//! (computer 工具契约). 本地 SQLite 存 `status` 字符串, 与这些枚举 1:1.

use serde::{Deserialize, Serialize};

/// 回合终态协议 — 直接复用 cumora `set_turn_status` 五态语义.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnStatus {
    Done,
    Continue,
    NeedsClarification,
    Blocked,
    Waiting,
}

impl TurnStatus {
    /// 字符串形式 (落库/CLI 协议用).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Continue => "continue",
            Self::NeedsClarification => "needs_clarification",
            Self::Blocked => "blocked",
            Self::Waiting => "waiting",
        }
    }

    /// 解析字符串, 未知返回 `None` (调用方按 fail-closed 处理).
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "done" => Some(Self::Done),
            "continue" => Some(Self::Continue),
            "needs_clarification" => Some(Self::NeedsClarification),
            "blocked" => Some(Self::Blocked),
            "waiting" => Some(Self::Waiting),
            _ => None,
        }
    }
}

/// 任务状态 — 对标 OpenMuse `AgentTask.status` + TaskWorker 租约机 (简化版).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Done,
    Failed,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "done" => Some(Self::Done),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// 本地任务 (durable, SQLite 持久化).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub created_at: String,
    pub updated_at: String,
}

/// token 用量 (OpenAI `usage` 形状子集, 落 `ledger` 表).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    /// 本地模型默认 0.0; 云端由上游账单回填.
    pub cost_usd: f64,
}

/// 极简 tool 名 — cumora 三板斧 + openbot computer 受控动作之最小交集.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolName {
    Bash,
    SetTurnStatus,
    ReadFile,
    WriteFile,
    EditFile,
    /// openbot `COMPUTER_ACTING_TOOLS` 本地占位: 当前只做策略门控,
    /// 具体 navigate/click/type 由后续 `nt_computer` 实现.
    ComputerAct,
}

impl ToolName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::SetTurnStatus => "set_turn_status",
            Self::ReadFile => "read_file",
            Self::WriteFile => "write_file",
            Self::EditFile => "edit_file",
            Self::ComputerAct => "computer_act",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "bash" => Some(Self::Bash),
            "set_turn_status" => Some(Self::SetTurnStatus),
            "read_file" => Some(Self::ReadFile),
            "write_file" => Some(Self::WriteFile),
            "edit_file" => Some(Self::EditFile),
            "computer_act" => Some(Self::ComputerAct),
            _ => None,
        }
    }
}

/// 一次工具调用 (LLM 侧发出, 网关先审后执).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: ToolName,
    /// 参数 JSON (如 `{"command":"ls"}` / `{"status":"done"}`).
    pub args: serde_json::Value,
}

/// 工具执行结果.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub ok: bool,
    pub output: String,
    pub truncated: bool,
}

#[cfg(test)]
mod tests {
    use super::{TaskStatus, ToolName, TurnStatus};

    #[test]
    fn turn_status_roundtrip() {
        for status in [
            TurnStatus::Done,
            TurnStatus::Continue,
            TurnStatus::NeedsClarification,
            TurnStatus::Blocked,
            TurnStatus::Waiting,
        ] {
            assert_eq!(TurnStatus::parse(status.as_str()), Some(status));
        }
        assert_eq!(TurnStatus::parse("nope"), None);
    }

    #[test]
    fn unknown_tool_is_none_fail_closed() {
        assert_eq!(ToolName::parse("rm_rf_root"), None);
        assert_eq!(TaskStatus::parse("bogus"), None);
    }
}
