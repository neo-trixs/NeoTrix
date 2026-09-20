#![deny(clippy::unwrap_used)]

use super::event_log::{AgentEvent, EventLog, TimestampedEvent};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Periodic state snapshot for replay reconstruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub index: usize,
    pub timestamp_ms: u128,
    pub accumulated_tokens_in: u64,
    pub accumulated_tokens_out: u64,
    pub accumulated_api_calls: u64,
    pub accumulated_compute_ms: u64,
    pub error_count: usize,
    pub tools_called: Vec<String>,
}

/// Session replay engine for navigating agent event history.
pub struct SessionReplay {
    event_log: EventLog,
    snapshots: Vec<StateSnapshot>,
    cursor: usize,
}

impl SessionReplay {
    /// Create a new replay engine from an event log.
    /// Pre-builds snapshots for efficient random access.
    pub fn new(event_log: EventLog) -> Self {
        let mut replay = Self {
            event_log,
            snapshots: Vec::new(),
            cursor: 0,
        };
        replay.build_snapshots();
        replay
    }

    /// Rebuild snapshots from the event log.
    fn build_snapshots(&mut self) {
        self.snapshots.clear();
        let events = self.event_log.get_events();
        let mut acc_tokens_in: u64 = 0;
        let mut acc_tokens_out: u64 = 0;
        let mut acc_api_calls: u64 = 0;
        let mut acc_compute_ms: u64 = 0;
        let mut error_count: usize = 0;
        let mut tools_called: Vec<String> = Vec::new();

        for (idx, te) in events.iter().enumerate() {
            match &te.event {
                AgentEvent::LLMRequest {
                    tokens_in,
                    tokens_out,
                    latency_ms,
                    ..
                } => {
                    acc_tokens_in += tokens_in;
                    acc_tokens_out += tokens_out;
                    acc_compute_ms += latency_ms;
                    acc_api_calls += 1;
                }
                AgentEvent::ToolCall {
                    tool, duration_ms, ..
                } => {
                    acc_compute_ms += duration_ms;
                    tools_called.push(tool.clone());
                }
                AgentEvent::Error { .. } => {
                    error_count += 1;
                }
                AgentEvent::Decision { .. } => {}
            }

            self.snapshots.push(StateSnapshot {
                index: idx,
                timestamp_ms: te.timestamp_ms,
                accumulated_tokens_in: acc_tokens_in,
                accumulated_tokens_out: acc_tokens_out,
                accumulated_api_calls: acc_api_calls,
                accumulated_compute_ms: acc_compute_ms,
                error_count,
                tools_called: tools_called.clone(),
            });
        }

        self.cursor = self.snapshots.len();
    }

    /// Replay state at a given event index (0-based).
    /// Returns None if index is out of range.
    pub fn replay_from(&mut self, index: usize) -> Option<StateSnapshot> {
        if index < self.snapshots.len() {
            self.cursor = index;
            Some(self.snapshots[index].clone())
        } else {
            None
        }
    }

    /// Current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Step forward one event. Returns the new state.
    pub fn step_forward(&mut self) -> Option<StateSnapshot> {
        if self.cursor + 1 < self.snapshots.len() {
            self.cursor += 1;
            Some(self.snapshots[self.cursor].clone())
        } else {
            None
        }
    }

    /// Step back one event. Returns the new state.
    pub fn step_back(&mut self) -> Option<StateSnapshot> {
        if self.cursor > 0 {
            self.cursor -= 1;
            Some(self.snapshots[self.cursor].clone())
        } else {
            None
        }
    }

    /// Jump to a specific index.
    pub fn jump_to(&mut self, index: usize) -> Option<StateSnapshot> {
        self.replay_from(index)
    }

    /// Get the final state (all events processed).
    pub fn final_state(&self) -> Option<StateSnapshot> {
        self.snapshots.last().cloned()
    }

    /// Total events in the log.
    pub fn event_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Export a human-readable timeline of all events.
    pub fn export_timeline(&self) -> String {
        let events = self.event_log.get_events();
        let mut out = format!("Session: {}\n", self.event_log.session_id());
        out.push_str(&format!("Total events: {}\n", events.len()));
        out.push_str("---\n");

        for (idx, te) in events.iter().enumerate() {
            let ts = format_timestamp(te.timestamp_ms);
            let summary = event_summary(&te.event);
            out.push_str(&format!("[{ts}] #{idx}: {summary}\n"));

            if let Some(meta) = &te.metadata {
                if !meta.is_empty() {
                    let pairs: Vec<String> = meta.iter().map(|(k, v)| format!("{k}={v}")).collect();
                    out.push_str(&format!("         meta: {}\n", pairs.join(", ")));
                }
            }
        }

        if let Some(final_s) = self.snapshots.last() {
            out.push_str("---\n");
            out.push_str(&format!(
                "Final: {} tokens in, {} tokens out, {} API calls, {} compute ms, {} errors\n",
                final_s.accumulated_tokens_in,
                final_s.accumulated_tokens_out,
                final_s.accumulated_api_calls,
                final_s.accumulated_compute_ms,
                final_s.error_count,
            ));
        }

        out
    }
}

fn event_summary(event: &AgentEvent) -> String {
    match event {
        AgentEvent::ToolCall {
            tool, duration_ms, ..
        } => format!("ToolCall({tool}, {duration_ms}ms)"),
        AgentEvent::LLMRequest {
            model,
            tokens_in,
            tokens_out,
            latency_ms,
        } => format!("LLM({model}, in={tokens_in}, out={tokens_out}, {latency_ms}ms)"),
        AgentEvent::Decision { node, .. } => format!("Decision({node})"),
        AgentEvent::Error { kind, message } => format!("Error({kind}: {message})"),
    }
}

fn format_timestamp(ts_ms: u128) -> String {
    let secs = (ts_ms / 1000) as u64;
    let millis = (ts_ms % 1000) as u32;
    format!("{secs}.{millis:03}")
}

impl fmt::Display for StateSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Snapshot@{}: tokens_in={} tokens_out={} calls={} compute={}ms errors={} tools={}",
            self.index,
            self.accumulated_tokens_in,
            self.accumulated_tokens_out,
            self.accumulated_api_calls,
            self.accumulated_compute_ms,
            self.error_count,
            self.tools_called.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn populate_log() -> EventLog {
        let log = EventLog::new("replay-test");
        log.record(AgentEvent::LLMRequest {
            model: "gpt-4".into(),
            tokens_in: 100,
            tokens_out: 50,
            latency_ms: 200,
        });
        log.record(AgentEvent::ToolCall {
            tool: "web_search".into(),
            input: "query".into(),
            output: "results".into(),
            duration_ms: 150,
        });
        log.record(AgentEvent::Error {
            kind: "timeout".into(),
            message: "slow".into(),
        });
        log.record(AgentEvent::LLMRequest {
            model: "gpt-4".into(),
            tokens_in: 200,
            tokens_out: 80,
            latency_ms: 300,
        });
        log
    }

    #[test]
    fn test_replay_from() {
        let log = populate_log();
        let mut replay = SessionReplay::new(log);
        let state = replay.replay_from(1).unwrap();
        assert_eq!(state.index, 1);
        assert_eq!(state.accumulated_tokens_in, 100);
        assert_eq!(state.accumulated_api_calls, 1);
    }

    #[test]
    fn test_step_forward_back() {
        let log = populate_log();
        let mut replay = SessionReplay::new(log);
        let initial = replay.cursor();
        let fwd = replay.step_forward().unwrap();
        assert_eq!(fwd.index, initial);
        let back = replay.step_back().unwrap();
        assert_eq!(back.index, initial.saturating_sub(1));
    }

    #[test]
    fn test_final_state() {
        let log = populate_log();
        let mut replay = SessionReplay::new(log);
        let final_s = replay.final_state().unwrap();
        assert_eq!(final_s.index, 3);
        assert_eq!(final_s.accumulated_tokens_in, 300);
        assert_eq!(final_s.error_count, 1);
    }

    #[test]
    fn test_export_timeline() {
        let log = populate_log();
        let replay = SessionReplay::new(log);
        let timeline = replay.export_timeline();
        assert!(timeline.contains("replay-test"));
        assert!(timeline.contains("LLM("));
        assert!(timeline.contains("ToolCall("));
        assert!(timeline.contains("Error("));
        assert!(timeline.contains("Final:"));
    }

    #[test]
    fn test_jump_to() {
        let log = populate_log();
        let mut replay = SessionReplay::new(log);
        assert!(replay.jump_to(2).is_some());
        assert!(replay.jump_to(100).is_none());
    }
}
