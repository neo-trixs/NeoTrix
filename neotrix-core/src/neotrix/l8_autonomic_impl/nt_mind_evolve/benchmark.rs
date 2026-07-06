use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    CodeGeneration,
    Reasoning,
    KnowledgeQA,
    ToolUse,
    Safety,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BenchmarkCategory {
    Reasoning,
    Memory,
    ToolUse,
    CodeGeneration,
    Security,
    Social,
    Metacognitive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoringFn {
    Binary,
    StepWeighted { weights: Vec<f64> },
    PrecisionAtK { k: usize },
    RecallAtK { k: usize },
    CompileAndTest,
    RejectionAccuracy,
    Custom(String),
}

impl ScoringFn {
    pub fn score(&self, correct: bool, partial: Option<f64>) -> f64 {
        match self {
            ScoringFn::Binary => {
                if correct { 1.0 } else { 0.0 }
            }
            ScoringFn::StepWeighted { weights } => {
                let total: f64 = weights.iter().sum();
                if total == 0.0 {
                    return 0.0;
                }
                let correct_steps = weights.iter().filter(|&&w| w > 0.0).count() as f64;
                (correct_steps / weights.len() as f64).max(0.0).min(1.0)
            }
            ScoringFn::PrecisionAtK { .. } => partial.unwrap_or(0.0),
            ScoringFn::RecallAtK { .. } => partial.unwrap_or(0.0),
            ScoringFn::CompileAndTest => {
                if correct { 1.0 } else { 0.0 }
            }
            ScoringFn::RejectionAccuracy => {
                if correct { 1.0 } else { 0.0 }
            }
            ScoringFn::Custom(_) => partial.unwrap_or(if correct { 1.0 } else { 0.0 }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTask {
    pub id: String,
    pub description: String,
    pub task_type: TaskType,
    pub expected_difficulty: f64,
    pub timeout_s: u64,
    pub scoring_fn: ScoringFn,
    pub category: BenchmarkCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub task_id: String,
    pub score: f64,
    pub latency_ms: u64,
    pub tokens_used: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    pub name: String,
    pub version: String,
    pub tasks: Vec<BenchmarkTask>,
}

impl BenchmarkSuite {
    pub fn new(name: impl Into<String>, version: impl Into<String>, tasks: Vec<BenchmarkTask>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            tasks,
        }
    }

    pub fn pass_rate(&self, results: &[BenchmarkResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        let sum: f64 = results.iter().map(|r| r.score).sum();
        sum / results.len() as f64
    }

    pub fn get_task(&self, id: &str) -> Option<&BenchmarkTask> {
        self.tasks.iter().find(|t| t.id == id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MutationScope {
    Comprehensive,
    Targeted,
    Minimal,
}

impl MutationScope {
    pub fn from_pass_rate(rate: f64) -> Self {
        if rate < 0.3 {
            MutationScope::Comprehensive
        } else if rate <= 0.7 {
            MutationScope::Targeted
        } else {
            MutationScope::Minimal
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvolutionStrategy {
    AdaptiveEvolve,
    GuidedSynthesis,
    SkillForge,
    Recombination,
    ParameterSearch,
}

impl EvolutionStrategy {
    pub fn select(scope: MutationScope, pass_rate: f64, cycle_count: u64) -> Self {
        if cycle_count > 0 && cycle_count.is_multiple_of(5) {
            return EvolutionStrategy::Recombination;
        }
        if pass_rate < 0.3 {
            return EvolutionStrategy::AdaptiveEvolve;
        }
        match scope {
            MutationScope::Comprehensive => EvolutionStrategy::SkillForge,
            MutationScope::Targeted => EvolutionStrategy::GuidedSynthesis,
            MutationScope::Minimal => EvolutionStrategy::ParameterSearch,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BenchmarkGate {
    pub required_pass_rate: f64,
    pub max_regression: f64,
}

impl Default for BenchmarkGate {
    fn default() -> Self {
        Self {
            required_pass_rate: 0.7,
            max_regression: -0.05,
        }
    }
}

impl BenchmarkGate {
    pub fn is_accepted(&self, pass_rate: f64, previous_pass_rate: f64) -> bool {
        if pass_rate < self.required_pass_rate {
            return false;
        }
        let delta = pass_rate - previous_pass_rate;
        delta >= self.max_regression
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_variants() {
        let variants = [
            TaskType::CodeGeneration,
            TaskType::Reasoning,
            TaskType::KnowledgeQA,
            TaskType::ToolUse,
            TaskType::Safety,
        ];
        assert_eq!(variants.len(), 5);
    }

    #[test]
    fn test_scoring_fn_binary_correct() {
        let fn_bin = ScoringFn::Binary;
        assert_eq!(fn_bin.score(true, None), 1.0);
        assert_eq!(fn_bin.score(false, None), 0.0);
    }

    #[test]
    fn test_scoring_fn_step_weighted() {
        let fn_step = ScoringFn::StepWeighted {
            weights: vec![1.0, 0.0, 1.0, 0.0],
        };
        let score = fn_step.score(true, None);
        assert!((score - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_scoring_fn_step_weighted_all_zero() {
        let fn_step = ScoringFn::StepWeighted {
            weights: vec![0.0, 0.0],
        };
        assert_eq!(fn_step.score(true, None), 0.0);
    }

    #[test]
    fn test_benchmark_suite_pass_rate() {
        let suite = BenchmarkSuite::new("test", "1.0", vec![]);
        let results = vec![
            BenchmarkResult {
                task_id: "t1".into(),
                score: 0.8,
                latency_ms: 100,
                tokens_used: 50,
                error: None,
            },
            BenchmarkResult {
                task_id: "t2".into(),
                score: 0.6,
                latency_ms: 200,
                tokens_used: 100,
                error: None,
            },
        ];
        let pr = suite.pass_rate(&results);
        assert!((pr - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_benchmark_suite_pass_rate_empty() {
        let suite = BenchmarkSuite::new("test", "1.0", vec![]);
        assert_eq!(suite.pass_rate(&[]), 0.0);
    }

    #[test]
    fn test_benchmark_suite_get_task() {
        let task = BenchmarkTask {
            id: "task-1".into(),
            description: "test".into(),
            task_type: TaskType::Reasoning,
            expected_difficulty: 0.5,
            timeout_s: 30,
            scoring_fn: ScoringFn::Binary,
            category: BenchmarkCategory::Reasoning,
        };
        let suite = BenchmarkSuite::new("test", "1.0", vec![task]);
        assert!(suite.get_task("task-1").is_some());
        assert!(suite.get_task("nonexistent").is_none());
    }

    #[test]
    fn test_mutation_scope_from_pass_rate() {
        assert_eq!(MutationScope::from_pass_rate(0.0), MutationScope::Comprehensive);
        assert_eq!(MutationScope::from_pass_rate(0.29), MutationScope::Comprehensive);
        assert_eq!(MutationScope::from_pass_rate(0.3), MutationScope::Targeted);
        assert_eq!(MutationScope::from_pass_rate(0.5), MutationScope::Targeted);
        assert_eq!(MutationScope::from_pass_rate(0.7), MutationScope::Targeted);
        assert_eq!(MutationScope::from_pass_rate(0.71), MutationScope::Minimal);
        assert_eq!(MutationScope::from_pass_rate(1.0), MutationScope::Minimal);
    }

    #[test]
    fn test_evolution_strategy_recombination_every_5() {
        let scope_all = [MutationScope::Comprehensive, MutationScope::Targeted, MutationScope::Minimal];
        for &scope in &scope_all {
            let strat = EvolutionStrategy::select(scope, 0.5, 5);
            assert_eq!(strat, EvolutionStrategy::Recombination);
        }
        let strat = EvolutionStrategy::select(MutationScope::Targeted, 0.5, 10);
        assert_eq!(strat, EvolutionStrategy::Recombination);
    }

    #[test]
    fn test_evolution_strategy_adaptive_at_low_pass_rate() {
        let strat = EvolutionStrategy::select(MutationScope::Comprehensive, 0.29, 1);
        assert_eq!(strat, EvolutionStrategy::AdaptiveEvolve);
        let strat = EvolutionStrategy::select(MutationScope::Targeted, 0.2, 2);
        assert_eq!(strat, EvolutionStrategy::AdaptiveEvolve);
    }

    #[test]
    fn test_evolution_strategy_by_scope() {
        let s1 = EvolutionStrategy::select(MutationScope::Comprehensive, 0.5, 1);
        assert_eq!(s1, EvolutionStrategy::SkillForge);

        let s2 = EvolutionStrategy::select(MutationScope::Targeted, 0.5, 1);
        assert_eq!(s2, EvolutionStrategy::GuidedSynthesis);

        let s3 = EvolutionStrategy::select(MutationScope::Minimal, 0.8, 1);
        assert_eq!(s3, EvolutionStrategy::ParameterSearch);
    }

    #[test]
    fn test_benchmark_gate_default() {
        let gate = BenchmarkGate::default();
        assert!((gate.required_pass_rate - 0.7).abs() < 1e-6);
        assert!((gate.max_regression - (-0.05)).abs() < 1e-6);
    }

    #[test]
    fn test_benchmark_gate_is_accepted() {
        let gate = BenchmarkGate::default();
        assert!(gate.is_accepted(0.8, 0.7));
        assert!(!gate.is_accepted(0.6, 0.7));
        assert!(gate.is_accepted(0.72, 0.7));
        assert!(!gate.is_accepted(0.7, 0.8));
    }
}
