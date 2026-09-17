use crate::core::nt_core_e8::domain_transition::E8TaskType;
use crate::core::nt_core_e8::nt_core_trajectory_prm::{TrajectoryPrm, TrajectoryScoreReport};
use crate::core::nt_core_e8::E8TransitionMatrix;
use crate::core::nt_core_prm::{AgentTrajectory, TrajectoryStep};
use crate::core::{FullReasoningState, MetaState};
use std::collections::HashMap;

// ─── SWE-TRACE Rubric-Based PRM ─────────────────────────────────
//
// Inspired by "SWE-TRACE: Trajectory Reduction and Agentic Criteria
// Evaluation" — a rubric-based Process Reward Model that scores each
// reasoning step across multiple criteria. Each criterion captures a
// different dimension of reasoning quality.

/// Rubric criteria for SWE-TRACE-style process reward modeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RubricCriterion {
    /// Directional coherence: does the transition move toward convergence?
    Directionality,
    /// State diversity: is the system exploring new states?
    Diversity,
    /// Efficiency: is the trajectory free of oscillation/redundancy?
    Efficiency,
    /// Novelty: does this step introduce new reasoning patterns?
    Novelty,
    /// Depth: is the reasoning at appropriate depth for the problem?
    Depth,
    /// Alignment: does the transition align with task type priors?
    TaskAlignment,
}

impl RubricCriterion {
    pub const ALL: [RubricCriterion; 6] = [
        RubricCriterion::Directionality,
        RubricCriterion::Diversity,
        RubricCriterion::Efficiency,
        RubricCriterion::Novelty,
        RubricCriterion::Depth,
        RubricCriterion::TaskAlignment,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            RubricCriterion::Directionality => "Directionality",
            RubricCriterion::Diversity => "Diversity",
            RubricCriterion::Efficiency => "Efficiency",
            RubricCriterion::Novelty => "Novelty",
            RubricCriterion::Depth => "Depth",
            RubricCriterion::TaskAlignment => "TaskAlignment",
        }
    }
}

/// Domain-specific rubric weights for PRM scoring.
/// Each task type has a weight profile reflecting which criteria matter most.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RubricConfig {
    /// Weights for each criterion (must sum to 1.0)
    pub weights: [f64; 6],
    /// Task type this config applies to
    pub task_type: E8TaskType,
}

impl RubricConfig {
    /// Default weights for each task type, derived from SWE-TRACE analysis
    /// of the Complete-FABLE.5-traces-2M corpus.
    pub fn for_task_type(task_type: E8TaskType) -> Self {
        let weights = match task_type {
            // General: balanced
            E8TaskType::General => [0.20, 0.15, 0.20, 0.15, 0.15, 0.15],
            // Reasoning: directionality + depth most important
            E8TaskType::Reasoning => [0.30, 0.10, 0.15, 0.10, 0.25, 0.10],
            // Math: precision + directionality
            E8TaskType::Math => [0.35, 0.10, 0.15, 0.05, 0.25, 0.10],
            // Coding: efficiency + task alignment
            E8TaskType::Coding => [0.20, 0.15, 0.25, 0.10, 0.10, 0.20],
            // Agentic: task alignment + diversity
            E8TaskType::Agentic => [0.15, 0.25, 0.15, 0.15, 0.05, 0.25],
            // Creative: novelty + diversity
            E8TaskType::Creative => [0.10, 0.25, 0.10, 0.35, 0.10, 0.10],
        };
        Self { weights, task_type }
    }

    /// Score a set of rubric assessments using these weights.
    pub fn score(&self, assessments: &[f64; 6]) -> f64 {
        let weighted: f64 = self
            .weights
            .iter()
            .zip(assessments.iter())
            .map(|(w, s)| w * s)
            .sum();
        weighted.max(0.0).min(1.0)
    }

    /// Default config for general tasks.
    pub fn general() -> Self {
        Self::for_task_type(E8TaskType::General)
    }
}

impl Default for RubricConfig {
    fn default() -> Self {
        Self::general()
    }
}

/// Full rubric assessment of a single reasoning transition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RubricAssessment {
    /// Per-criterion scores (0.0–1.0)
    pub scores: [f64; 6],
    /// Weighted composite score
    pub composite: f64,
    /// The rubrics used
    pub config: RubricConfig,
    /// Detailed breakdown as strings
    pub breakdown: [String; 6],
}

/// Observer report: summary of a reasoning trajectory.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObserverReport {
    pub trajectory_len: usize,
    pub distinct_states: usize,
    pub patterns: Vec<String>,
    pub step_qualities: Vec<f64>,
    pub quality_score: f64,
    pub recommended_meta: Option<MetaState>,
    pub capability_deltas: Vec<(String, f64)>,
    pub has_actionable_insight: bool,
    pub critical_patterns: Vec<String>,
    /// Trajectory-level weighted score (step-attention)
    pub trajectory_weighted_score: Option<f64>,
    /// Convergence score from trajectory PRM
    pub convergence_score: Option<f64>,
    /// Whether trajectory PRM recommends early exit
    pub should_exit_early: Option<bool>,
    /// Step-level attention weights
    pub step_attention: Option<Vec<f64>>,
}

impl ObserverReport {
    pub fn has_critical_pattern(&self) -> bool {
        !self.critical_patterns.is_empty()
    }

    pub fn is_degraded(&self) -> bool {
        self.quality_score < 0.3
    }
}

/// Unified observer that wraps PRM + metacognitive observation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OneObserver {
    pub prm: PrmHead,
    pub e8: E8Observer,
    pub step_rewards: Vec<StepReward>,
    pub trajectory_bank: Vec<Vec<u8>>,
    pub analysis_count: usize,
    pub trajectory_history: Vec<FullReasoningState>,
    pub transition_matrix: Option<E8TransitionMatrix>,
    /// Trajectory-aware PRM (ReasonFlux-style)
    pub trajectory_prm: Option<TrajectoryPrm>,
    /// Last trajectory score report
    pub last_trajectory_report: Option<TrajectoryScoreReport>,
    /// Last full ObserverReport (set by analyze() for downstream querying)
    pub last_report: Option<ObserverReport>,
}

impl OneObserver {
    pub fn new() -> Self {
        let mut prm = PrmHead::new();
        prm.init_transition_matrix();
        Self {
            prm,
            e8: E8Observer::new(),
            step_rewards: Vec::new(),
            trajectory_bank: Vec::new(),
            analysis_count: 0,
            trajectory_history: Vec::new(),
            transition_matrix: None,
            trajectory_prm: Some(TrajectoryPrm::default()),
            last_trajectory_report: None,
            last_report: None,
        }
    }

    pub fn with_transition_matrix(mut self, matrix: E8TransitionMatrix) -> Self {
        let m = matrix.clone();
        self.transition_matrix = Some(matrix);
        self.prm.set_transition_matrix(m);
        self
    }

    /// Enable or disable trajectory-aware PRM scoring.
    pub fn with_trajectory_prm(mut self, enabled: bool) -> Self {
        self.trajectory_prm = if enabled {
            Some(TrajectoryPrm::default())
        } else {
            None
        };
        self
    }

    pub fn analyze(
        &mut self,
        trajectory: &[FullReasoningState],
        keywords: &[&str],
    ) -> ObserverReport {
        self.analysis_count += 1;
        self.trajectory_history.extend_from_slice(trajectory);

        let mut deltas = Vec::new();
        let mut patterns = Vec::new();

        // Record transitions into the PRM trajectory buffer
        for (i, state) in trajectory.iter().enumerate() {
            self.prm.record_state(state);
            if i > 0 {
                let prev = trajectory[i - 1].mode.0;
                let cur = state.mode.0;
                self.prm.score_e8_transition(prev, cur, i, &[]);
                if let Some(ref mut tm) = self.transition_matrix {
                    tm.record_transition(prev, cur);
                }
            }
        }

        // Trajectory-aware PRM scoring (ReasonFlux-style)
        let traj_weighted_score: Option<f64>;
        let conv_score: Option<f64>;
        let should_exit_early: Option<bool>;
        let step_attention_vec: Option<Vec<f64>>;
        if let Some(ref mut tp) = self.trajectory_prm {
            let mut agent_traj = AgentTrajectory::new(
                self.analysis_count as u64,
                keywords.first().unwrap_or(&"").to_string(),
            );
            for (i, state) in trajectory.iter().enumerate() {
                agent_traj.push(TrajectoryStep {
                    step_idx: i,
                    specialist: crate::core::nt_core_traits::SpecialistType::ReflectionEngine,
                    e8_mode: state.mode,
                    action: format!("e8_transition_{}", i),
                    input: String::new(),
                    output: String::new(),
                    duration_ms: None,
                    success: true,
                    external_reward: Some(state.meta.0 as f64 / 3.0),
                });
            }
            let step_scores: Vec<f64> = self
                .prm
                .step_scores
                .iter()
                .rev()
                .take(trajectory.len())
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let report = tp.score_trajectory(&agent_traj, &step_scores);
            self.last_trajectory_report = Some(report.clone());
            traj_weighted_score = Some(report.weighted_score);
            conv_score = Some(report.convergence_score);
            should_exit_early = Some(report.should_exit);
            step_attention_vec = Some(report.attention);
        } else {
            traj_weighted_score = None;
            conv_score = None;
            should_exit_early = None;
            step_attention_vec = None;
        }

        if trajectory.len() >= 3 {
            let modes: Vec<u8> = trajectory.iter().map(|s| s.mode.0).collect();
            if modes.len() >= 3
                && modes[modes.len() - 1] == modes[modes.len() - 3]
                && modes[modes.len() - 2] != modes[modes.len() - 1]
            {
                patterns.push("oscillation".to_string());
            }
            if trajectory.last().map(|s| s.meta.0).unwrap_or(0) < 2 {
                patterns.push("stuck".to_string());
            }
        }

        // Check for oscillations via transition matrix
        if let Some(ref tm) = self.transition_matrix {
            if let Some(period) = tm.detect_oscillation(2) {
                patterns.push(format!("oscillation_p{}", period));
            }
            if let Some(period) = tm.detect_oscillation(3) {
                patterns.push(format!("oscillation_p{}", period));
            }
        }

        let mut quality_sum = 0.0;
        for (step, state) in trajectory.iter().enumerate() {
            let score = self
                .prm
                .score_state(state, step, trajectory.len(), keywords);
            quality_sum += score;
            let name = format!("mode_{}", state.mode.0);
            deltas.push((name, score * 0.05));
        }

        let quality_score = if trajectory.is_empty() {
            0.0
        } else {
            quality_sum / trajectory.len() as f64
        };

        let step_qualities: Vec<f64> = trajectory
            .iter()
            .enumerate()
            .map(|(i, s)| self.prm.score_state(s, i, trajectory.len(), keywords))
            .collect();

        let distinct = trajectory
            .iter()
            .map(|s| s.mode.0)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let report = ObserverReport {
            trajectory_len: trajectory.len(),
            distinct_states: distinct,
            patterns,
            step_qualities,
            quality_score,
            recommended_meta: None,
            capability_deltas: deltas,
            has_actionable_insight: quality_score < 0.5,
            critical_patterns: Vec::new(),
            trajectory_weighted_score: traj_weighted_score,
            convergence_score: conv_score,
            should_exit_early,
            step_attention: step_attention_vec,
        };
        self.last_report = Some(report.clone());
        report
    }

    pub fn record_trajectory(&mut self, traj: Vec<FullReasoningState>) {
        self.trajectory_history.extend(traj);
    }
}

impl Default for OneObserver {
    fn default() -> Self {
        Self::new()
    }
}

/// E8 observer for transition analysis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct E8Observer {
    pub transition_counts: HashMap<(u8, u8), usize>,
}

impl E8Observer {
    pub fn new() -> Self {
        Self {
            transition_counts: HashMap::new(),
        }
    }

    pub fn record_transition(&mut self, from: u8, to: u8) {
        *self.transition_counts.entry((from, to)).or_insert(0) += 1;
    }

    pub fn most_common_transition(&self) -> Option<((u8, u8), usize)> {
        self.transition_counts
            .iter()
            .max_by_key(|(_, &c)| c)
            .map(|(k, v)| (*k, *v))
    }
}

impl Default for E8Observer {
    fn default() -> Self {
        Self::new()
    }
}

/// Trajectory-aware PRM head: scores reasoning steps/transitions with
/// ReasonFlux-inspired step-level tracking and oscillation detection.
///
/// Unlike the previous stateless PrmHead, this version maintains a trajectory
/// buffer and scores each transition in context of the full reasoning path.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrmHead {
    /// Recent mode sequence for transition-aware scoring
    pub mode_buffer: Vec<u8>,
    /// Step scores recorded along the trajectory
    pub step_scores: Vec<f64>,
    /// Number of direction changes detected (flip of ABST bit)
    pub direction_changes: usize,
    /// Total transitions scored
    pub total_transitions: u64,
    /// Transition matrix for next-state prediction bias
    pub transition_matrix: Option<E8TransitionMatrix>,
    /// Maximum buffer size
    pub max_buffer: usize,
    /// SWE-TRACE rubric configuration for per-criterion scoring
    pub rubric_config: RubricConfig,
    /// Rubric assessment history (one per scored transition)
    pub rubric_history: Vec<RubricAssessment>,
}

impl PrmHead {
    pub fn new() -> Self {
        Self {
            mode_buffer: Vec::with_capacity(32),
            step_scores: Vec::with_capacity(32),
            direction_changes: 0,
            total_transitions: 0,
            transition_matrix: None,
            max_buffer: 32,
            rubric_config: RubricConfig::general(),
            rubric_history: Vec::with_capacity(32),
        }
    }

    /// Initialize the transition matrix with trace patterns.
    pub fn init_transition_matrix(&mut self) {
        let mut tm = E8TransitionMatrix::new();
        tm.init_from_trace_patterns();
        self.transition_matrix = Some(tm);
    }

    /// Set the transition matrix externally.
    pub fn set_transition_matrix(&mut self, tm: E8TransitionMatrix) {
        self.transition_matrix = Some(tm);
    }

    /// Get immutable reference to transition matrix.
    pub fn transition_matrix(&self) -> Option<&E8TransitionMatrix> {
        self.transition_matrix.as_ref()
    }

    /// Get mutable reference to transition matrix.
    pub fn transition_matrix_mut(&mut self) -> Option<&mut E8TransitionMatrix> {
        self.transition_matrix.as_mut()
    }

    /// Record a reasoning state into the trajectory buffer.
    pub fn record_state(&mut self, state: &FullReasoningState) {
        self.mode_buffer.push(state.mode.0);
        if self.mode_buffer.len() > self.max_buffer {
            self.mode_buffer.remove(0);
        }
    }

    /// Trajectory-aware transition scoring.
    ///
    /// Scores incorporate:
    /// 1. Direction change reward (flipping ABST axis = exploring new paradigm)
    /// 2. Magnitude of exploration (distance in state space)
    /// 3. Oscillation penalty (repeating modes wastes compute)
    /// 4. Transition matrix likelihood (empirically good transitions get bonus)
    /// 5. Step position bonus (early steps get lower base, later steps higher)
    pub fn score_e8_transition(
        &mut self,
        from: u8,
        to: u8,
        step: usize,
        task_embedding: &[f64],
    ) -> f64 {
        self.total_transitions += 1;

        // 1. Direction change reward (flip ABST bit = paradigm shift)
        let direction_change = if (from & 0x20) != (to & 0x20) {
            0.25
        } else {
            0.0
        };

        // 2. Magnitude of exploration (distance / 64)
        let magnitude = (to as f64 - from as f64).abs() / 64.0;

        // 3. Oscillation penalty: if we've seen this to→from pattern recently
        let osc_penalty = if self.mode_buffer.len() >= 3 {
            let len = self.mode_buffer.len();
            if self.mode_buffer[len - 2] == to && self.mode_buffer[len - 3] == from {
                -0.15 // oscillating: penalize
            } else if self.mode_buffer[len - 1] == from && self.mode_buffer[len - 2] == to {
                -0.10 // return oscillation: slight penalty
            } else {
                0.0
            }
        } else {
            0.0
        };

        // 4. Transition matrix likelihood bonus
        let tm_bonus = self
            .transition_matrix
            .as_ref()
            .map(|tm| tm.transition_prob(from, to) * 0.5)
            .unwrap_or(0.0);

        // 5. Step position: score increases with step (later steps more valuable)
        let step_bonus = (step as f64 / 64.0).min(0.1);

        // 6. Task alignment (if embedding provided)
        let task_alignment = if !task_embedding.is_empty() {
            let task_mean = task_embedding.iter().sum::<f64>() / task_embedding.len() as f64;
            0.2 * (1.0 - (task_mean - (to as f64 / 64.0)).abs())
        } else {
            0.1
        };

        let raw_score = direction_change
            + magnitude * 0.3
            + osc_penalty
            + tm_bonus
            + step_bonus
            + task_alignment;
        let clamped = raw_score.max(0.0).min(1.0);
        let final_score = (clamped * 100.0).round() / 100.0;

        self.step_scores.push(final_score);
        final_score
    }

    /// Step-aware state scoring.
    ///
    /// Scores incorporate:
    /// 1. Base meta-state score
    /// 2. Trajectory position (early = lower confidence, later = higher)
    /// 3. Mode novelty (new modes get a bonus for exploration)
    /// 4. Recent transition quality
    pub fn score_state(
        &mut self,
        state: &FullReasoningState,
        step: usize,
        total_steps: usize,
        _keywords: &[&str],
    ) -> f64 {
        // 1. Meta-state base (0.0-0.75)
        let meta_base = (state.meta.0 as f64) * 0.25;

        // 2. Trajectory position: normalize to [0.0, 0.15]
        let pos_bonus = if total_steps > 1 {
            0.15 * (step as f64 / (total_steps - 1) as f64)
        } else {
            0.075
        };

        // 3. Mode novelty: check if this mode is new in the buffer
        let novelty = if self.mode_buffer.len() >= 2 {
            let current = state.mode.0;
            if self.mode_buffer.iter().rev().skip(1).any(|&m| m == current) {
                0.0 // already visited
            } else {
                0.1 // new mode: exploration bonus
            }
        } else {
            0.05
        };

        // 4. Recent transition quality (from step_scores)
        let transition_quality = if step > 0 && !self.step_scores.is_empty() {
            let recent_scores: Vec<&f64> = self.step_scores.iter().rev().take(3).collect();
            recent_scores.iter().copied().copied().sum::<f64>() / recent_scores.len() as f64 * 0.1
        } else {
            0.05
        };

        let score = meta_base + pos_bonus + novelty + transition_quality;
        score.max(0.0).min(1.0)
    }

    /// ─── SWE-TRACE Rubric-Based Scoring ──────────────────────────
    ///
    /// Each transition is scored across 6 criteria (Directionality, Diversity,
    /// Efficiency, Novelty, Depth, TaskAlignment) with task-type-adaptive weights.
    /// Set the rubric config for task-type-aware scoring.
    pub fn set_rubric_config(&mut self, task_type: E8TaskType) {
        self.rubric_config = RubricConfig::for_task_type(task_type);
    }

    /// Score a transition using the rubric-based PRM.
    /// Returns the rubic assessment with per-criterion breakdown.
    pub fn assess_transition_rubrics(
        &self,
        from: u8,
        to: u8,
        step: usize,
        total_steps: usize,
    ) -> RubricAssessment {
        let config = &self.rubric_config;

        // 1. Directionality: moving toward task-appropriate blocks
        let directionality = {
            let block_from = from & 0xF8;
            let block_to = to & 0xF8;
            // Block transitions are directional, within-block is incremental
            if block_from == block_to {
                0.5 // within-block refinement
            } else if block_to < block_from {
                // Moving "up" toward synthesis (lower block numbers)
                let progress = ((block_from - block_to) as f64 / 64.0).min(1.0);
                0.5 + progress * 0.5
            } else {
                // Moving "down" toward decomposition (higher block numbers)
                0.4 // may be backtracking, but sometimes necessary
            }
        };

        // 2. Diversity: exploring distinct states
        let diversity = {
            let distinct = {
                let mut seen = std::collections::HashSet::new();
                for &m in &self.mode_buffer {
                    seen.insert(m & 0xF8);
                }
                seen.len()
            };
            let diversity_ratio = if self.mode_buffer.is_empty() {
                0.5
            } else {
                (distinct as f64 / self.mode_buffer.len() as f64).min(1.0)
            };
            // New state gets bonus
            let is_new = !self.mode_buffer.iter().rev().skip(1).any(|&m| m == to);
            if is_new {
                (0.5 + diversity_ratio * 0.5).min(1.0)
            } else {
                diversity_ratio * 0.6
            }
        };

        // 3. Efficiency: no oscillation, good transition matrix likelihood
        let efficiency = {
            let osc_penalty = if self.mode_buffer.len() >= 3 {
                let len = self.mode_buffer.len();
                if self.mode_buffer[len - 2] == to && self.mode_buffer[len - 3] == from {
                    0.3 // oscillation: severe penalty
                } else if self.mode_buffer[len - 1] == from && self.mode_buffer[len - 2] == to {
                    0.4 // return oscillation: moderate penalty
                } else {
                    0.8
                }
            } else {
                0.8
            };
            let tm_bonus = self
                .transition_matrix
                .as_ref()
                .map(|tm| tm.transition_prob(from, to))
                .unwrap_or(1.0 / 64.0);
            (osc_penalty + tm_bonus * 0.2).min(1.0)
        };

        // 4. Novelty: new modes and direction changes
        let novelty = {
            let direction_change = if (from & 0x20) != (to & 0x20) {
                0.3
            } else {
                0.0
            };
            let magnitude = (to as f64 - from as f64).abs() / 64.0;
            let is_new_block = (from & 0xF8) != (to & 0xF8);
            let block_bonus = if is_new_block { 0.2 } else { 0.0 };
            (direction_change + magnitude * 0.5 + block_bonus).min(1.0)
        };

        // 5. Depth: appropriate for step position
        let depth = {
            if total_steps > 0 {
                let progress = step as f64 / total_steps as f64;
                if progress < 0.2 {
                    // Early: prefer exploration (wider jumps)
                    let jump_mag = (to as f64 - from as f64).abs() / 64.0;
                    0.3 + jump_mag * 0.7
                } else if progress < 0.7 {
                    // Middle: prefer systematic progress
                    0.6 + ((to & 0x07) as f64 / 7.0) * 0.3
                } else {
                    // Late: converge toward synthesis (block 0-7)
                    let target_block = to & 0xF8;
                    if target_block <= 8 {
                        0.9 // converging
                    } else {
                        0.5 // still exploring late
                    }
                }
            } else {
                0.5
            }
        };

        // 6. Task Alignment: how well this matches the expected chain
        let task_alignment = {
            let chain = config.task_type.e8_chain();
            let chain_len = chain.len();
            // Determine where in the chain we are based on step position
            let expected_idx = if total_steps > 0 {
                ((step as f64 / total_steps as f64) * (chain_len - 1) as f64).round() as usize
            } else {
                0
            };
            let expected_state = chain[expected_idx.min(chain_len - 1)];
            let block_match = (to & 0xF8) == (expected_state & 0xF8);
            if block_match {
                0.8 + (1.0 - (to as f64 - expected_state as f64).abs() / 64.0) * 0.2
            } else {
                // Check if adjacent to expected block
                let diff = ((to & 0xF8) as i16 - (expected_state & 0xF8) as i16).abs();
                if diff <= 8 {
                    0.5
                } else {
                    0.2
                }
            }
        };

        let scores = [
            directionality,
            diversity,
            efficiency,
            novelty,
            depth,
            task_alignment,
        ];
        let composite = config.score(&scores);

        let breakdown = [
            format!("{:.2}", directionality),
            format!("{:.2}", diversity),
            format!("{:.2}", efficiency),
            format!("{:.2}", novelty),
            format!("{:.2}", depth),
            format!("{:.2}", task_alignment),
        ];

        RubricAssessment {
            scores,
            composite,
            config: config.clone(),
            breakdown,
        }
    }

    /// Score a transition using rubrics and record in history.
    pub fn score_transition_with_rubrics(
        &mut self,
        from: u8,
        to: u8,
        step: usize,
        total_steps: usize,
    ) -> f64 {
        self.total_transitions += 1;
        let assessment = self.assess_transition_rubrics(from, to, step, total_steps);
        let score = assessment.composite;
        self.rubric_history.push(assessment);
        self.step_scores.push(score);
        score
    }

    /// Get the last N rubric assessments.
    pub fn last_rubrics(&self, n: usize) -> &[RubricAssessment] {
        let start = self.rubric_history.len().saturating_sub(n);
        &self.rubric_history[start..]
    }

    /// Get average rubric scores across all scored transitions.
    pub fn average_rubric_scores(&self) -> [f64; 6] {
        if self.rubric_history.is_empty() {
            return [0.0; 6];
        }
        let mut sums = [0.0f64; 6];
        for assessment in &self.rubric_history {
            for (i, &s) in assessment.scores.iter().enumerate() {
                sums[i] += s;
            }
        }
        let n = self.rubric_history.len() as f64;
        for s in &mut sums {
            *s /= n;
        }
        sums
    }

    /// Clear trajectory buffer, step scores, and rubric history.
    pub fn clear(&mut self) {
        self.mode_buffer.clear();
        self.step_scores.clear();
        self.direction_changes = 0;
        self.rubric_history.clear();
    }
}

impl Default for PrmHead {
    fn default() -> Self {
        Self::new()
    }
}

/// PRM observer for the reasoning engine.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrmObserver {
    pub head: PrmHead,
    pub rewards: Vec<StepReward>,
    pub feedback_history: Vec<StepRewardFeedback>,
}

impl PrmObserver {
    pub fn new() -> Self {
        Self {
            head: PrmHead::new(),
            rewards: Vec::new(),
            feedback_history: Vec::new(),
        }
    }

    pub fn record_reward(&mut self, reward: StepReward) {
        self.rewards.push(reward);
    }

    pub fn feed_step_reward(&mut self, reward: StepReward) {
        self.rewards.push(reward);
    }

    pub fn get_step_rewards(&self) -> &[StepReward] {
        &self.rewards
    }

    pub fn clear_step_rewards(&mut self) {
        self.rewards.clear();
    }
}

impl Default for PrmObserver {
    fn default() -> Self {
        Self::new()
    }
}

/// Step-level reward for process reward model.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StepReward {
    pub step: usize,
    pub score: f64,
    pub confidence: f64,
}

impl StepReward {
    pub fn new(step: usize, score: f64, confidence: f64) -> Self {
        Self {
            step,
            score,
            confidence,
        }
    }
}

/// Feedback from step reward evaluation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StepRewardFeedback {
    Correct(usize),
    Incorrect(usize, String),
    Partial(usize, f64),
    Skip(usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hex::{FullReasoningState, MetaState, ReasoningHexagram};

    fn make_state(mode: u8, meta: u8) -> FullReasoningState {
        FullReasoningState::new(ReasoningHexagram::new(mode % 64), MetaState::new(meta % 4))
    }

    #[test]
    fn test_observer_config_default() {
        let cfg = RubricConfig::default();
        assert_eq!(cfg.task_type, E8TaskType::General);
        let expected = [0.20, 0.15, 0.20, 0.15, 0.15, 0.15];
        for (i, &w) in cfg.weights.iter().enumerate() {
            assert!(
                (w - expected[i]).abs() < 1e-10,
                "weight {i} mismatch: {} != {}",
                w,
                expected[i]
            );
        }
        let sum: f64 = cfg.weights.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-10,
            "weights must sum to 1.0, got {sum}"
        );
    }

    #[test]
    fn test_observer_initial_state() {
        let observer = OneObserver::new();
        assert_eq!(observer.analysis_count, 0);
        assert!(observer.step_rewards.is_empty());
        assert!(observer.trajectory_bank.is_empty());
        assert!(observer.trajectory_history.is_empty());
        assert!(observer.transition_matrix.is_none());
        assert!(observer.last_trajectory_report.is_none());
        assert!(observer.trajectory_prm.is_some());
        assert!(observer.prm.step_scores.is_empty());
        assert_eq!(observer.prm.total_transitions, 0);
    }

    #[test]
    fn test_prm_head_initialization() {
        let prm = PrmHead::new();
        assert!(prm.mode_buffer.is_empty());
        assert!(prm.step_scores.is_empty());
        assert_eq!(prm.direction_changes, 0);
        assert_eq!(prm.total_transitions, 0);
        assert!(prm.transition_matrix.is_none());
        assert_eq!(prm.max_buffer, 32);
        assert_eq!(prm.rubric_config.task_type, E8TaskType::General);

        let mut prm = PrmHead::new();
        prm.init_transition_matrix();
        assert!(prm.transition_matrix.is_some());

        let matrix = prm.transition_matrix();
        assert!(matrix.is_some());
        assert_eq!(matrix.unwrap().max_recent, 256);
    }

    #[test]
    fn test_process_reward_scoring() {
        let mut prm = PrmHead::new();
        prm.init_transition_matrix();

        let score_forward = prm.score_e8_transition(0, 10, 1, &[]);
        assert!(
            score_forward >= 0.0 && score_forward <= 1.0,
            "forward transition score {score_forward} out of [0,1]"
        );

        let score_self = prm.score_e8_transition(10, 10, 2, &[]);
        assert!(
            score_self >= 0.0 && score_self <= 1.0,
            "self transition score {score_self} out of [0,1]"
        );

        let score_back = prm.score_e8_transition(10, 0, 3, &[]);
        assert!(
            score_back >= 0.0 && score_back <= 1.0,
            "backward transition score {score_back} out of [0,1]"
        );

        assert!(
            score_forward != score_back || score_forward != score_self,
            "different transitions should yield different scores"
        );

        assert_eq!(prm.total_transitions, 3);
        assert_eq!(prm.step_scores.len(), 3);
    }

    #[test]
    fn test_observer_report_structure() {
        let mut observer = OneObserver::new();
        observer.prm.init_transition_matrix();

        let trajectory = vec![
            make_state(0, 0),
            make_state(10, 1),
            make_state(20, 2),
            make_state(30, 1),
            make_state(40, 2),
        ];

        let report = observer.analyze(&trajectory, &["test"]);

        assert_eq!(report.trajectory_len, 5);
        assert!(report.distinct_states > 0);
        assert!(!report.step_qualities.is_empty());
        assert_eq!(report.step_qualities.len(), 5);
        assert!(report.quality_score >= 0.0 && report.quality_score <= 1.0);
        assert!(!report.capability_deltas.is_empty());
        assert!(report.trajectory_weighted_score.is_some());
        assert!(report.convergence_score.is_some());
        assert!(report.should_exit_early.is_some());
        assert!(report.step_attention.is_some());
        assert!(!report.has_critical_pattern());

        let degraded = ObserverReport {
            quality_score: 0.2,
            ..report.clone()
        };
        assert!(degraded.is_degraded());

        let healthy = ObserverReport {
            quality_score: 0.8,
            ..report.clone()
        };
        assert!(!healthy.is_degraded());

        let critical = ObserverReport {
            critical_patterns: vec!["error".to_string()],
            ..report
        };
        assert!(critical.has_critical_pattern());
    }
}
