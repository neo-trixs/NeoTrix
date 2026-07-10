#![forbid(unsafe_code)]

pub mod curriculum;
pub mod grpo;
pub mod self_edit_gen;

use serde::{Deserialize, Serialize};

pub use self::curriculum::{
    CalibratedCurriculumGenerator, CurriculumRecord, IterationValidator, LearnabilityWindowAnalyzer,
    ValidationResult,
};
pub use self::grpo::{GRPOLoop, GrpoConfig, GrpoReport};
pub use self::self_edit_gen::{EditType, SelfEdit, SelfEditGen};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealIterationReport {
    pub iteration: u32,
    pub task: String,
    pub edits_generated: usize,
    pub best_reward: f64,
    pub avg_reward: f64,
    pub policy_improvement: f64,
    pub curriculum_difficulty: f64,
    pub convergence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumTask {
    pub description: String,
    pub difficulty: f64,
    pub prerequisites: Vec<String>,
}

pub struct SealPipeline {
    pub generator: SelfEditGen,
    pub evaluator: GRPOLoop,
    pub curriculum: CalibratedCurriculumGenerator,
    pub validator: IterationValidator,
    pub analyzer: LearnabilityWindowAnalyzer,
    iteration_count: u32,
}

impl SealPipeline {
    pub fn new(vocab_size: usize, hidden_dim: usize) -> Self {
        Self {
            generator: SelfEditGen::new(vocab_size, hidden_dim, 0.8),
            evaluator: GRPOLoop::new(GrpoConfig::default(), vocab_size * hidden_dim),
            curriculum: CalibratedCurriculumGenerator::new(0.7, 10),
            validator: IterationValidator::new(0.3, 3, 5),
            analyzer: LearnabilityWindowAnalyzer::new(10),
            iteration_count: 0,
        }
    }

    pub fn run_iteration(&mut self, task: &str, context: &[&str]) -> SealIterationReport {
        self.iteration_count += 1;
        let code_context = context.join("\n");
        let edits = self.generator.generate_edits(&code_context, task);
        let rewards: Vec<f64> = edits.iter().map(|e| self.evaluator.evaluate(e)).collect();
        let best_reward = rewards.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg_reward = if rewards.is_empty() {
            0.0
        } else {
            rewards.iter().sum::<f64>() / rewards.len() as f64
        };

        let policy_improvement = if self.iteration_count > 1 {
            (avg_reward - 0.5).max(0.0)
        } else {
            0.0
        };

        self.curriculum.record_outcome(CurriculumRecord {
            task_id: task.to_string(),
            difficulty: self.curriculum.difficulty_level,
            success: best_reward > 0.5,
            reward: best_reward,
            iterations: self.iteration_count,
        });
        self.curriculum.adjust_difficulty();

        self.analyzer.add_performance(avg_reward);
        let validation = self.validator.validate(avg_reward, &rewards);

        let convergence = if validation.is_valid && validation.reason == "converged" {
            1.0
        } else {
            self.analyzer.variance()
        };

        SealIterationReport {
            iteration: self.iteration_count,
            task: task.to_string(),
            edits_generated: edits.len(),
            best_reward,
            avg_reward,
            policy_improvement,
            curriculum_difficulty: self.curriculum.difficulty_level,
            convergence,
        }
    }

    pub fn run_curriculum(&mut self, tasks: &[CurriculumTask]) -> Vec<SealIterationReport> {
        let mut reports = Vec::new();
        for task in tasks {
            let context: Vec<&str> = task.prerequisites.iter().map(|s| s.as_str()).collect();
            let report = self.run_iteration(&task.description, &context);
            reports.push(report);
        }
        reports
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_new() {
        let p = SealPipeline::new(100, 32);
        assert_eq!(p.iteration_count, 0);
        assert_eq!(p.generator.vocab_size, 100);
        assert_eq!(p.generator.hidden_dim, 32);
    }

    #[test]
    fn test_run_single_iteration() {
        let mut p = SealPipeline::new(100, 32);
        let report = p.run_iteration("test_task", &["fn foo() {}"]);
        assert_eq!(report.iteration, 1);
        assert_eq!(report.task, "test_task");
        assert!(report.edits_generated > 0);
        assert!(report.best_reward >= 0.0);
        assert!(report.avg_reward >= 0.0);
        assert!(report.curriculum_difficulty >= 0.0);
    }

    #[test]
    fn test_run_two_iterations() {
        let mut p = SealPipeline::new(100, 32);
        let r1 = p.run_iteration("a", &["fn a() {}"]);
        let r2 = p.run_iteration("b", &["fn b() {}"]);
        assert_eq!(r2.iteration, 2);
        assert!(r2.policy_improvement >= 0.0);
    }

    #[test]
    fn test_run_curriculum() {
        let mut p = SealPipeline::new(100, 32);
        let tasks = vec![
            CurriculumTask {
                description: "refactor".into(),
                difficulty: 0.5,
                prerequisites: vec!["fn old() {}".into()],
            },
            CurriculumTask {
                description: "optimize".into(),
                difficulty: 0.6,
                prerequisites: vec!["perf loop".into()],
            },
        ];
        let reports = p.run_curriculum(&tasks);
        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0].iteration, 1);
        assert_eq!(reports[1].iteration, 2);
    }

    #[test]
    fn test_curriculum_tasks_have_difficulty() {
        let tasks = vec![
            CurriculumTask {
                description: "a".into(),
                difficulty: 0.3,
                prerequisites: vec![],
            },
            CurriculumTask {
                description: "b".into(),
                difficulty: 0.7,
                prerequisites: vec![],
            },
        ];
        assert!((tasks[0].difficulty - 0.3).abs() < 1e-6);
        assert!((tasks[1].difficulty - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_report_has_all_fields() {
        let mut p = SealPipeline::new(100, 32);
        let r = p.run_iteration("test", &["code"]);
        assert_eq!(r.task, "test");
        assert!(r.edits_generated > 0);
        assert!(r.best_reward.is_finite());
        assert!(r.avg_reward.is_finite());
        assert!(r.policy_improvement.is_finite());
        assert!(r.curriculum_difficulty.is_finite());
        assert!(r.convergence.is_finite());
    }

    #[test]
    fn test_difficulty_increases_with_success() {
        let mut p = SealPipeline::new(100, 32);
        for _ in 0..8 {
            p.run_iteration("fn task() {}", &["function body"]);
        }
        assert!(p.curriculum.difficulty_level >= 0.5);
    }

    #[test]
    fn test_pipeline_uses_analyzer() {
        let mut p = SealPipeline::new(100, 32);
        for i in 0..6 {
            p.run_iteration(&format!("iter_{}", i), &["fn code() {}"]);
        }
        assert!(p.analyzer.recent_performance.len() >= 6);
    }
}
