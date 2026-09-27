//! `nt_types` — neobot 核心类型（任务/回合终态/工具名/转录项）。
//! 本地 SQLite 存 `status` 字符串，与这些枚举 1:1.

use serde::{Deserialize, Serialize};

/// 回合终态协议（五态：done/continue/waiting/failed/cancelled）.
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

/// 任务状态（pending/running/done/failed/cancelled，租约机简化版）.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Done,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "done" => Some(Self::Done),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
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
    /// 认领者 actor（原子认领；None=未认领）
    #[serde(default)]
    pub claimed_by: Option<String>,
    /// 所属会话 id（`conversations` 表；落库恒有值，IM 语义：同会话发送即追加）
    #[serde(default)]
    pub conversation_id: Option<String>,
    /// 认领时刻（RFC3339；过期可扫回）
    #[serde(default)]
    pub claimed_at: Option<String>,
    /// 可见性：team（默认）| private
    #[serde(default = "default_visibility")]
    pub visibility: String,
    /// 运行租约（崩溃恢复凭据）
    #[serde(default)]
    pub lease_id: Option<String>,
    #[serde(default)]
    pub lease_until: Option<String>,
    /// 已执行次数（含崩溃后重跑）
    #[serde(default)]
    pub attempts: i64,
    /// 末次失败原因（成功则空）
    #[serde(default)]
    pub error: Option<String>,
}

/// 新任务默认可见性（Team AI：默认队内可见）。
pub fn default_visibility() -> String {
    "team".to_owned()
}

/// token 用量 (OpenAI `usage` 形状子集, 落 `ledger` 表).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    /// 本地模型默认 0.0; 云端由上游账单回填.
    pub cost_usd: f64,
}

/// 极简 tool 名（bash + 状态机 + 3×FS + computer 受控动作 + 联网读写 + 未知兜底）.
/// `Unknown` 保留原始名用于审计，网关一律拒绝（fail-closed）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolName {
    Bash,
    SetTurnStatus,
    ReadFile,
    WriteFile,
    EditFile,
    /// computer 受控动作占位：当前只做策略门控，
    /// 具体 navigate/click/type 由后续 `nt_computer` 实现.
    ComputerAct,
    /// 联网搜索（DDG → Wikipedia 回退，客户端直调，对话即 crystal 全能力外表）。
    WebSearch,
    /// 网页抓取（http/https 门控，4000 字截断）。
    WebFetch,
    Unknown(String),
}

impl ToolName {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bash => "bash",
            Self::SetTurnStatus => "set_turn_status",
            Self::ReadFile => "read_file",
            Self::WriteFile => "write_file",
            Self::EditFile => "edit_file",
            Self::ComputerAct => "computer_act",
            Self::WebSearch => "web_search",
            Self::WebFetch => "web_fetch",
            Self::Unknown(_) => "unknown_tool",
        }
    }

    /// 未知名 → `Unknown(raw)` (审计保留原名, 策略层拒绝).
    pub fn parse(raw: &str) -> Self {
        match raw {
            "bash" => Self::Bash,
            "set_turn_status" => Self::SetTurnStatus,
            "read_file" => Self::ReadFile,
            "write_file" => Self::WriteFile,
            "edit_file" => Self::EditFile,
            "computer_act" => Self::ComputerAct,
            "web_search" | "search" => Self::WebSearch,
            "web_fetch" | "fetch" | "browse" => Self::WebFetch,
            _ => Self::Unknown(raw.to_owned()),
        }
    }

    /// 效果视角：机制名回答“调了什么”，intent 回答“改变了什么”。
    /// 审计明细用它，方便人按效果读。
    pub fn intent(&self) -> &'static str {
        match self {
            Self::Bash => "run_command",
            Self::SetTurnStatus => "turn_status",
            Self::ReadFile => "read_file",
            Self::WriteFile | Self::EditFile => "write_file",
            Self::ComputerAct => "computer_act",
            Self::WebSearch => "web_search",
            Self::WebFetch => "web_fetch",
            Self::Unknown(_) => "unknown_tool",
        }
    }
}

/// 一次工具调用 (LLM 侧发出, 网关先审后执).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// OpenAI `tool_calls[].id` 回填 (`tool` role 必需); 本地引擎合成.
    pub id: String,
    pub name: ToolName,
    /// 参数 JSON (如 `{"command":"ls"}` / `{"status":"done"}`).
    pub args: serde_json::Value,
}

/// 对话历史项 — 多跳 loop 的记忆 (OpenAI messages 兼容子集).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptItem {
    pub role: TranscriptRole,
    pub content: String,
    /// assistant 本轮发出的调用 (仅 Assistant).
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    /// tool 结果对应的调用 id (仅 Tool).
    #[serde(default)]
    pub tool_call_id: Option<String>,
}

/// 历史角色.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptRole {
    User,
    Assistant,
    Tool,
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
    fn unknown_tool_is_named_and_deniable() {
        let name = ToolName::parse("rm_rf_root");
        assert!(matches!(name, ToolName::Unknown(_)));
        assert_eq!(name.as_str(), "unknown_tool");
        assert_eq!(TaskStatus::parse("bogus"), None);
    }
}
