use super::*;
use std::collections::{HashSet, VecDeque};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 聊天状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatState {
    pub messages: VecDeque<ChatMessage>,
    pub streaming: bool,
    pub streaming_role: String,
    pub streaming_text: String,
    pub streaming_model: Option<String>,
    pub streaming_tool_calls: Vec<ToolCall>,
    pub thinking_expanded: HashSet<String>,
    pub tool_calls_expanded: HashSet<String>,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub model: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub thinking_blocks: Vec<ThinkingBlock>,
    pub image_name: Option<String>,
}

impl ChatMessage {
    pub fn new(role: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            timestamp: Utc::now(),
            model: None,
            tool_calls: Vec::new(),
            thinking_blocks: Vec::new(),
            image_name: None,
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: String,
    pub result: Option<String>,
    pub success: bool,
    pub duration_ms: u64,
    pub started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

impl ToolCall {
    pub fn new(name: String, args: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            args,
            result: None,
            success: false,
            duration_ms: 0,
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn complete(mut self, result: String, success: bool) -> Self {
        self.result = Some(result);
        self.success = success;
        self.completed_at = Some(Utc::now());
        self
    }
}

/// 思考块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlock {
    pub content: String,
    pub signature: Option<String>,
}

/// 工具结果
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn ok(output: String) -> Self {
        Self {
            success: true,
            output,
            error: None,
        }
    }

    pub fn err(error: String) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error),
        }
    }
}
