use std::collections::VecDeque;
use crate::memory::hypercube::HyperCube;

const MAX_BACKTRACK_DEPTH: usize = 16;
const RECENT_WINDOW: usize = 5;
const BACKTRACK_CONFIDENCE_THRESHOLD: f64 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureType {
    HypothesisFailure,
    ExecutionFailure,
    BudgetExhaustion,
    ContextOverflow,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct BacktrackEvent {
    pub trigger_step: u64,
    pub failure_type: FailureType,
    pub confidence_before: f64,
    pub confidence_after: f64,
    pub rollback_depth: usize,
    pub hypothesis_vsa: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct BacktrackDetector {
    trace: VecDeque<(Vec<f64>, f64, u64)>,
    events: Vec<BacktrackEvent>,
    step_counter: u64,
    stuck_threshold: usize,
    stuck_count: usize,
}

impl BacktrackDetector {
    pub fn new(stuck_threshold: usize) -> Self {
        Self {
            trace: VecDeque::with_capacity(MAX_BACKTRACK_DEPTH),
            events: Vec::new(),
            step_counter: 0,
            stuck_threshold,
            stuck_count: 0,
        }
    }

    pub fn record_step(&mut self, thought: Vec<f64>, confidence: f64) {
        self.step_counter += 1;
        if self.trace.len() >= MAX_BACKTRACK_DEPTH {
            self.trace.pop_front();
        }
        self.trace.push_back((thought, confidence, self.step_counter));
    }

    pub fn detect_dead_end(&mut self) -> Option<FailureType> {
        if self.trace.len() < 3 {
            return None;
        }

        let entries: Vec<&(Vec<f64>, f64, u64)> = self.trace.iter().rev().take(RECENT_WINDOW).collect();
        if entries.len() < 3 {
            return None;
        }

        let avg_confidence: f64 = entries.iter().map(|e| e.1).sum::<f64>() / entries.len() as f64;
        if avg_confidence < BACKTRACK_CONFIDENCE_THRESHOLD {
            self.stuck_count += 1;
        } else {
            self.stuck_count = 0;
            return None;
        }

        if self.stuck_count < self.stuck_threshold {
            return None;
        }

        let first = &entries[0].0;
        let last = entries.last().map(|e| &e.0).unwrap_or(first);
        let similarity = HyperCube::similarity(first, last);

        if similarity > 0.85 {
            self.stuck_count = 0;
            return Some(FailureType::HypothesisFailure);
        }

        if avg_confidence < 0.15 && entries.len() >= 4 {
            let mid = entries.len() / 2;
            let mid_sim = HyperCube::similarity(&entries[0].0, &entries[mid].0);
            let end_sim = HyperCube::similarity(&entries[0].0, &entries[entries.len() - 1].0);
            if mid_sim < 0.5 && end_sim < 0.3 {
                self.stuck_count = 0;
                return Some(FailureType::ExecutionFailure);
            }
        }

        if self.stuck_count >= self.stuck_threshold * 2 {
            self.stuck_count = 0;
            return Some(FailureType::BudgetExhaustion);
        }

        None
    }

    pub fn needs_backtrack(&mut self) -> Option<(FailureType, usize)> {
        let failure = self.detect_dead_end()?;
        let rollback_depth = self.compute_rollback_depth(&failure);

        if let Some(last) = self.trace.back() {
            self.events.push(BacktrackEvent {
                trigger_step: self.step_counter,
                failure_type: failure,
                confidence_before: last.1,
                confidence_after: 0.0,
                rollback_depth,
                hypothesis_vsa: last.0.clone(),
            });
        }

        Some((failure, rollback_depth))
    }

    fn compute_rollback_depth(&self, failure: &FailureType) -> usize {
        match failure {
            FailureType::HypothesisFailure => {
                let mut best = 0;
                let entries: Vec<&(Vec<f64>, f64, u64)> = self.trace.iter().rev().take(MAX_BACKTRACK_DEPTH).collect();
                for (i, entry) in entries.iter().enumerate() {
                    if entry.1 > 0.6 {
                        best = i;
                        break;
                    }
                }
                best.min(self.trace.len() / 2)
            }
            FailureType::ExecutionFailure => 1.max(self.trace.len() / 4),
            FailureType::BudgetExhaustion => 0,
            FailureType::ContextOverflow => self.trace.len() / 3,
            FailureType::Unknown => 1,
        }
    }

    pub fn apply_backtrack(&mut self, depth: usize) {
        for _ in 0..depth {
            self.trace.pop_back();
        }
        self.stuck_count = 0;
    }

    pub fn events(&self) -> &[BacktrackEvent] {
        &self.events
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn summary(&self) -> String {
        let hyp = self.events.iter().filter(|e| e.failure_type == FailureType::HypothesisFailure).count();
        let exec = self.events.iter().filter(|e| e.failure_type == FailureType::ExecutionFailure).count();
        let budget = self.events.iter().filter(|e| e.failure_type == FailureType::BudgetExhaustion).count();
        format!("Backtrack[events={} hyp={} exec={} budget={} stuck={}]",
            self.events.len(), hyp, exec, budget, self.stuck_count)
    }
}

impl Default for BacktrackDetector {
    fn default() -> Self {
        Self::new(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_detector_no_dead_end() {
        let mut bd = BacktrackDetector::new(3);
        assert!(bd.detect_dead_end().is_none());
    }

    #[test]
    fn test_high_confidence_no_dead_end() {
        let mut bd = BacktrackDetector::new(3);
        for _ in 0..6 {
            bd.record_step(vec![1.0; 8], 0.8);
        }
        assert!(bd.detect_dead_end().is_none());
    }

    #[test]
    fn test_low_confidence_triggers_hypothesis() {
        let mut bd = BacktrackDetector::new(3);
        for _ in 0..10 {
            bd.record_step(vec![1.0; 8], 0.1);
        }
        let result = bd.detect_dead_end();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), FailureType::HypothesisFailure);
    }

    #[test]
    fn test_backtrack_clears_stuck() {
        let mut bd = BacktrackDetector::new(3);
        for _ in 0..10 {
            bd.record_step(vec![1.0; 8], 0.1);
        }
        let depth = 3;
        bd.apply_backtrack(depth);
        assert_eq!(bd.stuck_count, 0);
    }

    #[test]
    fn test_needs_backtrack_returns_depth() {
        let mut bd = BacktrackDetector::new(2);
        for _ in 0..8 {
            bd.record_step(vec![1.0; 8], 0.1);
        }
        let result = bd.needs_backtrack();
        assert!(result.is_some());
        let (failure, depth) = result.unwrap();
        assert_eq!(failure, FailureType::HypothesisFailure);
        assert!(depth <= MAX_BACKTRACK_DEPTH);
    }

    #[test]
    fn test_event_logging() {
        let mut bd = BacktrackDetector::new(2);
        for _ in 0..8 {
            bd.record_step(vec![1.0; 8], 0.1);
        }
        bd.needs_backtrack();
        assert_eq!(bd.event_count(), 1);
    }

    #[test]
    fn test_summary_format() {
        let mut bd = BacktrackDetector::new(2);
        for _ in 0..8 {
            bd.record_step(vec![0.5; 8], 0.1);
        }
        bd.needs_backtrack();
        let s = bd.summary();
        assert!(s.starts_with("Backtrack["));
        assert!(s.contains("events="));
    }

    #[test]
    fn test_dynamic_similarity_detects_execution_failure() {
        let mut bd = BacktrackDetector::new(2);
        let a = vec![0.1, 0.2, 0.3, 0.4];
        let b = vec![0.9, 0.8, 0.7, 0.6];
        let c = vec![0.9, 0.81, 0.71, 0.61];
        bd.record_step(a, 0.2);
        bd.record_step(b, 0.3);
        bd.record_step(c, 0.25);
        for _ in 0..5 {
            bd.record_step(vec![0.9, 0.8, 0.7, 0.6], 0.2);
        }
        let result = bd.needs_backtrack();
        assert!(result.is_some());
    }
}
