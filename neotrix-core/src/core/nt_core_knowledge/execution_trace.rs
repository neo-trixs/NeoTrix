use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Types of actions in a trace
#[derive(Debug, Clone)]
pub enum TraceAction {
    HandlerCall(String),
    KnowledgeQuery(String),
    AgentMessage(String),
    VsaOperation(String),
    Decision {
        description: String,
        confidence: f64,
    },
}

/// A single step in an execution trace
#[derive(Debug, Clone)]
pub struct TraceStep {
    pub seq: u64,
    pub action: TraceAction,
    pub input_summary: String,
    pub output_summary: String,
    pub timestamp_ns: u64,
    pub duration_us: u64,
    pub success: bool,
}

/// A complete execution trace for one task
#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    pub trace_id: u64,
    pub agent_id: String,
    pub task: String,
    pub started_at_ns: u64,
    pub steps: Vec<TraceStep>,
    pub succeeded: bool,
    pub total_duration_us: u64,
}

/// Manages execution traces
pub struct TraceManager {
    pub traces: HashMap<u64, ExecutionTrace>,
    next_id: u64,
    pub max_traces: usize,
    pub enabled: bool,
}

impl TraceManager {
    pub fn new(max_traces: usize) -> Self {
        Self {
            traces: HashMap::new(),
            next_id: 1,
            max_traces,
            enabled: true,
        }
    }

    pub fn start_trace(&mut self, agent_id: &str, task: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        self.traces.insert(
            id,
            ExecutionTrace {
                trace_id: id,
                agent_id: agent_id.to_string(),
                task: task.to_string(),
                started_at_ns: now,
                steps: Vec::new(),
                succeeded: false,
                total_duration_us: 0,
            },
        );
        if self.traces.len() > self.max_traces {
            if let Some(oldest) = self.traces.keys().min().copied() {
                self.traces.remove(&oldest);
            }
        }
        id
    }

    pub fn record_step(
        &mut self,
        trace_id: u64,
        action: TraceAction,
        input: &str,
        output: &str,
        duration_us: u64,
        success: bool,
    ) -> Result<(), String> {
        let trace = self.traces.get_mut(&trace_id).ok_or("Trace not found")?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        trace.steps.push(TraceStep {
            seq: trace.steps.len() as u64 + 1,
            action,
            input_summary: input.chars().take(128).collect(),
            output_summary: output.chars().take(128).collect(),
            timestamp_ns: now,
            duration_us,
            success,
        });
        Ok(())
    }

    pub fn complete_trace(&mut self, trace_id: u64, succeeded: bool) -> Result<(), String> {
        let trace = self.traces.get_mut(&trace_id).ok_or("Trace not found")?;
        trace.succeeded = succeeded;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        trace.total_duration_us = (now - trace.started_at_ns) / 1000;
        Ok(())
    }

    pub fn trace_report(&self, trace_id: u64) -> String {
        match self.traces.get(&trace_id) {
            Some(t) => {
                let ok = t.steps.iter().filter(|s| s.success).count();
                format!(
                    "Trace #{} [{}]: {} steps ({}/{} ok), {}µs, verdict={}",
                    t.trace_id,
                    t.task,
                    t.steps.len(),
                    ok,
                    t.steps.len(),
                    t.total_duration_us,
                    if t.succeeded { "PASS" } else { "FAIL" }
                )
            }
            None => "Trace not found".into(),
        }
    }

    pub fn recent_summary(&self, limit: usize) -> Vec<(u64, String, usize, bool)> {
        let mut v: Vec<_> = self.traces.values().collect();
        v.sort_by(|a, b| b.started_at_ns.cmp(&a.started_at_ns));
        v.iter()
            .take(limit)
            .map(|t| (t.trace_id, t.task.clone(), t.steps.len(), t.succeeded))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_record_complete() {
        let mut tm = TraceManager::new(100);
        let id = tm.start_trace("agent1", "test_task");
        tm.record_step(
            id,
            TraceAction::HandlerCall("handle_x".into()),
            "in",
            "out",
            100,
            true,
        )
        .unwrap();
        tm.complete_trace(id, true).unwrap();
        assert!(tm.trace_report(id).contains("PASS"));
        assert!(tm.trace_report(id).contains("1 steps"));
    }

    #[test]
    fn test_record_nonexistent_trace() {
        let mut tm = TraceManager::new(100);
        let result = tm.record_step(
            999,
            TraceAction::HandlerCall("x".into()),
            "in",
            "out",
            0,
            true,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_lru_eviction() {
        let mut tm = TraceManager::new(2);
        let id1 = tm.start_trace("a", "t1");
        let id2 = tm.start_trace("b", "t2");
        let id3 = tm.start_trace("c", "t3");
        assert!(tm.traces.get(&id1).is_none());
        assert!(tm.traces.get(&id2).is_some());
        assert!(tm.traces.get(&id3).is_some());
    }
}
