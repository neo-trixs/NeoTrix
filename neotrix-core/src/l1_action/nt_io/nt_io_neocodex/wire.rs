// ── Wire Format (from Kimi Code: JSONL event stream) ──

use super::provider::NeoCodexMode;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WireEvent {
    UserMessage {
        content: String,
        timestamp: i64,
        #[serde(default)]
        attachments: Option<Vec<NeoCodexAttachment>>,
    },
    AgentMessage {
        content: String,
        timestamp: i64,
    },
    ToolCall {
        name: String,
        args: String,
        result: String,
        duration_ms: u64,
        success: bool,
    },
    SystemEvent {
        kind: String,
        detail: String,
        timestamp: i64,
    },
    GoalUpdate {
        id: String,
        state: String,
        description: String,
    },
    ModeChange {
        from: NeoCodexMode,
        to: NeoCodexMode,
    },
    SessionMeta {
        name: String,
        timestamp: i64,
        #[serde(default)]
        tags: Vec<String>,
    },
    // P1-2: side chat now carries a role so the UI can render a real answer
    // bubble; `role` defaults to "user" so pre-fix wire lines stay compatible.
    SideChatMessage {
        content: String,
        timestamp: i64,
        #[serde(default)]
        role: String,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NeoCodexAttachment {
    pub name: String,
    pub size: u64,
    pub mime_type: String,
    #[serde(default)]
    pub data: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WireSession {
    pub session_id: String,
    pub events: Vec<WireEvent>,
    pub path: std::path::PathBuf,
}

impl WireSession {
    pub fn new(session_id: &str) -> Self {
        let base = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from(".neocodex"))
            .join("neocodex")
            .join("sessions");
        Self {
            session_id: session_id.to_string(),
            events: Vec::new(),
            path: base.join(format!("{}.jsonl", session_id)),
        }
    }

    pub fn record(&mut self, event: WireEvent) {
        // Bound in-memory growth: long-running sessions otherwise accumulate
        // every message/tool event in memory forever. The JSONL file below
        // keeps the full history for replay/load, so only the live Vec is
        // capped.
        const MAX_IN_MEMORY_EVENTS: usize = 10_000;
        self.events.push(event.clone());
        if self.events.len() > MAX_IN_MEMORY_EVENTS {
            let drop_to = self.events.len() - MAX_IN_MEMORY_EVENTS;
            self.events.drain(0..drop_to);
        }
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(line) = serde_json::to_string(&event) {
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)
                .and_then(|f| {
                    use std::io::Write;
                    writeln!(&f, "{}", line)
                });
        }
    }

    pub fn replay(path: &std::path::Path) -> Vec<WireEvent> {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        content
            .lines()
            .filter_map(|l| serde_json::from_str::<WireEvent>(l).ok())
            .collect()
    }

    /// Load all events for this session (empty if none recorded yet).
    pub fn load(&self) -> Vec<WireEvent> {
        if self.path.exists() {
            Self::replay(&self.path)
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wire_format_roundtrip() {
        let mut session = WireSession::new("test-wire");
        session.record(WireEvent::UserMessage {
            content: "hello".into(),
            timestamp: 1000,
            attachments: None,
        });
        assert_eq!(session.events.len(), 1);
    }

    #[test]
    fn test_wire_replay() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("neocodex_test_wire");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("replay_test.jsonl");
        let event = WireEvent::UserMessage {
            content: "replay".into(),
            timestamp: 42,
            attachments: None,
        };
        let line = serde_json::to_string(&event).unwrap();
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "{}", line).unwrap();
        drop(f);

        let events = WireSession::replay(&path);
        assert_eq!(events.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}