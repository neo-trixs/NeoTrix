pub use crate::core::nt_core_hex::ReasoningHexagram;
pub use crate::core::nt_core_traits::SpecialistType;
use serde::{Deserialize, Serialize};

/// One step in a multi-agent reasoning trajectory.
///
/// NOTE: f64 fields may produce NaN in edge cases. Use
/// `serde_json::Builder::new().nan_infinity(true)` to handle during (de)serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    /// Step index within the episode (0-based).
    pub step_idx: usize,
    /// Which specialist executed this step.
    pub specialist: SpecialistType,
    /// The E8 reasoning mode active during this step.
    pub e8_mode: ReasoningHexagram,
    /// Short description of the action taken.
    pub action: String,
    /// Input context passed to this step.
    pub input: String,
    /// Output/result produced by this step.
    pub output: String,
    /// Duration in milliseconds (if available).
    pub duration_ms: Option<u64>,
    /// Whether the step completed without error.
    pub success: bool,
    /// External reward signal (if any, e.g. test pass/fail).
    pub external_reward: Option<f64>,
}

/// A full multi-step reasoning episode (trajectory).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTrajectory {
    /// Unique identifier for this trajectory.
    pub trajectory_id: u64,
    /// The task description / prompt.
    pub task: String,
    /// All steps in execution order.
    pub steps: Vec<TrajectoryStep>,
    /// Final outcome reward (e.g. solution quality).
    pub outcome_reward: Option<f64>,
    /// Whether the episode completed successfully.
    pub completed: bool,
    /// Total wall-clock time in ms.
    pub total_duration_ms: Option<u64>,
}

impl AgentTrajectory {
    pub fn new(trajectory_id: u64, task: String) -> Self {
        Self {
            trajectory_id,
            task,
            steps: Vec::new(),
            outcome_reward: None,
            completed: false,
            total_duration_ms: None,
        }
    }

    pub fn push(&mut self, step: TrajectoryStep) {
        self.steps.push(step);
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// A criterion used to score a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredCriterion {
    /// Criterion name (e.g. "correctness", "efficiency", "clarity").
    pub name: String,
    /// Score in [0.0, 1.0].
    pub score: f64,
    /// Optional justification.
    pub rationale: Option<String>,
}

/// Process reward score for a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessScore {
    /// Step index this score applies to.
    pub step_idx: usize,
    /// Overall process reward in [0.0, 1.0].
    pub score: f64,
    /// Coach confidence in [0.0, 1.0].
    pub confidence: f64,
    /// Per-criterion breakdown.
    pub criteria: Vec<ScoredCriterion>,
    /// Semantic attribution tags (e.g. "correct_logic", "missing_edge_case").
    pub attribution_tags: Vec<String>,
}

impl ProcessScore {
    pub fn new(step_idx: usize) -> Self {
        Self {
            step_idx,
            score: 0.5,
            confidence: 0.0,
            criteria: Vec::new(),
            attribution_tags: Vec::new(),
        }
    }
}

/// Context passed to the Coach alongside each step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachContext {
    /// The full trajectory up to (and including) the current step.
    pub trajectory_so_far: Vec<TrajectoryStep>,
    /// Aggregated E8 transition data for the episode.
    pub transition_patterns: Vec<String>,
    /// Whether this is the final step of the trajectory.
    pub is_terminal: bool,
}

impl CoachContext {
    pub fn new(is_terminal: bool) -> Self {
        Self {
            trajectory_so_far: Vec::new(),
            transition_patterns: Vec::new(),
            is_terminal,
        }
    }
}

/// A Coach assigns process rewards by observing trajectory steps.
///
/// This is the core MAPPA abstraction: an LLM-as-judge (or analytic heuristic)
/// that scores each agent action and provides semantic attribution.
pub trait Coach: Send + Sync {
    /// Human-readable name for this coach (e.g. "llm-judge", "heuristic-v1").
    fn name(&self) -> &str;

    /// Score a single trajectory step in context.
    fn score_step(&self, step: &TrajectoryStep, context: &CoachContext) -> ProcessScore;

    /// Score an entire episode trajectory, returning per-step scores.
    fn score_episode(&self, trajectory: &AgentTrajectory) -> Vec<ProcessScore> {
        let terminal = CoachContext {
            trajectory_so_far: trajectory.steps.clone(),
            transition_patterns: Vec::new(),
            is_terminal: true,
        };
        trajectory
            .steps
            .iter()
            .enumerate()
            .map(|(i, step)| {
                self.score_step(
                    step,
                    &CoachContext {
                        trajectory_so_far: trajectory.steps[..=i].to_vec(),
                        ..terminal.clone()
                    },
                )
            })
            .collect()
    }

    /// Update internal parameters based on trajectory + score feedback.
    fn learn(&mut self, _trajectory: &AgentTrajectory, _scores: &[ProcessScore]) {}
}

