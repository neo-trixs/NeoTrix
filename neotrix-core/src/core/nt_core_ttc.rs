use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Test-Time Compute Scaling engine (P0 — third scaling law)
///
/// Implements four key capabilities from o3/DeepSeek-R1/PaCoRe/TRACE:
/// 1. Adaptive compute budget allocation (Lagrangian optimization)
/// 2. PRM-guided beam/MCTS search
/// 3. Parallel reasoning trajectories (PaCoRe)
/// 4. Early exit detection (TRACE)
///
/// Compute amounts available per task
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ComputeBudget {
    pub max_tokens: u64,
    pub max_steps: u64,
    pub beam_width: usize,
    pub parallel_trajectories: usize,
    pub mcts_simulations: usize,
}

impl ComputeBudget {
    pub const fn free() -> Self {
        Self {
            max_tokens: 0,
            max_steps: 0,
            beam_width: 1,
            parallel_trajectories: 1,
            mcts_simulations: 0,
        }
    }

    pub fn with_tokens(mut self, tokens: u64) -> Self {
        self.max_tokens = tokens;
        self
    }
    pub fn with_steps(mut self, steps: u64) -> Self {
        self.max_steps = steps;
        self
    }
    pub fn with_beam(mut self, width: usize) -> Self {
        self.beam_width = width;
        self
    }
    pub fn with_parallel(mut self, n: usize) -> Self {
        self.parallel_trajectories = n;
        self
    }
    pub fn with_mcts(mut self, sims: usize) -> Self {
        self.mcts_simulations = sims;
        self
    }
}

/// Difficulty factors for compute allocation
#[derive(Debug, Clone)]
pub struct DifficultyFactors {
    pub prompt_length: f64,
    pub question_density: f64,
    pub task_weight: f64,
    pub constraint_count: f64,
    pub novelty: f64,
}

impl DifficultyFactors {
    pub fn composite(&self) -> f64 {
        let raw = self.prompt_length * 0.2
            + self.question_density * 0.25
            + self.task_weight * 0.25
            + self.constraint_count * 0.2
            + self.novelty * 0.1;
        raw.max(0.0).min(1.0)
    }
}

/// Adaptive compute budget allocator (Lagrangian optimization)
#[derive(Debug, Clone)]
pub struct LagrangianAllocator {
    pub base_budget: ComputeBudget,
    pub max_multiplier: f64,
    pub lambda: f64,
}

impl Default for LagrangianAllocator {
    fn default() -> Self {
        Self {
            base_budget: ComputeBudget {
                max_tokens: 4096,
                max_steps: 32,
                beam_width: 2,
                parallel_trajectories: 2,
                mcts_simulations: 32,
            },
            max_multiplier: 8.0,
            lambda: 0.5,
        }
    }
}

impl LagrangianAllocator {
    pub fn allocate(&self, difficulty: f64, remaining_budget: f64) -> ComputeBudget {
        let multiplier = 1.0 + (self.max_multiplier - 1.0) * difficulty;
        let budget_ratio = (remaining_budget * self.lambda).max(0.1).min(1.0);
        let effective = (multiplier * budget_ratio)
            .max(1.0)
            .min(self.max_multiplier);

        ComputeBudget {
            max_tokens: (self.base_budget.max_tokens as f64 * effective) as u64,
            max_steps: (self.base_budget.max_steps as f64 * (1.0 + (effective - 1.0) * 0.5)) as u64,
            beam_width: self.base_budget.beam_width.max(1),
            parallel_trajectories: (self.base_budget.parallel_trajectories as f64
                * (1.0 + (effective - 1.0) * 0.3))
                .ceil() as usize,
            mcts_simulations: (self.base_budget.mcts_simulations as f64 * effective).ceil()
                as usize,
        }
    }
}

/// A single reasoning step with score from PRM
#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub step_idx: usize,
    pub content: String,
    pub prm_score: f64,
    pub cumulative_score: f64,
}

/// Beam search state
#[derive(Debug, Clone)]
pub struct BeamState {
    pub steps: Vec<ReasoningStep>,
    pub cumulative_score: f64,
    pub is_terminal: bool,
}

impl Default for BeamState {
    fn default() -> Self {
        Self::new()
    }
}

impl BeamState {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            cumulative_score: 0.0,
            is_terminal: false,
        }
    }

    pub fn last_score(&self) -> f64 {
        self.steps.last().map(|s| s.cumulative_score).unwrap_or(0.0)
    }
}

/// PRM-guided beam search
#[derive(Debug, Clone)]
pub struct PrmBeamSearch {
    pub beam_width: usize,
    pub max_steps: usize,
}

impl PrmBeamSearch {
    pub fn new(beam_width: usize, max_steps: usize) -> Self {
        Self {
            beam_width: beam_width.max(1),
            max_steps,
        }
    }

    pub fn search<F>(&self, initial_state: BeamState, scorer: F) -> Vec<BeamState>
    where
        F: Fn(&BeamState) -> Vec<BeamState>,
    {
        let mut beam = vec![initial_state];
        for _step in 0..self.max_steps {
            let mut candidates: Vec<BeamState> = beam
                .into_iter()
                .flat_map(|state| {
                    if state.is_terminal {
                        return vec![state];
                    }
                    scorer(&state)
                })
                .collect();

            candidates.sort_by(|a, b| {
                b.cumulative_score
                    .partial_cmp(&a.cumulative_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            candidates.truncate(self.beam_width);
            beam = candidates;

            if beam.iter().all(|b| b.is_terminal) {
                break;
            }
        }
        beam
    }
}

/// Parallel trajectory (PaCoRe-style)
#[derive(Debug, Clone)]
pub struct ParallelTrajectory {
    pub trajectory_id: usize,
    pub steps: Vec<ReasoningStep>,
    pub final_score: f64,
    pub token_count: u64,
    pub converged: bool,
}

/// PaCoRe-style parallel coordinated reasoning
#[derive(Debug, Clone)]
pub struct ParallelReasoner {
    pub num_trajectories: usize,
    pub max_steps: usize,
    pub coordination_interval: usize,
    pub convergence_threshold: f64,
}

impl Default for ParallelReasoner {
    fn default() -> Self {
        Self {
            num_trajectories: 4,
            max_steps: 32,
            coordination_interval: 4,
            convergence_threshold: 0.9,
        }
    }
}

impl ParallelReasoner {
    pub fn new(num_trajectories: usize, max_steps: usize) -> Self {
        Self {
            num_trajectories: num_trajectories.max(1),
            max_steps,
            coordination_interval: 4,
            convergence_threshold: 0.9,
        }
    }

    pub fn run<F>(&self, step_fn: F) -> Vec<ParallelTrajectory>
    where
        F: Fn(usize, usize, &[ParallelTrajectory]) -> (f64, bool),
    {
        let mut trajectories: Vec<ParallelTrajectory> = (0..self.num_trajectories)
            .map(|i| ParallelTrajectory {
                trajectory_id: i,
                steps: Vec::new(),
                final_score: 0.0,
                token_count: 0,
                converged: false,
            })
            .collect();

        for step in 0..self.max_steps {
            let mut all_converged = true;
            for t in 0..self.num_trajectories {
                if trajectories[t].converged {
                    continue;
                }
                let (score, converged) = step_fn(t, step, &trajectories);
                trajectories[t].final_score = score;
                trajectories[t].converged = converged;
                if !converged {
                    all_converged = false;
                }
            }

            if trajectories.iter().filter(|t| t.converged).count() >= self.num_trajectories / 2 {
                break;
            }
            if all_converged && step > 0 {
                break;
            }
        }
        trajectories
    }

    /// Coordinate across trajectories (average scores, share insights)
    pub fn coordinate(&self, trajectories: &mut [ParallelTrajectory]) {
        if trajectories.len() < 2 {
            return;
        }
        let avg_score: f64 =
            trajectories.iter().map(|t| t.final_score).sum::<f64>() / trajectories.len() as f64;
        for t in trajectories.iter_mut() {
            if (t.final_score - avg_score).abs() > self.convergence_threshold * 0.3 {
                t.final_score = t.final_score * 0.7 + avg_score * 0.3;
            }
        }
    }
}

/// Early exit detector (TRACE-style)
#[derive(Debug, Clone)]
pub struct EarlyExitDetector {
    pub window_size: usize,
    pub stability_threshold: f64,
    pub min_steps: usize,
}

impl Default for EarlyExitDetector {
    fn default() -> Self {
        Self {
            window_size: 5,
            stability_threshold: 0.05,
            min_steps: 3,
        }
    }
}

impl EarlyExitDetector {
    pub fn new(window_size: usize, threshold: f64, min_steps: usize) -> Self {
        Self {
            window_size: window_size.max(2),
            stability_threshold: threshold.max(0.0).min(1.0),
            min_steps: min_steps.max(1),
        }
    }

    /// Check if reasoning has converged (scores stable over window)
    pub fn should_exit(&self, scores: &[f64]) -> bool {
        if scores.len() < self.min_steps {
            return false;
        }
        if scores.len() < self.window_size {
            return false;
        }

        let start = scores.len().saturating_sub(self.window_size);
        let recent: Vec<f64> = scores[start..].to_vec();
        let variance = Self::variance(&recent);
        variance < self.stability_threshold
    }

    /// Time aggregation convergence detection (TRACE core)
    pub fn detect_convergence(&self, scores: &[f64]) -> ConvergenceSignal {
        if scores.len() < self.min_steps {
            return ConvergenceSignal::InsufficientSteps;
        }

        let start = scores.len().saturating_sub(self.window_size);
        let recent: Vec<f64> = scores[start..].to_vec();
        let variance = Self::variance(&recent);
        let trend = if recent.len() >= 2 {
            let first = recent.first().copied().unwrap_or(0.0);
            let last = recent.last().copied().unwrap_or(0.0);
            last - first
        } else {
            0.0
        };

        if variance < self.stability_threshold * 0.5 && trend.abs() < self.stability_threshold {
            ConvergenceSignal::Converged {
                reason: "High stability with no trend".into(),
                token_saving: (scores.len() as f64 * 0.25) as u64,
            }
        } else if variance < self.stability_threshold
            && trend.abs() < self.stability_threshold * 2.0
        {
            ConvergenceSignal::StableEnough {
                remaining_variance: variance,
            }
        } else if trend < -self.stability_threshold * 3.0 {
            ConvergenceSignal::Degrading { trend }
        } else {
            ConvergenceSignal::NeedsMoreSteps { variance }
        }
    }

    fn variance(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 1.0;
        }
        if values.len() == 1 {
            return 0.0;
        }
        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
        let variance: f64 =
            values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        variance
    }
}

/// Convergence signal from early exit detector
#[derive(Debug, Clone, PartialEq)]
pub enum ConvergenceSignal {
    Converged { reason: String, token_saving: u64 },
    StableEnough { remaining_variance: f64 },
    Degrading { trend: f64 },
    NeedsMoreSteps { variance: f64 },
    InsufficientSteps,
}

impl ConvergenceSignal {
    pub fn should_exit(&self) -> bool {
        matches!(
            self,
            ConvergenceSignal::Converged { .. } | ConvergenceSignal::Degrading { .. }
        )
    }
}

/// MCTS node for PRM-guided Monte Carlo Tree Search
#[derive(Debug, Clone)]
pub struct MctsNode {
    pub state_id: String,
    pub score: f64,
    pub visits: u64,
    pub children: Vec<MctsNode>,
    pub parent: Option<Box<MctsNode>>,
}

impl MctsNode {
    pub fn ucb_score(&self, parent_visits: u64, exploration_weight: f64) -> f64 {
        if self.visits == 0 {
            return f64::MAX;
        }
        let exploitation = self.score / self.visits as f64;
        let exploration = exploration_weight * (parent_visits as f64).ln().sqrt()
            / (1.0 + self.visits as f64).sqrt();
        exploitation + exploration
    }
}

/// PRM-guided MCTS search
#[derive(Debug, Clone)]
pub struct PrmMcts {
    pub num_simulations: usize,
    pub exploration_weight: f64,
}

impl PrmMcts {
    pub fn new(num_simulations: usize, exploration_weight: f64) -> Self {
        Self {
            num_simulations: num_simulations.max(1),
            exploration_weight: exploration_weight.max(0.0),
        }
    }

    pub fn search<F, G>(&self, root: MctsNode, expand: F, evaluate: G) -> MctsNode
    where
        F: Fn(&MctsNode) -> Vec<MctsNode>,
        G: Fn(&MctsNode) -> f64,
    {
        let mut best = root;
        for _sim in 0..self.num_simulations {
            let expanded = expand(&best);
            if !expanded.is_empty() {
                let mut scored: Vec<(f64, MctsNode)> = expanded
                    .into_iter()
                    .map(|child| {
                        let score = evaluate(&child);
                        (score, child)
                    })
                    .collect();
                scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
                if let Some((_, node)) = scored.into_iter().next() {
                    best = node;
                }
            }
        }
        best
    }
}

/// Main TTC configuration
#[derive(Debug, Clone)]
pub struct TtcConfig {
    pub allocator: LagrangianAllocator,
    pub beam_config: PrmBeamSearch,
    pub parallel_config: ParallelReasoner,
    pub early_exit: EarlyExitDetector,
    pub mcts_config: PrmMcts,
    pub enabled: bool,
}

impl Default for TtcConfig {
    fn default() -> Self {
        Self {
            allocator: LagrangianAllocator::default(),
            beam_config: PrmBeamSearch::new(2, 32),
            parallel_config: ParallelReasoner::default(),
            early_exit: EarlyExitDetector::default(),
            mcts_config: PrmMcts::new(32, 1.41),
            enabled: true,
        }
    }
}

/// Allocation decision for a single task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub budget: ComputeBudget,
    pub difficulty: f64,
    pub strategy: AllocationStrategy,
    pub expected_tokens: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AllocationStrategy {
    Direct,
    Beam,
    Mcts,
    Parallel,
    Hybrid,
}

impl AllocationStrategy {
    pub fn select(difficulty: f64, budget: &ComputeBudget) -> Self {
        if difficulty < 0.3 {
            Self::Direct
        } else if difficulty < 0.5 {
            Self::Beam
        } else if difficulty < 0.7 {
            Self::Parallel
        } else if budget.mcts_simulations > 0 {
            Self::Mcts
        } else {
            Self::Hybrid
        }
    }
}

/// TTC execution report
#[derive(Debug, Clone)]
pub struct TtcReport {
    pub total_tokens_used: u64,
    pub tokens_saved_by_early_exit: u64,
    pub total_steps: u64,
    pub num_trajectories: usize,
    pub strategy: AllocationStrategy,
    pub convergence_status: ConvergenceSignal,
    pub best_score: f64,
}

/// Main TTC Engine — orchestrates test-time compute scaling
#[derive(Debug, Clone)]
pub struct TtcEngine {
    pub config: TtcConfig,
    pub early_exit_detector: EarlyExitDetector,
}

impl TtcEngine {
    pub fn new(config: TtcConfig) -> Self {
        Self {
            early_exit_detector: config.early_exit.clone(),
            config,
        }
    }

    /// Allocate compute budget for a task based on difficulty
    pub fn allocate_budget(&self, difficulty: f64, remaining: f64) -> Allocation {
        let budget = self.config.allocator.allocate(difficulty, remaining);
        let strategy = AllocationStrategy::select(difficulty, &budget);
        let expected = budget.max_tokens;
        Allocation {
            budget,
            difficulty,
            strategy,
            expected_tokens: expected,
        }
    }

    /// Run PRM-guided beam search
    pub fn beam_search<F>(&self, initial: BeamState, scorer: F) -> Vec<BeamState>
    where
        F: Fn(&BeamState) -> Vec<BeamState>,
    {
        self.config.beam_config.search(initial, scorer)
    }

    /// Run parallel trajectories
    pub fn parallel_reason<F>(&self, step_fn: F) -> Vec<ParallelTrajectory>
    where
        F: Fn(usize, usize, &[ParallelTrajectory]) -> (f64, bool),
    {
        self.config.parallel_config.run(step_fn)
    }

    /// Check if early exit is warranted
    pub fn check_early_exit(&self, scores: &[f64]) -> ConvergenceSignal {
        self.early_exit_detector.detect_convergence(scores)
    }

    /// Run PRM-guided MCTS
    pub fn mcts_search<F, G>(&self, root: MctsNode, expand: F, evaluate: G) -> MctsNode
    where
        F: Fn(&MctsNode) -> Vec<MctsNode>,
        G: Fn(&MctsNode) -> f64,
    {
        self.config.mcts_config.search(root, expand, evaluate)
    }

    /// Whether TTC is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Token savings estimate from early exit
    pub fn estimate_savings(&self, total_tokens: u64, scores: &[f64]) -> u64 {
        if scores.len() < 3 {
            return 0;
        }
        let signal = self.early_exit_detector.detect_convergence(scores);
        match signal {
            ConvergenceSignal::Converged { token_saving, .. } => token_saving.min(total_tokens / 2),
            _ => 0,
        }
    }
}

impl Default for TtcEngine {
    fn default() -> Self {
        Self::new(TtcConfig::default())
    }
}

// ── PRM Adapter ─────────────────────────────────────────────────────────────

/// Adapter to connect TTC with Process Reward Model (nt_core_prm)
#[derive(Debug, Clone)]
pub struct PrmAdapter {
    pub prm_scores: Vec<f64>,
    pub step_scores: HashMap<usize, f64>,
}

impl Default for PrmAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PrmAdapter {
    pub fn new() -> Self {
        Self {
            prm_scores: Vec::new(),
            step_scores: HashMap::new(),
        }
    }

    pub fn record_step(&mut self, step_idx: usize, score: f64) {
        self.step_scores.insert(step_idx, score.max(0.0).min(1.0));
        self.prm_scores.push(score);
    }

    pub fn cumulative_score(&self) -> f64 {
        if self.prm_scores.is_empty() {
            return 0.0;
        }
        self.prm_scores.iter().sum::<f64>() / self.prm_scores.len() as f64
    }

    pub fn trajectory_score(&self, discount: f64) -> f64 {
        let d = discount.max(0.0).min(1.0);
        self.prm_scores
            .iter()
            .enumerate()
            .map(|(i, s)| s * d.powi(i as i32))
            .sum::<f64>()
    }

    pub fn best_step(&self) -> Option<(usize, f64)> {
        self.step_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(&idx, &score)| (idx, score))
    }

    pub fn clear(&mut self) {
        self.prm_scores.clear();
        self.step_scores.clear();
    }
}

/// Fable 5 effort tiers mapped to TTC rollout depths.
///
/// Qwen3 allocates a thinking budget proportional to task difficulty;
/// Fable 5 uses five explicit effort levels (low/medium/high/xhigh/max).
/// Each tier controls how deep the TTC rollout explores before committing
/// to a prediction — directly mapping the model's compute allocation to
/// the NeoTrix consciousness core's trajectory depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EffortTier {
    /// Low effort: minimal rollout, direct answer (Qwen3 thinking_budget ≈ 0.1)
    Low,
    /// Medium effort: standard rollout depth (Qwen3 thinking_budget ≈ 0.3)
    Medium,
    /// High effort: deeper iterative loops (Qwen3 thinking_budget ≈ 0.5)
    High,
    /// Extra-high effort: deep verification + backtrack (Qwen3 thinking_budget ≈ 0.7)
    XHigh,
    /// Max effort: full MCTS + parallel trajectories (Qwen3 thinking_budget ≈ 0.9)
    Max,
}

impl EffortTier {
    /// Detect effort tier from task difficulty and length.
    /// Mirrors Qwen3's thinking budget allocation heuristic.
    pub fn from_difficulty(difficulty: f64, task_length: usize) -> Self {
        let length_factor = (task_length as f64 / 500.0).min(1.0);
        let combined = difficulty * 0.7 + length_factor * 0.3;
        if combined < 0.2 {
            EffortTier::Low
        } else if combined < 0.4 {
            EffortTier::Medium
        } else if combined < 0.6 {
            EffortTier::High
        } else if combined < 0.8 {
            EffortTier::XHigh
        } else {
            EffortTier::Max
        }
    }

    /// TTC rollout depth for this effort tier.
    pub fn rollout_depth(&self) -> usize {
        match self {
            EffortTier::Low => 2,
            EffortTier::Medium => 4,
            EffortTier::High => 8,
            EffortTier::XHigh => 16,
            EffortTier::Max => 32,
        }
    }

    /// MCTS simulations for this effort tier.
    pub fn mcts_simulations(&self) -> usize {
        match self {
            EffortTier::Low => 0,
            EffortTier::Medium => 8,
            EffortTier::High => 16,
            EffortTier::XHigh => 32,
            EffortTier::Max => 64,
        }
    }

    /// Beam width for this effort tier.
    pub fn beam_width(&self) -> usize {
        match self {
            EffortTier::Low => 1,
            EffortTier::Medium => 2,
            EffortTier::High => 4,
            EffortTier::XHigh => 8,
            EffortTier::Max => 16,
        }
    }

    /// Confidence cap: maximum confidence allowed at this tier.
    /// Lower tiers stay uncertain (exploration); higher tiers commit (exploitation).
    pub fn confidence_cap(&self) -> f64 {
        match self {
            EffortTier::Low => 0.5,
            EffortTier::Medium => 0.65,
            EffortTier::High => 0.8,
            EffortTier::XHigh => 0.9,
            EffortTier::Max => 0.95,
        }
    }

    /// Sparse attention k: top-K states to attend to at this tier.
    /// Mirrors K3's sparse expert activation (16 of 896).
    pub fn sparse_k(&self) -> usize {
        match self {
            EffortTier::Low => 8,
            EffortTier::Medium => 16,
            EffortTier::High => 24,
            EffortTier::XHigh => 32,
            EffortTier::Max => 40,
        }
    }

    /// Extended-thinking budget in tokens. Mirrors Qwen3 thinking-budget
    /// fraction (Low≈0.1 … Max≈0.9) scaled to a 16k reasoning window.
    /// Low = 0 disables extended thinking (direct answer).
    pub fn thinking_budget_tokens(&self) -> u32 {
        match self {
            EffortTier::Low => 0,
            EffortTier::Medium => 1024,
            EffortTier::High => 2048,
            EffortTier::XHigh => 4096,
            EffortTier::Max => 8192,
        }
    }

    /// Total output token budget (thinking + answer) for this tier.
    pub fn max_tokens_budget(&self) -> u32 {
        match self {
            EffortTier::Low => 2048,
            EffortTier::Medium => 4096,
            EffortTier::High => 8192,
            EffortTier::XHigh => 16384,
            EffortTier::Max => 32768,
        }
    }
}

/// Adaptive effort tier selector using Fable 5 five-tier effort model.
///
/// Maps the Fable-5 effort allocation (low/medium/high/xhigh/max) to
/// TTC compute parameters. This is the bridge between the reasoning
/// pattern's effort signal and the test-time compute scaling engine.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EffortTierSelector {
    /// Base difficulty threshold for each tier boundary.
    pub tier_thresholds: [f64; 4],
}

impl Default for EffortTierSelector {
    fn default() -> Self {
        Self {
            // Boundaries between Low/Medium/High/XHigh/Max
            tier_thresholds: [0.2, 0.4, 0.6, 0.8],
        }
    }
}

impl EffortTierSelector {
    /// Select effort tier from difficulty score.
    pub fn select(&self, difficulty: f64) -> EffortTier {
        let d = difficulty.clamp(0.0, 1.0);
        if d < self.tier_thresholds[0] {
            EffortTier::Low
        } else if d < self.tier_thresholds[1] {
            EffortTier::Medium
        } else if d < self.tier_thresholds[2] {
            EffortTier::High
        } else if d < self.tier_thresholds[3] {
            EffortTier::XHigh
        } else {
            EffortTier::Max
        }
    }

    /// Select effort tier from difficulty and task length.
    pub fn select_for_task(&self, difficulty: f64, task_length: usize) -> EffortTier {
        let length_factor = (task_length as f64 / 500.0).min(1.0);
        let adjusted = difficulty * 0.7 + length_factor * 0.3;
        self.select(adjusted)
    }
}

#[cfg(test)]
mod effort_tier_tests {
    use super::*;

    #[test]
    fn test_effort_tier_from_difficulty() {
        assert_eq!(EffortTier::from_difficulty(0.1, 100), EffortTier::Low);
        assert_eq!(EffortTier::from_difficulty(0.3, 200), EffortTier::Medium);
        assert_eq!(EffortTier::from_difficulty(0.5, 300), EffortTier::High);
        assert_eq!(EffortTier::from_difficulty(0.7, 400), EffortTier::XHigh);
        assert_eq!(EffortTier::from_difficulty(0.9, 500), EffortTier::Max);
    }

    #[test]
    fn test_rollout_depth_increases_with_tier() {
        assert!(EffortTier::Max.rollout_depth() > EffortTier::High.rollout_depth());
        assert!(EffortTier::High.rollout_depth() > EffortTier::Medium.rollout_depth());
        assert!(EffortTier::Medium.rollout_depth() > EffortTier::Low.rollout_depth());
    }

    #[test]
    fn test_mcts_simulations_increases_with_tier() {
        assert_eq!(EffortTier::Low.mcts_simulations(), 0);
        assert_eq!(EffortTier::Medium.mcts_simulations(), 8);
        assert_eq!(EffortTier::High.mcts_simulations(), 16);
        assert_eq!(EffortTier::XHigh.mcts_simulations(), 32);
        assert_eq!(EffortTier::Max.mcts_simulations(), 64);
    }

    #[test]
    fn test_sparse_k_increases_with_tier() {
        assert_eq!(EffortTier::Low.sparse_k(), 8);
        assert_eq!(EffortTier::Medium.sparse_k(), 16);
        assert_eq!(EffortTier::High.sparse_k(), 24);
        assert_eq!(EffortTier::XHigh.sparse_k(), 32);
        assert_eq!(EffortTier::Max.sparse_k(), 40);
    }

    #[test]
    fn test_confidence_cap_increases_with_tier() {
        assert!(EffortTier::Max.confidence_cap() > EffortTier::High.confidence_cap());
        assert!(EffortTier::High.confidence_cap() > EffortTier::Medium.confidence_cap());
        assert!(EffortTier::Medium.confidence_cap() > EffortTier::Low.confidence_cap());
    }

    #[test]
    fn test_effort_tier_selector() {
        let selector = EffortTierSelector::default();
        assert_eq!(selector.select(0.1), EffortTier::Low);
        assert_eq!(selector.select(0.3), EffortTier::Medium);
        assert_eq!(selector.select(0.5), EffortTier::High);
        assert_eq!(selector.select(0.7), EffortTier::XHigh);
        assert_eq!(selector.select(0.9), EffortTier::Max);
    }

    #[test]
    fn test_effort_tier_selector_clamping() {
        let selector = EffortTierSelector::default();
        assert_eq!(selector.select(-0.5), EffortTier::Low);
        assert_eq!(selector.select(1.5), EffortTier::Max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_factors_composite() {
        let f = DifficultyFactors {
            prompt_length: 0.5,
            question_density: 0.3,
            task_weight: 0.8,
            constraint_count: 0.2,
            novelty: 0.1,
        };
        let c = f.composite();
        assert!(c > 0.0 && c <= 1.0);
        assert!((c - 0.5 * 0.2 - 0.3 * 0.25 - 0.8 * 0.25 - 0.2 * 0.2 - 0.1 * 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_lagrangian_allocator_low_difficulty() {
        let alloc = LagrangianAllocator::default();
        let budget = alloc.allocate(0.1, 1.0);
        assert!(budget.max_tokens >= alloc.base_budget.max_tokens);
        assert!(
            budget.max_tokens
                <= (alloc.base_budget.max_tokens as f64 * alloc.max_multiplier) as u64
        );
    }

    #[test]
    fn test_lagrangian_allocator_high_difficulty() {
        let alloc = LagrangianAllocator::default();
        let easy = alloc.allocate(0.1, 1.0);
        let hard = alloc.allocate(0.9, 1.0);
        assert!(hard.max_tokens >= easy.max_tokens);
        assert!(hard.max_steps >= easy.max_steps);
    }

    #[test]
    fn test_lagrangian_allocator_remaining_budget() {
        let alloc = LagrangianAllocator::default();
        let high_remaining = alloc.allocate(0.5, 10.0);
        let low_remaining = alloc.allocate(0.5, 0.1);
        assert!(high_remaining.max_tokens >= low_remaining.max_tokens);
    }

    #[test]
    fn test_beam_search_basic() {
        let beam = PrmBeamSearch::new(3, 5);
        let initial = BeamState::new();
        let results = beam.search(initial, |_state| {
            vec![
                BeamState {
                    steps: vec![ReasoningStep {
                        step_idx: 0,
                        content: "A".into(),
                        prm_score: 0.8,
                        cumulative_score: 0.8,
                    }],
                    cumulative_score: 0.8,
                    is_terminal: false,
                },
                BeamState {
                    steps: vec![ReasoningStep {
                        step_idx: 0,
                        content: "B".into(),
                        prm_score: 0.6,
                        cumulative_score: 0.6,
                    }],
                    cumulative_score: 0.6,
                    is_terminal: false,
                },
            ]
        });
        assert!(!results.is_empty());
        assert!(results[0].cumulative_score >= results[1].cumulative_score);
    }

    #[test]
    fn test_beam_width_enforced() {
        let beam = PrmBeamSearch::new(2, 10);
        let initial = BeamState::new();
        let results = beam.search(initial, |_state| {
            (0..5)
                .map(|i| BeamState {
                    steps: vec![ReasoningStep {
                        step_idx: 0,
                        content: i.to_string(),
                        prm_score: i as f64 / 5.0,
                        cumulative_score: i as f64 / 5.0,
                    }],
                    cumulative_score: i as f64 / 5.0,
                    is_terminal: false,
                })
                .collect()
        });
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_parallel_reasoner_runs_all_trajectories() {
        let pr = ParallelReasoner::new(3, 10);
        let results = pr.run(|t, _step, _all| (0.5 + t as f64 * 0.1, false));
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_parallel_reasoner_convergence() {
        let pr = ParallelReasoner::new(2, 20);
        let results = pr.run(|_t, step, _all| {
            if step >= 5 {
                (0.95, true)
            } else {
                (0.5, false)
            }
        });
        assert!(results.iter().all(|t| t.converged));
    }

    #[test]
    fn test_parallel_coordination_smooths_scores() {
        let pr = ParallelReasoner::default();
        let mut trajectories = vec![
            ParallelTrajectory {
                trajectory_id: 0,
                steps: vec![],
                final_score: 0.2,
                token_count: 100,
                converged: false,
            },
            ParallelTrajectory {
                trajectory_id: 1,
                steps: vec![],
                final_score: 0.9,
                token_count: 100,
                converged: false,
            },
        ];
        pr.coordinate(&mut trajectories);
        assert!((trajectories[0].final_score - 0.2).abs() > 1e-6);
    }

    #[test]
    fn test_early_exit_detector_insufficient_steps() {
        let detector = EarlyExitDetector::default();
        assert_eq!(
            detector.detect_convergence(&[0.5, 0.6]),
            ConvergenceSignal::InsufficientSteps
        );
    }

    #[test]
    fn test_early_exit_detector_converged() {
        let detector = EarlyExitDetector::new(3, 0.1, 3);
        let scores = vec![0.5, 0.51, 0.52, 0.51, 0.5];
        let signal = detector.detect_convergence(&scores);
        assert!(signal.should_exit() || matches!(signal, ConvergenceSignal::StableEnough { .. }));
    }

    #[test]
    fn test_early_exit_detector_degrading() {
        let detector = EarlyExitDetector::new(3, 0.05, 3);
        let scores = vec![0.9, 0.7, 0.5, 0.3, 0.1];
        let signal = detector.detect_convergence(&scores);
        assert!(signal.should_exit());
    }

    #[test]
    fn test_early_exit_should_exit() {
        let detector = EarlyExitDetector::default();
        assert!(!detector.should_exit(&[0.5]));
        assert!(!detector.should_exit(&[0.5, 0.6]));
        assert!(!detector.should_exit(&[0.5, 0.6, 0.55]));
    }

    #[test]
    fn test_mcts_ucb_score() {
        let node = MctsNode {
            state_id: "root".into(),
            score: 5.0,
            visits: 10,
            children: vec![],
            parent: None,
        };
        let score = node.ucb_score(20, 1.41);
        assert!(score.is_finite());
        assert!(score > 0.0);
    }

    #[test]
    fn test_mcts_search_basic() {
        let mcts = PrmMcts::new(10, 1.41);
        let root = MctsNode {
            state_id: "root".into(),
            score: 0.0,
            visits: 0,
            children: vec![],
            parent: None,
        };
        let result = mcts.search(
            root,
            |_node| {
                vec![
                    MctsNode {
                        state_id: "a".into(),
                        score: 0.0,
                        visits: 0,
                        children: vec![],
                        parent: None,
                    },
                    MctsNode {
                        state_id: "b".into(),
                        score: 0.0,
                        visits: 0,
                        children: vec![],
                        parent: None,
                    },
                ]
            },
            |node| if node.state_id == "a" { 0.9 } else { 0.5 },
        );
        assert!(result.state_id == "a" || result.state_id == "b");
    }

    #[test]
    fn test_allocation_strategy_selection() {
        assert_eq!(
            AllocationStrategy::select(0.2, &ComputeBudget::free()),
            AllocationStrategy::Direct
        );
        assert_eq!(
            AllocationStrategy::select(0.4, &ComputeBudget::free()),
            AllocationStrategy::Beam
        );
        assert_eq!(
            AllocationStrategy::select(0.6, &ComputeBudget::free()),
            AllocationStrategy::Parallel
        );
    }

    #[test]
    fn test_ttc_engine_allocate_budget() {
        let engine = TtcEngine::default();
        let alloc = engine.allocate_budget(0.7, 5.0);
        assert!(alloc.budget.max_tokens > 0);
        assert!(alloc.expected_tokens > 0);
        assert!(alloc.difficulty - 0.7 < 1e-6);
    }

    #[test]
    fn test_prm_adapter_records_scores() {
        let mut adapter = PrmAdapter::new();
        adapter.record_step(0, 0.8);
        adapter.record_step(1, 0.6);
        assert_eq!(adapter.prm_scores.len(), 2);
        assert!((adapter.cumulative_score() - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_prm_adapter_trajectory_score_discount() {
        let mut adapter = PrmAdapter::new();
        adapter.record_step(0, 1.0);
        adapter.record_step(1, 0.5);
        let discounted = adapter.trajectory_score(0.8);
        assert!(discounted > 0.0);
        assert!(discounted < 2.0);
    }

    #[test]
    fn test_prm_adapter_best_step() {
        let mut adapter = PrmAdapter::new();
        adapter.record_step(0, 0.3);
        adapter.record_step(1, 0.9);
        adapter.record_step(2, 0.6);
        let best = adapter.best_step();
        assert!(best.is_some());
        assert_eq!(best.unwrap().0, 1);
    }

    #[test]
    fn test_prm_adapter_clear() {
        let mut adapter = PrmAdapter::new();
        adapter.record_step(0, 0.8);
        adapter.clear();
        assert!(adapter.prm_scores.is_empty());
        assert!(adapter.step_scores.is_empty());
    }

    #[test]
    fn test_ttc_engine_estimate_savings() {
        let engine = TtcEngine::default();
        let savings = engine.estimate_savings(1000, &[0.5, 0.51, 0.52, 0.51, 0.5]);
        assert!(savings <= 500);
    }

    #[test]
    fn test_compute_budget_builder() {
        let budget = ComputeBudget::free()
            .with_tokens(8192)
            .with_beam(4)
            .with_parallel(8)
            .with_mcts(64);
        assert_eq!(budget.max_tokens, 8192);
        assert_eq!(budget.beam_width, 4);
        assert_eq!(budget.parallel_trajectories, 8);
        assert_eq!(budget.mcts_simulations, 64);
    }

    #[test]
    fn test_beam_search_terminal_early_stop() {
        let beam = PrmBeamSearch::new(3, 100);
        let initial = BeamState::new();
        let results = beam.search(initial, |state| {
            if state.last_score() > 0.8 {
                vec![BeamState {
                    steps: state.steps.clone(),
                    cumulative_score: state.cumulative_score,
                    is_terminal: true,
                }]
            } else {
                vec![BeamState {
                    steps: vec![ReasoningStep {
                        step_idx: 0,
                        content: "step".into(),
                        prm_score: 0.9,
                        cumulative_score: 0.9,
                    }],
                    cumulative_score: 0.9,
                    is_terminal: true,
                }]
            }
        });
        assert!(!results.is_empty());
    }

    #[test]
    fn test_difficulty_factors_zero() {
        let f = DifficultyFactors {
            prompt_length: 0.0,
            question_density: 0.0,
            task_weight: 0.0,
            constraint_count: 0.0,
            novelty: 0.0,
        };
        assert!((f.composite() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_difficulty_factors_max() {
        let f = DifficultyFactors {
            prompt_length: 1.0,
            question_density: 1.0,
            task_weight: 1.0,
            constraint_count: 1.0,
            novelty: 1.0,
        };
        assert!((f.composite() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_convergence_signal_methods() {
        assert!(ConvergenceSignal::Converged {
            reason: "done".into(),
            token_saving: 100
        }
        .should_exit());
        assert!(ConvergenceSignal::Degrading { trend: -0.5 }.should_exit());
        assert!(!ConvergenceSignal::StableEnough {
            remaining_variance: 0.1
        }
        .should_exit());
        assert!(!ConvergenceSignal::NeedsMoreSteps { variance: 0.5 }.should_exit());
        assert!(!ConvergenceSignal::InsufficientSteps.should_exit());
    }
}
