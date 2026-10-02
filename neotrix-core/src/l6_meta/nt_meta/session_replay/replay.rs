#![deny(clippy::unwrap_used)]

// 2026-09-30 修正导入：编译器报 `TimestampedEvent` unused（核实为真）⇒ 去掉；
// 而 serde 的一行报告是**误报** —— 它在下方 `#[derive(Serialize, Deserialize)]`
// 里被使用（我曾据 grep 结论删掉它，编译器立即报「cannot find derive macro」）。
// ⇒ 教训：**与编译器冲突时以编译器为准**，别用自己的 grep 覆盖它。
use super::event_log::{AgentEvent, EventLog};
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

        // ⚠️ 2026-09-30 修正：原先 `self.cursor = self.snapshots.len()`，
        // 即**构造完成即把游标置于末尾**（「已处理全部事件」状态）。
        // 但 `new()` 的字段初值是 `0`，且没有任何方法把游标送回起点
        // ⇒ 刚构造出的 `SessionReplay` **无法从头回放**：
        // `step_forward()` 立刻返回 `None`（已在最后一项）。
        //
        // 实测症状：自带测试 `test_step_forward_back` 里
        // `replay.step_forward().unwrap()` 直接 panic（None）。
        //
        // 判定为**真缺陷**而非「测试写错」：
        // · `final_state()` 已单独提供「取末态」的语义，
        //   构造即末尾让「从起点逐步前进」这一**主要用途**无法使用；
        // · `SessionReplay` 全仓**零生产消费者**
        //   （`anti_distillation` 里的 `SessionReplayGuard` 是**同名不同类型**，
        //   已核实二者无关）⇒ 改动不影响任何生产行为。
        //
        // ⇒ 改为从**起点**开始；需要末态用 `final_state()`。
        self.cursor = 0;
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

            // ⚠️ 2026-09-30 修正：`te.metadata` 的类型是
            // `HashMap<String, String>`（**不是** `Option<_>`），
            // 原代码按 `Option` 写 `if let Some(meta) = …`
            // ⇒ `E0308: expected HashMap<…>, found Option<_>`。
            // ⇒ 直接用 `is_empty()` 判空即可（空 map 自然跳过）。
            {
                let meta = &te.metadata;
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
        // ⚠️ 2026-09-30 修正：原断言是
        //   `fwd.index == initial` 且 `back.index == initial.saturating_sub(1)`，
        // 即「前进后 index 不变、后退后 index 减一」——
        // 这**既与实现不符、自身也自相矛盾**（前进本就该改变 index）。
        // 实现语义（已逐行核实 `step_forward` / `step_back`）：
        //   `cursor += 1` 后返回 `snapshots[cursor]` ⇒ index **加一**；
        //   `cursor -= 1` 后返回 `snapshots[cursor]` ⇒ index **减一**。
        // ⇒ 按实现语义断言，并补上「回到起点」的往返校验。
        let fwd = replay.step_forward().unwrap();
        assert_eq!(fwd.index, initial + 1);
        assert_eq!(replay.cursor(), initial + 1);
        let back = replay.step_back().unwrap();
        assert_eq!(back.index, initial);
        assert_eq!(replay.cursor(), initial);
        // 在起点再后退应返回 None（不可越界）
        assert!(replay.step_back().is_none());
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
