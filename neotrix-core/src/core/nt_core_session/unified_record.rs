use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Single canonical record for ALL session persistence.
/// Every subsystem writes this format; readers are backward-compat shims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSessionRecord {
    pub version: u8,
    pub timestamp: u64,
    pub session_id: String,
    pub record_type: RecordType,
    pub payload: RecordPayload,
    pub source: String,
}

impl UnifiedSessionRecord {
    pub fn new(
        session_id: impl Into<String>,
        record_type: RecordType,
        payload: RecordPayload,
        source: impl Into<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            version: 1,
            timestamp: now,
            session_id: session_id.into(),
            record_type,
            payload,
            source: source.into(),
        }
    }

    pub fn is_chat(&self) -> bool {
        matches!(self.record_type, RecordType::ChatMessage)
    }

    pub fn is_system(&self) -> bool {
        matches!(self.record_type, RecordType::SystemEvent)
    }

    pub fn extract_text(&self) -> Option<&str> {
        match &self.payload {
            RecordPayload::Text { content, .. } => Some(content.as_str()),
            _ => None,
        }
    }

    pub fn text_or_empty(&self) -> &str {
        self.extract_text().unwrap_or("")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecordType {
    ChatMessage,
    ConsciousnessState,
    AgentAction,
    TranscriptEntry,
    KnowledgeEntry,
    ThinkingTrace,
    GoalUpdate,
    ToolCall,
    SystemEvent,
    Custom(String),
}

impl RecordType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ChatMessage => "chat_message",
            Self::ConsciousnessState => "consciousness_state",
            Self::AgentAction => "agent_action",
            Self::TranscriptEntry => "transcript_entry",
            Self::KnowledgeEntry => "knowledge_entry",
            Self::ThinkingTrace => "thinking_trace",
            Self::GoalUpdate => "goal_update",
            Self::ToolCall => "tool_call",
            Self::SystemEvent => "system_event",
            Self::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordPayload {
    Text {
        role: String,
        content: String,
    },
    Stats {
        c_score: f64,
        coherence: f64,
        cycle: u64,
    },
    JSON {
        data: HashMap<String, String>,
    },
    Binary {
        content_type: String,
        len: usize,
    },
}

impl RecordPayload {
    pub fn text(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Text { role: role.into(), content: content.into() }
    }

    pub fn stats(c_score: f64, coherence: f64, cycle: u64) -> Self {
        Self::Stats { c_score, coherence, cycle }
    }

    pub fn json(data: HashMap<String, String>) -> Self {
        Self::JSON { data }
    }

    pub fn binary(content_type: impl Into<String>, len: usize) -> Self {
        Self::Binary { content_type: content_type.into(), len }
    }
}
