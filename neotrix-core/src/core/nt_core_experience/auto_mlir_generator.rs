use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    InProgress,
    GatePassed,
    GateFailed,
    Committed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationStep {
    pub id: u64,
    pub description: String,
    pub target_file: String,
    pub mutation_kind: String,
    pub status: StepStatus,
    pub cycle_created: u64,
    pub gate_score: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoMlirConfig {
    pub max_iterations_per_cycle: usize,
    pub min_gate_score: f64,
    pub source_dir: String,
    pub auto_commit: bool,
    pub max_history: usize,
}

impl Default for AutoMlirConfig {
    fn default() -> Self {
        Self {
            max_iterations_per_cycle: 3,
            min_gate_score: 0.5,
            source_dir: "src/".to_string(),
            auto_commit: false,
            max_history: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoMlirGenerator {
    pub config: AutoMlirConfig,
    pub steps: Vec<MutationStep>,
    pub next_id: u64,
    pub total_committed: u64,
    pub total_rolled_back: u64,
    pub consecutive_failures: u32,
}

impl AutoMlirGenerator {
    pub fn new() -> Self {
        Self {
            config: AutoMlirConfig::default(),
            steps: Vec::new(),
            next_id: 1,
            total_committed: 0,
            total_rolled_back: 0,
            consecutive_failures: 0,
        }
    }

    pub fn new_with_config(config: AutoMlirConfig) -> Self {
        Self {
            config,
            steps: Vec::new(),
            next_id: 1,
            total_committed: 0,
            total_rolled_back: 0,
            consecutive_failures: 0,
        }
    }

    pub fn propose_mutations(
        &mut self,
        cycle: u64,
        source_files: &[String],
        meta_accuracy: f64,
    ) -> Vec<MutationStep> {
        let mut created = Vec::new();

        let (kind, desc, target) = if meta_accuracy < 0.5 {
            (
                "AddFunction".to_string(),
                "AddFunction — add calibration helper".to_string(),
                source_files.first().cloned().unwrap_or_else(|| "src/calib.rs".to_string()),
            )
        } else {
            (
                "AddImport".to_string(),
                "AddImport — add std::collections::HashMap import".to_string(),
                source_files.first().cloned().unwrap_or_else(|| "src/main.rs".to_string()),
            )
        };

        let step = MutationStep {
            id: self.next_id,
            description: desc,
            target_file: target,
            mutation_kind: kind,
            status: StepStatus::Pending,
            cycle_created: cycle,
            gate_score: 0.0,
            error: None,
        };
        self.next_id += 1;
        self.steps.push(step.clone());
        created.push(step);

        while self.steps.len() > self.config.max_history {
            self.steps.remove(0);
        }

        created
    }

    pub fn run_gate(&mut self, step_id: u64, simulated_score: f64) -> bool {
        let step = match self.steps.iter_mut().find(|s| s.id == step_id) {
            Some(s) => s,
            None => return false,
        };
        step.status = StepStatus::InProgress;
        step.gate_score = simulated_score;
        if simulated_score >= self.config.min_gate_score {
            step.status = StepStatus::GatePassed;
            true
        } else {
            step.status = StepStatus::GateFailed;
            self.consecutive_failures += 1;
            false
        }
    }

    pub fn commit(&mut self, step_id: u64) -> bool {
        let step = match self.steps.iter_mut().find(|s| s.id == step_id) {
            Some(s) => s,
            None => return false,
        };
        step.status = StepStatus::Committed;
        self.total_committed += 1;
        self.consecutive_failures = 0;
        true
    }

    pub fn rollback(&mut self, step_id: u64) -> bool {
        let step = match self.steps.iter_mut().find(|s| s.id == step_id) {
            Some(s) => s,
            None => return false,
        };
        step.status = StepStatus::RolledBack;
        step.error = Some("gate score below threshold".to_string());
        self.total_rolled_back += 1;
        true
    }

    pub fn run_iteration(
        &mut self,
        cycle: u64,
        source_files: &[String],
        meta_accuracy: f64,
        simulated_gate_score: f64,
    ) -> Vec<String> {
        let mut events = Vec::new();

        if self.consecutive_failures >= 3 {
            events.push("too many consecutive failures".to_string());
            return events;
        }

        let proposals = self.propose_mutations(cycle, source_files, meta_accuracy);
        for step in &proposals {
            let passed = self.run_gate(step.id, simulated_gate_score);
            if passed {
                if self.config.auto_commit {
                    self.commit(step.id);
                    events.push(format!(
                        "cycle={} step={} committed (gate={})",
                        cycle, step.id, simulated_gate_score
                    ));
                } else {
                    events.push(format!(
                        "cycle={} step={} gate_passed (score={}) awaiting_commit",
                        cycle, step.id, simulated_gate_score
                    ));
                }
            } else {
                self.rollback(step.id);
                events.push(format!(
                    "cycle={} step={} rolled_back (gate={} < min={})",
                    cycle, step.id, simulated_gate_score, self.config.min_gate_score
                ));
            }
        }

        events
    }

    pub fn stats(&self) -> String {
        format!(
            "AutoMlirGenerator: steps={} committed={} rolled_back={} consec_fails={}",
            self.steps.len(),
            self.total_committed,
            self.total_rolled_back,
            self.consecutive_failures,
        )
    }

    pub fn summary(&self) -> String {
        let total = self.total_committed + self.total_rolled_back;
        let rate = if total > 0 {
            self.total_committed as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        format!(
            "AutoMlirGenerator: committed={} rolled={} rate={:.1}% consec_fails={}",
            self.total_committed,
            self.total_rolled_back,
            rate,
            self.consecutive_failures,
        )
    }

    pub fn clear_history(&mut self) {
        self.steps.clear();
    }
}

impl Default for AutoMlirGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let gen = AutoMlirGenerator::new();
        assert_eq!(gen.config.max_iterations_per_cycle, 3);
        assert_eq!(gen.config.min_gate_score, 0.5);
        assert_eq!(gen.config.source_dir, "src/");
        assert!(!gen.config.auto_commit);
        assert_eq!(gen.config.max_history, 100);
        assert_eq!(gen.next_id, 1);
        assert_eq!(gen.total_committed, 0);
        assert_eq!(gen.total_rolled_back, 0);
        assert_eq!(gen.consecutive_failures, 0);
        assert!(gen.steps.is_empty());
    }

    #[test]
    fn test_new_with_config() {
        let config = AutoMlirConfig {
            max_iterations_per_cycle: 5,
            min_gate_score: 0.7,
            source_dir: "lib/".to_string(),
            auto_commit: true,
            max_history: 200,
        };
        let gen = AutoMlirGenerator::new_with_config(config);
        assert_eq!(gen.config.max_iterations_per_cycle, 5);
        assert_eq!(gen.config.min_gate_score, 0.7);
        assert_eq!(gen.config.source_dir, "lib/");
        assert!(gen.config.auto_commit);
        assert_eq!(gen.config.max_history, 200);
    }

    #[test]
    fn test_propose_low_meta() {
        let mut gen = AutoMlirGenerator::new();
        let files = vec!["src/core.rs".to_string()];
        let steps = gen.propose_mutations(1, &files, 0.3);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].mutation_kind, "AddFunction");
        assert!(steps[0].description.contains("calibration"));
        assert_eq!(steps[0].cycle_created, 1);
        assert_eq!(gen.steps.len(), 1);
    }

    #[test]
    fn test_propose_high_meta() {
        let mut gen = AutoMlirGenerator::new();
        let files = vec!["src/main.rs".to_string()];
        let steps = gen.propose_mutations(5, &files, 0.7);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].mutation_kind, "AddImport");
        assert!(steps[0].description.contains("HashMap"));
        assert_eq!(steps[0].cycle_created, 5);
    }

    #[test]
    fn test_run_gate_passed() {
        let mut gen = AutoMlirGenerator::new();
        let files = vec!["src/core.rs".to_string()];
        let steps = gen.propose_mutations(1, &files, 0.3);
        let result = gen.run_gate(steps[0].id, 0.8);
        assert!(result);
        let step = gen.steps.iter().find(|s| s.id == steps[0].id).unwrap();
        assert_eq!(step.status, StepStatus::GatePassed);
        assert_eq!(step.gate_score, 0.8);
    }

    #[test]
    fn test_run_gate_failed() {
        let mut gen = AutoMlirGenerator::new();
        let files = vec!["src/core.rs".to_string()];
        let steps = gen.propose_mutations(1, &files, 0.3);
        let result = gen.run_gate(steps[0].id, 0.2);
        assert!(!result);
        let step = gen.steps.iter().find(|s| s.id == steps[0].id).unwrap();
        assert_eq!(step.status, StepStatus::GateFailed);
        assert_eq!(step.gate_score, 0.2);
        assert_eq!(gen.consecutive_failures, 1);
    }

    #[test]
    fn test_commit() {
        let mut gen = AutoMlirGenerator::new();
        let files = vec!["src/core.rs".to_string()];
        let steps = gen.propose_mutations(1, &files, 0.3);
        gen.run_gate(steps[0].id, 0.8);
        let result = gen.commit(steps[0].id);
        assert!(result);
        let step = gen.steps.iter().find(|s| s.id == steps[0].id).unwrap();
        assert_eq!(step.status, StepStatus::Committed);
        assert_eq!(gen.total_committed, 1);
        assert_eq!(gen.consecutive_failures, 0);
    }

    #[test]
    fn test_rollback() {
        let mut gen = AutoMlirGenerator::new();
        let files = vec!["src/core.rs".to_string()];
        let steps = gen.propose_mutations(1, &files, 0.3);
        let result = gen.rollback(steps[0].id);
        assert!(result);
        let step = gen.steps.iter().find(|s| s.id == steps[0].id).unwrap();
        assert_eq!(step.status, StepStatus::RolledBack);
        assert_eq!(gen.total_rolled_back, 1);
        assert!(step.error.is_some());
    }

    #[test]
    fn test_run_iteration() {
        let mut gen = AutoMlirGenerator::new();
        gen.config.auto_commit = true;
        let files = vec!["src/core.rs".to_string()];
        let events = gen.run_iteration(10, &files, 0.3, 0.8);
        assert!(!events.is_empty());
        assert!(events[0].contains("committed"));
        assert_eq!(gen.total_committed, 1);
    }

    #[test]
    fn test_run_iteration_low_score() {
        let mut gen = AutoMlirGenerator::new();
        let files = vec!["src/core.rs".to_string()];
        let events = gen.run_iteration(10, &files, 0.3, 0.2);
        assert!(!events.is_empty());
        assert!(events[0].contains("rolled_back"));
        assert_eq!(gen.total_rolled_back, 1);
    }

    #[test]
    fn test_consecutive_failures_block() {
        let mut gen = AutoMlirGenerator::new();
        gen.consecutive_failures = 3;
        let files = vec!["src/core.rs".to_string()];
        let events = gen.run_iteration(10, &files, 0.3, 0.8);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "too many consecutive failures");
        assert_eq!(gen.total_committed, 0);
    }

    #[test]
    fn test_summary() {
        let mut gen = AutoMlirGenerator::new();
        gen.total_committed = 5;
        gen.total_rolled_back = 3;
        let s = gen.summary();
        assert!(s.contains("committed=5"));
        assert!(s.contains("rolled=3"));
        assert!(s.contains("62.5%"));
        assert!(s.contains("consec_fails=0"));
    }
}
