use serde::{Deserialize, Serialize};

fn now_secs() -> i64 {
    crate::core::nt_core_time::unix_now_secs() as i64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ResearchStatus {
    Draft,
    InProgress,
    Completed,
    Failed,
    Superseded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchHypothesis {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: ResearchStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub budget_cycles: u64,
    pub cycles_used: u64,
    pub score_before: Option<f64>,
    pub score_after: Option<f64>,
    pub conclusion: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub handler_target: String,
    pub mutation_description: String,
    pub expected_outcome: String,
    pub max_cycles: u64,
    pub measurement_fn: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub hypothesis_id: String,
    pub config: ExperimentConfig,
    pub passed: bool,
    pub score_delta: f64,
    pub cycles_taken: u64,
    pub error_log: Vec<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchLedger {
    hypotheses: Vec<ResearchHypothesis>,
    results: Vec<ExperimentResult>,
    max_entries: usize,
}

impl ResearchLedger {
    pub fn new() -> Self {
        Self {
            hypotheses: Vec::new(),
            results: Vec::new(),
            max_entries: 200,
        }
    }

    pub fn add_hypothesis(&mut self, hypothesis: ResearchHypothesis) {
        self.hypotheses.push(hypothesis);
        if self.hypotheses.len() > self.max_entries {
            self.hypotheses.remove(0);
        }
    }

    pub fn record_result(&mut self, result: ExperimentResult) {
        self.results.push(result);
        if self.results.len() > self.max_entries {
            self.results.remove(0);
        }
    }

    pub fn get_hypothesis(&self, id: &str) -> Option<&ResearchHypothesis> {
        self.hypotheses.iter().find(|h| h.id == id)
    }

    pub fn get_hypothesis_mut(&mut self, id: &str) -> Option<&mut ResearchHypothesis> {
        self.hypotheses.iter_mut().find(|h| h.id == id)
    }

    pub fn all_hypotheses(&self) -> &[ResearchHypothesis] {
        &self.hypotheses
    }

    pub fn results_for(&self, hypothesis_id: &str) -> Vec<&ExperimentResult> {
        self.results
            .iter()
            .filter(|r| r.hypothesis_id == hypothesis_id)
            .collect()
    }

    pub fn success_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let passed = self.results.iter().filter(|r| r.passed).count();
        passed as f64 / self.results.len() as f64
    }

    pub fn avg_score_delta(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        self.results.iter().map(|r| r.score_delta).sum::<f64>() / self.results.len() as f64
    }

    pub fn recent_report(&self, n: usize) -> String {
        let n = n.min(self.results.len());
        let recent: Vec<&ExperimentResult> = self.results.iter().rev().take(n).collect();
        if recent.is_empty() {
            return "research:no_results".to_string();
        }
        let passed = recent.iter().filter(|r| r.passed).count();
        let avg_delta = recent.iter().map(|r| r.score_delta).sum::<f64>() / recent.len() as f64;
        format!(
            "research:{}_recent|{}_passed|{:.4}_avg_delta|{}_total",
            n,
            passed,
            avg_delta,
            self.results.len()
        )
    }
}

pub struct ResearchPlanner {
    next_id: u64,
    pending: Vec<ResearchHypothesis>,
}

impl ResearchPlanner {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            pending: Vec::new(),
        }
    }

    pub fn propose(
        &mut self,
        title: &str,
        desc: &str,
        budget: u64,
        tags: Vec<String>,
    ) -> ResearchHypothesis {
        let id = format!("research_{}", self.next_id);
        self.next_id += 1;
        ResearchHypothesis {
            id,
            title: title.to_string(),
            description: desc.to_string(),
            status: ResearchStatus::Draft,
            created_at: now_secs(),
            updated_at: now_secs(),
            budget_cycles: budget,
            cycles_used: 0,
            score_before: None,
            score_after: None,
            conclusion: None,
            tags,
        }
    }

    pub fn enqueue(&mut self, hypothesis: ResearchHypothesis) {
        self.pending.push(hypothesis);
    }

    pub fn dequeue(&mut self) -> Option<ResearchHypothesis> {
        self.pending.pop()
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

pub struct AutoResearchEngine {
    pub ledger: ResearchLedger,
    pub planner: ResearchPlanner,
    pub active_hypothesis: Option<ResearchHypothesis>,
    pub active_config: Option<ExperimentConfig>,
    pub active_cycles: u64,
    pub enabled: bool,
}

impl AutoResearchEngine {
    pub fn new() -> Self {
        Self {
            ledger: ResearchLedger::new(),
            planner: ResearchPlanner::new(),
            active_hypothesis: None,
            active_config: None,
            active_cycles: 0,
            enabled: true,
        }
    }

    pub fn propose_research(
        &mut self,
        title: &str,
        desc: &str,
        budget: u64,
        tags: Vec<String>,
    ) -> String {
        let h = self.planner.propose(title, desc, budget, tags);
        let id = h.id.clone();
        self.planner.enqueue(h);
        format!("research:proposed|{}", id)
    }

    pub fn start_next(&mut self, current_score: f64) -> Option<String> {
        let hypothesis = self.planner.dequeue()?;
        let id = hypothesis.id.clone();
        self.active_hypothesis = Some(ResearchHypothesis {
            score_before: Some(current_score),
            ..hypothesis
        });
        self.active_cycles = 0;
        Some(id)
    }

    pub fn tick(&mut self, current_score: f64) -> String {
        if !self.enabled {
            return "research:disabled".to_string();
        }
        if self.active_hypothesis.is_none() {
            if self.planner.pending_count() > 0 {
                self.start_next(current_score);
                return "research:started_next".to_string();
            }
            return self.ledger.recent_report(5);
        }

        self.active_cycles += 1;
        let Some(hypothesis) = self.active_hypothesis.as_mut() else {
            log::warn!("research: tick called without active hypothesis");
            return "research:no_active".to_string();
        };
        hypothesis.cycles_used = self.active_cycles;

        if self.active_cycles >= hypothesis.budget_cycles {
            let before = hypothesis.score_before.unwrap_or(current_score);
            let delta = current_score - before;
            let passed = delta > 0.01;
            let id = hypothesis.id.clone();
            hypothesis.status = if passed {
                ResearchStatus::Completed
            } else {
                ResearchStatus::Failed
            };
            hypothesis.score_after = Some(current_score);
            hypothesis.updated_at = now_secs();
            hypothesis.conclusion = Some(if passed {
                format!("improved by {:.4}", delta)
            } else {
                format!("no improvement (delta={:.4})", delta)
            });

            let result = ExperimentResult {
                hypothesis_id: id.clone(),
                config: ExperimentConfig {
                    handler_target: hypothesis.tags.first().cloned().unwrap_or_default(),
                    mutation_description: hypothesis.description.clone(),
                    expected_outcome: hypothesis.title.clone(),
                    max_cycles: hypothesis.budget_cycles,
                    measurement_fn: "composite_score".to_string(),
                },
                passed,
                score_delta: delta,
                cycles_taken: self.active_cycles,
                error_log: Vec::new(),
                timestamp: now_secs(),
            };
            self.ledger.record_result(result);
            self.ledger.add_hypothesis(hypothesis.clone());
            self.active_hypothesis = None;
            self.active_cycles = 0;

            format!(
                "research:completed|{}|passed={}|delta={:.4}",
                id, passed, delta
            )
        } else {
            format!(
                "research:running|{}|cycle={}/{}",
                hypothesis.id, self.active_cycles, hypothesis.budget_cycles
            )
        }
    }

    pub fn stats(&self) -> String {
        format!(
            "research:{}_pending|{}_completed|{}_total|{:.2}_succ_rate|{:.4}_avg_delta",
            self.planner.pending_count(),
            self.ledger
                .all_hypotheses()
                .iter()
                .filter(|h| matches!(h.status, ResearchStatus::Completed))
                .count(),
            self.ledger.all_hypotheses().len(),
            self.ledger.success_rate(),
            self.ledger.avg_score_delta(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propose_and_tick() {
        let mut engine = AutoResearchEngine::new();
        let r = engine.propose_research("test", "test hypothesis", 10, vec!["test".into()]);
        assert!(r.contains("research:proposed"));

        let r1 = engine.tick(0.5);
        assert!(r1.contains("research:started_next") || r1.contains("research:no_results"));

        for i in 0..15 {
            let r = engine.tick(0.5 + (i as f64 * 0.02));
            if r.contains("research:completed") {
                return;
            }
        }
        panic!("research should have completed");
    }

    #[test]
    fn test_ledger_success_rate() {
        let mut ledger = ResearchLedger::new();
        assert_eq!(ledger.success_rate(), 0.0);

        for i in 0..5 {
            ledger.record_result(ExperimentResult {
                hypothesis_id: format!("h{}", i),
                config: ExperimentConfig {
                    handler_target: "test".into(),
                    mutation_description: "test".into(),
                    expected_outcome: "test".into(),
                    max_cycles: 10,
                    measurement_fn: "test".into(),
                },
                passed: i < 3,
                score_delta: if i < 3 { 0.1 } else { -0.05 },
                cycles_taken: 10,
                error_log: Vec::new(),
                timestamp: now_secs(),
            });
        }
        assert!((ledger.success_rate() - 0.6).abs() < 0.001);
        assert!(ledger.avg_score_delta() > 0.0);
    }

    #[test]
    fn test_planner_roundtrip() {
        let mut planner = ResearchPlanner::new();
        let h = planner.propose("test", "desc", 10, vec![]);
        planner.enqueue(h);
        assert_eq!(planner.pending_count(), 1);
        let popped = planner.dequeue().unwrap();
        assert_eq!(popped.title, "test");
        assert_eq!(planner.pending_count(), 0);
    }

    #[test]
    fn test_stats_format() {
        let engine = AutoResearchEngine::new();
        let s = engine.stats();
        assert!(s.contains("research:"));
        assert!(s.contains("succ_rate"));
    }
}
