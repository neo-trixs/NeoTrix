use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub tokens: Option<u32>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Code,
    ToolCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub total_messages: u32,
    pub total_tokens: u32,
    pub last_model: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Markdown,
    Html,
    Csv,
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
    active_session: Option<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session: None,
        }
    }

    /// 创建新会话
    pub fn create_session(&mut self, title: &str, model: &str) -> Session {
        let id = format!("session_{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        let session = Session {
            id: id.clone(),
            title: title.to_string(),
            created_at: now,
            updated_at: now,
            model: model.to_string(),
            messages: Vec::new(),
            metadata: SessionMetadata {
                total_messages: 0,
                total_tokens: 0,
                last_model: model.to_string(),
                tags: Vec::new(),
            },
        };

        self.sessions.insert(id.clone(), session.clone());
        self.active_session = Some(id);

        session
    }

    /// 获取会话
    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }

    /// 获取可变会话
    fn get_session_mut(&mut self, session_id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }

    /// 添加消息
    pub fn add_message(&mut self, session_id: &str, message: Message) -> Result<(), String> {
        if let Some(session) = self.get_session_mut(session_id) {
            session.messages.push(message.clone());
            session.updated_at = Utc::now();
            session.metadata.total_messages += 1;
            session.metadata.total_tokens += message.tokens.unwrap_or(0);
            Ok(())
        } else {
            Err("Session not found".into())
        }
    }

    /// 获取会话历史
    pub fn get_history(&self, session_id: &str) -> Option<&Vec<Message>> {
        self.sessions.get(session_id).map(|s| &s.messages)
    }

    /// 列出所有会话
    pub fn list_sessions(&self) -> Vec<&Session> {
        self.sessions.values().collect()
    }

    /// 搜索会话
    pub fn search_sessions(&self, query: &str) -> Vec<&Session> {
        self.sessions
            .values()
            .filter(|s| {
                s.title.to_lowercase().contains(&query.to_lowercase())
                    || s.messages
                        .iter()
                        .any(|m| m.content.to_lowercase().contains(&query.to_lowercase()))
            })
            .collect()
    }

    /// 重命名会话
    pub fn rename_session(&mut self, session_id: &str, new_title: &str) -> Result<(), String> {
        if let Some(session) = self.get_session_mut(session_id) {
            session.title = new_title.to_string();
            session.updated_at = Utc::now();
            Ok(())
        } else {
            Err("Session not found".into())
        }
    }

    /// 删除会话
    pub fn delete_session(&mut self, session_id: &str) -> Result<(), String> {
        if self.sessions.remove(session_id).is_some() {
            if self.active_session.as_deref() == Some(session_id) {
                self.active_session = None;
            }
            Ok(())
        } else {
            Err("Session not found".into())
        }
    }

    /// 设置活跃会话
    pub fn set_active_session(&mut self, session_id: &str) -> Result<(), String> {
        if self.sessions.contains_key(session_id) {
            self.active_session = Some(session_id.to_string());
            Ok(())
        } else {
            Err("Session not found".into())
        }
    }

    /// 获取活跃会话
    pub fn get_active_session(&self) -> Option<&Session> {
        self.active_session
            .as_ref()
            .and_then(|id| self.sessions.get(id))
    }

    /// 导出会话
    pub async fn export_session(
        &self,
        session_id: &str,
        format: ExportFormat,
    ) -> Result<Vec<u8>, String> {
        let session = self.sessions.get(session_id).ok_or("Session not found")?;

        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(session)
                    .map_err(|e| format!("Failed to serialize: {}", e))?;
                Ok(json.into_bytes())
            }
            ExportFormat::Markdown => {
                let mut md = format!("# {}\n\n", session.title);
                md.push_str(&format!("Model: {}\n", session.model));
                md.push_str(&format!("Created: {}\n\n", session.created_at));

                for msg in &session.messages {
                    let role = match msg.role {
                        MessageRole::User => "User",
                        MessageRole::Assistant => "Assistant",
                        MessageRole::System => "System",
                    };
                    md.push_str(&format!("## {}\n\n{}\n\n", role, msg.content));
                }

                Ok(md.into_bytes())
            }
            ExportFormat::Html => {
                let mut html = format!("<h1>{}</h1>\n", session.title);
                html.push_str(&format!("<p>Model: {}</p>\n", session.model));

                for msg in &session.messages {
                    let role = match msg.role {
                        MessageRole::User => "User",
                        MessageRole::Assistant => "Assistant",
                        MessageRole::System => "System",
                    };
                    html.push_str(&format!("<h2>{}</h2>\n<p>{}</p>\n", role, msg.content));
                }

                Ok(html.into_bytes())
            }
            ExportFormat::Csv => {
                let mut csv = "Role,Content,Timestamp,Tokens\n".to_string();
                for msg in &session.messages {
                    let role = match msg.role {
                        MessageRole::User => "User",
                        MessageRole::Assistant => "Assistant",
                        MessageRole::System => "System",
                    };
                    csv.push_str(&format!(
                        "\"{}\",\"{}\",\"{}\",{}\n",
                        role,
                        msg.content.replace('"', "\"\""),
                        msg.timestamp,
                        msg.tokens.unwrap_or(0)
                    ));
                }

                Ok(csv.into_bytes())
            }
        }
    }

    /// 获取会话统计
    pub fn get_stats(&self) -> SessionStats {
        let total_sessions = self.sessions.len() as u32;
        let total_messages: u32 = self
            .sessions
            .values()
            .map(|s| s.metadata.total_messages)
            .sum();
        let total_tokens: u32 = self
            .sessions
            .values()
            .map(|s| s.metadata.total_tokens)
            .sum();

        SessionStats {
            total_sessions,
            total_messages,
            total_tokens,
            active_session: self.active_session.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: u32,
    pub total_messages: u32,
    pub total_tokens: u32,
    pub active_session: Option<String>,
}
