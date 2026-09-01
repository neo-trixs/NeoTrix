#![forbid(unsafe_code)]

pub mod constitution_gate;
pub mod curriculum;
pub mod grpo;
pub mod self_edit_gen;

use serde::{Deserialize, Serialize};

use crate::core::nt_core_self::self_audit::{converge_check, AuditReport};
use crate::core::nt_core_self::pilot_steering::{PilotSupervisor, SupervisorConfig, SupervisorDecision, TracePoint, TraceResult};

pub use self::constitution_gate::{ConstitutionGate, SELF_EDIT_MIN_CONSCIOUSNESS};
pub use self::curriculum::{
    CalibratedCurriculumGenerator, CurriculumRecord, IterationValidator,
    LearnabilityWindowAnalyzer, ValidationResult,
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
    pub supervisor: PilotSupervisor,
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
            supervisor: PilotSupervisor::new(SupervisorConfig::default()),
            iteration_count: 0,
        }
    }

    pub fn run_iteration(&mut self, task: &str, context: &[&str]) -> SealIterationReport {
        self.iteration_count += 1;

        // PILOT: 启动 worker 监控
        self.supervisor.launch_worker(format!("iter_{}", self.iteration_count));

        let code_context = context.join("\n");
        let edits = self.generator.generate_edits(&code_context, task);
        let rewards: Vec<f64> = edits.iter().map(|e| self.evaluator.evaluate(e)).collect();
        let best_reward = rewards.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg_reward = if rewards.is_empty() {
            0.0
        } else {
            rewards.iter().sum::<f64>() / rewards.len() as f64
        };

        // PILOT: 记录执行轨迹
        self.supervisor.record_trace(TracePoint {
            timestamp: std::time::Instant::now(),
            action: format!("generate_{}_edits", edits.len()),
            result: if avg_reward > 0.5 {
                TraceResult::Success
            } else {
                TraceResult::Failure(format!("low reward: {:.3}", avg_reward))
            },
            context: std::collections::HashMap::new(),
        });

        // PILOT: 评估是否需要重定向
        let decision = self.supervisor.evaluate();
        if let SupervisorDecision::Redirect { new_task: _, reason } = &decision {
            log::debug!("[seal] PILOT redirect suggestion: {}", reason);
        }

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

    pub fn converge_check(&self, src_dir: &str) -> AuditReport {
        converge_check(src_dir)
    }

    /// 使用 CUDA Agent RL 选择优化策略
    ///
    /// 参考: arXiv:2602.24286 "CUDA-Agent: Skill-Augmented..."
    /// 整合强化学习策略选择器, 根据历史效果选择最佳优化策略。
    pub fn select_optimization_strategy(&self, _task: &str, language: &str) -> Option<String> {
        use crate::core::nt_core_self::cuda_agent::StrategyManager;
        use crate::core::nt_core_self::cuda_agent::OptimizationStrategy;

        let mut manager = StrategyManager::new();

        // 注册默认优化策略
        manager.register(OptimizationStrategy {
            id: "loop_unrolling".to_string(),
            name: "loop_unrolling".to_string(),
            description: "循环展开优化".to_string(),
            applicable_languages: vec!["rust".to_string(), "c".to_string(), "cpp".to_string()],
            expected_improvement: 0.15,
            success_rate: 0.7,
            usage_count: 0,
        });

        manager.register(OptimizationStrategy {
            id: "inlining".to_string(),
            name: "inlining".to_string(),
            description: "函数内联优化".to_string(),
            applicable_languages: vec!["rust".to_string(), "c".to_string(), "cpp".to_string()],
            expected_improvement: 0.12,
            success_rate: 0.8,
            usage_count: 0,
        });

        manager.register(OptimizationStrategy {
            id: "parallelization".to_string(),
            name: "parallelization".to_string(),
            description: "并行化优化".to_string(),
            applicable_languages: vec!["rust".to_string()],
            expected_improvement: 0.25,
            success_rate: 0.6,
            usage_count: 0,
        });

        manager.register(OptimizationStrategy {
            id: "memory_optimization".to_string(),
            name: "memory_optimization".to_string(),
            description: "内存优化".to_string(),
            applicable_languages: vec!["rust".to_string(), "c".to_string(), "cpp".to_string()],
            expected_improvement: 0.18,
            success_rate: 0.65,
            usage_count: 0,
        });

        // 根据语言和任务选择策略
        let strategy = manager.select_strategy(language, "execution_time");
        strategy.map(|s| s.name.clone())
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

    /// 获取 PILOT 失败模式
    pub fn get_failure_patterns(&self) -> Vec<crate::core::nt_core_self::pilot_steering::FailurePattern> {
        self.supervisor.get_failure_patterns()
    }

    /// Human Approval 集成: 为 SEAL pipeline 添加审批工作流
    ///
    /// 参考: "Human-in-the-Loop Agent Patterns"
    /// 在关键 SEAL 阶段添加人工审批点。
    pub fn request_approval_for_iteration(
        &self,
        iteration: &str,
        changes: &[String],
    ) -> crate::core::nt_core_self::human_approval::ApprovalRequest {
        use crate::core::nt_core_self::human_approval::*;

        // 根据变更类型确定风险级别
        let risk_level = if changes.iter().any(|c| c.contains("core") || c.contains("security")) {
            RiskLevel::High
        } else if changes.iter().any(|c| c.contains("test") || c.contains("doc")) {
            RiskLevel::Low
        } else {
            RiskLevel::Medium
        };

        // 使用 Builder 创建审批请求
        ApprovalRequestBuilder::new(
            &format!("SEAL_iteration_{}", iteration),
            "seal_iteration",
        )
        .description(&changes.join("\n"))
        .risk_level(risk_level)
        .timeout(std::time::Duration::from_secs(300))
        .build()
    }

    /// Human Approval 集成: 根据审批结果决定是否继续执行
    pub fn should_continue_with_approval(
        &self,
        approval: &crate::core::nt_core_self::human_approval::ApprovalStatus,
    ) -> bool {
        matches!(approval, crate::core::nt_core_self::human_approval::ApprovalStatus::Approved)
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
        let _r1 = p.run_iteration("a", &["fn a() {}"]);
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
