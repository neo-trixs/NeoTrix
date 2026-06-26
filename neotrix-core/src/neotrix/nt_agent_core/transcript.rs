use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// Session transcript — append-only JSONL structured session log.
/// Each line is a self-contained `TranscriptEvent`.
/// Inspired by Claude Code's structured session persistence.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum TranscriptEvent {
    #[serde(rename = "session_start")]
    SessionStart {
        uuid: String,
        timestamp: u64,
        session_id: String,
        label: String,
        init_data: String,
    },
    #[serde(rename = "session_end")]
    SessionEnd {
        uuid: String,
        timestamp: u64,
        session_id: String,
        duration_cycles: u64,
        summary: String,
    },
    #[serde(rename = "handler_dispatch")]
    HandlerDispatch {
        uuid: String,
        timestamp: u64,
        session_id: String,
        handler: String,
        input_summary: String,
        output_summary: String,
        duration_ms: u64,
        tier: String,
    },
    #[serde(rename = "user_input")]
    UserInput {
        uuid: String,
        timestamp: u64,
        session_id: String,
        content: String,
        input_type: String,
    },
    #[serde(rename = "agent_communication")]
    AgentCommunication {
        uuid: String,
        timestamp: u64,
        session_id: String,
        sender: String,
        recipients: Vec<String>,
        message_type: String,
        summary: String,
    },
    #[serde(rename = "permission_check")]
    PermissionCheck {
        uuid: String,
        timestamp: u64,
        session_id: String,
        handler: String,
        decision: String,
        mode: String,
    },
    #[serde(rename = "verify_check")]
    VerifyCheck {
        uuid: String,
        timestamp: u64,
        session_id: String,
        handler: String,
        passed: bool,
        issues: Vec<String>,
    },
}

impl TranscriptEvent {
    pub fn as_json_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct SessionTranscript {
    session_id: String,
    path: Option<PathBuf>,
    buffer: VecDeque<TranscriptEvent>,
    max_buffer: usize,
    total_events: u64,
    label: String,
    enabled: bool,
}

impl SessionTranscript {
    pub fn new(session_id: String, label: String) -> Self {
        Self {
            session_id,
            path: None,
            buffer: VecDeque::with_capacity(64),
            max_buffer: 1_000,
            total_events: 0,
            label,
            enabled: true,
        }
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        self.path = Some(path);
        self
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn total_events(&self) -> u64 {
        self.total_events
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_path(&mut self, path: PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        self.path = Some(path);
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    fn record(&mut self, event: TranscriptEvent) {
        if !self.enabled {
            return;
        }
        self.total_events += 1;
        if self.buffer.len() >= self.max_buffer {
            self.buffer.pop_front();
        }
        self.buffer.push_back(event);
    }

    pub fn start_session(&mut self, init_data: String) {
        let uuid = format!(
            "{}-start",
            &self.session_id[..self.session_id.len().min(36)]
        );
        let event = TranscriptEvent::SessionStart {
            uuid,
            timestamp: unix_now(),
            session_id: self.session_id.clone(),
            label: self.label.clone(),
            init_data,
        };
        self.record(event);
        let _ = self.flush();
    }

    pub fn end_session(&mut self, duration_cycles: u64, summary: String) {
        let uuid = format!("{}-end", &self.session_id[..self.session_id.len().min(36)]);
        let event = TranscriptEvent::SessionEnd {
            uuid,
            timestamp: unix_now(),
            session_id: self.session_id.clone(),
            duration_cycles,
            summary,
        };
        self.record(event);
        let _ = self.flush();
    }

    pub fn record_handler(
        &mut self,
        handler: &str,
        input_summary: &str,
        output_summary: &str,
        duration_ms: u64,
        tier: &str,
    ) {
        let uuid = format!("h-{}-{}", handler, unix_now());
        let event = TranscriptEvent::HandlerDispatch {
            uuid,
            timestamp: unix_now(),
            session_id: self.session_id.clone(),
            handler: handler.to_string(),
            input_summary: input_summary.to_string(),
            output_summary: output_summary.to_string(),
            duration_ms,
            tier: tier.to_string(),
        };
        self.record(event);
    }

    pub fn record_user_input(&mut self, content: &str, input_type: &str) {
        let uuid = format!("u-{}", unix_now());
        let event = TranscriptEvent::UserInput {
            uuid,
            timestamp: unix_now(),
            session_id: self.session_id.clone(),
            content: truncate(content, 500),
            input_type: input_type.to_string(),
        };
        self.record(event);
    }

    pub fn record_agent_communication(
        &mut self,
        sender: &str,
        recipients: &[String],
        message_type: &str,
        summary: &str,
    ) {
        let uuid = format!("a-{}-{}", sender, unix_now());
        let event = TranscriptEvent::AgentCommunication {
            uuid,
            timestamp: unix_now(),
            session_id: self.session_id.clone(),
            sender: sender.to_string(),
            recipients: recipients.to_vec(),
            message_type: message_type.to_string(),
            summary: truncate(summary, 200),
        };
        self.record(event);
    }

    pub fn record_permission_check(&mut self, handler: &str, decision: &str, mode: &str) {
        let uuid = format!("p-{}-{}", handler, unix_now());
        let event = TranscriptEvent::PermissionCheck {
            uuid,
            timestamp: unix_now(),
            session_id: self.session_id.clone(),
            handler: handler.to_string(),
            decision: decision.to_string(),
            mode: mode.to_string(),
        };
        self.record(event);
    }

    pub fn record_verify_check(&mut self, handler: &str, passed: bool, issues: &[String]) {
        let uuid = format!("v-{}-{}", handler, unix_now());
        let event = TranscriptEvent::VerifyCheck {
            uuid,
            timestamp: unix_now(),
            session_id: self.session_id.clone(),
            handler: handler.to_string(),
            passed,
            issues: issues.to_vec(),
        };
        self.record(event);
    }

    /// Flush buffered events to disk as JSONL
    pub fn flush(&mut self) -> std::io::Result<u64> {
        let count = self.buffer.len() as u64;
        if count == 0 {
            return Ok(0);
        }
        if let Some(ref path) = self.path {
            let file = OpenOptions::new().create(true).append(true).open(path)?;
            let mut writer = BufWriter::new(file);
            while let Some(event) = self.buffer.pop_front() {
                writeln!(writer, "{}", event.as_json_line())?;
            }
            writer.flush()?;
        }
        Ok(count)
    }

    /// Read back all events from disk
    pub fn replay(&self) -> Vec<TranscriptEvent> {
        let Some(ref path) = self.path else {
            return Vec::new();
        };
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        content
            .lines()
            .filter_map(|line| serde_json::from_str::<TranscriptEvent>(line).ok())
            .collect()
    }

    /// Count events by type from disk
    pub fn event_counts(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for event in self.replay() {
            let label = match event {
                TranscriptEvent::SessionStart { .. } => "session_start",
                TranscriptEvent::SessionEnd { .. } => "session_end",
                TranscriptEvent::HandlerDispatch { .. } => "handler_dispatch",
                TranscriptEvent::UserInput { .. } => "user_input",
                TranscriptEvent::AgentCommunication { .. } => "agent_communication",
                TranscriptEvent::PermissionCheck { .. } => "permission_check",
                TranscriptEvent::VerifyCheck { .. } => "verify_check",
            };
            *counts.entry(label.to_string()).or_insert(0) += 1;
        }
        counts
    }
}

impl Default for SessionTranscript {
    fn default() -> Self {
        Self::new("default".into(), "default".into())
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}... (truncated {} chars)", &s[..max], s.len() - max)
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;

    #[test]
    fn test_session_lifecycle() {
        let dir = std::env::temp_dir().join("nt_transcript_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test.jsonl");
        let mut t =
            SessionTranscript::new("test-session".into(), "test".into()).with_path(path.clone());

        t.start_session("boot".into());
        t.record_handler("process", "input", "output", 42, "hot");
        t.record_user_input("hello", "text");
        let _ = t.flush();

        let events = t.replay();
        assert_eq!(events.len(), 3);
        assert_eq!(t.total_events(), 3);

        t.end_session(100, "done".into());
        let _ = t.flush();
        let events = t.replay();
        assert_eq!(events.len(), 4);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_event_counts() {
        let dir = std::env::temp_dir().join("nt_transcript_counts");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("counts.jsonl");
        let mut t =
            SessionTranscript::new("count-test".into(), "test".into()).with_path(path.clone());

        t.start_session("init".into());
        t.record_handler("h1", "in", "out", 10, "hot");
        t.record_handler("h2", "in", "out", 20, "warm");
        t.record_user_input("hi", "text");
        let _ = t.flush();

        let counts = t.event_counts();
        assert_eq!(counts.get("session_start").copied().unwrap_or(0), 1);
        assert_eq!(counts.get("handler_dispatch").copied().unwrap_or(0), 2);
        assert_eq!(counts.get("user_input").copied().unwrap_or(0), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_disabled_does_not_record() {
        let mut t = SessionTranscript::new("disabled".into(), "test".into());
        t.set_enabled(false);
        t.record_handler("h", "in", "out", 0, "hot");
        assert_eq!(t.total_events(), 0);
    }

    #[test]
    fn test_truncate_long_content() {
        let long = "a".repeat(1000);
        let truncated = truncate(&long, 10);
        assert!(truncated.len() < 1000);
        assert!(truncated.contains("truncated"));
    }
}
