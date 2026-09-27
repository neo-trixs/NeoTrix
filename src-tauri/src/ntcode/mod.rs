//! ntcode — 桌面 app 默认对话模型
//!
//! 封装 neotrix-core 的晶体闭环，通过 Tauri IPC 暴露给前端。
//! 前端围绕对话界面进行可视化：消息列表 + 流式渲染 + 模型切换 + 对话管理。

pub mod commands;
pub mod streaming;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

// ============================================================================
// 核心类型
// ============================================================================

/// 对话中的单条消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub timestamp: u64,
    /// 生成此消息的模型（仅 Assistant 消息有）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// 消息角色。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

/// 会话状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Idle,
    Streaming,
    WaitingHuman,
}

/// 模型信息（供前端展示）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub source: String,
    pub tier: String,
    pub is_free: bool,
}

/// 对话摘要（供前端列表展示）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub id: String,
    pub goal: String,
    pub message_count: usize,
    pub model: Option<String>,
    pub updated_at: u64,
}

/// 桌面端会话状态（Tauri managed）。
pub struct NtcodeSession {
    /// 当前对话 ID。
    pub conversation_id: Option<String>,
    /// 当前目标/话题。
    pub goal: String,
    /// 当前定点模型。
    pub model: Option<String>,
    /// 对话历史。
    pub transcript: Vec<ChatMessage>,
    /// 当前状态。
    pub status: SessionStatus,
    /// 流式任务句柄（用于取消）。
    pub cancel: Option<Arc<tokio::sync::Notify>>,
}

impl NtcodeSession {
    pub fn new(goal: &str) -> Self {
        Self {
            conversation_id: None,
            goal: goal.to_string(),
            model: None,
            transcript: Vec::new(),
            status: SessionStatus::Idle,
            cancel: None,
        }
    }

    /// 添加用户消息。
    pub fn push_user(&mut self, content: &str) {
        self.transcript.push(ChatMessage {
            role: Role::User,
            content: content.to_string(),
            timestamp: unix_now(),
            model: None,
        });
    }

    /// 添加助手消息。
    pub fn push_assistant(&mut self, content: &str, model: Option<String>) {
        self.transcript.push(ChatMessage {
            role: Role::Assistant,
            content: content.to_string(),
            timestamp: unix_now(),
            model,
        });
    }
}

/// Tauri managed state：全局共享的 ntcode 会话。
pub struct NtcodeState {
    pub inner: Arc<RwLock<NtcodeSession>>,
}

impl NtcodeState {
    pub fn new(goal: &str) -> Self {
        Self {
            inner: Arc::new(RwLock::new(NtcodeSession::new(goal))),
        }
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
