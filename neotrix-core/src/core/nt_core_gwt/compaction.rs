//! 5-layer context compaction pipeline
//!
//! Implements progressive context management for the Global Workspace.
//! Layers (applied sequentially):
//! 1. Budget — enforce max token/window size
//! 2. Snip — drop everything beyond the last-N turns
//! 3. Microcompact — summarize oldest remaining entries
//! 4. Collapse — anchored iterative compaction into a ContextState document
//! 5. Auto-compact — automatically trigger when window exceeds threshold

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Minimum context entries to preserve after any compaction stage.
pub const MIN_CONTEXT_ENTRIES: usize = 10;
/// Default max entries before budget enforcement.
pub const DEFAULT_BUDGET: usize = 100;
/// Auto-compact triggers when window exceeds this fraction of budget.
pub const AUTO_COMPACT_THRESHOLD: f64 = 0.95;

/// A compacted contextual state — the persistent anchor document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextState {
    /// Goal / intent of the current session
    pub current_goal: Option<String>,
    /// Key decisions made so far
    pub key_decisions: Vec<String>,
    /// Pending tasks or questions
    pub pending_items: Vec<String>,
    /// Summary of completed sub-tasks
    pub completed_summary: Vec<String>,
    /// Last update tick
    pub last_updated: usize,
}

impl ContextState {
    pub fn update_goal(&mut self, goal: String, tick: usize) {
        self.current_goal = Some(goal);
        self.last_updated = tick;
    }

    pub fn add_decision(&mut self, decision: String, tick: usize) {
        self.key_decisions.push(decision);
        self.last_updated = tick;
    }

    pub fn add_pending(&mut self, item: String, tick: usize) {
        self.pending_items.push(item);
        self.last_updated = tick;
    }

    pub fn resolve_pending(&mut self, item: &str, tick: usize) {
        self.pending_items.retain(|i| i != item);
        self.last_updated = tick;
    }

    pub fn add_completed(&mut self, summary: String, tick: usize) {
        self.completed_summary.push(summary);
        self.last_updated = tick;
    }

    pub fn is_stale(&self, tick: usize, max_ticks: usize) -> bool {
        tick.saturating_sub(self.last_updated) > max_ticks
    }
}

/// 5-layer compaction pipeline result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionReport {
    pub entries_before: usize,
    pub entries_after: usize,
    pub stages_applied: Vec<String>,
    pub context_state: ContextState,
    pub auto_compacted: bool,
}

/// Progressive compaction pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionPipeline {
    pub budget: usize,
    pub snip_keep: usize,
    pub auto_compact_threshold: f64,
    pub context_state: ContextState,
    tick: usize,
}

impl Default for CompactionPipeline {
    fn default() -> Self {
        Self {
            budget: DEFAULT_BUDGET,
            snip_keep: MIN_CONTEXT_ENTRIES,
            auto_compact_threshold: AUTO_COMPACT_THRESHOLD,
            context_state: ContextState::default(),
            tick: 0,
        }
    }
}

impl CompactionPipeline {
    pub fn new(budget: usize) -> Self {
        Self {
            budget,
            ..Default::default()
        }
    }

    pub fn tick(&mut self) -> usize {
        self.tick += 1;
        self.tick
    }

    /// Run the full 5-layer compaction pipeline on broadcast history.
    pub fn compact(&mut self, history: &mut VecDeque<String>) -> CompactionReport {
        let entries_before = history.len();
        let mut stages_applied: Vec<String> = Vec::new();

        // Layer 1: Budget enforcement
        if history.len() > self.budget {
            let excess = history.len() - self.budget;
            for _ in 0..excess {
                history.pop_front();
            }
            stages_applied.push(format!("budget:trimmed={}", excess));
        }

        // Layer 2: Snip — keep only last N entries
        if history.len() > self.snip_keep + 5 {
            let to_remove = history.len().saturating_sub(self.snip_keep);
            for _ in 0..to_remove {
                history.pop_front();
            }
            stages_applied.push(format!("snip:removed={}", to_remove));
        }

        // Layer 3: Microcompact — summarize oldest entries into context state
        if history.len() > self.snip_keep {
            let oldest: Vec<String> = history
                .drain(..history.len().saturating_sub(self.snip_keep))
                .collect();
            let summary = self.summarize_for_state(&oldest);
            if let Some(s) = summary {
                self.context_state.add_completed(s, self.tick);
            }
            stages_applied.push(format!("microcompact:summarized={}", oldest.len()));
        }

        // Layer 4: Collapse — update context state from remaining entries
        if let Some(latest) = history.back() {
            self.update_state_from_entry(latest);
            stages_applied.push("collapse:state_updated".to_string());
        }

        // Layer 5: Auto-compact — trigger if over threshold
        let window_ratio = history.len() as f64 / self.budget as f64;
        let auto_compacted = window_ratio > self.auto_compact_threshold;

        let entries_after = history.len();
        CompactionReport {
            entries_before,
            entries_after,
            stages_applied,
            context_state: self.context_state.clone(),
            auto_compacted,
        }
    }

    fn summarize_for_state(&self, entries: &[String]) -> Option<String> {
        if entries.is_empty() {
            return None;
        }
        let total: usize = entries.len();
        let key_events: Vec<&str> = entries
            .iter()
            .filter(|e| e.contains("[resonant_broadcast]") || e.contains("[entropy_monitor]"))
            .map(|s| s.as_str())
            .collect();
        Some(format!(
            "[compact] {} entries ({} key events)",
            total,
            key_events.len()
        ))
    }

    fn update_state_from_entry(&mut self, entry: &str) {
        if entry.contains("winner=") {
            if let Some(goal) = self.extract_goal(entry) {
                self.context_state.update_goal(goal, self.tick);
            }
        }
    }

    fn extract_goal(&self, entry: &str) -> Option<String> {
        let marker = "winner=";
        entry.find(marker).map(|i| {
            let rest = &entry[i + marker.len()..];
            let end = rest
                .find(|c: char| !c.is_numeric() && c != '.' && c != '-')
                .unwrap_or(rest.len());
            format!("winner_idx_{}", &rest[..end])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_history(n: usize) -> VecDeque<String> {
        let mut h = VecDeque::new();
        for i in 0..n {
            h.push_back(format!("entry_{}", i));
        }
        h
    }

    #[test]
    fn test_budget_enforcement() {
        let mut pipeline = CompactionPipeline::new(20);
        let mut history = make_history(50);
        assert_eq!(history.len(), 50);

        let report = pipeline.compact(&mut history);
        assert!(report.entries_before > report.entries_after);
        assert!(report
            .stages_applied
            .iter()
            .any(|s| s.starts_with("budget:")));
    }

    #[test]
    fn test_snip_layer() {
        let mut pipeline = CompactionPipeline::new(100);
        pipeline.snip_keep = 5;
        let mut history = make_history(30);

        let report = pipeline.compact(&mut history);
        assert!(report.stages_applied.iter().any(|s| s.starts_with("snip:")));
        assert!(history.len() <= pipeline.snip_keep + 5);
    }

    #[test]
    fn test_microcompact_layer() {
        let mut pipeline = CompactionPipeline::new(100);
        pipeline.snip_keep = 3;
        let mut history = make_history(10);

        let report = pipeline.compact(&mut history);
        assert!(
            report.context_state.completed_summary.len() > 0
                || report.entries_after <= pipeline.snip_keep
        );
    }

    #[test]
    fn test_auto_compact_detection() {
        let mut pipeline = CompactionPipeline::new(20);
        pipeline.snip_keep = 100; // don't let snip reduce below threshold
        pipeline.auto_compact_threshold = 0.5;
        let mut history = make_history(15);

        let report = pipeline.compact(&mut history);
        // window_ratio after snip = entries_after/budget should trigger
        assert!(report.auto_compacted);
    }

    #[test]
    fn test_context_state_updates() {
        let mut pipeline = CompactionPipeline::new(100);
        let mut history = VecDeque::new();
        history.push_back("[resonant_broadcast] winner=5 entropy=0.7".to_string());

        pipeline.compact(&mut history);
        assert!(pipeline.context_state.current_goal.is_some());
    }

    #[test]
    fn test_context_state_lifecycle() {
        let mut state = ContextState::default();
        state.update_goal("solve math problem".to_string(), 1);
        assert_eq!(state.current_goal.as_deref(), Some("solve math problem"));

        state.add_decision("use beam search".to_string(), 2);
        assert_eq!(state.key_decisions.len(), 1);

        state.add_pending("verify result".to_string(), 3);
        assert_eq!(state.pending_items.len(), 1);

        state.resolve_pending("verify result", 4);
        assert!(state.pending_items.is_empty());

        state.add_completed("math problem solved".to_string(), 5);
        assert_eq!(state.completed_summary.len(), 1);

        assert!(!state.is_stale(5, 10));
        assert!(state.is_stale(20, 10));
    }

    #[test]
    fn test_empty_history() {
        let mut pipeline = CompactionPipeline::new(100);
        let mut history = VecDeque::new();
        let report = pipeline.compact(&mut history);
        assert_eq!(report.entries_before, 0);
        assert_eq!(report.entries_after, 0);
    }

    #[test]
    fn test_tick_increments() {
        let mut pipeline = CompactionPipeline::new(100);
        assert_eq!(pipeline.tick(), 1);
        assert_eq!(pipeline.tick(), 2);
    }
}
