#![deny(clippy::unwrap_used)]

use serde::{Deserialize, Serialize};
use crate::core::nt_core_hex::FullReasoningState;
use crate::core::nt_core_policy::E8Policy;

/// E8 Plan Mode — 将推理轨迹编码为结构化计划，每个步骤是对应 E8 卦象状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8Plan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<PlanStep>,
    pub e8_sequence: Vec<u8>,
    pub metrics: PlanMetrics,
    pub created_at: u64,
    pub execution_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub index: usize,
    pub e8_mode: u8,
    pub action: String,
    pub expected_outcome: String,
    pub prm_score: f64,
    pub status: StepStatus,
    pub actual_outcome: Option<String>,
    pub completion_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMetrics {
    pub total_steps: usize,
    pub completed_steps: usize,
    pub avg_prm_score: f64,
    pub estimated_cost: f64,
    pub est_completion_ms: u64,
    pub e8_mode_stability: f64,
    pub goal_alignment: f64,
}

/// 计划生成器 — 利用 E8 状态机 + PRM 策略生成最优计划
pub struct PlanGenerator {
    pub policy: Option<E8Policy>,
    pub planner_mode: u8,
    pub max_steps: usize,
    pub prm_threshold: f64,
}

impl PlanGenerator {
    pub fn new() -> Self {
        Self {
            policy: None,
            planner_mode: 7,
            max_steps: 12,
            prm_threshold: 0.3,
        }
    }

    pub fn with_policy(mut self, policy: E8Policy) -> Self {
        self.policy = Some(policy);
        self
    }

    pub fn generate_plan(&self, goal: &str, context: &[FullReasoningState]) -> E8Plan {
        let steps = self.generate_steps(goal, context);
        let scores: Vec<f64> = steps.iter().map(|s| s.prm_score).collect();
        let avg_prm = if scores.is_empty() { 0.0 } else { scores.iter().sum::<f64>() / scores.len() as f64 };
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

        E8Plan {
            id,
            goal: goal.to_string(),
            e8_sequence: steps.iter().map(|s| s.e8_mode).collect(),
            metrics: PlanMetrics {
                total_steps: steps.len(),
                completed_steps: 0,
                avg_prm_score: avg_prm,
                estimated_cost: steps.len() as f64 * 0.005,
                est_completion_ms: steps.len() as u64 * 2000,
                e8_mode_stability: self.compute_mode_stability(&steps),
                goal_alignment: self.compute_goal_alignment(&steps, goal),
            },
            execution_count: 0,
            steps,
            created_at,
        }
    }

    fn generate_steps(&self, goal: &str, context: &[FullReasoningState]) -> Vec<PlanStep> {
        let mut steps = Vec::new();
        let task_len = goal.len().min(200);

        // Plan phases mapped to E8 modes
        let phase_modes: [u8; 6] = [1, 9, 17, 33, 41, 57];
        let phase_actions = [
            "analyze_goal",
            "gather_context",
            "generate_strategy",
            "execute",
            "verify",
            "reflect",
        ];

        for (i, (&mode, action)) in phase_modes.iter().zip(phase_actions.iter()).enumerate() {
            if i >= self.max_steps {
                break;
            }
            let score = self.score_mode_for_goal(mode, goal, context);
            if score >= self.prm_threshold {
                steps.push(PlanStep {
                    index: i,
                    e8_mode: mode,
                    action: action.to_string(),
                    expected_outcome: format!("Execute phase {} with E8 mode {}", action, mode),
                    prm_score: score,
                    status: StepStatus::Pending,
                    actual_outcome: None,
                    completion_time_ms: None,
                });
            }
        }

        if steps.is_empty() {
            steps.push(PlanStep {
                index: 0,
                e8_mode: self.planner_mode,
                action: "default_execute".to_string(),
                expected_outcome: format!("Default execution for: {}", &goal[..task_len.min(60)]),
                prm_score: 0.5,
                status: StepStatus::Pending,
                actual_outcome: None,
                completion_time_ms: None,
            });
        }

        steps
    }

    fn score_mode_for_goal(&self, mode: u8, _goal: &str, context: &[FullReasoningState]) -> f64 {
        if self.policy.is_some() {
            0.5 + (mode as f64) / 128.0
        } else if !context.is_empty() {
            let recent = context.last().unwrap_or(&context[0]);
            let similarity = 1.0 - (recent.mode.0 as f64 - mode as f64).abs() / 64.0;
            0.3 + similarity * 0.5
        } else {
            0.4
        }
    }

    fn compute_mode_stability(&self, steps: &[PlanStep]) -> f64 {
        if steps.len() < 2 {
            return 1.0;
        }
        let transitions = steps.windows(2).filter(|w| w[0].e8_mode != w[1].e8_mode).count();
        1.0 - transitions as f64 / steps.len() as f64
    }

    fn compute_goal_alignment(&self, steps: &[PlanStep], _goal: &str) -> f64 {
        if steps.is_empty() { return 0.0; }
        steps.iter().map(|s| s.prm_score).sum::<f64>() / steps.len() as f64
    }

    pub fn execute_step(&self, step: &mut PlanStep, outcome: &str, duration_ms: u64) {
        step.status = StepStatus::Completed;
        step.actual_outcome = Some(outcome.to_string());
        step.completion_time_ms = Some(duration_ms);
    }

    pub fn fail_step(&self, step: &mut PlanStep, error: &str) {
        step.status = StepStatus::Failed(error.to_string());
    }
}

impl Default for PlanGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl E8Plan {
    pub fn next_pending(&self) -> Option<&PlanStep> {
        self.steps.iter().find(|s| matches!(s.status, StepStatus::Pending))
    }

    pub fn completion_pct(&self) -> f64 {
        if self.steps.is_empty() { return 1.0; }
        self.steps.iter().filter(|s| matches!(s.status, StepStatus::Completed)).count() as f64 / self.steps.len() as f64
    }

    pub fn is_complete(&self) -> bool {
        self.steps.iter().all(|s| matches!(s.status, StepStatus::Completed | StepStatus::Skipped))
    }

    pub fn duration_ms(&self) -> u64 {
        self.steps.iter().filter_map(|s| s.completion_time_ms).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_generation() {
        let gen = PlanGenerator::new();
        let plan = gen.generate_plan("Build a web search tool", &[]);
        assert!(!plan.steps.is_empty());
        assert!(plan.metrics.total_steps > 1);
        assert_eq!(plan.steps[0].index, 0);
    }

    #[test]
    fn test_plan_step_execution() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Test step execution", &[]);
        let step = plan.steps.first_mut().unwrap();
        gen.execute_step(step, "completed successfully", 1500);
        assert!(matches!(step.status, StepStatus::Completed));
        assert_eq!(step.completion_time_ms, Some(1500));
    }

    #[test]
    fn test_plan_completion() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Test completion", &[]);
        for step in plan.steps.iter_mut() {
            gen.execute_step(step, "done", 100);
        }
        assert!(plan.is_complete());
        assert!(plan.completion_pct() > 0.99);
    }

    #[test]
    fn test_plan_metrics() {
        let gen = PlanGenerator::new();
        let plan = gen.generate_plan("Test metrics", &[]);
        assert!(plan.metrics.avg_prm_score >= 0.0);
        assert!(plan.metrics.e8_mode_stability >= 0.0);
        assert!(plan.metrics.goal_alignment >= 0.0);
    }

    #[test]
    fn test_empty_context_plan() {
        let gen = PlanGenerator::new();
        let plan = gen.generate_plan("High threshold plan", &[]);
        assert!(!plan.steps.is_empty());
    }
}
