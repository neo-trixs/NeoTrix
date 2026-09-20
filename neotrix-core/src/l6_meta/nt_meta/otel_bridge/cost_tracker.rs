#![deny(clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Cost record for a single API call or operation.
#[derive(Debug, Clone)]
pub struct CostEntry {
    pub timestamp_ms: u128,
    pub kind: CostEntryKind,
    pub model: Option<String>,
    pub agent_id: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub duration_ms: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CostEntryKind {
    LlmRequest,
    ToolCall,
    Embedding,
    MemoryOp,
}

/// Aggregated cost for a single agent or session.
#[derive(Debug, Clone, Default)]
pub struct CostSummary {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_api_calls: u64,
    pub total_compute_ms: u64,
    pub by_model: HashMap<String, ModelCost>,
    pub by_kind: HashMap<CostEntryKind, KindCost>,
}

#[derive(Debug, Clone, Default)]
pub struct ModelCost {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub api_calls: u64,
    pub compute_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct KindCost {
    pub count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub compute_ms: u64,
}

/// Per-session cost aggregator with thread-safe storage.
#[derive(Debug, Clone)]
pub struct SessionCost {
    pub session_id: String,
    pub entries: Arc<Mutex<Vec<CostEntry>>>,
    pub start: Instant,
}

impl SessionCost {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            entries: Arc::new(Mutex::new(Vec::new())),
            start: Instant::now(),
        }
    }

    pub fn record(&self, entry: CostEntry) {
        let mut entries = self.entries.lock().expect("session cost lock poisoned");
        entries.push(entry);
    }

    pub fn summary(&self) -> CostSummary {
        let entries = self.entries.lock().expect("session cost lock poisoned");
        summarize_entries(&entries)
    }

    pub fn entries(&self) -> Vec<CostEntry> {
        let entries = self.entries.lock().expect("session cost lock poisoned");
        entries.clone()
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}

/// Multi-agent cost tracker with per-agent and global aggregation.
pub struct CostTracker {
    sessions: HashMap<String, SessionCost>,
    global_entries: Vec<CostEntry>,
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            global_entries: Vec::new(),
        }
    }

    pub fn session(&mut self, session_id: &str) -> &SessionCost {
        self.sessions
            .entry(session_id.to_string())
            .or_insert_with(|| SessionCost::new(session_id))
    }

    pub fn record(&mut self, session_id: &str, entry: CostEntry) {
        self.global_entries.push(entry.clone());
        self.session(session_id).record(entry);
    }

    pub fn record_llm(
        &mut self,
        session_id: &str,
        agent_id: &str,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        duration_ms: u64,
    ) {
        let entry = CostEntry {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            kind: CostEntryKind::LlmRequest,
            model: Some(model.to_string()),
            agent_id: agent_id.to_string(),
            input_tokens,
            output_tokens,
            duration_ms,
            metadata: HashMap::new(),
        };
        self.record(session_id, entry);
    }

    pub fn record_tool(
        &mut self,
        session_id: &str,
        agent_id: &str,
        tool_name: &str,
        duration_ms: u64,
    ) {
        let mut metadata = HashMap::new();
        metadata.insert("tool".to_string(), tool_name.to_string());
        let entry = CostEntry {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            kind: CostEntryKind::ToolCall,
            model: None,
            agent_id: agent_id.to_string(),
            input_tokens: 0,
            output_tokens: 0,
            duration_ms,
            metadata,
        };
        self.record(session_id, entry);
    }

    /// Cost report for a specific session.
    pub fn cost_report(&self, session_id: &str) -> CostReport {
        let session = self
            .sessions
            .get(session_id)
            .map(|s| s.summary())
            .unwrap_or_default();
        let global = summarize_entries(&self.global_entries);
        let elapsed = self
            .sessions
            .get(session_id)
            .map(|s| s.elapsed_secs())
            .unwrap_or(0.0);

        CostReport {
            session_id: session_id.to_string(),
            session,
            global,
            elapsed_secs: elapsed,
        }
    }

    /// Global cost report across all sessions.
    pub fn global_report(&self) -> CostSummary {
        summarize_entries(&self.global_entries)
    }

    /// Per-agent breakdown for a session.
    pub fn agent_breakdown(&self, session_id: &str) -> HashMap<String, CostSummary> {
        let session = match self.sessions.get(session_id) {
            Some(s) => s,
            None => return HashMap::new(),
        };
        let entries = session.entries();
        let mut by_agent: HashMap<String, Vec<&CostEntry>> = HashMap::new();
        for e in &entries {
            by_agent.entry(e.agent_id.clone()).or_default().push(e);
        }
        by_agent
            .into_iter()
            .map(|(agent, entries)| {
                let summaries: Vec<CostEntry> = entries.into_iter().cloned().collect();
                (agent, summarize_entries(&summaries))
            })
            .collect()
    }

    pub fn entry_count(&self) -> usize {
        self.global_entries.len()
    }

    pub fn session_ids(&self) -> Vec<&str> {
        self.sessions.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Cost report combining session and global views.
#[derive(Debug, Clone)]
pub struct CostReport {
    pub session_id: String,
    pub session: CostSummary,
    pub global: CostSummary,
    pub elapsed_secs: f64,
}

fn summarize_entries(entries: &[CostEntry]) -> CostSummary {
    let mut summary = CostSummary::default();

    for entry in entries {
        summary.total_input_tokens += entry.input_tokens;
        summary.total_output_tokens += entry.output_tokens;
        summary.total_api_calls += 1;
        summary.total_compute_ms += entry.duration_ms;

        let model_cost = summary
            .by_model
            .entry(entry.model.clone().unwrap_or_else(|| "unknown".into()))
            .or_default();
        model_cost.input_tokens += entry.input_tokens;
        model_cost.output_tokens += entry.output_tokens;
        model_cost.api_calls += 1;
        model_cost.compute_ms += entry.duration_ms;

        let kind_cost = summary.by_kind.entry(entry.kind.clone()).or_default();
        kind_cost.count += 1;
        kind_cost.input_tokens += entry.input_tokens;
        kind_cost.output_tokens += entry.output_tokens;
        kind_cost.compute_ms += entry.duration_ms;
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_tracker_record() {
        let mut tracker = CostTracker::new();
        tracker.record_llm("s1", "agent-1", "gpt-4", 100, 50, 200);
        tracker.record_llm("s1", "agent-1", "gpt-4", 200, 100, 300);
        let report = tracker.cost_report("s1");
        assert_eq!(report.session.total_api_calls, 2);
        assert_eq!(report.session.total_input_tokens, 300);
        assert_eq!(report.session.total_output_tokens, 150);
    }

    #[test]
    fn test_agent_breakdown() {
        let mut tracker = CostTracker::new();
        tracker.record_llm("s1", "a1", "gpt-4", 100, 50, 100);
        tracker.record_llm("s1", "a2", "gpt-4", 200, 100, 200);
        let breakdown = tracker.agent_breakdown("s1");
        assert_eq!(breakdown.len(), 2);
        assert!(breakdown.contains_key("a1"));
        assert!(breakdown.contains_key("a2"));
    }

    #[test]
    fn test_tool_cost() {
        let mut tracker = CostTracker::new();
        tracker.record_tool("s1", "a1", "web_search", 500);
        let report = tracker.cost_report("s1");
        assert_eq!(report.session.total_api_calls, 1);
        assert_eq!(report.session.total_compute_ms, 500);
    }

    #[test]
    fn test_global_report() {
        let mut tracker = CostTracker::new();
        tracker.record_llm("s1", "a1", "gpt-4", 100, 50, 100);
        tracker.record_llm("s2", "a2", "claude-3", 200, 100, 200);
        let global = tracker.global_report();
        assert_eq!(global.total_api_calls, 2);
    }

    #[test]
    fn test_session_cost_default() {
        let summary = CostSummary::default();
        assert_eq!(summary.total_input_tokens, 0);
        assert!(summary.by_model.is_empty());
    }
}
