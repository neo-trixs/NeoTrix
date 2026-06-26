use super::error::AgentError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct RemoteSessionId(pub String);

impl RemoteSessionId {
    pub fn new() -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let random_part: u64 = ts as u64;
        Self(format!("rs-{:x}", random_part))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteSessionState {
    Connected,
    Disconnected,
    Expired,
}

#[derive(Debug, Clone)]
pub struct RemoteSession {
    pub id: RemoteSessionId,
    pub label: String,
    pub created_at: u64,
    pub last_active: u64,
    pub state: RemoteSessionState,
    pub message_queue: Vec<RemoteMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteMessage {
    pub id: String,
    pub seq: u64,
    pub direction: MessageDirection,
    pub content: MessageContent,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageDirection {
    Outgoing,
    Incoming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    ToolCall {
        tool: String,
        args: serde_json::Value,
    },
    ToolResult {
        tool: String,
        output: String,
        success: bool,
    },
    StatusUpdate {
        state: String,
        detail: String,
    },
    System {
        action: String,
        payload: serde_json::Value,
    },
}

impl MessageContent {
    pub fn content_type(&self) -> &'static str {
        match self {
            MessageContent::Text(_) => "text",
            MessageContent::ToolCall { .. } => "tool_call",
            MessageContent::ToolResult { .. } => "tool_result",
            MessageContent::StatusUpdate { .. } => "status",
            MessageContent::System { .. } => "system",
        }
    }
}

pub struct RemoteControlServer {
    sessions: Arc<Mutex<HashMap<String, RemoteSession>>>,
    seq_counter: Arc<Mutex<u64>>,
    relay_url: String,
}

impl RemoteControlServer {
    pub fn new(relay_url: String) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            seq_counter: Arc::new(Mutex::new(0)),
            relay_url,
        }
    }

    pub fn sessions(&self) -> Arc<Mutex<HashMap<String, RemoteSession>>> {
        self.sessions.clone()
    }

    pub fn create_session(&self, label: String) -> RemoteSessionId {
        let id = RemoteSessionId::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let session = RemoteSession {
            id: id.clone(),
            label,
            created_at: now,
            last_active: now,
            state: RemoteSessionState::Connected,
            message_queue: Vec::new(),
        };
        let mut sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        sessions.insert(id.0.clone(), session);
        id
    }

    pub fn push_message(
        &self,
        session_id: &str,
        content: MessageContent,
        direction: MessageDirection,
    ) -> Result<u64, AgentError> {
        let mut sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        let session = sessions.get_mut(session_id).ok_or("Session not found")?;
        let mut seq = self.seq_counter.lock().unwrap_or_else(|e| e.into_inner());
        *seq += 1;
        let seq_val = *seq;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        session.message_queue.push(RemoteMessage {
            id: format!("msg-{}", seq_val),
            seq: seq_val,
            direction,
            content,
            timestamp: now,
        });
        const MAX_MESSAGE_QUEUE: usize = 500;
        if session.message_queue.len() > MAX_MESSAGE_QUEUE {
            session.message_queue.drain(0..MAX_MESSAGE_QUEUE / 5);
        }
        session.last_active = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Ok(seq_val)
    }

    pub fn poll_messages(
        &self,
        session_id: &str,
        since_seq: u64,
    ) -> Result<Vec<RemoteMessage>, AgentError> {
        let sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        let session = sessions.get(session_id).ok_or("Session not found")?;
        Ok(session
            .message_queue
            .iter()
            .filter(|m| m.seq > since_seq)
            .cloned()
            .collect())
    }

    pub fn get_relay_url(&self, session_id: &str) -> String {
        format!(
            "{}/session/{}",
            self.relay_url.trim_end_matches('/'),
            session_id
        )
    }

    pub fn get_qr_url(&self, session_id: &str) -> String {
        format!("neotrix://remote/{}", session_id)
    }

    pub fn remove_session(&self, session_id: &str) {
        let mut sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        sessions.remove(session_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let server = RemoteControlServer::new("https://relay.test".into());
        let id = server.create_session("test-session".into());
        assert!(id.0.starts_with("rs-"));
        assert!(id.0.len() > 3);
    }

    #[test]
    fn test_push_and_poll_messages() {
        let server = RemoteControlServer::new("https://relay.test".into());
        let id = server.create_session("test".into());
        let sid = id.0.clone();

        let seq = server
            .push_message(
                &sid,
                MessageContent::Text("hello".into()),
                MessageDirection::Outgoing,
            )
            .unwrap();
        assert_eq!(seq, 1);

        let seq2 = server
            .push_message(
                &sid,
                MessageContent::Text("world".into()),
                MessageDirection::Incoming,
            )
            .unwrap();
        assert_eq!(seq2, 2);

        let msgs = server.poll_messages(&sid, 0).unwrap();
        assert_eq!(msgs.len(), 2);

        let msgs_since = server.poll_messages(&sid, 1).unwrap();
        assert_eq!(msgs_since.len(), 1);
        assert_eq!(msgs_since[0].seq, 2);
    }

    #[test]
    fn test_poll_nonexistent_session() {
        let server = RemoteControlServer::new("https://relay.test".into());
        assert!(server.poll_messages("nonexistent", 0).is_err());
    }

    #[test]
    fn test_get_urls() {
        let server = RemoteControlServer::new("https://relay.example.com".into());
        let id = server.create_session("test".into());
        let sid = id.0.clone();

        let relay = server.get_relay_url(&sid);
        assert!(relay.starts_with("https://relay.example.com/session/"));

        let qr = server.get_qr_url(&sid);
        assert_eq!(qr, format!("neotrix://remote/{}", sid));
    }

    #[test]
    fn test_remove_session() {
        let server = RemoteControlServer::new("https://relay.test".into());
        let id = server.create_session("test".into());
        let sid = id.0.clone();
        server.remove_session(&sid);
        let sessions = server.sessions.lock().unwrap_or_else(|e| e.into_inner());
        assert!(!sessions.contains_key(&sid));
    }

    #[test]
    fn test_multiple_sessions() {
        let server = RemoteControlServer::new("https://relay.test".into());
        let a = server.create_session("A".into());
        let b = server.create_session("B".into());
        assert_ne!(a.0, b.0);
    }

    #[test]
    fn test_message_content_type() {
        assert_eq!(MessageContent::Text("hello".into()).content_type(), "text");
        assert_eq!(
            MessageContent::ToolCall {
                tool: "search".into(),
                args: serde_json::json!({})
            }
            .content_type(),
            "tool_call"
        );
        assert_eq!(
            MessageContent::StatusUpdate {
                state: "ok".into(),
                detail: "".into()
            }
            .content_type(),
            "status"
        );
    }
}
