#![allow(dead_code)]
use crate::core::nt_core_consciousness::{
    sleep_gate::SleepGate, CognitiveLoadMonitor, ConsciousnessStream, FirstPersonRef, InnerCritic,
    NarrativeSelf, SpeciousPresent, ValenceAxis,
};
use crate::neotrix::nt_expert_routing::TaskType;
use crate::neotrix::nt_mind::core::RewardSource;
use crate::neotrix::nt_mind::self_edit::MicroEdit;
use crate::neotrix::nt_mind::self_iterating::aging_monitor::AgingMonitor;
use crate::neotrix::nt_mind::self_iterating::goal_contract::{GoalContract, PhaseEvidence};
use crate::neotrix::nt_mind::self_iterating::harness_adapter::HarnessAdapter;
use crate::neotrix::nt_mind::self_iterating::pipeline::{BrainSnapshot, StageResult};
use crate::neotrix::nt_mind::self_iterating::skillopt::{
    LrScheduler, RejectedEditBuffer, ValidationGate,
};
use std::collections::VecDeque;

/// Transient per-task state (pub(crate) access only)
pub(crate) struct TaskScratch {
    pub current_task: String,
    pub current_task_type: TaskType,
    pub task_embedding: Option<Vec<f64>>,
    pub external_reward: Option<f64>,
    pub snapshot: Option<BrainSnapshot>,
    pub reward: f64,
    pub reward_source: RewardSource,
    pub micro_edits: Vec<MicroEdit>,
}

/// Evo pipeline internal state
pub(crate) struct EvoPipelineState {
    pub task_horizon: usize,
    pub velocity_history: VecDeque<f64>,
    pub velocity_momentum: f64,
    pub agent_specs: Vec<String>,
    pub diversity_scores: VecDeque<f64>,
    pub diversity_curiosity: f64,
    pub dense_advantage: f64,
    pub open_source_insights: Option<String>,
    pub open_source_edits: Vec<MicroEdit>,
}

/// Consciousness system state
pub(crate) struct ConsciousnessState {
    pub first_person: FirstPersonRef,
    pub specious_present: SpeciousPresent,
    pub consciousness_stream: ConsciousnessStream,
    pub inner_critic: InnerCritic,
    pub cognitive_load: CognitiveLoadMonitor,
    pub narrative_self: NarrativeSelf,
    pub sleep_gate: SleepGate,
    pub valence_axis: ValenceAxis,
}

/// Goal contract tracking state
pub(crate) struct GoalContractState {
    pub goal_contract: Option<GoalContract>,
    pub phase_evidence: VecDeque<PhaseEvidence>,
    pub goal_complete: bool,
}

/// 6-dimensional self-review audit result produced by SelfReviewStage.
#[derive(Debug, Clone, Default)]
pub(crate) struct SelfReviewReport {
    /// D1: Architecture cycle health (0.0 = all cycles increment, 1.0 = all broken)
    pub cycle_risk: f64,
    /// D2: Panic path density (ratio of unwrap/panic to total fn calls)
    pub panic_density: f64,
    /// D3: Unbounded collection ratio (collections without MAX_*/drain)
    pub unbounded_ratio: f64,
    /// D4: Dead code ratio (orphan modules / total modules)
    pub dead_code_ratio: f64,
    /// D5: Shutdown signal coverage (1.0 = fully covered)
    pub shutdown_coverage: f64,
    /// D6: Feature gate integrity (1.0 = all cfg features declared)
    pub feature_integrity: f64,
    /// D7: External knowledge coverage — ratio of weak dimensions that triggered
    /// successful external search → code fix mapping. 0.0 = isolated review
    /// (defects found but no external reference absorbed), 1.0 = every defect
    /// triggered external literature/project search that yielded actionable fixes.
    pub external_ref_coverage: f64,
    /// External references found for each weak dimension (paper/ project URLs)
    pub external_references: Vec<String>,
    /// Composite self-review health score (0.0 = critical, 1.0 = healthy)
    pub composite_health: f64,
    /// When this audit was produced (brain iteration)
    pub iteration: u64,
}

/// SEAL RL training internals
#[derive(Default)]
pub(crate) struct SealRlState {
    pub lr_scheduler: LrScheduler,
    pub validation_gate: ValidationGate,
    pub rejected_buffer: RejectedEditBuffer,
    pub aging_monitor: AgingMonitor,
    pub harness_adapter: HarnessAdapter,
    pub stage_results: Vec<StageResult>,
    /// Most recent self-review report (populated by SelfReviewStage)
    pub self_review_report: Option<SelfReviewReport>,
    /// Previous cycle's composite health for tracking improvement delta
    pub prev_composite_health: Option<f64>,
    /// External reference hits: (dimension_name, paper_or_project_url, description)
    /// Fed by external search agents during parallel literature search phase.
    /// SelfReviewStage uses these to compute D7 external_ref_coverage.
    pub external_references: Vec<(String, String, String)>,
}

/// Max external references to keep before pruning oldest
const MAX_EXTERNAL_REFS: usize = 100;

impl SealRlState {
    /// Register an external reference found for a given audit dimension.
    /// Called by external search agents running in parallel with fix waves.
    pub fn add_external_reference(&mut self, dim: &str, url: &str, description: &str) {
        self.external_references
            .push((dim.to_string(), url.to_string(), description.to_string()));
        if self.external_references.len() > MAX_EXTERNAL_REFS {
            self.external_references.remove(0);
        }
    }

    /// Number of dimensions (0-6) that have at least one external reference mapped.
    /// Used by SelfReviewStage to compute D7 external_ref_coverage.
    pub fn covered_dimension_count(&self) -> usize {
        let dims: std::collections::HashSet<&str> = self
            .external_references
            .iter()
            .map(|(d, _, _)| d.as_str())
            .collect();
        dims.len()
    }
}

impl Default for TaskScratch {
    fn default() -> Self {
        Self {
            current_task: String::new(),
            current_task_type: TaskType::General,
            task_embedding: None,
            external_reward: None,
            snapshot: None,
            reward: 0.0,
            reward_source: RewardSource::Internal,
            micro_edits: Vec::new(),
        }
    }
}

impl Default for EvoPipelineState {
    fn default() -> Self {
        Self {
            task_horizon: 1,
            velocity_history: VecDeque::with_capacity(8),
            velocity_momentum: 0.0,
            agent_specs: Vec::new(),
            diversity_scores: VecDeque::with_capacity(16),
            diversity_curiosity: 0.0,
            dense_advantage: 0.0,
            open_source_insights: None,
            open_source_edits: Vec::new(),
        }
    }
}

impl Default for ConsciousnessState {
    fn default() -> Self {
        Self {
            first_person: FirstPersonRef::bootstrap(0),
            specious_present: SpeciousPresent::new(5),
            consciousness_stream: ConsciousnessStream::new(1024),
            inner_critic: InnerCritic::new(),
            cognitive_load: CognitiveLoadMonitor::new(),
            narrative_self: NarrativeSelf::new(),
            sleep_gate: SleepGate::new(),
            valence_axis: ValenceAxis::new(),
        }
    }
}

impl Default for GoalContractState {
    fn default() -> Self {
        Self {
            goal_contract: None,
            phase_evidence: VecDeque::with_capacity(32),
            goal_complete: false,
        }
    }
}
