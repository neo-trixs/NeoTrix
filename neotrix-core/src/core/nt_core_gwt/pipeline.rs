use serde::{Deserialize, Serialize};

use super::module_def::SpecialistType;
use super::workspace::GlobalWorkspace;
use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_policy::{E8Policy, NUM_E8_FACTORS};
use crate::core::nt_core_prm::{AgentTrajectory, ProcessScore, TrajectoryCollector};

/// Role in the sequential MAPPA pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineRole {
    ProblemSolver,
    CodeExecutor,
    Verifier,
}

impl PipelineRole {
    pub fn all() -> [Self; 3] {
        [Self::ProblemSolver, Self::CodeExecutor, Self::Verifier]
    }

    /// Map pipeline role to the closest GWT specialist type for activation tracking.
    pub fn specialist_type(&self) -> SpecialistType {
        match self {
            Self::ProblemSolver => SpecialistType::Planner,
            Self::CodeExecutor => SpecialistType::CodeAnalyzer,
            Self::Verifier => SpecialistType::MetaCognitionAnalyst,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::ProblemSolver => "problem-solver",
            Self::CodeExecutor => "code-executor",
            Self::Verifier => "verifier",
        }
    }
}

/// One stage in the sequential pipeline specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub role: PipelineRole,
    pub specialist_name: String,
    pub description: String,
}

impl PipelineStage {
    pub fn new(role: PipelineRole, specialist_name: &str, description: &str) -> Self {
        Self {
            role,
            specialist_name: specialist_name.to_string(),
            description: description.to_string(),
        }
    }
}

/// Specification of the sequential MAPPA pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSpec {
    pub stages: Vec<PipelineStage>,
}

impl PipelineSpec {
    /// Default 3-stage MAPPA pipeline: ProblemSolver → CodeExecutor → Verifier.
    pub fn mappa_default() -> Self {
        Self {
            stages: vec![
                PipelineStage::new(
                    PipelineRole::ProblemSolver,
                    "planner",
                    "Analyze the task and produce a solution plan",
                ),
                PipelineStage::new(
                    PipelineRole::CodeExecutor,
                    "code-analyzer",
                    "Implement the solution as working code",
                ),
                PipelineStage::new(
                    PipelineRole::Verifier,
                    "meta-cognition-analyst",
                    "Verify the implementation for correctness and quality",
                ),
            ],
        }
    }
}

/// Result from executing one pipeline step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStepResult {
    pub role: PipelineRole,
    pub output: String,
    pub success: bool,
    pub duration_ms: u64,
    pub e8_mode: ReasoningHexagram,
}

/// Result from executing the full pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub task: String,
    pub step_results: Vec<PipelineStepResult>,
    pub final_output: String,
    pub all_success: bool,
    pub total_duration_ms: u64,
}

/// Executes the sequential MAPPA pipeline, collecting trajectory data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecutor {
    pub spec: PipelineSpec,
    pub trajectory_collector: TrajectoryCollector,
}

impl PipelineExecutor {
    pub fn new(spec: PipelineSpec) -> Self {
        Self {
            spec,
            trajectory_collector: TrajectoryCollector::new(),
        }
    }

    /// Execute the pipeline for a given task.
    ///
    /// Each stage:
    /// 1. Looks up the specialist from GWT
    /// 2. Records the step in the trajectory collector
    /// 3. Passes output as input to the next stage
    ///
    /// The actual "execution" is a placeholder — the integrating crate
    /// provides the real LLM-backed execution via `PipelineHandler`.
    pub fn execute(
        &mut self,
        task: &str,
        _gwt: &GlobalWorkspace,
        handler: &dyn PipelineHandler,
    ) -> PipelineResult {
        self.trajectory_collector.begin(task.to_string());

        let mut input = task.to_string();
        let mut step_results = Vec::new();
        let mut all_success = true;
        let start = std::time::Instant::now();

        for stage in &self.spec.stages {
            let step_start = std::time::Instant::now();
            let e8_mode = stage.role.e8_mode_default();

            let (output, success) = handler.execute_stage(stage, &input);

            let duration_ms = step_start.elapsed().as_millis() as u64;

            self.trajectory_collector.record_step(
                stage.role.specialist_type(),
                e8_mode,
                stage.description.clone(),
                input.clone(),
                output.clone(),
                Some(duration_ms),
                success,
                None,
            );

            step_results.push(PipelineStepResult {
                role: stage.role,
                output: output.clone(),
                success,
                duration_ms,
                e8_mode,
            });

            if !success {
                all_success = false;
            }
            input = output;
        }

        let total_duration_ms = start.elapsed().as_millis() as u64;

        self.trajectory_collector.finish(
            if all_success { Some(1.0) } else { Some(0.0) },
            all_success,
        );

        PipelineResult {
            task: task.to_string(),
            step_results,
            final_output: input,
            all_success,
            total_duration_ms,
        }
    }
}

/// Trait for the actual execution logic (LLM-backed step).
pub trait PipelineHandler {
    fn execute_stage(&self, stage: &PipelineStage, input: &str) -> (String, bool);
}

/// Distributes process rewards from Coach back to E8Policy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CreditArbiter;

impl CreditArbiter {
    /// Apply step-level process scores to update E8Policy.
    ///
    /// Each step's score becomes a TD-update signal for the mode used in that step.
    pub fn distribute(
        policy: &mut E8Policy,
        trajectory: &AgentTrajectory,
        scores: &[ProcessScore],
    ) {
        for score in scores {
            if let Some(step) = trajectory.steps.get(score.step_idx) {
                policy.set_previous(step.e8_mode);
                policy.update(score.score);
            }
        }
        policy.decay_epsilon();
    }

    /// Distribute with factorized deltas for more granular credit assignment.
    pub fn distribute_factorized(
        policy: &mut E8Policy,
        trajectory: &AgentTrajectory,
        scores: &[ProcessScore],
    ) {
        for score in scores {
            if let Some(step) = trajectory.steps.get(score.step_idx) {
                policy.set_previous(step.e8_mode);

                let mut factor_deltas = [0.0; NUM_E8_FACTORS];
                for (i, tag) in score.attribution_tags.iter().enumerate() {
                    if i < NUM_E8_FACTORS {
                        factor_deltas[i] = if tag.contains("good") || tag.contains("ok") {
                            0.1
                        } else {
                            -0.1
                        };
                    }
                }
                policy.update_factorized(score.score, &factor_deltas);
            }
        }
        policy.decay_epsilon();
    }
}

impl PipelineRole {
    /// Default E8 mode for each pipeline role.
    pub fn e8_mode_default(&self) -> ReasoningHexagram {
        match self {
            // Planner modes: Analytical+Focused (bit pattern varies)
            Self::ProblemSolver => ReasoningHexagram::new(0b001010),
            // CodeAnalyzer modes: Analytical+Deep
            Self::CodeExecutor => ReasoningHexagram::new(0b000010),
            // MetaCognitionAnalyst modes: Meta+Analytical
            Self::Verifier => ReasoningHexagram::new(0b101010),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_prm::{ScoredCriterion, TrajectoryStep};

    struct MockHandler {
        results: Vec<(String, bool)>,
        call_count: std::sync::atomic::AtomicUsize,
    }

    impl MockHandler {
        fn new(results: Vec<(&str, bool)>) -> Self {
            Self {
                results: results.into_iter().map(|(o, s)| (o.to_string(), s)).collect(),
                call_count: std::sync::atomic::AtomicUsize::new(0),
            }
        }
    }

    impl PipelineHandler for MockHandler {
        fn execute_stage(&self, _stage: &PipelineStage, _input: &str) -> (String, bool) {
            let idx = self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if idx < self.results.len() {
                self.results[idx].clone()
            } else {
                ("overflow".to_string(), false)
            }
        }
    }

    #[test]
    fn test_pipeline_role_all_returns_3() {
        assert_eq!(PipelineRole::all().len(), 3);
    }

    #[test]
    fn test_pipeline_role_specialist_mapping() {
        assert_eq!(PipelineRole::ProblemSolver.specialist_type(), SpecialistType::Planner);
        assert_eq!(PipelineRole::CodeExecutor.specialist_type(), SpecialistType::CodeAnalyzer);
        assert_eq!(PipelineRole::Verifier.specialist_type(), SpecialistType::MetaCognitionAnalyst);
    }

    #[test]
    fn test_pipeline_role_labels() {
        assert_eq!(PipelineRole::ProblemSolver.label(), "problem-solver");
        assert_eq!(PipelineRole::CodeExecutor.label(), "code-executor");
        assert_eq!(PipelineRole::Verifier.label(), "verifier");
    }

    #[test]
    fn test_pipeline_stage_new() {
        let stage = PipelineStage::new(PipelineRole::ProblemSolver, "planner", "plan it");
        assert_eq!(stage.role, PipelineRole::ProblemSolver);
        assert_eq!(stage.specialist_name, "planner");
        assert_eq!(stage.description, "plan it");
    }

    #[test]
    fn test_mappa_default_has_3_stages() {
        let spec = PipelineSpec::mappa_default();
        assert_eq!(spec.stages.len(), 3);
        assert_eq!(spec.stages[0].role, PipelineRole::ProblemSolver);
        assert_eq!(spec.stages[1].role, PipelineRole::CodeExecutor);
        assert_eq!(spec.stages[2].role, PipelineRole::Verifier);
    }

    #[test]
    fn test_pipeline_executor_default_spec() {
        let executor = PipelineExecutor::new(PipelineSpec::mappa_default());
        assert_eq!(executor.spec.stages.len(), 3);
    }

    #[test]
    fn test_execute_all_success() {
        let mut executor = PipelineExecutor::new(PipelineSpec::mappa_default());
        let handler = MockHandler::new(vec![
            ("plan output", true),
            ("code output", true),
            ("verification ok", true),
        ]);
        let gwt = GlobalWorkspace::new(0.3);
        let result = executor.execute("test task", &gwt, &handler);
        assert!(result.all_success);
        assert_eq!(result.step_results.len(), 3);
        assert_eq!(result.final_output, "verification ok");
    }

    #[test]
    fn test_execute_mid_pipeline_failure() {
        let mut executor = PipelineExecutor::new(PipelineSpec::mappa_default());
        let handler = MockHandler::new(vec![
            ("plan output", true),
            ("code failed", false),
            ("should not run", true),
        ]);
        let gwt = GlobalWorkspace::new(0.3);
        let result = executor.execute("failing task", &gwt, &handler);
        assert!(!result.all_success);
        assert_eq!(result.step_results[1].success, false);
    }

    #[test]
    fn test_execute_trajectory_recorded() {
        let mut executor = PipelineExecutor::new(PipelineSpec::mappa_default());
        let handler = MockHandler::new(vec![
            ("step1", true),
            ("step2", true),
            ("step3", true),
        ]);
        let gwt = GlobalWorkspace::new(0.3);
        let result = executor.execute("traj test", &gwt, &handler);
        let traj = executor.trajectory_collector.latest();
        assert!(traj.is_some());
        assert_eq!(traj.unwrap().steps.len(), 3);
        assert!(result.all_success);
    }

    #[test]
    fn test_credit_arbiter_distribute_basic() {
        let mut policy = E8Policy::default();
        let mode = ReasoningHexagram::new(10);
        let step = TrajectoryStep {
            step_idx: 0,
            specialist: SpecialistType::Planner,
            e8_mode: mode,
            action: "test step".to_string(),
            input: "in".to_string(),
            output: "out".to_string(),
            duration_ms: Some(100),
            success: true,
            external_reward: None,
        };
        let trajectory = AgentTrajectory {
            trajectory_id: 1,
            task: "test".to_string(),
            steps: vec![step],
            outcome_reward: None,
            completed: true,
            total_duration_ms: None,
        };
        let scores = vec![ProcessScore {
            step_idx: 0,
            score: 0.8,
            confidence: 0.9,
            criteria: vec![ScoredCriterion {
                name: "correctness".to_string(),
                score: 0.8,
                rationale: None,
            }],
            attribution_tags: vec!["good".to_string()],
        }];
        CreditArbiter::distribute(&mut policy, &trajectory, &scores);
        assert!(policy.epsilon() < 1.0);
    }

    #[test]
    fn test_e8_mode_defaults() {
        let solver = PipelineRole::ProblemSolver.e8_mode_default();
        let coder = PipelineRole::CodeExecutor.e8_mode_default();
        let verifier = PipelineRole::Verifier.e8_mode_default();
        assert_eq!(solver.0 & 0x3F, solver.0);
        assert_eq!(coder.0 & 0x3F, coder.0);
        assert_eq!(verifier.0 & 0x3F, verifier.0);
        assert!(solver.0 != coder.0 || coder.0 != verifier.0);
    }
}

