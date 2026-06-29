use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use super::sia_loop::{
    ExecutionMetrics, ExecutionTrajectory, FeedbackAgent, MetaAgent, SIAController, SIAImprovement,
    TargetAgent, ToolCallRecord, TrajectoryStep,
};

const MAX_GENERATIONS: usize = 10;
#[allow(dead_code)]
const CONVERGENCE_THRESHOLD: f64 = 0.05;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriAgentPipeline<M, T, F>
where
    M: MetaAgent,
    T: TargetAgent,
    F: FeedbackAgent,
{
    meta: M,
    target: T,
    feedback: F,
    controller: SIAController,
    task_spec: String,
    generation: usize,
    converged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriAgentReport {
    pub generations_completed: usize,
    pub final_score: f64,
    pub best_score: f64,
    pub converged: bool,
    pub num_improvements: usize,
    pub total_trajectories: usize,
}

impl<M, T, F> TriAgentPipeline<M, T, F>
where
    M: MetaAgent,
    T: TargetAgent<Trajectory = ExecutionTrajectory>,
    F: FeedbackAgent<Improvement = SIAImprovement>,
{
    pub fn new(meta: M, target: T, feedback: F, task_spec: &str) -> Self {
        Self {
            meta,
            target,
            feedback,
            controller: SIAController::new(MAX_GENERATIONS),
            task_spec: task_spec.to_string(),
            generation: 0,
            converged: false,
        }
    }

    pub fn run_generation(&mut self) {
        if self.converged || self.generation >= MAX_GENERATIONS {
            return;
        }

        let scaffold = self.meta.generate_initial_scaffold(&self.task_spec);
        let trajectory = self.target.execute(&self.task_spec, &scaffold);
        let improvement = self
            .feedback
            .analyze_and_improve(&trajectory, &trajectory.metrics);

        self.controller.record_trajectory(trajectory);
        self.controller.record_improvement(improvement);
        self.controller.next_generation();

        self.generation += 1;
        self.converged = self.check_convergence();
    }

    pub fn run_full(&mut self) -> TriAgentReport {
        while !self.converged && self.generation < MAX_GENERATIONS {
            self.run_generation();
        }
        self.report()
    }

    pub fn run_n_generations(&mut self, n: usize) -> TriAgentReport {
        let target = (self.generation + n).min(MAX_GENERATIONS);
        while self.generation < target && !self.converged {
            self.run_generation();
        }
        self.report()
    }

    pub fn report(&self) -> TriAgentReport {
        TriAgentReport {
            generations_completed: self.generation,
            final_score: self.controller.latest_score(),
            best_score: self.controller.best_score(),
            converged: self.converged,
            num_improvements: self.controller.improvements.len(),
            total_trajectories: self.controller.trajectories.len(),
        }
    }

    pub fn controller(&self) -> &SIAController {
        &self.controller
    }

    pub fn controller_mut(&mut self) -> &mut SIAController {
        &mut self.controller
    }

    fn check_convergence(&self) -> bool {
        if self.controller.improvements.is_empty() {
            return false;
        }
        matches!(
            self.controller.improvements.last(),
            Some(SIAImprovement::Converged)
        )
    }
}

pub struct DefaultMeta;

impl MetaAgent for DefaultMeta {
    fn generate_initial_scaffold(&self, task_spec: &str) -> String {
        format!(
            "You are a specialized agent executing: {}\n\
             Follow these steps:\n\
             1. Understand the task requirements\n\
             2. Plan your approach\n\
             3. Execute with verification at each step\n\
             4. Record all actions and outcomes",
            task_spec
        )
    }
}

pub struct DefaultTarget;

impl TargetAgent for DefaultTarget {
    type Trajectory = ExecutionTrajectory;

    fn execute(&mut self, task_spec: &str, scaffold: &str) -> ExecutionTrajectory {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let step = TrajectoryStep {
            prompt: format!("Task: {}\nScaffold: {}", task_spec, scaffold),
            response: "Executing task per scaffold guidelines.".to_string(),
            tool_calls: vec![ToolCallRecord {
                tool_name: "execute".to_string(),
                args: task_spec.to_string(),
                output: "completed".to_string(),
                success: true,
            }],
            result: "Task execution completed successfully.".to_string(),
            timestamp,
        };

        ExecutionTrajectory {
            steps: vec![step],
            metrics: ExecutionMetrics {
                score: 0.8,
                instances_completed: 1,
                total_instances: 1,
                avg_time_ms: 100.0,
                error_count: 0,
            },
            scaffold: scaffold.to_string(),
            generation: 0,
        }
    }
}

pub struct DefaultFeedback;

impl FeedbackAgent for DefaultFeedback {
    type Improvement = SIAImprovement;

    fn analyze_and_improve(
        &self,
        _trajectory: &ExecutionTrajectory,
        metrics: &ExecutionMetrics,
    ) -> SIAImprovement {
        if metrics.score > 0.95 {
            return SIAImprovement::Converged;
        }
        if metrics.error_count > 0 {
            SIAImprovement::HarnessUpdate {
                new_scaffold: format!(
                    "Improved scaffold (errors: {}, score: {:.2}): add validation steps",
                    metrics.error_count, metrics.score
                ),
                rationale: format!(
                    "Reducing errors from {} to improve score {:.2} → {:.2}",
                    metrics.error_count,
                    metrics.score,
                    (metrics.score + 0.1).min(1.0)
                ),
            }
        } else {
            SIAImprovement::HarnessUpdate {
                new_scaffold: format!(
                    "Refined scaffold (score: {:.2}): increase task complexity handling",
                    metrics.score
                ),
                rationale: format!("Incremental improvement from {:.2}", metrics.score),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pipeline() {
        let pipeline =
            TriAgentPipeline::new(DefaultMeta, DefaultTarget, DefaultFeedback, "test task");
        assert_eq!(pipeline.generation, 0);
        assert!(!pipeline.converged);
    }

    #[test]
    fn test_run_single_generation() {
        let mut pipeline =
            TriAgentPipeline::new(DefaultMeta, DefaultTarget, DefaultFeedback, "test task");
        pipeline.run_generation();
        assert_eq!(pipeline.generation, 1);
        assert_eq!(pipeline.controller.trajectories.len(), 1);
        assert_eq!(pipeline.controller.improvements.len(), 1);
    }

    #[test]
    fn test_run_full() {
        let mut pipeline =
            TriAgentPipeline::new(DefaultMeta, DefaultTarget, DefaultFeedback, "test task");
        let report = pipeline.run_full();
        assert!(report.generations_completed > 0);
        assert!(report.total_trajectories > 0);
    }

    #[test]
    fn test_run_n_generations() {
        let mut pipeline =
            TriAgentPipeline::new(DefaultMeta, DefaultTarget, DefaultFeedback, "test task");
        let report = pipeline.run_n_generations(3);
        assert_eq!(report.generations_completed, 3);
    }

    #[test]
    fn test_report() {
        let mut pipeline =
            TriAgentPipeline::new(DefaultMeta, DefaultTarget, DefaultFeedback, "test task");
        pipeline.run_generation();
        let report = pipeline.report();
        assert!(report.final_score > 0.0);
        assert!(report.best_score > 0.0);
    }

    #[test]
    fn test_max_generations() {
        let mut pipeline =
            TriAgentPipeline::new(DefaultMeta, DefaultTarget, DefaultFeedback, "test task");
        let report = pipeline.run_n_generations(MAX_GENERATIONS + 5);
        assert_eq!(report.generations_completed, MAX_GENERATIONS);
    }

    #[test]
    fn test_default_meta_generates_scaffold() {
        let meta = DefaultMeta;
        let scaffold = meta.generate_initial_scaffold("write code");
        assert!(scaffold.contains("write code"));
        assert!(scaffold.contains("specialized agent"));
    }

    #[test]
    fn test_default_target_produces_trajectory() {
        let mut target = DefaultTarget;
        let traj = target.execute("test", "scaffold");
        assert!(!traj.steps.is_empty());
        assert!(traj.metrics.score > 0.0);
    }

    #[test]
    fn test_default_feedback_improves() {
        let feedback = DefaultFeedback;
        let mut target = DefaultTarget;
        let traj = target.execute("test", "scaffold");
        let improvement = feedback.analyze_and_improve(&traj, &traj.metrics);
        match improvement {
            SIAImprovement::HarnessUpdate { .. } => {}
            SIAImprovement::Converged => {}
            _ => panic!("unexpected improvement type"),
        }
    }
}
