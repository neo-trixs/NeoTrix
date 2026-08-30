use super::*;
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionState {
    pub sessions: HashMap<String, Session>,
    pub active_id: Option<String>,
    pub order: Vec<String>,
}

/// 会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub messages: VecDeque<crate::cli::tui::app::state::chat::ChatMessage>,
    pub created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl Session {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            messages: VecDeque::new(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn add_message(&mut self, msg: crate::cli::tui::app::state::chat::ChatMessage) {
        self.messages.push_back(msg);
        self.updated_at = Utc::now();
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}
