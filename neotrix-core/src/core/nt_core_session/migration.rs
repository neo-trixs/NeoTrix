use std::collections::HashMap;

use crate::core::nt_core_session::unified_record::{
    RecordPayload, RecordType, UnifiedSessionRecord,
};

/// Migrate from CrossSessionMemory JSON format (HashMap-based, custom schema).
///
/// The old format serializes the entire `CrossSessionMemory` struct as JSON:
/// ```json
/// { "store": { "key": { "key": "...", "value": "...", "category": "Principle", ... } }, ... }
/// ```
/// Each entry becomes a `KnowledgeEntry` record.
pub fn from_cross_session_memory_json(data: &str, session_id: &str) -> Vec<UnifiedSessionRecord> {
    #[derive(serde::Deserialize)]
    struct OldEntry {
        key: String,
        value: String,
        category: String,
        confidence: f64,
        created_at: u64,
        last_accessed: u64,
        access_count: u64,
    }

    #[derive(serde::Deserialize)]
    struct OldStore {
        store: HashMap<String, OldEntry>,
    }

    let Ok(old) = serde_json::from_str::<OldStore>(data) else {
        log::warn!("[migration] from_cross_session_memory_json: failed to parse");
        return Vec::new();
    };

    old.store
        .into_values()
        .map(|e| {
            let mut fields = HashMap::new();
            fields.insert("category".to_string(), e.category);
            fields.insert("confidence".to_string(), e.confidence.to_string());
            fields.insert("created_at".to_string(), e.created_at.to_string());
            fields.insert("last_accessed".to_string(), e.last_accessed.to_string());
            fields.insert("access_count".to_string(), e.access_count.to_string());

            UnifiedSessionRecord::new(
                session_id,
                RecordType::KnowledgeEntry,
                RecordPayload::JSON { data: fields },
                "cross_session_memory",
            )
        })
        .collect()
}

/// Migrate from SessionTranscript JSONL format.
///
/// The old format writes one `TranscriptEvent` per line as JSONL.
/// Each line is a serde-tagged enum:
/// ```json
/// {"type":"session_start","uuid":"...","timestamp":...,"session_id":"...",...}
/// ```
pub fn from_session_transcript_jsonl(data: &str, session_id: &str) -> Vec<UnifiedSessionRecord> {
    #[derive(serde::Deserialize)]
    struct OldTranscriptEvent {
        #[serde(rename = "type")]
        event_type: String,
        uuid: Option<String>,
        timestamp: Option<u64>,
        content: Option<String>,
        summary: Option<String>,
        handler: Option<String>,
        decision: Option<String>,
        passed: Option<bool>,
    }

    let mut results = Vec::new();

    for line in data.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Ok(event) = serde_json::from_str::<OldTranscriptEvent>(line) else {
            log::warn!("[migration] from_session_transcript_jsonl: skipping unparseable line");
            continue;
        };

        let mut fields = HashMap::new();
        if let Some(u) = &event.uuid {
            fields.insert("uuid".to_string(), u.clone());
        }
        if let Some(h) = &event.handler {
            fields.insert("handler".to_string(), h.clone());
        }
        if let Some(d) = &event.decision {
            fields.insert("decision".to_string(), d.clone());
        }
        if let Some(p) = event.passed {
            fields.insert("passed".to_string(), p.to_string());
        }
        if let Some(s) = &event.summary {
            fields.insert("summary".to_string(), s.clone());
        }

        let record_type = match event.event_type.as_str() {
            "session_start" | "session_end" => RecordType::SystemEvent,
            "user_input" => RecordType::ChatMessage,
            "handler_dispatch" => RecordType::AgentAction,
            "agent_communication" => RecordType::AgentAction,
            "permission_check" | "verify_check" => RecordType::SystemEvent,
            other => RecordType::Custom(other.to_string()),
        };

        let payload = if let Some(c) = event.content {
            RecordPayload::Text { role: "user".to_string(), content: c }
        } else {
            RecordPayload::JSON { data: fields }
        };

        results.push(UnifiedSessionRecord::new(session_id, record_type, payload, "transcript"));
    }

    results
}

/// Migrate from a plain text/JSON chat log format.
/// Tries to parse as JSONL first, falls back to line-by-line text.
pub fn from_chat_log(data: &str, session_id: &str) -> Vec<UnifiedSessionRecord> {
    let mut results = Vec::new();

    for line in data.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Try to parse as a simple {"role":"...","content":"..."} line
        if let Ok(obj) = serde_json::from_str::<HashMap<String, String>>(line) {
            let role = obj.get("role").cloned().unwrap_or_default();
            let content = obj.get("content").cloned().unwrap_or_default();
            results.push(UnifiedSessionRecord::new(
                session_id,
                RecordType::ChatMessage,
                RecordPayload::text(role, content),
                "chat_log",
            ));
        } else {
            // Plain text line — treat as assistant message
            results.push(UnifiedSessionRecord::new(
                session_id,
                RecordType::ChatMessage,
                RecordPayload::text("assistant", line),
                "chat_log",
            ));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_cross_session_memory_json() {
        let json = r#"{
            "store": {
                "k1": { "key": "k1", "value": "v1", "category": "Principle", "confidence": 0.9, "created_at": 1000, "last_accessed": 1001, "access_count": 2 },
                "k2": { "key": "k2", "value": "v2", "category": "Pattern", "confidence": 0.8, "created_at": 1002, "last_accessed": 1003, "access_count": 1 }
            },
            "max_entries": 1000
        }"#;
        let records = from_cross_session_memory_json(json, "test-session");
        assert_eq!(records.len(), 2);
        for r in &records {
            assert_eq!(r.session_id, "test-session");
            assert_eq!(r.source, "cross_session_memory");
            assert_eq!(r.record_type, RecordType::KnowledgeEntry);
        }
    }

    #[test]
    fn test_from_session_transcript_jsonl() {
        let jsonl = r#"{"type":"session_start","uuid":"u1","timestamp":1000,"session_id":"s1","label":"test","init_data":""}
{"type":"user_input","uuid":"u2","timestamp":1001,"session_id":"s1","content":"hello","input_type":"text"}"#;
        let records = from_session_transcript_jsonl(jsonl, "s1");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].record_type, RecordType::SystemEvent);
        assert_eq!(records[0].source, "transcript");
        assert_eq!(records[1].record_type, RecordType::ChatMessage);
        if let RecordPayload::Text { ref role, ref content } = records[1].payload {
            assert_eq!(role, "user");
            assert_eq!(content, "hello");
        } else {
            panic!("expected Text payload");
        }
    }

    #[test]
    fn test_from_chat_log_jsonl() {
        let data = r#"{"role":"user","content":"hi"}
{"role":"assistant","content":"hello"}"#;
        let records = from_chat_log(data, "chat-1");
        assert_eq!(records.len(), 2);
        for r in &records {
            assert_eq!(r.record_type, RecordType::ChatMessage);
        }
    }

    #[test]
    fn test_from_chat_log_plain() {
        let data = "hello world\nhow are you";
        let records = from_chat_log(data, "chat-2");
        assert_eq!(records.len(), 2);
        for r in &records {
            assert_eq!(r.record_type, RecordType::ChatMessage);
            if let RecordPayload::Text { ref role, .. } = r.payload {
                assert_eq!(role, "assistant");
            } else {
                panic!("expected Text payload");
            }
        }
    }
}
