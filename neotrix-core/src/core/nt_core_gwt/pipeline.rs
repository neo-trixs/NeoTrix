use super::module_def::SpecialistType;
use super::workspace::GlobalWorkspace;
use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_policy::{E8Policy, NUM_E8_FACTORS};
use crate::core::nt_core_prm::{AgentTrajectory, ProcessScore, TrajectoryCollector};

/// Role in the sequential MAPPA pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct PipelineStepResult {
    pub role: PipelineRole,
    pub output: String,
    pub success: bool,
    pub duration_ms: u64,
    pub e8_mode: ReasoningHexagram,
}

/// Result from executing the full pipeline.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub task: String,
    pub step_results: Vec<PipelineStepResult>,
    pub final_output: String,
    pub all_success: bool,
    pub total_duration_ms: u64,
}

/// Executes the sequential MAPPA pipeline, collecting trajectory data.
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
