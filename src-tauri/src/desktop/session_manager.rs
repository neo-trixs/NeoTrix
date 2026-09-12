//! Session Manager — 统一会话管理
//!
//! 提供会话生命周期管理：创建、消息追加、搜索、导出。
//! 与 domain/plugins/session.rs 互补（后者提供 Domain Plugin 接口）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

// ========== Core Types ==========

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// 消息内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Markdown,
    Code { language: String },
    Image { data: String, mime_type: String },
    ToolCall { name: String, args: serde_json::Value },
    ToolResult { name: String, result: serde_json::Value },
}

/// 会话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: ContentType,
    pub timestamp: DateTime<Utc>,
    pub token_count: Option<u32>,
    pub model: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// 会话元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub project: String,
    pub tags: Vec<String>,
    pub model: String,
    pub total_tokens: u64,
    pub message_count: usize,
    pub last_activity: DateTime<Utc>,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            project: String::new(),
            tags: vec![],
            model: String::new(),
            total_tokens: 0,
            message_count: 0,
            last_activity: Utc::now(),
        }
    }
}

/// 会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub model: String,
    pub messages: Vec<Message>,
    pub metadata: SessionMetadata,
}

/// 导出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Markdown,
    Html,
    Csv,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Markdown => write!(f, "markdown"),
            Self::Html => write!(f, "html"),
            Self::Csv => write!(f, "csv"),
        }
    }
}

impl std::str::FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "markdown" | "md" => Ok(Self::Markdown),
            "html" => Ok(Self::Html),
            "csv" => Ok(Self::Csv),
            _ => Err(format!("未知导出格式: {s}")),
        }
    }
}

// ========== Session Manager ==========

/// 会话管理器
///
/// 管理会话生命周期，提供：
/// - 会话 CRUD
/// - 消息追加与检索
/// - 搜索与过滤
/// - 导出（JSON/Markdown/HTML）
pub struct SessionManager {
    sessions: RwLock<HashMap<String, Session>>,
    active_session: RwLock<Option<String>>,
}

impl SessionManager {
    /// 创建空管理器
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            active_session: RwLock::new(None),
        }
    }

    /// 创建新会话
    pub async fn create_session(&self, title: &str, model: &str) -> Session {
        let id = format!("s-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let now = Utc::now();

        let session = Session {
            id: id.clone(),
            title: title.to_string(),
            created_at: now,
            updated_at: now,
            model: model.to_string(),
            messages: vec![],
            metadata: SessionMetadata {
                model: model.to_string(),
                last_activity: now,
                ..Default::default()
            },
        };

        self.sessions
            .write()
            .await
            .insert(id, session.clone());

        session
    }

    /// 获取会话
    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// 列出所有会话
    pub async fn list_sessions(&self) -> Vec<Session> {
        self.sessions
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// 获取当前活跃会话
    pub async fn active_session(&self) -> Option<Session> {
        let active_id = self.active_session.read().await.clone();
        active_id
            .as_deref()
            .and_then(|id| {
                // 需要在 runtime 中获取，这里简化处理
                None
            })
    }

    /// 设置活跃会话
    pub async fn set_active_session(&self, session_id: &str) -> Result<(), String> {
        let exists = self.sessions.read().await.contains_key(session_id);
        if !exists {
            return Err(format!("会话不存在: {session_id}"));
        }
        *self.active_session.write().await = Some(session_id.to_string());
        Ok(())
    }

    /// 添加消息
    pub async fn add_message(
        &self,
        session_id: &str,
        message: Message,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("会话不存在: {session_id}"))?;

        session.messages.push(message);
        session.updated_at = Utc::now();
        session.metadata.message_count = session.messages.len();
        session.metadata.last_activity = Utc::now();

        // 更新 token 计数
        if let Some(tokens) = &session.messages.last().and_then(|m| m.token_count) {
            session.metadata.total_tokens += *tokens as u64;
        }

        Ok(())
    }

    /// 获取会话历史
    pub async fn get_history(&self, session_id: &str) -> Option<Vec<Message>> {
        self.sessions
            .read()
            .await
            .get(session_id)
            .map(|s| s.messages.clone())
    }

    /// 搜索会话
    pub async fn search_sessions(&self, query: &str) -> Vec<Session> {
        let q = query.to_lowercase();
        self.sessions
            .read()
            .await
            .values()
            .filter(|s| {
                s.title.to_lowercase().contains(&q)
                    || s.messages.iter().any(|m| match &m.content {
                        ContentType::Text(t) => t.to_lowercase().contains(&q),
                        ContentType::Markdown(t) => t.to_lowercase().contains(&q),
                        _ => false,
                    })
            })
            .cloned()
            .collect()
    }

    /// 重命名会话
    pub async fn rename_session(
        &self,
        session_id: &str,
        new_title: &str,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("会话不存在: {session_id}"))?;
        session.title = new_title.to_string();
        session.updated_at = Utc::now();
        Ok(())
    }

    /// 删除会话
    pub async fn delete_session(&self, session_id: &str) -> Result<(), String> {
        let removed = self.sessions.write().await.remove(session_id);
        if removed.is_none() {
            return Err(format!("会话不存在: {session_id}"));
        }

        // 如果删除的是活跃会话，清空活跃状态
        let mut active = self.active_session.write().await;
        if active.as_deref() == Some(session_id) {
            *active = None;
        }

        Ok(())
    }

    /// 清空会话消息
    pub async fn clear_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("会话不存在: {session_id}"))?;
        session.messages.clear();
        session.metadata.message_count = 0;
        session.metadata.total_tokens = 0;
        session.updated_at = Utc::now();
        Ok(())
    }

    /// 导出会话
    pub async fn export_session(
        &self,
        session_id: &str,
        format: ExportFormat,
    ) -> Result<Vec<u8>, String> {
        let session = self
            .sessions
            .read()
            .await
            .get(session_id)
            .cloned()
            .ok_or_else(|| format!("会话不存在: {session_id}"))?;

        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&session)
                    .map_err(|e| format!("JSON 序列化失败: {e}"))?;
                Ok(json.into_bytes())
            }
            ExportFormat::Markdown => {
                let md = self.session_to_markdown(&session);
                Ok(md.into_bytes())
            }
            ExportFormat::Html => {
                let html = self.session_to_html(&session);
                Ok(html.into_bytes())
            }
            ExportFormat::Csv => {
                let csv = self.session_to_csv(&session);
                Ok(csv.into_bytes())
            }
        }
    }

    /// 会话转 Markdown
    fn session_to_markdown(&self, session: &Session) -> String {
        let mut md = format!(
            "# {}\n\n**模型**: {} | **创建时间**: {} | **消息数**: {}\n\n---\n\n",
            session.title,
            session.model,
            session.created_at.format("%Y-%m-%d %H:%M"),
            session.messages.len()
        );

        for msg in &session.messages {
            let role = match msg.role {
                MessageRole::User => "**用户**",
                MessageRole::Assistant => "**助手**",
                MessageRole::System => "**系统**",
            };
            md.push_str(&format!("### {}\n\n", role));

            match &msg.content {
                ContentType::Text(t) | ContentType::Markdown(t) => {
                    md.push_str(t);
                    md.push_str("\n\n");
                }
                ContentType::Code { language, .. } => {
                    md.push_str(&format!("```{}\n<code>\n```\n\n", language));
                }
                ContentType::ToolCall { name, .. } => {
                    md.push_str(&format!("*调用工具: {}*\n\n", name));
                }
                ContentType::ToolResult { name, .. } => {
                    md.push_str(&format!("*工具结果: {}*\n\n", name));
                }
                _ => {}
            }
        }

        md
    }

    /// 会话转 HTML
    fn session_to_html(&self, session: &Session) -> String {
        let mut html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        .message {{ margin: 10px 0; padding: 10px; border-radius: 8px; }}
        .user {{ background: #e3f2fd; }}
        .assistant {{ background: #f5f5f5; }}
        .system {{ background: #fff3e0; }}
        pre {{ background: #263238; color: #eeffff; padding: 12px; border-radius: 4px; overflow-x: auto; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p><strong>模型</strong>: {} | <strong>创建时间</strong>: {} | <strong>消息数**: {}</p>
    <hr>"#,
            session.title,
            session.title,
            session.model,
            session.created_at.format("%Y-%m-%d %H:%M"),
            session.messages.len()
        );

        for msg in &session.messages {
            let role_class = match msg.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::System => "system",
            };
            let role_label = match msg.role {
                MessageRole::User => "用户",
                MessageRole::Assistant => "助手",
                MessageRole::System => "系统",
            };

            html.push_str(&format!(
                r#"<div class="message {}"><strong>{}:</strong><br>"#,
                role_class, role_label
            ));

            match &msg.content {
                ContentType::Text(t) | ContentType::Markdown(t) => {
                    html.push_str(&html_escape(t));
                }
                ContentType::Code { language, .. } => {
                    html.push_str(&format!(
                        r#"<pre><code class="{}">{}</code></pre>"#,
                        language,
                        "<code>"
                    ));
                }
                ContentType::ToolCall { name, .. } => {
                    html.push_str(&format!("<em>调用工具: {}</em>", name));
                }
                ContentType::ToolResult { name, .. } => {
                    html.push_str(&format!("<em>工具结果: {}</em>", name));
                }
                _ => {}
            }

            html.push_str("</div>\n");
        }

        html.push_str("</body></html>");
        html
    }

    /// 会话转 CSV
    fn session_to_csv(&self, session: &Session) -> String {
        let mut csv = "role,content,timestamp,model\n".to_string();

        for msg in &session.messages {
            let role = match msg.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::System => "system",
            };
            let content = match &msg.content {
                ContentType::Text(t) | ContentType::Markdown(t) => t.clone(),
                ContentType::Code { language, .. } => format!("[code: {}]", language),
                ContentType::ToolCall { name, .. } => format!("[tool_call: {}]", name),
                ContentType::ToolResult { name, .. } => format!("[tool_result: {}]", name),
                _ => "[binary]".to_string(),
            };
            let model = msg.model.as_deref().unwrap_or("");
            let timestamp = msg.timestamp.format("%Y-%m-%dT%H:%M:%SZ");

            // CSV 转义
            let escaped = content.replace('"', "\"\"");
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\"\n",
                role, escaped, timestamp, model
            ));
        }

        csv
    }

    /// 获取会话统计
    pub async fn stats(&self) -> SessionStats {
        let sessions = self.sessions.read().await;
        let total_sessions = sessions.len();
        let total_messages: usize = sessions.values().map(|s| s.messages.len()).sum();
        let total_tokens: u64 = sessions.values().map(|s| s.metadata.total_tokens).sum();

        SessionStats {
            total_sessions,
            total_messages,
            total_tokens,
        }
    }
}

/// 会话统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub total_messages: usize,
    pub total_tokens: u64,
}

// ========== Helper Functions ==========

/// HTML 转义
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 创建用户消息
pub fn user_message(content: &str) -> Message {
    Message {
        id: format!("msg-{}", &uuid::Uuid::new_v4().to_string()[..8]),
        role: MessageRole::User,
        content: ContentType::Text(content.to_string()),
        timestamp: Utc::now(),
        token_count: None,
        model: None,
        metadata: None,
    }
}

/// 创建助手消息
pub fn assistant_message(content: &str, model: &str) -> Message {
    Message {
        id: format!("msg-{}", &uuid::Uuid::new_v4().to_string()[..8]),
        role: MessageRole::Assistant,
        content: ContentType::Text(content.to_string()),
        timestamp: Utc::now(),
        token_count: None,
        model: Some(model.to_string()),
        metadata: None,
    }
}

/// 创建系统消息
pub fn system_message(content: &str) -> Message {
    Message {
        id: format!("msg-{}", &uuid::Uuid::new_v4().to_string()[..8]),
        role: MessageRole::System,
        content: ContentType::Text(content.to_string()),
        timestamp: Utc::now(),
        token_count: None,
        model: None,
        metadata: None,
    }
}

// ========== Default ==========

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
