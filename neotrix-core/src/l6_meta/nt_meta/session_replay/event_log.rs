#![deny(clippy::unwrap_used)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Agent-level event types for session replay (agentops pattern).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentEvent {
    ToolCall {
        tool: String,
        input: String,
        output: String,
        duration_ms: u64,
    },
    LLMRequest {
        model: String,
        tokens_in: u64,
        tokens_out: u64,
        latency_ms: u64,
    },
    Decision {
        node: String,
        reasoning: String,
    },
    Error {
        kind: String,
        message: String,
    },
}

impl AgentEvent {
    pub fn kind_name(&self) -> &'static str {
        match self {
            AgentEvent::ToolCall { .. } => "tool_call",
            AgentEvent::LLMRequest { .. } => "llm_request",
            AgentEvent::Decision { .. } => "decision",
            AgentEvent::Error { .. } => "error",
        }
    }
}

/// Timestamped event wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedEvent {
    pub timestamp_ms: u128,
    pub event: AgentEvent,
    pub metadata: HashMap<String, String>,
}

/// Metadata about a session event log.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventLogMetadata {
    pub agent_id: String,
    pub model: Option<String>,
    pub tags: Vec<String>,
    pub extra: HashMap<String, String>,
}

/// Thread-safe event log for recording and querying agent events.
#[derive(Debug, Clone)]
pub struct EventLog {
    /// 2026-09-30 新增：已记录事件的**最大**时间戳，保证写入单调。
    last_ts: Arc<Mutex<u128>>,
    session_id: String,
    events: Arc<Mutex<Vec<TimestampedEvent>>>,
    metadata: Arc<Mutex<EventLogMetadata>>,
}

impl EventLog {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            events: Arc::new(Mutex::new(Vec::new())),
            metadata: Arc::new(Mutex::new(EventLogMetadata::default())),
            // 2026-09-30 新增：单调时间戳游标（见 `record_with_metadata`）。
            last_ts: Arc::new(Mutex::new(0)),
        }
    }

    pub fn with_metadata(session_id: impl Into<String>, meta: EventLogMetadata) -> Self {
        Self {
            session_id: session_id.into(),
            events: Arc::new(Mutex::new(Vec::new())),
            metadata: Arc::new(Mutex::new(meta)),
            // 2026-09-30 新增：与 `new()` 一致（见 `record_with_metadata`）。
            last_ts: Arc::new(Mutex::new(0)),
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Record an event with auto-generated timestamp.
    pub fn record(&self, event: AgentEvent) {
        self.record_with_metadata(event, HashMap::new());
    }

    /// Record an event with custom metadata.
    pub fn record_with_metadata(&self, event: AgentEvent, metadata: HashMap<String, String>) {
        // ⚠️ 2026-09-30 修正：原先直接用 `now_ms()`。但 `now_ms()` 是**毫秒**精度，
        // 同一进程内快速连续 record（测试里一连 4 条）会拿到**完全相同**的时间戳
        // ⇒ `get_events_in_range(mid, mid)` 一次返回**多条**事件，
        // 而调用方（按时间定位单条事件）期望恰好 1 条。
        // 实测症状：自带测试 `test_get_events_in_range` 期望 1 条、实得 3 条。
        //
        // ⇒ 改为**单调递增**时间戳：严格大于此前所有已记录事件；
        // 仍保留毫秒语义（不引入新单位），并用 `max` 保证时钟回拨时也不倒退。
        // ⚠️ 这里必须 `max(now) + 1` 而**不是** `max(now)`：
        // 后者只保证「不倒退」，**不保证递增** —— 同一毫秒内连写 4 条时
        // `now_ms()` 4 次都返回同一个值，`max` 结果仍是同一个值
        // ⇒ 时间戳依旧重复（我第一版就这么写，测试仍然返回 3 条）。
        let ts = {
            let mut last = self.last_ts.lock().expect("event_log lock poisoned");
            let next = now_ms().max(*last + 1);
            *last = next;
            next
        };
        let entry = TimestampedEvent {
            timestamp_ms: ts,
            event,
            metadata,
        };
        let mut events = self.events.lock().expect("event_log lock poisoned");
        events.push(entry);
    }

    /// Get all events in chronological order.
    pub fn get_events(&self) -> Vec<TimestampedEvent> {
        let events = self.events.lock().expect("event_log lock poisoned");
        events.clone()
    }

    /// Get events within a timestamp range (inclusive).
    pub fn get_events_in_range(&self, start_ms: u128, end_ms: u128) -> Vec<TimestampedEvent> {
        let events = self.events.lock().expect("event_log lock poisoned");
        events
            .iter()
            .filter(|e| e.timestamp_ms >= start_ms && e.timestamp_ms <= end_ms)
            .cloned()
            .collect()
    }

    /// Get events by kind.
    pub fn get_events_by_kind(&self, kind: &str) -> Vec<TimestampedEvent> {
        let events = self.events.lock().expect("event_log lock poisoned");
        events
            .iter()
            .filter(|e| e.event.kind_name() == kind)
            .cloned()
            .collect()
    }

    /// Total event count.
    pub fn len(&self) -> usize {
        let events = self.events.lock().expect("event_log lock poisoned");
        events.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Update session metadata.
    pub fn set_metadata(&self, meta: EventLogMetadata) {
        let mut m = self
            .metadata
            .lock()
            .expect("event_log metadata lock poisoned");
        *m = meta;
    }

    /// Get current metadata.
    pub fn metadata(&self) -> EventLogMetadata {
        let m = self
            .metadata
            .lock()
            .expect("event_log metadata lock poisoned");
        m.clone()
    }

    /// Clear all events.
    pub fn clear(&self) {
        let mut events = self.events.lock().expect("event_log lock poisoned");
        events.clear();
    }

    /// Drain events, returning them and clearing the log.
    pub fn drain(&self) -> Vec<TimestampedEvent> {
        let mut events = self.events.lock().expect("event_log lock poisoned");
        std::mem::take(&mut *events)
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tool_event(tool: &str) -> AgentEvent {
        AgentEvent::ToolCall {
            tool: tool.to_string(),
            input: "{}".into(),
            output: "ok".into(),
            duration_ms: 100,
        }
    }

    #[test]
    fn test_record_and_get() {
        let log = EventLog::new("s1");
        log.record(make_tool_event("web_search"));
        log.record(AgentEvent::LLMRequest {
            model: "gpt-4".into(),
            tokens_in: 100,
            tokens_out: 50,
            latency_ms: 200,
        });
        assert_eq!(log.len(), 2);
        let events = log.get_events();
        assert_eq!(events[0].event.kind_name(), "tool_call");
        assert_eq!(events[1].event.kind_name(), "llm_request");
    }

    #[test]
    fn test_get_events_in_range() {
        let log = EventLog::new("s1");
        log.record(make_tool_event("a"));
        log.record(make_tool_event("b"));
        log.record(make_tool_event("c"));
        let events = log.get_events();
        let mid = events[1].timestamp_ms;
        let range = log.get_events_in_range(mid, mid);
        assert_eq!(range.len(), 1);
    }

    #[test]
    fn test_get_events_by_kind() {
        let log = EventLog::new("s1");
        log.record(make_tool_event("a"));
        log.record(AgentEvent::Error {
            kind: "timeout".into(),
            message: "slow".into(),
        });
        assert_eq!(log.get_events_by_kind("tool_call").len(), 1);
        assert_eq!(log.get_events_by_kind("error").len(), 1);
        assert_eq!(log.get_events_by_kind("decision").len(), 0);
    }

    #[test]
    fn test_drain() {
        let log = EventLog::new("s1");
        log.record(make_tool_event("a"));
        let drained = log.drain();
        assert_eq!(drained.len(), 1);
        assert!(log.is_empty());
    }

    #[test]
    fn test_metadata() {
        let log = EventLog::new("s1");
        let mut meta = EventLogMetadata::default();
        meta.agent_id = "agent-1".into();
        meta.tags.push("production".into());
        log.set_metadata(meta);
        let m = log.metadata();
        assert_eq!(m.agent_id, "agent-1");
        assert_eq!(m.tags, vec!["production"]);
    }

    #[test]
    fn test_event_kind_name() {
        assert_eq!(
            AgentEvent::ToolCall {
                tool: "x".into(),
                input: "".into(),
                output: "".into(),
                duration_ms: 0,
            }
            .kind_name(),
            "tool_call"
        );
        assert_eq!(
            AgentEvent::Decision {
                node: "n".into(),
                reasoning: "r".into(),
            }
            .kind_name(),
            "decision"
        );
    }
}
