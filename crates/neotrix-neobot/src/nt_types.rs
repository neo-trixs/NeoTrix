//! `nt_types` — neobot 核心类型（任务/回合终态/工具名/转录项）。
//! 本地 SQLite 存 `status` 字符串，与这些枚举 1:1.

use serde::{Deserialize, Serialize};

/// 回合终态协议（五态：done/continue/needs_clarification/blocked/waiting）.
///
/// **没有 `failed`，更没有 `cancelled`** —— 早先这行注释写着
/// 「done/continue/waiting/failed/cancelled」，两个都不存在。写错的后果很具体：
/// 照着它去实现 `/stop` 的人会以为已经有个取消态可用，实际上**整条链路上没有
/// 任何地方能产出取消**（`nt_agent` 也没有 stop hook），于是 `/stop` 只能回一句
/// 「停不了」。
///
/// **但别把这句读成「整个 neobot 没有取消概念」** —— 取消在**另一层**已经存在：
/// 落库用的是 [`TaskStatus`]，它**本来就有 `Cancelled`**（`pending/running/done/
/// failed/cancelled`），`cancel_task` 早就在产出它。真正缺的不是那个状态值，
/// 而是「**在飞的那一轮**能被就地转成它」—— 即 `nt_agent` 侧的取消钩子。
/// 故做 `/stop` 时**不必**给本枚举新增变体：把中止落成 `TaskStatus::Cancelled`
/// 即可，理由与管线见 `docs/architecture/DESIGN-CHANNEL-DISPATCH.md` §10.5。
///
/// 落点澄清（本行早先写着「与 `nt_store/nt_store_turns.rs` 的落库字符串 1:1」——
/// **那个文件不存在，也没有 `turns` 表**，是又一处说谎的注释）：本枚举是
/// **模型侧协议**（引擎产出、dispatch 翻译成用户文案），**不落库**；
/// 落库的是 `tasks.status` 里的 [`TaskStatus`]。
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
    /// 看图（工作区内的图片 → 真正的多模态 content part）。
    ///
    /// **与 `read_file` 分家是规则性的**：文本走 `read_file`（进 `content` 字符串），
    /// 图像走这里（进 `TranscriptItem.image`，随下一次请求以 `image_url` 部件抵达
    /// 模型）。若把 base64 塞进 `content`，模型收到的是一堵字符墙 —— 那比不给更糟，
    /// 因为它会开始「描述」它其实什么都没看见的东西。
    ReadImage,
    /// computer 受控动作占位：当前只做策略门控，
    /// 具体 navigate/click/type 由后续 `nt_computer` 实现.
    ComputerAct,
    /// 联网搜索（DDG → Wikipedia 回退，客户端直调，对话即 crystal 全能力外表）。
    WebSearch,
    /// 网页抓取（http/https 门控，4000 字截断）。
    WebFetch,
    /// 侧边栏导航：模型主动让 UI 打开文件 / 文件夹 / 任务面板。
    ///
    /// **只出指令、不碰 UI**：Rust 侧把 `topic` + `target` 记成一步，
    /// 由前端（唯一持有界面状态的一方）解释并执行。模型因此只能「提议打开」，
    /// 开不开、开哪个 tab 永远由界面说了算 —— 与 better-sidebar 的
    /// `sidebar_open` 同构。
    SidebarOpen,
    Unknown(String),
}

impl ToolName {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bash => "bash",
            Self::SetTurnStatus => "set_turn_status",
            Self::ReadFile => "read_file",
            Self::ReadImage => "read_image",
            Self::WriteFile => "write_file",
            Self::EditFile => "edit_file",
            Self::ComputerAct => "computer_act",
            Self::WebSearch => "web_search",
            Self::WebFetch => "web_fetch",
            Self::SidebarOpen => "sidebar_open",
            Self::Unknown(_) => "unknown_tool",
        }
    }

    /// 未知名 → `Unknown(raw)` (审计保留原名, 策略层拒绝).
    pub fn parse(raw: &str) -> Self {
        match raw {
            "bash" => Self::Bash,
            "set_turn_status" => Self::SetTurnStatus,
            "read_file" => Self::ReadFile,
            // 别名：模型常把「看图」说成 view/see。收窄到这三个，
            // 免得 `image` 这种裸词日后被别的工具含义劫持。
            "read_image" | "view_image" | "see_image" => Self::ReadImage,
            "write_file" => Self::WriteFile,
            "edit_file" => Self::EditFile,
            "computer_act" => Self::ComputerAct,
            "web_search" | "search" => Self::WebSearch,
            "web_fetch" | "fetch" | "browse" => Self::WebFetch,
            "sidebar_open" | "open_sidebar" | "sidebar" => Self::SidebarOpen,
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
            Self::ReadImage => "read_image",
            Self::WriteFile | Self::EditFile => "write_file",
            Self::ComputerAct => "computer_act",
            Self::WebSearch => "web_search",
            Self::WebFetch => "web_fetch",
            Self::SidebarOpen => "navigate",
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

/// 一张要送进模型的图像（base64 data URL 形态）。
///
/// 只在**真的要送**的时候构造：base64 膨胀 4/3，一张 4 MiB 的图会变成约
/// 5.6 MiB 的请求体。落盘/序列化进 SQLite 都不是它的归宿 —— 图像的旅程
/// 只有一个终点：下一次 `/v1/chat/completions` 的 `content` 数组。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImagePart {
    /// IANA media type（如 `image/png`）。
    ///
    /// 来自**魔数**而非扩展名：`.png` 里装的是 zip 就不是 png，
    /// 照扩展名填 media type 等于让解码器去接一颗不属于它的炸弹。
    pub media_type: String,
    /// 纯 base64（无换行、无 `data:` 前缀；由 `nt_vision::base64_encode` 产出）。
    pub base64: String,
}

impl ImagePart {
    /// OpenAI 兼容请求里 `image_url.url` 的取值：`data:<media_type>;base64,<payload>`。
    pub fn data_url(&self) -> String {
        format!("data:{};base64,{}", self.media_type, self.base64)
    }
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
    /// 工具产出的图像部件（仅 Tool；`read_image` 命中时为 `Some`）。
    ///
    /// 为什么单开一个字段而不拼进 `content`：`content` 是**纯文本**通道，
    /// OpenAI 兼容形状下它会被序列化成字符串，图像塞进去只会得到一堵 base64
    /// 字符墙。这个字段是那趟运输的载体 —— `nt_http_engine::transcript_message`
    /// 见到它就把该行 content 从字符串升级成 `[{type:text},{type:image_url}]` 数组，
    /// 图像这才能以**真正的多模态部件**抵达模型。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<ImagePart>,
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

    #[test]
    fn read_image_wires_every_layer() {
        // as_str ↔ parse 必须闭合：schema 里的名字要能被 parse 原样认回来。
        for raw in ["read_image", "view_image", "see_image"] {
            let name = ToolName::parse(raw);
            assert_eq!(name, ToolName::ReadImage, "alias '{raw}' must parse");
            assert_eq!(name.as_str(), "read_image");
        }
        assert_eq!(ToolName::ReadImage.intent(), "read_image");
        // 别名不得过宽：裸 `image` 留给未来的别的语义。
        assert!(matches!(ToolName::parse("image"), ToolName::Unknown(_)));
    }

    #[test]
    fn image_part_data_url_is_openai_shaped() {
        let part = super::ImagePart {
            media_type: "image/png".to_owned(),
            base64: "iVBORw0KGgo=".to_owned(),
        };
        assert_eq!(part.data_url(), "data:image/png;base64,iVBORw0KGgo=");
        // 反序列化时缺 image 字段（旧库/旧行）不得炸。
        let item: super::TranscriptItem = serde_json::from_str(
            r#"{"role":"tool","content":"ok","tool_call_id":"c1"}"#,
        )
        .expect("old transcript row still parses");
        assert!(item.image.is_none());
    }
}
