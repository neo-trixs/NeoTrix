//! # IM Domain Plugin
//!
//! 基于 DSH-IM 架构的多渠道 IM 插件：
//! - 9 个内置渠道：微信、飞书、钉钉、企业微信、QQ、Slack、Telegram、Discord、WhatsApp
//! - 流式响应支持
//! - 访问模式控制（白名单/响应模式）
//! - 超时恢复机制
//! - 会话渠道前缀路由

use crate::atomic_io;
use crate::domain::app_handle::{get_app_handle, set_app_handle};
use crate::domain::registry::DomainRegistry;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin, ParamSpec};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Emitter;

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

// ═══════════════════════════════════════════════
// IM Plugin
// ═══════════════════════════════════════════════

pub struct ImPlugin {
    config_path: PathBuf,
    timeout_recovery: Arc<tokio::sync::Mutex<HashMap<String, TimeoutRecoveryState>>>,
    timeout_config: TimeoutRecoveryConfig,
    registry: Arc<tokio::sync::RwLock<DomainRegistry>>,
    /// 消息处理状态（用于幂等处理）
    message_states: Arc<tokio::sync::Mutex<HashMap<String, MessageProcessingState>>>,
    /// 机器人间消息队列
    bot_message_queue: Arc<tokio::sync::Mutex<Vec<BotMessage>>>,
    /// 工作流模板
    workflow_templates: Arc<tokio::sync::Mutex<HashMap<String, WorkflowTemplate>>>,
    /// 用户身份映射（channel_user_id -> global_user_id）
    identity_map: Arc<tokio::sync::Mutex<HashMap<String, String>>>,
}

impl ImPlugin {
    pub fn new(registry: Arc<tokio::sync::RwLock<DomainRegistry>>) -> Self {
        let config_path = crate::config::AppConfig::base_dir()
            .map(|h| h.join("im_channels.json"))
            .unwrap_or_else(|| PathBuf::from(".neotrix/im_channels.json"));

        Self {
            config_path,
            timeout_recovery: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            timeout_config: TimeoutRecoveryConfig::default(),
            registry,
            message_states: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            bot_message_queue: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            workflow_templates: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            identity_map: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    /// 检查消息是否已处理（幂等性检查）
    async fn is_message_processed(&self, message_id: &str) -> bool {
        let states = self.message_states.lock().await;
        states
            .get(message_id)
            .map(|s| s.status == ProcessingStatus::Completed)
            .unwrap_or(false)
    }

    /// 记录消息处理状态
    async fn record_message_state(&self, message_id: &str, status: ProcessingStatus) {
        let mut states = self.message_states.lock().await;
        let now = chrono::Utc::now().timestamp() as u64;

        let state =
            states
                .entry(message_id.to_string())
                .or_insert_with(|| MessageProcessingState {
                    message_id: message_id.to_string(),
                    status: ProcessingStatus::Pending,
                    started_at: now,
                    completed_at: None,
                    result: None,
                    error: None,
                    retry_count: 0,
                });

        state.status = status;
        if state.status == ProcessingStatus::Completed || state.status == ProcessingStatus::Failed {
            state.completed_at = Some(now);
        }
    }

    /// 获取全局用户 ID（五层身份模型）
    async fn get_global_user_id(&self, channel: &ChannelType, channel_user_id: &str) -> String {
        let key = format!("{}:{}", channel, channel_user_id);
        let mut map = self.identity_map.lock().await;

        map.get(&key).cloned().unwrap_or_else(|| {
            let short_id = uuid::Uuid::new_v4().to_string();
            let global_id = format!("user-{}", &short_id[..8]);
            map.insert(key, global_id.clone());
            global_id
        })
    }

    /// 发送机器人间消息
    async fn send_bot_message(&self, message: BotMessage) -> Result<(), DomainError> {
        let mut queue = self.bot_message_queue.lock().await;
        queue.push(message);
        Ok(())
    }

    /// 接收机器人间消息
    async fn receive_bot_messages(&self, bot_id: &str) -> Vec<BotMessage> {
        let mut queue = self.bot_message_queue.lock().await;
        let mut received = Vec::new();
        let mut remaining = Vec::new();

        for msg in queue.drain(..) {
            if msg.to_bot_id == bot_id {
                received.push(msg);
            } else {
                remaining.push(msg);
            }
        }

        queue.extend(remaining);
        received
    }

    /// 保存工作流模板
    async fn save_workflow_template(&self, template: WorkflowTemplate) -> Result<(), DomainError> {
        let mut templates = self.workflow_templates.lock().await;
        templates.insert(template.id.clone(), template);
        Ok(())
    }

    /// 获取工作流模板
    async fn get_workflow_template(&self, template_id: &str) -> Option<WorkflowTemplate> {
        let templates = self.workflow_templates.lock().await;
        templates.get(template_id).cloned()
    }

    /// 列出所有工作流模板
    async fn list_workflow_templates(&self) -> Vec<WorkflowTemplate> {
        let templates = self.workflow_templates.lock().await;
        templates.values().cloned().collect()
    }

    fn base_dir(&self) -> std::path::PathBuf {
        self.config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from(".neotrix"))
    }

    fn load_channels(&self) -> Result<Vec<ChannelConfig>, DomainError> {
        crate::domain::im_service::load_channels(&self.base_dir()).map_err(|e| DomainError {
            code: "CONFIG_READ_ERROR".into(),
            message: format!("读取 IM 配置失败: {}", e),
            recoverable: true,
        })
    }

    fn save_channels(&self, channels: &[ChannelConfig]) -> Result<(), DomainError> {
        crate::domain::im_service::save_channels(&self.base_dir(), channels).map_err(|e| {
            DomainError {
                code: "CONFIG_WRITE_ERROR".into(),
                message: format!("写入配置失败: {}", e),
                recoverable: true,
            }
        })
    }

    fn default_channels(&self) -> Vec<ChannelConfig> {
        crate::domain::im_service::default_channels()
    }

    /// 检查是否应该响应消息
    fn should_respond(&self, bot: &BotConfig, chat_id: &str, sender_id: &str, text: &str) -> bool {
        match &bot.response_mode {
            ResponseMode::Private => {
                // 私聊模式：检查白名单
                if bot.whitelist.is_empty() {
                    true // 白名单为空时允许所有
                } else {
                    bot.whitelist.contains(sender_id)
                }
            }
            ResponseMode::GroupInvite => {
                // 群聊中需要被 @
                text.contains(&format!("@{}", bot.name))
            }
            ResponseMode::GroupKeyword { keyword } => {
                // 群聊中需要关键词触发
                text.contains(keyword)
            }
            ResponseMode::GroupAll => {
                // 群聊中响应所有消息
                true
            }
        }
    }

    /// 生成会话 ID（带渠道前缀）
    fn make_session_id(&self, channel: &ChannelType, bot_id: &str, chat_id: &str) -> String {
        format!("{}:{}:{}", channel, bot_id, chat_id)
    }

    /// 处理超时恢复
    async fn handle_timeout_recovery(&self, bot_id: &str, message_id: &str) -> Result<(), DomainError> {
        let mut state = self.timeout_recovery.lock().await;

        let entry = state
            .entry(message_id.to_string())
            .or_insert_with(|| TimeoutRecoveryState {
                bot_id: bot_id.to_string(),
                message_id: message_id.to_string(),
                retry_count: 0,
                last_attempt: 0,
                status: TimeoutRecoveryStatus::Pending,
            });

        if entry.retry_count >= self.timeout_config.max_retries {
            entry.status = TimeoutRecoveryStatus::Failed;
            return Ok(());
        }

        entry.retry_count += 1;
        entry.status = TimeoutRecoveryStatus::Retrying;
        entry.last_attempt = chrono::Utc::now().timestamp() as u64;

        Ok(())
    }

    /// 发送消息到渠道（模拟）
    #[tracing::instrument(skip(self, content), fields(channel = %channel, bot_id = %bot_id, chat_id = %chat_id))]
    async fn send_to_channel(
        &self,
        channel: &ChannelType,
        bot_id: &str,
        chat_id: &str,
        content: &str,
    ) -> Result<String, DomainError> {
        // 模拟发送消息
        let message_id = format!("msg-{}", chrono::Utc::now().timestamp_millis());

        // 发射事件到前端
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit(
                "im_message_sent",
                serde_json::json!({
                    "channel": channel.to_string(),
                    "bot_id": bot_id,
                    "chat_id": chat_id,
                    "message_id": message_id,
                    "content": content,
                }),
            ) {
                tracing::warn!("Failed to emit im_message_sent: {e}");
            }
        }

        Ok(message_id)
    }

    /// 流式发送消息
    async fn send_streaming(
        &self,
        channel: &ChannelType,
        bot_id: &str,
        chat_id: &str,
        content: &str,
    ) -> Result<String, DomainError> {
        // 模拟流式发送
        let message_id = format!("msg-{}", chrono::Utc::now().timestamp_millis());

        // 发射流式事件
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit(
                "im_stream_start",
                serde_json::json!({
                    "channel": channel.to_string(),
                    "bot_id": bot_id,
                    "chat_id": chat_id,
                    "message_id": message_id,
                }),
            ) {
                tracing::warn!("Failed to emit im_stream_start: {e}");
            }

            // 模拟流式 token
            for token in content.chars() {
                if let Err(e) = app.emit(
                    "im_stream_token",
                    serde_json::json!({
                        "message_id": message_id,
                        "token": token.to_string(),
                    }),
                ) {
                    tracing::warn!("Failed to emit im_stream_token: {e}");
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }

            if let Err(e) = app.emit(
                "im_stream_end",
                serde_json::json!({
                    "message_id": message_id,
                    "content": content,
                }),
            ) {
                tracing::warn!("Failed to emit im_stream_end: {e}");
            }
        }

        Ok(message_id)
    }

    /// DSH 市场配置路径
    fn dsh_market_path(&self) -> PathBuf {
        crate::domain::im_service::dsh_market_path()
    }

    /// 加载 DSH 市场配置
    fn load_dsh_market(&self) -> Result<DshMarketConfig, DomainError> {
        crate::domain::im_service::load_dsh_market(&self.base_dir()).map_err(|e| DomainError {
            code: "CONFIG_READ_ERROR".into(),
            message: format!("读取 DSH 市场配置失败: {}", e),
            recoverable: true,
        })
    }

    /// 保存 DSH 市场配置
    fn save_dsh_market(&self, config: &DshMarketConfig) -> Result<(), DomainError> {
        crate::domain::im_service::save_dsh_market(&self.base_dir(), config).map_err(|e| {
            DomainError {
                code: "CONFIG_WRITE_ERROR".into(),
                message: format!("写入 DSH 市场配置失败: {}", e),
                recoverable: true,
            }
        })
    }
}

#[async_trait]
impl DomainPlugin for ImPlugin {
    fn name(&self) -> &str {
        "im"
    }

    fn description(&self) -> &str {
        "IM 多渠道管理：微信、飞书、钉钉、企业微信、QQ、Slack、Telegram、Discord、WhatsApp"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            // 渠道管理
            ActionSpec {
                name: "status".into(),
                description: "获取 IM 系统状态".into(),
                params: vec![],
                returns: "ImStatus".into(),
            },
            ActionSpec {
                name: "list_channels".into(),
                description: "获取所有渠道配置".into(),
                params: vec![],
                returns: "Vec<ChannelConfig>".into(),
            },
            ActionSpec {
                name: "get_channel".into(),
                description: "获取单个渠道配置".into(),
                params: vec![ParamSpec {
                    name: "channel".into(),
                    r#type: "string".into(),
                    description: "渠道标识".into(),
                    optional: false,
                }],
                returns: "ChannelConfig".into(),
            },
            ActionSpec {
                name: "toggle_channel".into(),
                description: "启用/禁用渠道".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "enabled".into(),
                        r#type: "bool".into(),
                        description: "是否启用".into(),
                        optional: false,
                    },
                ],
                returns: "ChannelConfig".into(),
            },
            // 机器人管理
            ActionSpec {
                name: "add_bot".into(),
                description: "添加机器人".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "name".into(),
                        r#type: "string".into(),
                        description: "机器人名称".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "credential_type".into(),
                        r#type: "string".into(),
                        description: "凭证类型".into(),
                        optional: true,
                    },
                    ParamSpec {
                        name: "workspace".into(),
                        r#type: "string".into(),
                        description: "工作空间".into(),
                        optional: true,
                    },
                    ParamSpec {
                        name: "model".into(),
                        r#type: "string".into(),
                        description: "模型标识".into(),
                        optional: true,
                    },
                ],
                returns: "BotConfig".into(),
            },
            ActionSpec {
                name: "remove_bot".into(),
                description: "删除机器人".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "bot_id".into(),
                        r#type: "string".into(),
                        description: "机器人 ID".into(),
                        optional: false,
                    },
                ],
                returns: "bool".into(),
            },
            ActionSpec {
                name: "update_bot".into(),
                description: "更新机器人配置".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "bot_id".into(),
                        r#type: "string".into(),
                        description: "机器人 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "name".into(),
                        r#type: "string".into(),
                        description: "机器人名称".into(),
                        optional: true,
                    },
                    ParamSpec {
                        name: "workspace".into(),
                        r#type: "string".into(),
                        description: "工作空间".into(),
                        optional: true,
                    },
                    ParamSpec {
                        name: "model".into(),
                        r#type: "string".into(),
                        description: "模型标识".into(),
                        optional: true,
                    },
                ],
                returns: "BotConfig".into(),
            },
            ActionSpec {
                name: "set_response_mode".into(),
                description: "设置机器人响应模式".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "bot_id".into(),
                        r#type: "string".into(),
                        description: "机器人 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "mode".into(),
                        r#type: "string".into(),
                        description: "响应模式".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "keyword".into(),
                        r#type: "string".into(),
                        description: "触发关键词".into(),
                        optional: true,
                    },
                ],
                returns: "BotConfig".into(),
            },
            ActionSpec {
                name: "add_whitelist".into(),
                description: "添加白名单用户".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "bot_id".into(),
                        r#type: "string".into(),
                        description: "机器人 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "user_id".into(),
                        r#type: "string".into(),
                        description: "用户 ID".into(),
                        optional: false,
                    },
                ],
                returns: "BotConfig".into(),
            },
            ActionSpec {
                name: "remove_whitelist".into(),
                description: "移除白名单用户".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "bot_id".into(),
                        r#type: "string".into(),
                        description: "机器人 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "user_id".into(),
                        r#type: "string".into(),
                        description: "用户 ID".into(),
                        optional: false,
                    },
                ],
                returns: "BotConfig".into(),
            },
            // 功能配置
            ActionSpec {
                name: "set_context_enhancement".into(),
                description: "设置上下文增强".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "enabled".into(),
                        r#type: "bool".into(),
                        description: "是否启用".into(),
                        optional: false,
                    },
                ],
                returns: "ChannelConfig".into(),
            },
            ActionSpec {
                name: "set_proactive_delivery".into(),
                description: "设置主动投递".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "enabled".into(),
                        r#type: "bool".into(),
                        description: "是否启用".into(),
                        optional: false,
                    },
                ],
                returns: "ChannelConfig".into(),
            },
            // 消息处理
            ActionSpec {
                name: "send_message".into(),
                description: "发送消息到渠道".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "bot_id".into(),
                        r#type: "string".into(),
                        description: "机器人 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "chat_id".into(),
                        r#type: "string".into(),
                        description: "聊天会话 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "content".into(),
                        r#type: "string".into(),
                        description: "消息内容".into(),
                        optional: false,
                    },
                ],
                returns: "String".into(),
            },
            ActionSpec {
                name: "send_streaming".into(),
                description: "流式发送消息到渠道".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "bot_id".into(),
                        r#type: "string".into(),
                        description: "机器人 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "chat_id".into(),
                        r#type: "string".into(),
                        description: "聊天会话 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "content".into(),
                        r#type: "string".into(),
                        description: "消息内容".into(),
                        optional: false,
                    },
                ],
                returns: "String".into(),
            },
            ActionSpec {
                name: "should_respond".into(),
                description: "检查是否应该响应消息".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "bot_id".into(),
                        r#type: "string".into(),
                        description: "机器人 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "chat_id".into(),
                        r#type: "string".into(),
                        description: "聊天会话 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "sender_id".into(),
                        r#type: "string".into(),
                        description: "发送者 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "text".into(),
                        r#type: "string".into(),
                        description: "消息文本".into(),
                        optional: false,
                    },
                ],
                returns: "bool".into(),
            },
            // 会话路由
            ActionSpec {
                name: "make_session_id".into(),
                description: "生成会话 ID（带渠道前缀）".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "bot_id".into(),
                        r#type: "string".into(),
                        description: "机器人 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "chat_id".into(),
                        r#type: "string".into(),
                        description: "聊天会话 ID".into(),
                        optional: false,
                    },
                ],
                returns: "String".into(),
            },
            ActionSpec {
                name: "parse_session_id".into(),
                description: "解析会话 ID".into(),
                params: vec![ParamSpec {
                    name: "session_id".into(),
                    r#type: "string".into(),
                    description: "会话 ID".into(),
                    optional: false,
                }],
                returns: "SessionChannelPrefix".into(),
            },
            // 超时恢复
            ActionSpec {
                name: "timeout_recovery_status".into(),
                description: "获取超时恢复状态".into(),
                params: vec![],
                returns: "Vec<TimeoutRecoveryState>".into(),
            },
            ActionSpec {
                name: "retry_timeout".into(),
                description: "重试超时消息".into(),
                params: vec![ParamSpec {
                    name: "message_id".into(),
                    r#type: "string".into(),
                    description: "消息 ID".into(),
                    optional: false,
                }],
                returns: "TimeoutRecoveryState".into(),
            },
            // DSH 市场
            ActionSpec {
                name: "dsh_market_status".into(),
                description: "获取 DSH 市场配置".into(),
                params: vec![],
                returns: "DshMarketConfig".into(),
            },
            ActionSpec {
                name: "dsh_market_toggle".into(),
                description: "启用/禁用 DSH 市场".into(),
                params: vec![ParamSpec {
                    name: "enabled".into(),
                    r#type: "bool".into(),
                    description: "是否启用".into(),
                    optional: false,
                }],
                returns: "DshMarketConfig".into(),
            },
            ActionSpec {
                name: "dsh_market_config".into(),
                description: "更新 DSH 市场配置".into(),
                params: vec![
                    ParamSpec {
                        name: "api_endpoint".into(),
                        r#type: "string".into(),
                        description: "API 端点".into(),
                        optional: true,
                    },
                    ParamSpec {
                        name: "auth_token".into(),
                        r#type: "string".into(),
                        description: "认证令牌".into(),
                        optional: true,
                    },
                    ParamSpec {
                        name: "sync_enabled".into(),
                        r#type: "bool".into(),
                        description: "是否启用同步".into(),
                        optional: true,
                    },
                ],
                returns: "DshMarketConfig".into(),
            },
            ActionSpec {
                name: "dsh_market_sync".into(),
                description: "从 DSH 市场同步插件".into(),
                params: vec![],
                returns: "HashMap<String, String>".into(),
            },
        ]
    }

    #[tracing::instrument(skip(self, args), fields(action = %action))]
    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            // 渠道管理
            "status" => {
                let channels = self.load_channels()?;
                let total_bots: usize = channels.iter().map(|c| c.bots.len()).sum();
                let connected_bots: usize = channels
                    .iter()
                    .filter(|c| c.enabled)
                    .map(|c| c.bots.len())
                    .sum();

                Ok(serde_json::json!({
                    "channels": channels,
                    "total_bots": total_bots,
                    "connected_bots": connected_bots,
                    "dsh_market_enabled": false,
                }))
            }

            "list_channels" => {
                let channels = self.load_channels()?;
                Ok(serde_json::json!(channels))
            }

            "get_channel" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;
                let channels = self.load_channels()?;
                channels
                    .into_iter()
                    .find(|c| c.channel == channel_type)
                    .map(|c| serde_json::json!(c))
                    .ok_or_else(|| DomainError {
                        code: "NOT_FOUND".into(),
                        message: format!("渠道不存在: {}", channel),
                        recoverable: true,
                    })
            }

            "toggle_channel" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let enabled = args
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 enabled 参数".into(),
                        recoverable: true,
                    })?;
                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;
                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        ch.enabled = enabled;
                        self.save_channels(&channels)?;
                        return Ok(serde_json::json!(ch));
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("渠道不存在: {}", channel),
                    recoverable: true,
                })
            }

            // 机器人管理
            "add_bot" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let name =
                    args.get("name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 name 参数".into(),
                            recoverable: true,
                        })?;
                let credential_type = args
                    .get("credential_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("token");
                let workspace = args
                    .get("workspace")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let model = args.get("model").and_then(|v| v.as_str()).map(String::from);

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let bot_id = format!(
                    "bot-{}-{}",
                    channel_type,
                    &uuid::Uuid::new_v4().to_string()[..8]
                );

                let bot = BotConfig {
                    id: bot_id,
                    channel: channel_type.clone(),
                    name: name.to_string(),
                    credential_type: credential_type.to_string(),
                    workspace,
                    model,
                    enabled: true,
                    created_at: chrono::Utc::now().timestamp() as u64,
                    response_mode: ResponseMode::default(),
                    whitelist: HashSet::new(),
                };

                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        ch.bots.push(bot.clone());
                        self.save_channels(&channels)?;
                        return Ok(serde_json::json!(bot));
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("渠道不存在: {}", channel),
                    recoverable: true,
                })
            }

            "remove_bot" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;
                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        let original_len = ch.bots.len();
                        ch.bots.retain(|b| b.id != bot_id);
                        if ch.bots.len() < original_len {
                            self.save_channels(&channels)?;
                            return Ok(serde_json::json!(true));
                        }
                        return Ok(serde_json::json!(false));
                    }
                }
                Ok(serde_json::json!(false))
            }

            "update_bot" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let name = args.get("name").and_then(|v| v.as_str()).map(String::from);
                let workspace = args
                    .get("workspace")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let model = args.get("model").and_then(|v| v.as_str()).map(String::from);

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        for bot in &mut ch.bots {
                            if bot.id == bot_id {
                                if let Some(n) = name {
                                    bot.name = n;
                                }
                                if let Some(w) = workspace {
                                    bot.workspace = Some(w);
                                }
                                if let Some(m) = model {
                                    bot.model = Some(m);
                                }
                                self.save_channels(&channels)?;
                                return Ok(serde_json::json!(bot));
                            }
                        }
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("机器人不存在: {} / {}", channel, bot_id),
                    recoverable: true,
                })
            }

            "set_response_mode" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let mode =
                    args.get("mode")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 mode 参数".into(),
                            recoverable: true,
                        })?;
                let keyword = args
                    .get("keyword")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let response_mode = match mode {
                    "group_invite" => ResponseMode::GroupInvite,
                    "group_keyword" => ResponseMode::GroupKeyword {
                        keyword: keyword.unwrap_or_default(),
                    },
                    "group_all" => ResponseMode::GroupAll,
                    "private" => ResponseMode::Private,
                    _ => {
                        return Err(DomainError {
                            code: "INVALID_MODE".into(),
                            message: format!("未知响应模式: {}", mode),
                            recoverable: true,
                        })
                    }
                };

                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        for bot in &mut ch.bots {
                            if bot.id == bot_id {
                                bot.response_mode = response_mode;
                                self.save_channels(&channels)?;
                                return Ok(serde_json::json!(bot));
                            }
                        }
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("机器人不存在: {} / {}", channel, bot_id),
                    recoverable: true,
                })
            }

            "add_whitelist" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let user_id = args
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 user_id 参数".into(),
                        recoverable: true,
                    })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        for bot in &mut ch.bots {
                            if bot.id == bot_id {
                                bot.whitelist.insert(user_id.to_string());
                                self.save_channels(&channels)?;
                                return Ok(serde_json::json!(bot));
                            }
                        }
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("机器人不存在: {} / {}", channel, bot_id),
                    recoverable: true,
                })
            }

            "remove_whitelist" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let user_id = args
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 user_id 参数".into(),
                        recoverable: true,
                    })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        for bot in &mut ch.bots {
                            if bot.id == bot_id {
                                bot.whitelist.remove(user_id);
                                self.save_channels(&channels)?;
                                return Ok(serde_json::json!(bot));
                            }
                        }
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("机器人不存在: {} / {}", channel, bot_id),
                    recoverable: true,
                })
            }

            // 功能配置
            "set_context_enhancement" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let enabled = args
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 enabled 参数".into(),
                        recoverable: true,
                    })?;
                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;
                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        ch.context_enhancement = enabled;
                        self.save_channels(&channels)?;
                        return Ok(serde_json::json!(ch));
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("渠道不存在: {}", channel),
                    recoverable: true,
                })
            }

            "set_proactive_delivery" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let enabled = args
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 enabled 参数".into(),
                        recoverable: true,
                    })?;
                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;
                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        ch.proactive_delivery = enabled;
                        self.save_channels(&channels)?;
                        return Ok(serde_json::json!(ch));
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("渠道不存在: {}", channel),
                    recoverable: true,
                })
            }

            // 消息处理
            "send_message" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let chat_id = args
                    .get("chat_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 chat_id 参数".into(),
                        recoverable: true,
                    })?;
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 content 参数".into(),
                        recoverable: true,
                    })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let message_id = self.send_to_channel(&channel_type, bot_id, chat_id, content)
                    .await?;

                Ok(serde_json::json!({
                    "message_id": message_id,
                    "channel": channel,
                    "bot_id": bot_id,
                    "chat_id": chat_id,
                }))
            }

            "send_streaming" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let chat_id = args
                    .get("chat_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 chat_id 参数".into(),
                        recoverable: true,
                    })?;
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 content 参数".into(),
                        recoverable: true,
                    })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let message_id = self.send_streaming(&channel_type, bot_id, chat_id, content)
                    .await?;

                Ok(serde_json::json!({
                    "message_id": message_id,
                    "channel": channel,
                    "bot_id": bot_id,
                    "chat_id": chat_id,
                }))
            }

            "should_respond" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let chat_id = args
                    .get("chat_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 chat_id 参数".into(),
                        recoverable: true,
                    })?;
                let sender_id =
                    args.get("sender_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 sender_id 参数".into(),
                            recoverable: true,
                        })?;
                let text =
                    args.get("text")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 text 参数".into(),
                            recoverable: true,
                        })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let channels = self.load_channels()?;
                for ch in &channels {
                    if ch.channel == channel_type {
                        for bot in &ch.bots {
                            if bot.id == bot_id {
                                let should = self.should_respond(bot, chat_id, sender_id, text);
                                return Ok(serde_json::json!(should));
                            }
                        }
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("机器人不存在: {} / {}", channel, bot_id),
                    recoverable: true,
                })
            }

            // 会话路由
            "make_session_id" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let chat_id = args
                    .get("chat_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 chat_id 参数".into(),
                        recoverable: true,
                    })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let session_id = self.make_session_id(&channel_type, bot_id, chat_id);
                Ok(serde_json::json!(session_id))
            }

            "parse_session_id" => {
                let session_id =
                    args.get("session_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 session_id 参数".into(),
                            recoverable: true,
                        })?;

                SessionChannelPrefix::from_session_id(session_id)
                    .map(|prefix| serde_json::json!(prefix))
                    .ok_or_else(|| DomainError {
                        code: "INVALID_SESSION_ID".into(),
                        message: format!("无法解析会话 ID: {}", session_id),
                        recoverable: true,
                    })
            }

            // 超时恢复
            "timeout_recovery_status" => {
                let state = self.timeout_recovery.lock().await;
                let states: Vec<&TimeoutRecoveryState> = state.values().collect();
                Ok(serde_json::json!(states))
            }

            "retry_timeout" => {
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let message_id =
                    args.get("message_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 message_id 参数".into(),
                            recoverable: true,
                        })?;

                self.handle_timeout_recovery(bot_id, message_id).await?;

                let state = self.timeout_recovery.lock().await;

                state
                    .get(message_id)
                    .map(|entry| Ok(serde_json::json!(entry)))
                    .unwrap_or_else(|| {
                        Err(DomainError {
                            code: "NOT_FOUND".into(),
                            message: format!("消息不存在: {}", message_id),
                            recoverable: true,
                        })
                    })
            }

            // DSH 市场
            "dsh_market_status" => {
                let config = self.load_dsh_market()?;
                Ok(serde_json::json!(config))
            }

            "dsh_market_toggle" => {
                let enabled = args
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 enabled 参数".into(),
                        recoverable: true,
                    })?;
                let mut config = self.load_dsh_market()?;
                config.enabled = enabled;
                self.save_dsh_market(&config)?;
                Ok(serde_json::json!(config))
            }

            "dsh_market_config" => {
                let mut config = self.load_dsh_market()?;
                if let Some(ep) = args.get("api_endpoint").and_then(|v| v.as_str()) {
                    config.api_endpoint = ep.to_string();
                }
                if let Some(token) = args.get("auth_token").and_then(|v| v.as_str()) {
                    config.auth_token = Some(token.to_string());
                }
                if let Some(sync) = args.get("sync_enabled").and_then(|v| v.as_bool()) {
                    config.sync_enabled = sync;
                }
                self.save_dsh_market(&config)?;
                Ok(serde_json::json!(config))
            }

            "dsh_market_sync" => {
                let config = self.load_dsh_market()?;
                if !config.enabled {
                    return Err(DomainError {
                        code: "DSH_MARKET_DISABLED".into(),
                        message: "DSH market is not enabled".into(),
                        recoverable: true,
                    });
                }

                // TODO: 实现实际的 DSH 市场 API 调用
                // 目前返回模拟数据
                let mut plugins = HashMap::new();
                plugins.insert("dsh-im-core".into(), "1.0.0".into());
                plugins.insert("dsh-im-wechat".into(), "1.0.0".into());
                plugins.insert("dsh-im-feishu".into(), "1.0.0".into());
                plugins.insert("dsh-im-telegram".into(), "1.0.0".into());

                Ok(serde_json::json!(plugins))
            }

            // ═══════════════════════════════════════════════
            // Single Inbox Pattern (from Multi-Channel Architecture)
            // ═══════════════════════════════════════════════
            "receive_message" => {
                let envelope: MessageEnvelope = serde_json::from_value(
                    args.get("envelope")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                )
                .map_err(|e| DomainError {
                    code: "INVALID_ENVELOPE".into(),
                    message: format!("无效的消息信封: {}", e),
                    recoverable: true,
                })?;

                // 幂等性检查
                if self.is_message_processed(&envelope.id).await {
                    return Ok(serde_json::json!({
                        "status": "already_processed",
                        "message_id": envelope.id,
                    }));
                }

                // 记录处理状态
                self.record_message_state(&envelope.id, ProcessingStatus::Processing);

                // 发射事件到前端
                if let Some(app) = get_app_handle() {
                    if let Err(e) = app.emit(
                        "im_message_received",
                        serde_json::json!({
                            "envelope": envelope,
                        }),
                    ) {
                        tracing::warn!("Failed to emit im_message_received: {e}");
                    }
                }

                // 标记完成
                self.record_message_state(&envelope.id, ProcessingStatus::Completed);

                Ok(serde_json::json!({
                    "status": "processed",
                    "message_id": envelope.id,
                }))
            }

            "check_idempotency" => {
                let message_id =
                    args.get("message_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 message_id 参数".into(),
                            recoverable: true,
                        })?;

                let processed = self.is_message_processed(message_id).await;
                Ok(serde_json::json!(processed))
            }

            // ═══════════════════════════════════════════════
            // Five Identity Layers (from Multi-Channel Architecture)
            // ═══════════════════════════════════════════════
            "resolve_identity" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let channel_user_id = args
                    .get("channel_user_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel_user_id 参数".into(),
                        recoverable: true,
                    })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let global_user_id = self.get_global_user_id(&channel_type, channel_user_id).await;

                let identity = IdentityLayers {
                    channel_user_id: channel_user_id.to_string(),
                    global_user_id,
                    tenant_id: "default".to_string(),
                    session_id: format!("{}:{}", channel, channel_user_id),
                    roles: vec!["user".to_string()],
                };

                Ok(serde_json::json!(identity))
            }

            // ═══════════════════════════════════════════════
            // Bot-to-Bot Communication (from Grok Bot)
            // ═══════════════════════════════════════════════
            "send_bot_message" => {
                let from_bot_id = args
                    .get("from_bot_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 from_bot_id 参数".into(),
                        recoverable: true,
                    })?;
                let to_bot_id =
                    args.get("to_bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 to_bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 content 参数".into(),
                        recoverable: true,
                    })?;
                let message_type = args
                    .get("message_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 message_type 参数".into(),
                        recoverable: true,
                    })?;
                let context = args.get("context").cloned();

                let msg_type = match message_type {
                    "task_delegation" => BotMessageType::TaskDelegation,
                    "context_share" => BotMessageType::ContextShare,
                    "status_update" => BotMessageType::StatusUpdate,
                    "result_return" => BotMessageType::ResultReturn,
                    "heartbeat" => BotMessageType::Heartbeat,
                    _ => {
                        return Err(DomainError {
                            code: "INVALID_MESSAGE_TYPE".into(),
                            message: format!("未知消息类型: {}", message_type),
                            recoverable: true,
                        })
                    }
                };

                let message = BotMessage {
                    from_bot_id: from_bot_id.to_string(),
                    to_bot_id: to_bot_id.to_string(),
                    content: content.to_string(),
                    message_type: msg_type,
                    context,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                };

                self.send_bot_message(message.clone()).await?;

                // 发射事件到前端
                if let Some(app) = get_app_handle() {
                    if let Err(e) = app.emit(
                        "im_bot_message_sent",
                        serde_json::json!({
                            "message": message,
                        }),
                    ) {
                        tracing::warn!("Failed to emit im_bot_message_sent: {e}");
                    }
                }

                Ok(serde_json::json!(message))
            }

            "receive_bot_messages" => {
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;

                let messages = self.receive_bot_messages(bot_id).await;
                Ok(serde_json::json!(messages))
            }

            // ═══════════════════════════════════════════════
            // Workflow Learning (from Grok Bot)
            // ═══════════════════════════════════════════════
            "save_workflow" => {
                let name =
                    args.get("name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 name 参数".into(),
                            recoverable: true,
                        })?;
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 description 参数".into(),
                        recoverable: true,
                    })?;
                let steps: Vec<WorkflowStep> = serde_json::from_value(
                    args.get("steps")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                )
                .map_err(|e| DomainError {
                    code: "INVALID_STEPS".into(),
                    message: format!("无效的步骤列表: {}", e),
                    recoverable: true,
                })?;
                let trigger: WorkflowTrigger = serde_json::from_value(
                    args.get("trigger")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                )
                .map_err(|e| DomainError {
                    code: "INVALID_TRIGGER".into(),
                    message: format!("无效的触发条件: {}", e),
                    recoverable: true,
                })?;

                let short_id = uuid::Uuid::new_v4().to_string();
                let template = WorkflowTemplate {
                    id: format!("wf-{}", &short_id[..8]),
                    name: name.to_string(),
                    description: description.to_string(),
                    steps,
                    trigger,
                    created_at: chrono::Utc::now().timestamp() as u64,
                    last_used: None,
                    use_count: 0,
                };

                self.save_workflow_template(template.clone()).await?;

                Ok(serde_json::json!(template))
            }

            "get_workflow" => {
                let workflow_id = args
                    .get("workflow_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 workflow_id 参数".into(),
                        recoverable: true,
                    })?;

                self.get_workflow_template(workflow_id)
                    .await
                    .map(|t| Ok(serde_json::json!(t)))
                    .unwrap_or_else(|| {
                        Err(DomainError {
                            code: "NOT_FOUND".into(),
                            message: format!("工作流不存在: {}", workflow_id),
                            recoverable: true,
                        })
                    })
            }

            "list_workflows" => {
                let templates = self.list_workflow_templates().await;
                Ok(serde_json::json!(templates))
            }

            "execute_workflow" => {
                let workflow_id = args
                    .get("workflow_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 workflow_id 参数".into(),
                        recoverable: true,
                    })?;

                let mut template =
                    self.get_workflow_template(workflow_id)
                        .await
                        .ok_or_else(|| DomainError {
                            code: "NOT_FOUND".into(),
                            message: format!("工作流不存在: {}", workflow_id),
                            recoverable: true,
                        })?;

                // 更新使用统计
                template.last_used = Some(chrono::Utc::now().timestamp() as u64);
                template.use_count += 1;
                self.save_workflow_template(template.clone()).await?;

                // 发射事件到前端
                if let Some(app) = get_app_handle() {
                    if let Err(e) = app.emit(
                        "im_workflow_executed",
                        serde_json::json!({
                            "workflow_id": workflow_id,
                            "workflow_name": template.name,
                        }),
                    ) {
                        tracing::warn!("Failed to emit im_workflow_executed: {e}");
                    }
                }

                Ok(serde_json::json!({
                    "status": "executed",
                    "workflow_id": workflow_id,
                    "workflow_name": template.name,
                    "steps_count": template.steps.len(),
                }))
            }

            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("未知 action: {}", action),
                recoverable: true,
            }),
        }
    }

    async fn init(&mut self) -> Result<(), DomainError> {
        // 初始化配置文件
        if !self.config_path.exists() {
            let channels = self.default_channels();
            self.save_channels(&channels)?;
        }
        Ok(())
    }
}
