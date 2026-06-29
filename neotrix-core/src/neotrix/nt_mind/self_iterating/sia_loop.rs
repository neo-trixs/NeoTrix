use serde::{Deserialize, Serialize};

use super::brain_impl::RLAlgorithm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrajectory {
    pub steps: Vec<TrajectoryStep>,
    pub metrics: ExecutionMetrics,
    pub scaffold: String,
    pub generation: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub prompt: String,
    pub response: String,
    pub tool_calls: Vec<ToolCallRecord>,
    pub result: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub args: String,
    pub output: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub score: f64,
    pub instances_completed: usize,
    pub total_instances: usize,
    pub avg_time_ms: f64,
    pub error_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SIAImprovement {
    HarnessUpdate {
        new_scaffold: String,
        rationale: String,
    },
    WeightUpdate {
        algorithm: RLAlgorithm,
        config: RLConfig,
        rationale: String,
    },
    Converged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLConfig {
    pub learning_rate: f64,
    pub loja_rank: usize,
    pub num_rollouts: usize,
    pub kl_coeff: f64,
}

pub trait MetaAgent {
    fn generate_initial_scaffold(&self, task_spec: &str) -> String {
        format!("Scaffold for task: {}", task_spec)
    }
}

pub trait TargetAgent {
    type Trajectory;
    fn execute(&mut self, task_spec: &str, scaffold: &str) -> Self::Trajectory;
}

pub trait FeedbackAgent {
    type Improvement;
    fn analyze_and_improve(
        &self,
        trajectory: &ExecutionTrajectory,
        metrics: &ExecutionMetrics,
    ) -> Self::Improvement;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SIAController {
    pub current_generation: usize,
    pub max_generations: usize,
    pub trajectories: Vec<ExecutionTrajectory>,
    pub improvements: Vec<SIAImprovement>,
    pub current_scaffold: String,
}

impl SIAController {
    pub fn new(max_generations: usize) -> Self {
        Self {
            current_generation: 0,
            max_generations,
            trajectories: Vec::new(),
            improvements: Vec::new(),
            current_scaffold: String::new(),
        }
    }

    pub fn record_trajectory(&mut self, trajectory: ExecutionTrajectory) {
        self.trajectories.push(trajectory);
    }

    pub fn record_improvement(&mut self, improvement: SIAImprovement) {
        self.improvements.push(improvement);
    }

    pub fn best_score(&self) -> f64 {
        self.trajectories
            .iter()
            .map(|t| t.metrics.score)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    pub fn latest_score(&self) -> f64 {
        self.trajectories
            .last()
            .map(|t| t.metrics.score)
            .unwrap_or(0.0)
    }

    pub fn is_converged(&self) -> bool {
        self.improvements
            .last()
            .map_or(false, |i| matches!(i, SIAImprovement::Converged))
    }

    pub fn next_generation(&mut self) {
        self.current_generation += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sia_controller_new() {
        let controller = SIAController::new(10);
        assert_eq!(controller.current_generation, 0);
        assert!(controller.trajectories.is_empty());
        assert!(controller.improvements.is_empty());
        assert_eq!(controller.max_generations, 10);
    }

    #[test]
    fn test_sia_controller_record_trajectory() {
        let mut controller = SIAController::new(5);
        let metrics = ExecutionMetrics {
            score: 0.85,
            instances_completed: 8,
            total_instances: 10,
            avg_time_ms: 120.5,
            error_count: 1,
        };
        let trajectory = ExecutionTrajectory {
            steps: vec![],
            metrics,
            scaffold: "test scaffold".into(),
            generation: 0,
        };
        controller.record_trajectory(trajectory);
        assert_eq!(controller.trajectories.len(), 1);
        assert_eq!(controller.latest_score(), 0.85);
    }

    #[test]
    fn test_sia_controller_converged() {
        let mut controller = SIAController::new(10);
        assert!(!controller.is_converged());
        controller.record_improvement(SIAImprovement::Converged);
        assert!(controller.is_converged());
    }

    #[test]
    fn test_trajectory_step() {
        let tool_call = ToolCallRecord {
            tool_name: "search".into(),
            args: "query".into(),
            output: "results".into(),
            success: true,
        };
        let step = TrajectoryStep {
            prompt: "prompt".into(),
            response: "response".into(),
            tool_calls: vec![tool_call],
            result: "done".into(),
            timestamp: 1_234_567_890,
        };
        assert_eq!(step.prompt, "prompt");
        assert_eq!(step.response, "response");
        assert_eq!(step.tool_calls.len(), 1);
        assert!(step.tool_calls[0].success);
        assert_eq!(step.result, "done");
        assert_eq!(step.timestamp, 1_234_567_890);
    }

    #[test]
    fn test_execution_metrics() {
        let metrics = ExecutionMetrics {
            score: 0.92,
            instances_completed: 10,
            total_instances: 10,
            avg_time_ms: 95.0,
            error_count: 0,
        };
        assert_eq!(metrics.score, 0.92);
        assert_eq!(metrics.instances_completed, 10);
        assert_eq!(metrics.total_instances, 10);
        assert_eq!(metrics.avg_time_ms, 95.0);
        assert_eq!(metrics.error_count, 0);
    }
}
