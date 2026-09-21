//! IM domain types - channel, bot, message, recovery.
//!
//! Extracted from single-file im.rs; public paths unchanged via im/mod.rs re-exports.

use crate::domain::serde_json;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ═══════════════════════════════════════════════
// Core Types (from dsh-im)
// ═══════════════════════════════════════════════

/// IM 渠道类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    WeChat,
    Feishu,
    DingTalk,
    WeCom,
    QQ,
    Slack,
    Telegram,
    Discord,
    WhatsApp,
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WeChat => write!(f, "wechat"),
            Self::Feishu => write!(f, "feishu"),
            Self::DingTalk => write!(f, "dingtalk"),
            Self::WeCom => write!(f, "wecom"),
            Self::QQ => write!(f, "qq"),
            Self::Slack => write!(f, "slack"),
            Self::Telegram => write!(f, "telegram"),
            Self::Discord => write!(f, "discord"),
            Self::WhatsApp => write!(f, "whatsapp"),
        }
    }
}

/// 所有内置渠道类型（单一事实源）
pub const ALL_CHANNEL_TYPES: &[ChannelType] = &[
    ChannelType::WeChat,
    ChannelType::Feishu,
    ChannelType::DingTalk,
    ChannelType::WeCom,
    ChannelType::QQ,
    ChannelType::Slack,
    ChannelType::Telegram,
    ChannelType::Discord,
    ChannelType::WhatsApp,
];

impl ChannelType {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "wechat" | "微信" => Some(Self::WeChat),
            "feishu" | "飞书" => Some(Self::Feishu),
            "dingtalk" | "钉钉" => Some(Self::DingTalk),
            "wecom" | "企业微信" => Some(Self::WeCom),
            "qq" => Some(Self::QQ),
            "slack" => Some(Self::Slack),
            "telegram" => Some(Self::Telegram),
            "discord" => Some(Self::Discord),
            "whatsapp" => Some(Self::WhatsApp),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::WeChat => "微信",
            Self::Feishu => "飞书",
            Self::DingTalk => "钉钉",
            Self::WeCom => "企业微信",
            Self::QQ => "QQ",
            Self::Slack => "Slack",
            Self::Telegram => "Telegram",
            Self::Discord => "Discord",
            Self::WhatsApp => "WhatsApp",
        }
    }
}

/// 渠道连接状态
#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChannelStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl std::fmt::Display for ChannelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "disconnected"),
            Self::Connecting => write!(f, "connecting"),
            Self::Connected => write!(f, "connected"),
            Self::Error(msg) => write!(f, "error: {msg}"),
        }
    }
}

/// 机器人配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BotConfig {
    pub id: String,
    pub channel: ChannelType,
    pub name: String,
    pub credential_type: String,
    pub workspace: Option<String>,
    pub model: Option<String>,
    pub enabled: bool,
    pub created_at: u64,
    /// 响应模式：group_invite / group_keyword / group_all / private
    pub response_mode: ResponseMode,
    /// 白名单用户（仅私聊模式使用）
    #[serde(default)]
    pub whitelist: HashSet<String>,
}

/// 响应模式
#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseMode {
    /// 群聊中需要被 @
    GroupInvite,
    /// 群聊中需要关键词触发
    GroupKeyword { keyword: String },
    /// 群聊中响应所有消息
    GroupAll,
    /// 私聊模式
    Private,
}

impl Default for ResponseMode {
    fn default() -> Self {
        Self::GroupInvite
    }
}

impl std::fmt::Display for ResponseMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GroupInvite => write!(f, "group_invite"),
            Self::GroupKeyword { keyword } => write!(f, "group_keyword:{keyword}"),
            Self::GroupAll => write!(f, "group_all"),
            Self::Private => write!(f, "private"),
        }
    }
}

/// 渠道配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub channel: ChannelType,
    pub enabled: bool,
    pub bots: Vec<BotConfig>,
    pub context_enhancement: bool,
    pub proactive_delivery: bool,
}

/// IM 系统状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImStatus {
    pub channels: Vec<ChannelConfig>,
    pub total_bots: usize,
    pub connected_bots: usize,
    pub dsh_market_enabled: bool,
}

/// 会话渠道前缀
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionChannelPrefix {
    pub channel: ChannelType,
    pub bot_id: String,
    pub chat_id: String,
}

impl SessionChannelPrefix {
    pub fn to_session_id(&self) -> String {
        format!("{}:{}:{}", self.channel, self.bot_id, self.chat_id)
    }

    pub fn from_session_id(session_id: &str) -> Option<Self> {
        let parts: Vec<&str> = session_id.split(':').collect();
        if parts.len() >= 3 {
            let channel = ChannelType::from_name(parts[0])?;
            Some(Self {
                channel,
                bot_id: parts[1].to_string(),
                chat_id: parts[2].to_string(),
            })
        } else {
            None
        }
    }
}

/// DSH 市场配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DshMarketConfig {
    pub enabled: bool,
    pub api_endpoint: String,
    pub auth_token: Option<String>,
    pub sync_enabled: bool,
    pub last_sync: Option<u64>,
}

// ═══════════════════════════════════════════════
// Single Inbox Pattern (from Multi-Channel Architecture)
// ═══════════════════════════════════════════════

/// 统一消息信封 — 所有渠道消息标准化为统一格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// 消息唯一 ID（用于幂等处理）
    pub id: String,
    /// 渠道类型
    pub channel: ChannelType,
    /// 机器人 ID
    pub bot_id: String,
    /// 聊天 ID（群聊/私聊）
    pub chat_id: String,
    /// 发送者 ID
    pub sender_id: String,
    /// 发送者显示名称
    pub sender_name: String,
    /// 消息内容
    pub content: MessageContent,
    /// 消息时间戳
    pub timestamp: u64,
    /// 渠道特定元数据
    pub channel_metadata: serde_json::Value,
    /// 消息类型
    pub message_type: MessageType,
    /// 回复目标消息 ID（如果是回复）
    pub reply_to: Option<String>,
    /// 线程 ID（群聊中）
    pub thread_id: Option<String>,
}

/// 消息内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Image {
        url: String,
        alt: Option<String>,
    },
    File {
        url: String,
        filename: String,
        mime_type: String,
    },
    Audio {
        url: String,
        duration_ms: Option<u64>,
    },
    Video {
        url: String,
        thumbnail: Option<String>,
    },
    Sticker {
        url: String,
    },
    Location {
        lat: f64,
        lng: f64,
        name: Option<String>,
    },
    Command {
        command: String,
        args: Vec<String>,
    },
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    UserMessage,
    BotMessage,
    SystemMessage,
    ActionMessage,
}

// ═══════════════════════════════════════════════
// Five Identity Layers (from Multi-Channel Architecture)
// ═══════════════════════════════════════════════

/// 五层身份模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityLayers {
    /// 1. 渠道用户 ID（平台特定）
    pub channel_user_id: String,
    /// 2. 全局用户 ID（跨渠道统一）
    pub global_user_id: String,
    /// 3. 租户 ID（多租户隔离）
    pub tenant_id: String,
    /// 4. 会话 ID（对话上下文）
    pub session_id: String,
    /// 5. 角色/权限
    pub roles: Vec<String>,
}

// ═══════════════════════════════════════════════
// Bot-to-Bot Communication (from Grok Bot)
// ═══════════════════════════════════════════════

/// 机器人间通信消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotMessage {
    /// 发送方机器人 ID
    pub from_bot_id: String,
    /// 接收方机器人 ID
    pub to_bot_id: String,
    /// 消息内容
    pub content: String,
    /// 消息类型
    pub message_type: BotMessageType,
    /// 共享上下文
    pub context: Option<serde_json::Value>,
    /// 时间戳
    pub timestamp: u64,
}

/// 机器人间消息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BotMessageType {
    /// 任务委派
    TaskDelegation,
    /// 上下文共享
    ContextShare,
    /// 状态更新
    StatusUpdate,
    /// 结果返回
    ResultReturn,
    /// 心跳
    Heartbeat,
}

// ═══════════════════════════════════════════════
// Workflow Learning (from Grok Bot)
// ═══════════════════════════════════════════════

/// 工作流模板（从演示中学习）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    /// 工作流 ID
    pub id: String,
    /// 工作流名称
    pub name: String,
    /// 工作流描述
    pub description: String,
    /// 步骤列表
    pub steps: Vec<WorkflowStep>,
    /// 触发条件
    pub trigger: WorkflowTrigger,
    /// 创建时间
    pub created_at: u64,
    /// 最后使用时间
    pub last_used: Option<u64>,
    /// 使用次数
    pub use_count: u32,
}

/// 工作流步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// 步骤 ID
    pub id: String,
    /// 步骤描述
    pub description: String,
    /// 操作类型
    pub action: String,
    /// 参数
    pub params: serde_json::Value,
    /// 预期输出
    pub expected_output: Option<String>,
}

/// 工作流触发条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowTrigger {
    /// 手动触发
    Manual,
    /// 消息触发
    Message { pattern: String },
    /// 定时触发
    Schedule { cron: String },
    /// 事件触发
    Event { event_type: String },
}

// ═══════════════════════════════════════════════
// Idempotent Processing
// ═══════════════════════════════════════════════

/// 消息处理状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageProcessingState {
    /// 消息 ID
    pub message_id: String,
    /// 处理状态
    pub status: ProcessingStatus,
    /// 处理开始时间
    pub started_at: u64,
    /// 处理完成时间
    pub completed_at: Option<u64>,
    /// 处理结果
    pub result: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 重试次数
    pub retry_count: u32,
}

/// 处理状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retry,
}

impl Default for DshMarketConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_endpoint: "https://dshfind.com/api".into(),
            auth_token: None,
            sync_enabled: true,
            last_sync: None,
        }
    }
}

/// 超时恢复配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutRecoveryConfig {
    /// 超时时间（秒）
    pub timeout_secs: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔（秒）
    pub retry_interval_secs: u64,
}

impl Default for TimeoutRecoveryConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_retries: 3,
            retry_interval_secs: 5,
        }
    }
}

/// 超时恢复状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutRecoveryState {
    pub bot_id: String,
    pub message_id: String,
    pub retry_count: u32,
    pub last_attempt: u64,
    pub status: TimeoutRecoveryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeoutRecoveryStatus {
    Pending,
    Retrying,
    Completed,
    Failed,
}

