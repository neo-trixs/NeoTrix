use super::pipeline::{BrainStage, StageDecision};
use super::SelfIteratingBrain;
use crate::core::nt_core_knowledge::TaskType;
use crate::neotrix::nt_core_error::NeoTrixError;
use std::collections::HashMap;

/// A benchmark task consisting of a prompt and an expected output pattern.
pub struct BenchmarkTask {
    pub name: String,
    pub prompt: String,
    pub task_type: TaskType,
}

impl BenchmarkTask {
    pub fn new(name: &str, prompt: &str, task_type: TaskType) -> Self {
        Self {
            name: name.to_string(),
            prompt: prompt.to_string(),
            task_type,
        }
    }
}

/// Configuration for the BenchmarkGate stage.
pub struct BenchmarkSuite {
    pub tasks: Vec<BenchmarkTask>,
    /// Minimum acceptable improvement delta (default: -0.05 = allow 5% regression).
    pub threshold: f64,
    /// Cached pre-edit scores.
    pub pre_scores: HashMap<String, f64>,
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self {
            tasks: vec![
                BenchmarkTask::new("reasoning", "Solve: if x + 5 = 12, what is x?", TaskType::General),
                BenchmarkTask::new("code_gen", "Write a Rust function that sums a Vec<i32>", TaskType::CodeGeneration),
                BenchmarkTask::new("tool_use", "Search for the capital of France and summarize", TaskType::Research),
                BenchmarkTask::new("creative", "Write a haiku about artificial intelligence", TaskType::UIDesign),
                BenchmarkTask::new("analysis", "Compare REST and GraphQL APIs", TaskType::CodeAnalysis),
            ],
            threshold: -0.05,
            pre_scores: HashMap::new(),
        }
    }
}

impl BenchmarkSuite {
    /// Run the suite on a brain, returning per-task scores.
    pub fn run(&self, brain: &mut SelfIteratingBrain) -> HashMap<String, f64> {
        let mut scores = HashMap::new();
        for task in &self.tasks {
            let score = brain.brain.evaluate_capability(task.task_type);
            scores.insert(task.name.clone(), score);
        }
        scores
    }

    /// Compute the aggregate score delta: mean(post - pre).
    /// All tasks must be present in both maps; missing tasks score -1.0.
    pub fn compute_delta(pre: &HashMap<String, f64>, post: &HashMap<String, f64>) -> f64 {
        let mut total = 0.0;
        let mut count = 0;
        for (name, pre_score) in pre {
            let post_score = post.get(name).copied().unwrap_or(-1.0);
            total += post_score - pre_score;
            count += 1;
        }
        if count > 0 { total / count as f64 } else { 0.0 }
    }

    /// Gate decision based on delta vs threshold.
    pub fn gate(&self, delta: f64) -> BenchmarkGateDecision {
        if delta >= self.threshold {
            BenchmarkGateDecision::Accept
        } else if delta >= self.threshold * 2.0 {
            BenchmarkGateDecision::Retry
        } else {
            BenchmarkGateDecision::Rollback
        }
    }
}

pub enum BenchmarkGateDecision {
    Accept,
    Retry,
    Rollback,
}

/// (A-Evolve-inspired) BenchmarkGate stage inserted after ValidationGate.
///
/// Runs a benchmark suite pre- and post-edit, compares scores, and
/// rolls back if performance regresses beyond the threshold.
pub struct BenchmarkGateStage {
    pub suite: BenchmarkSuite,
    /// Cached pre-edit benchmark scores, so the gate can compare the next run
    /// against a true baseline instead of measuring a zero delta on the same state.
    pub baseline: std::sync::Mutex<Option<HashMap<String, f64>>>,
}

impl Default for BenchmarkGateStage {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkGateStage {
    pub fn new() -> Self {
        Self {
            suite: BenchmarkSuite::default(),
            baseline: std::sync::Mutex::new(None),
        }
    }

    pub fn with_suite(suite: BenchmarkSuite) -> Self {
        Self {
            suite,
            baseline: std::sync::Mutex::new(None),
        }
    }
}

impl BrainStage for BenchmarkGateStage {
    fn name(&self) -> &str {
        "benchmark_gate"
    }

    fn frequency(&self) -> usize {
        3
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let post_scores = self.suite.run(brain);

        let mut baseline_guard = self.baseline.lock().unwrap();
        let delta = match baseline_guard.as_ref() {
            Some(pre_scores) => {
                let d = BenchmarkSuite::compute_delta(pre_scores, &post_scores);
                // Baseline is consumed each time: the next process() records a fresh
                // baseline, so deltas always measure a genuine edit-induced change.
                *baseline_guard = Some(post_scores);
                d
            }
            None => {
                // First run: record the baseline, defer the gate decision to next run.
                log::info!("[benchmark-gate] baseline recorded ({} tasks)", post_scores.len());
                *baseline_guard = Some(post_scores);
                return Ok(StageDecision::Continue);
            }
        };

        match self.suite.gate(delta) {
            BenchmarkGateDecision::Accept => {
                log::info!("[benchmark-gate] accepted: delta={:.4} >= threshold={:.4}", delta, self.suite.threshold);
                Ok(StageDecision::Continue)
            }
            BenchmarkGateDecision::Retry => {
                log::info!("[benchmark-gate] retry: delta={:.4} below threshold={:.4}, retrying", delta, self.suite.threshold);
                Ok(StageDecision::Skip("benchmark_gate retry needed".to_string()))
            }
            BenchmarkGateDecision::Rollback => {
                log::warn!("[benchmark-gate] rollback: delta={:.4} << threshold={:.4}", delta, self.suite.threshold);
                brain._snapshot_restore();
                Ok(StageDecision::Rollback("benchmark_gate regression detected".to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_gate_accepts_improvement() {
        let suite = BenchmarkSuite {
            threshold: -0.05,
            ..Default::default()
        };
        let mut pre = HashMap::new();
        pre.insert("a".to_string(), 0.5);
        pre.insert("b".to_string(), 0.6);
        let mut post = HashMap::new();
        post.insert("a".to_string(), 0.7);
        post.insert("b".to_string(), 0.8);

        let delta = BenchmarkSuite::compute_delta(&pre, &post);
        assert!((delta - 0.2).abs() < 0.01, "delta should be +0.2, got {delta}");
        assert!(matches!(suite.gate(delta), BenchmarkGateDecision::Accept));
    }

    #[test]
    fn test_benchmark_gate_rolls_back_regression() {
        let suite = BenchmarkSuite {
            threshold: -0.05,
            ..Default::default()
        };
        let mut pre = HashMap::new();
        pre.insert("a".to_string(), 0.9);
        let mut post = HashMap::new();
        post.insert("a".to_string(), 0.3);

        let delta = BenchmarkSuite::compute_delta(&pre, &post);
        assert!(delta < 0.0, "delta should be negative, got {delta}");
        assert!(matches!(suite.gate(delta), BenchmarkGateDecision::Rollback));
    }

    #[test]
    fn test_benchmark_gate_retries_on_slight_regression() {
        let suite = BenchmarkSuite {
            threshold: -0.05,
            ..Default::default()
        };
        let mut pre = HashMap::new();
        pre.insert("x".to_string(), 1.0);
        let mut post = HashMap::new();
        post.insert("x".to_string(), 0.92);

        let delta = BenchmarkSuite::compute_delta(&pre, &post);
        assert!((delta - (-0.08)).abs() < 0.01);
        assert!(matches!(suite.gate(delta), BenchmarkGateDecision::Retry));
    }

    #[test]
    fn test_benchmark_gate_accepts_zero_delta() {
        let suite = BenchmarkSuite::default();
        let mut pre = HashMap::new();
        pre.insert("test".to_string(), 0.5);
        let post = pre.clone();
        let delta = BenchmarkSuite::compute_delta(&pre, &post);
        assert!((delta - 0.0).abs() < 0.01);
        assert!(matches!(suite.gate(delta), BenchmarkGateDecision::Accept));
        let _ = suite.threshold; // suppress unused warning
    }

    #[test]
    fn test_benchmark_gate_empty_suite() {
        let pre: HashMap<String, f64> = HashMap::new();
        let post: HashMap<String, f64> = HashMap::new();
        let delta = BenchmarkSuite::compute_delta(&pre, &post);
        assert!((delta - 0.0).abs() < 0.01, "empty suite should give zero delta");
    }

    #[test]
    fn test_benchmark_suite_default_has_five_tasks() {
        let suite = BenchmarkSuite::default();
        assert_eq!(suite.tasks.len(), 5);
    }

    #[test]
    fn test_process_records_baseline_first_run_then_compares() {
        let stage = BenchmarkGateStage::new();
        let mut brain = SelfIteratingBrain::new();
        // 首次 process: 记录 baseline, 不 gate
        let first = stage.process(&mut brain).unwrap();
        assert!(matches!(first, StageDecision::Continue));
        assert!(stage.baseline.lock().unwrap().is_some());
        // 第二次 process: 使用 baseline 计算真实 delta (当前仍 Accept, 因为 delta≈0)
        let second = stage.process(&mut brain).unwrap();
        assert!(matches!(second, StageDecision::Continue));
    }

    #[test]
    fn test_gate_threshold_logic() {
        let suite = BenchmarkSuite { threshold: -0.05, ..Default::default() };
        assert!(matches!(suite.gate(0.1), BenchmarkGateDecision::Accept));
        assert!(matches!(suite.gate(-0.02), BenchmarkGateDecision::Accept));
        assert!(matches!(suite.gate(-0.08), BenchmarkGateDecision::Retry));
        assert!(matches!(suite.gate(-0.5), BenchmarkGateDecision::Rollback));
    }
}
