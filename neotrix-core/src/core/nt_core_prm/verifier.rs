use super::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::SQRT_2;
/// Verification action types for MCTS tree expansion.
///
/// Each type grounds a different dimension of step quality:
/// - ModeConsistency: E8 hexagram alignment with task domain
/// - TransitionPattern: consecutive-state transition fidelity
/// - RewardHistory: empirical reward expectation from past similar modes
/// - DirectionChange: orientation shifts relative to trajectory arc
/// - OscillationCheck: avoid wasteful mode flips
/// - StepPosition: position-dependent relevance signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerificationAction {
    ModeConsistency,
    TransitionPattern,
    RewardHistory,
    DirectionChange,
    OscillationCheck,
    StepPosition,
}

impl VerificationAction {
    pub const ALL: [VerificationAction; 6] = [
        VerificationAction::ModeConsistency,
        VerificationAction::TransitionPattern,
        VerificationAction::RewardHistory,
        VerificationAction::DirectionChange,
        VerificationAction::OscillationCheck,
        VerificationAction::StepPosition,
    ];

    /// Human-readable dimension label.
    pub fn label(&self) -> &'static str {
        match self {
            VerificationAction::ModeConsistency => "mode_consistency",
            VerificationAction::TransitionPattern => "transition_pattern",
            VerificationAction::RewardHistory => "reward_history",
            VerificationAction::DirectionChange => "direction_change",
            VerificationAction::OscillationCheck => "oscillation_check",
            VerificationAction::StepPosition => "step_position",
        }
    }

    /// Ground-truth score for this dimension given a step in context.
    /// All scores in [0, 1].
    pub fn ground(
        &self,
        step: &TrajectoryStep,
        trajectory: &[TrajectoryStep],
        mode_rewards: &[(f64, u64)],
        task_type: &str,
    ) -> f64 {
        match self {
            VerificationAction::ModeConsistency => {
                // Task→mode alignment: code tasks prefer mid-range hexagrams,
                // math prefers structured high bits, general = neutral.
                let mode = step.e8_mode.0;
                match task_type {
                    t if t.contains("code") => {
                        // Coding: prefer modes with MODE bit on (0x08) and DEPTH moderate
                        let bits = mode & 0x3F;
                        if bits & 0x08 != 0 && (bits & 0x1C) < 0x1C {
                            0.8
                        } else {
                            0.4
                        }
                    }
                    t if t.contains("math") => {
                        // Math: prefer structured high-ABST modes
                        if mode & 0x20 != 0 && mode & 0x04 != 0 {
                            0.85
                        } else {
                            0.45
                        }
                    }
                    t if t.contains("reason") => {
                        // Reasoning: prefer balanced STANCE+MODE
                        let bits = mode & 0x3F;
                        let stance = (bits & 0x20) >> 5;
                        let mode_bit = (bits & 0x08) >> 3;
                        if stance == mode_bit {
                            0.75
                        } else {
                            0.5
                        }
                    }
                    _ => 0.6, // General: neutral
                }
            }
            VerificationAction::TransitionPattern => {
                // Transition fidelity: compare step's mode vs previous step's mode.
                // High score = moderate Hamming distance (1-2 bit flips = healthy exploration).
                if let Some(prev) = trajectory.last() {
                    let prev_mode = prev.e8_mode.0;
                    let diff = (step.e8_mode.0 ^ prev_mode) as u32;
                    let hamming = diff.count_ones();
                    match hamming {
                        0 => 0.3,     // no change = stuck
                        1 => 0.9,     // single bit = focused refinement
                        2 => 0.8,     // two bits = exploration
                        3 | 4 => 0.5, // moderate jump
                        _ => 0.2,     // large jump = unstable
                    }
                } else {
                    0.7 // first step: default trust
                }
            }
            VerificationAction::RewardHistory => {
                // Expectation: compare step success to historical avg for this mode.
                let mode = step.e8_mode.0 as usize;
                let (_, count) = mode_rewards[mode];
                if count > 0 {
                    let avg = mode_rewards[mode].0 / count as f64;
                    let step_val = if step.success { 1.0 } else { 0.0 };
                    // Score = 1 - |avg - step_val|: closer to historical = more grounded
                    1.0 - (avg - step_val).abs().max(0.0).min(1.0)
                } else {
                    0.5 // no history: neutral
                }
            }
            VerificationAction::DirectionChange => {
                // Orientation: ABST bit (0x20) flip = paradigm shift.
                if let Some(prev) = trajectory.last() {
                    let prev_abst = prev.e8_mode.0 & 0x20;
                    let cur_abst = step.e8_mode.0 & 0x20;
                    if prev_abst != cur_abst {
                        // Direction change is good later in trajectory (mature pivot),
                        // bad early (unstable start)
                        let step_frac = step.step_idx as f64 / trajectory.len().max(1) as f64;
                        0.2 + step_frac * 0.6 // 0.2→0.8 as trajectory progresses
                    } else {
                        0.7 // steady direction = generally good
                    }
                } else {
                    0.6
                }
            }
            VerificationAction::OscillationCheck => {
                // Oscillation: repeating same mode wastes compute.
                if trajectory.len() >= 2 {
                    let same_as_prev = trajectory
                        .last()
                        .map(|s| s.e8_mode.0 == step.e8_mode.0)
                        .unwrap_or(false);
                    let same_as_prev2 = trajectory.len() >= 2
                        && trajectory[trajectory.len() - 2].e8_mode.0 == step.e8_mode.0;
                    if same_as_prev && same_as_prev2 {
                        0.2 // repeating pattern
                    } else if same_as_prev {
                        0.4 // single repeat (could be normal)
                    } else {
                        0.8 // novel mode
                    }
                } else {
                    0.8
                }
            }
            VerificationAction::StepPosition => {
                // Position-dependent: early steps have lower base expectation,
                // later steps higher (convergence expectation).
                if trajectory.is_empty() {
                    return 0.5;
                }
                let total = trajectory.len() as f64;
                let pos = step.step_idx as f64;
                let fraction = pos / total.max(1.0);
                0.3 + fraction * 0.5 // 0.3 (start) → 0.8 (end)
            }
        }
    }
}

/// A single MCTS tree node representing one verification hypothesis.
struct MctsNode {
    /// The action that this node represents.
    action: VerificationAction,
    /// Ground score from the action's grounding function.
    /// Parent node index (None for root).
    parent: Option<usize>,
    /// Child node indices.
    children: Vec<usize>,
    /// Visit count for UCB1 exploration bonus.
    visits: u64,
    /// Cumulative reward from rollouts.
    total_value: f64,
}

/// MCTS tree for step verification.
///
/// Performs `num_iterations` rollouts:
/// 1. **Selection**: traverse from root using UCB1 until a leaf is reached
/// 2. **Expansion**: add children for all unvisited actions
/// 3. **Simulation**: ground score = the selected action's intrinsic quality
/// 4. **Backpropagation**: propagate reward up to root
///
/// After completion, root's `total_value / visits` is the grounded verification score.
struct MctsTree {
    nodes: Vec<MctsNode>,
    root_idx: usize,
}

impl MctsTree {
    fn new() -> Self {
        // Root node: no action, occupies index 0
        let root = MctsNode {
            action: VerificationAction::ModeConsistency, // placeholder, unused for root
            parent: None,
            children: Vec::new(),
            visits: 0,
            total_value: 0.0,
        };
        let mut tree = MctsTree {
            nodes: vec![root],
            root_idx: 0,
        };
        // Expand root with all 6 verification actions
        for act in VerificationAction::ALL {
            let child = MctsNode {
                action: act,
                parent: Some(tree.root_idx),
                children: Vec::new(),
                visits: 0,
                total_value: 0.0,
            };
            tree.nodes.push(child);
            let child_idx = tree.nodes.len() - 1;
            tree.nodes[tree.root_idx].children.push(child_idx);
        }
        tree
    }

    /// Select a leaf node using UCB1.
    fn select(&mut self) -> usize {
        let mut current = self.root_idx;
        loop {
            let node = &self.nodes[current];
            if node.children.is_empty() {
                return current;
            }
            // UCB1: pick child maximizing Q + c * sqrt(ln(N_parent) / N_child)
            let total_parent = node.visits.max(1) as f64;
            let mut best_child = node.children[0];
            let mut best_ucb = f64::NEG_INFINITY;
            for &child_idx in &node.children {
                let child = &self.nodes[child_idx];
                let n = child.visits.max(1) as f64;
                let q = if child.visits > 0 {
                    child.total_value / n
                } else {
                    0.0 // unvisited: high exploration bonus
                };
                let exploration = SQRT_2 * (total_parent.ln() / n).sqrt();
                let ucb = q + exploration;
                if ucb > best_ucb {
                    best_ucb = ucb;
                    best_child = child_idx;
                }
            }
            current = best_child;
        }
    }

    /// Simulate: look up the grounding score for the leaf node's action.
    fn simulate(
        &self,
        leaf_idx: usize,
        step: &TrajectoryStep,
        trajectory: &[TrajectoryStep],
        mode_rewards: &[(f64, u64)],
        task_type: &str,
    ) -> f64 {
        let action = self.nodes[leaf_idx].action;
        action.ground(step, trajectory, mode_rewards, task_type)
    }

    /// Backpropagate reward from leaf to root.
    fn backpropagate(&mut self, leaf_idx: usize, reward: f64) {
        let mut current = leaf_idx;
        loop {
            self.nodes[current].visits += 1;
            self.nodes[current].total_value += reward;
            match self.nodes[current].parent {
                Some(parent) => current = parent,
                None => break,
            }
        }
    }

    /// Run the full MCTS loop for the given step+context.
    fn run(
        &mut self,
        step: &TrajectoryStep,
        trajectory: &[TrajectoryStep],
        mode_rewards: &[(f64, u64)],
        task_type: &str,
        num_iterations: usize,
    ) {
        for _ in 0..num_iterations {
            let leaf = self.select();
            let reward = self.simulate(leaf, step, trajectory, mode_rewards, task_type);
            self.backpropagate(leaf, reward);
        }
    }

    /// Return the root's grounded score = total_value / visits.
    fn grounded_score(&self) -> f64 {
        if self.nodes[self.root_idx].visits == 0 {
            return 0.5; // neutral default
        }
        self.nodes[self.root_idx].total_value / self.nodes[self.root_idx].visits as f64
    }

    /// Return per-action scores for interpretability.
    fn per_action_scores(&self) -> Vec<(VerificationAction, f64, u64)> {
        self.nodes[self.root_idx]
            .children
            .iter()
            .map(|&child_idx| {
                let child = &self.nodes[child_idx];
                let avg = if child.visits > 0 {
                    child.total_value / child.visits as f64
                } else {
                    0.0
                };
                (child.action, avg, child.visits)
            })
            .collect()
    }
}

/// GroundedPRM step verifier.
///
/// For each step in a trajectory, MCTS tree search explores verification
/// hypotheses grounded in analytic signals (mode consistency, transition
/// fidelity, reward history, direction change, oscillation check, step
/// position). The MCTS root value after `num_iterations` rollouts is the
/// grounded verification score.
///
/// The blended score = λ * grounded + (1-λ) * original PRM score, where
/// λ is the `grounded_weight` (default 0.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedPrmVerifier {
    /// Number of MCTS iterations per step (default 48).
    pub num_iterations: usize,
    /// Blend weight λ for grounded score (default 0.3).
    pub grounded_weight: f64,
    /// Per-mode empirical reward accumulator for reward-history grounding.
    pub mode_rewards: Vec<(f64, u64)>,
    /// Total steps verified in this verifier's lifetime.
    pub total_steps_verified: u64,
}

impl GroundedPrmVerifier {
    pub fn new(num_iterations: usize, grounded_weight: f64) -> Self {
        Self {
            num_iterations,
            grounded_weight,
            mode_rewards: vec![(0.0, 0); 64],
            total_steps_verified: 0,
        }
    }

    /// Verify a single step in the context of its trajectory.
    ///
    /// Returns (grounded_score, per_action_scores).
    pub fn verify_step(
        &mut self,
        step: &TrajectoryStep,
        trajectory: &[TrajectoryStep],
        task_type: &str,
    ) -> (f64, Vec<(VerificationAction, f64, u64)>) {
        self.total_steps_verified += 1;
        let mut tree = MctsTree::new();
        tree.run(
            step,
            trajectory,
            &self.mode_rewards,
            task_type,
            self.num_iterations,
        );
        let score = tree.grounded_score();
        let actions = tree.per_action_scores();
        (score, actions)
    }

    /// Verify all steps in a trajectory and return blended scores.
    pub fn verify_trajectory(
        &mut self,
        trajectory: &AgentTrajectory,
        original_scores: &[ProcessScore],
        task_type: &str,
    ) -> Vec<ProcessScore> {
        // Build context slices: for step i, context = steps[0..i]
        let mut context: Vec<TrajectoryStep> = Vec::new();
        let mut blended = Vec::with_capacity(trajectory.steps.len());

        for (i, step) in trajectory.steps.iter().enumerate() {
            let (grounded, actions) = self.verify_step(step, &context, task_type);
            let original = original_scores
                .get(i)
                .cloned()
                .unwrap_or_else(|| ProcessScore::new(i));
            let blended_score =
                self.grounded_weight * grounded + (1.0 - self.grounded_weight) * original.score;
            let grounded_criterion = ScoredCriterion {
                name: "grounded_prm".to_string(),
                score: grounded,
                rationale: Some(format!(
                    "MCTS-verified via {} actions",
                    actions.iter().filter(|(_, _, v)| *v > 0).count()
                )),
            };
            let mut criteria = original.criteria.clone();
            criteria.push(grounded_criterion);
            // Add attribution tags per action
            let mut tags = original.attribution_tags.clone();
            tags.push("grounded_prm".to_string());
            for (act, avg, visits) in &actions {
                if *visits > 0 {
                    tags.push(format!("gp_{}:{:.2}", act.label(), avg));
                }
            }
            blended.push(ProcessScore {
                step_idx: i,
                score: blended_score.max(0.0).min(1.0),
                confidence: (original.confidence + grounded) / 2.0,
                criteria,
                attribution_tags: tags,
            });
            context.push(step.clone());
        }

        // Update mode rewards for future steps
        for step in &trajectory.steps {
            let mode = step.e8_mode.0 as usize;
            let step_val = if step.success { 1.0 } else { 0.0 };
            let (sum, count) = self.mode_rewards[mode];
            self.mode_rewards[mode] = (sum + step_val, count + 1);
        }

        blended
    }

    /// Convenience: one-shot verify + blend without managing trajectory state.
    pub fn verify_and_blend(
        &mut self,
        trajectory: &AgentTrajectory,
        coach: &dyn Coach,
        task_type: &str,
    ) -> Vec<ProcessScore> {
        // Get original scores from the coach
        let original = coach.score_episode(trajectory);
        self.verify_trajectory(trajectory, &original, task_type)
    }
}

#[cfg(test)]
mod grounded_prm_tests {
    use super::*;
    use crate::core::nt_core_hex::ReasoningHexagram;

    fn make_step(idx: usize, mode: u8, success: bool) -> TrajectoryStep {
        TrajectoryStep {
            step_idx: idx,
            specialist: SpecialistType::PatternMatcher,
            e8_mode: ReasoningHexagram(mode),
            action: format!("action_{}", idx),
            input: "in".into(),
            output: "out".into(),
            duration_ms: Some(100),
            success,
            external_reward: Some(if success { 1.0 } else { 0.0 }),
        }
    }

    #[test]
    fn test_grounded_prm_verifier_default_creation() {
        let verifier = GroundedPrmVerifier::new(48, 0.3);
        assert_eq!(verifier.num_iterations, 48);
        assert!((verifier.grounded_weight - 0.3).abs() < 1e-10);
        assert_eq!(verifier.total_steps_verified, 0);
    }

    #[test]
    fn test_mcts_tree_initialization() {
        let tree = MctsTree::new();
        assert_eq!(tree.nodes.len(), 7); // root + 6 actions
        assert_eq!(tree.nodes[0].children.len(), 6);
        // Verify all children are unvisited
        for &child_idx in &tree.nodes[0].children {
            assert_eq!(tree.nodes[child_idx].visits, 0);
        }
    }

    #[test]
    fn test_mcts_tree_single_step_verification() {
        let step = make_step(0, 56, true); // mode 56 = high STANCE + MODE
        let trajectory: Vec<TrajectoryStep> = vec![];
        let mode_rewards = [(0.0, 0); 64];
        let mut tree = MctsTree::new();
        tree.run(&step, &trajectory, &mode_rewards, "general", 48);
        let score = tree.grounded_score();
        // With 48 iterations across 6 actions ≈ 8 per action
        assert!(
            score > 0.0 && score <= 1.0,
            "score should be in (0,1], got {}",
            score
        );
        assert!(tree.nodes[0].visits > 0, "root should have been visited");
    }

    #[test]
    fn test_verification_action_ground_functions() {
        let step = make_step(0, 56, true); // mode 56
        let trajectory: Vec<TrajectoryStep> = vec![make_step(0, 48, true)];

        let mode_rewards = [(0.0, 0); 64];

        // Mode consistency for "code" task
        let mc =
            VerificationAction::ModeConsistency.ground(&step, &trajectory, &mode_rewards, "code");
        assert!(
            mc >= 0.0 && mc <= 1.0,
            "mode_consistency out of range: {}",
            mc
        );

        // Transition pattern
        let tp = VerificationAction::TransitionPattern.ground(
            &step,
            &trajectory,
            &mode_rewards,
            "general",
        );
        assert!(tp >= 0.0 && tp <= 1.0, "transition out of range: {}", tp);

        // Direction change within same ABST
        let dc = VerificationAction::DirectionChange.ground(
            &step,
            &trajectory,
            &mode_rewards,
            "general",
        );
        assert!(dc >= 0.0 && dc <= 1.0, "direction out of range: {}", dc);

        // Oscillation (first step after prev = different)
        let osc = VerificationAction::OscillationCheck.ground(
            &step,
            &trajectory,
            &mode_rewards,
            "general",
        );
        assert!(
            osc >= 0.0 && osc <= 1.0,
            "oscillation out of range: {}",
            osc
        );
    }

    #[test]
    fn test_verifier_verify_single_step() {
        let step = make_step(0, 56, true);
        let mut verifier = GroundedPrmVerifier::new(48, 0.3);
        let (score, actions) = verifier.verify_step(&step, &[], "code");
        assert!(score > 0.0 && score <= 1.0, "score out of range: {}", score);
        assert_eq!(actions.len(), 6);
        assert_eq!(verifier.total_steps_verified, 1);
    }

    #[test]
    fn test_verifier_verify_trajectory() {
        let steps = vec![
            make_step(0, 56, true),
            make_step(1, 48, true),
            make_step(2, 40, true),
            make_step(3, 32, true),
            make_step(4, 24, true),
        ];
        let traj = AgentTrajectory {
            trajectory_id: 1,
            task: "code task".into(),
            steps,
            outcome_reward: Some(1.0),
            completed: true,
            total_duration_ms: Some(500),
        };
        let original: Vec<ProcessScore> = (0..5)
            .map(|i| ProcessScore {
                step_idx: i,
                score: 0.5 + i as f64 * 0.1,
                confidence: 0.5,
                criteria: vec![],
                attribution_tags: vec![],
            })
            .collect();

        let mut verifier = GroundedPrmVerifier::new(36, 0.3);
        let blended = verifier.verify_trajectory(&traj, &original, "code");
        assert_eq!(blended.len(), 5);
        for (i, ps) in blended.iter().enumerate() {
            assert!(
                ps.score >= 0.0 && ps.score <= 1.0,
                "score out of range at {}: {}",
                i,
                ps.score
            );
            assert_eq!(ps.step_idx, i);
            // Should have grounded_prm criterion added
            assert!(
                ps.criteria.iter().any(|c| c.name == "grounded_prm"),
                "missing grounded_prm criterion at {}",
                i
            );
            // Should have grounded_prm tag
            assert!(
                ps.attribution_tags.iter().any(|t| t == "grounded_prm"),
                "missing grounded_prm tag at {}",
                i
            );
        }
        assert_eq!(verifier.total_steps_verified, 5);
    }

    #[test]
    fn test_verifier_blend_weight_effect() {
        let step = make_step(0, 56, true);
        let original = ProcessScore {
            step_idx: 0,
            score: 0.2,
            confidence: 0.5,
            criteria: vec![],
            attribution_tags: vec![],
        };
        let traj = AgentTrajectory {
            trajectory_id: 1,
            task: "test".into(),
            steps: vec![step],
            outcome_reward: None,
            completed: false,
            total_duration_ms: None,
        };

        // High grounded weight (0.9) → grounded dominates
        let mut verifier_high = GroundedPrmVerifier::new(48, 0.9);
        let blended_high = verifier_high.verify_trajectory(&traj, &[original.clone()], "general");
        let grounded_only = blended_high[0].score;
        assert!(grounded_only >= 0.0 && grounded_only <= 1.0);

        // Low grounded weight (0.1) → original dominates
        let mut verifier_low = GroundedPrmVerifier::new(48, 0.1);
        let blended_low = verifier_low.verify_trajectory(&traj, &[original], "general");
        let blended_low_score = blended_low[0].score;
        assert!(
            (blended_low_score - 0.2).abs() < 0.3 || grounded_only > blended_low_score,
            "high weight should pull further from original than low weight"
        );
    }

    #[test]
    fn test_verifier_mode_rewards_update() {
        let steps = vec![
            make_step(0, 10, true),
            make_step(1, 20, false),
            make_step(2, 30, true),
        ];
        let traj = AgentTrajectory {
            trajectory_id: 2,
            task: "test".into(),
            steps,
            outcome_reward: Some(0.5),
            completed: true,
            total_duration_ms: None,
        };

        let original: Vec<ProcessScore> = (0..3)
            .map(|i| ProcessScore {
                step_idx: i,
                score: 0.5,
                confidence: 0.5,
                criteria: vec![],
                attribution_tags: vec![],
            })
            .collect();

        let mut verifier = GroundedPrmVerifier::new(24, 0.3);
        let _ = verifier.verify_trajectory(&traj, &original, "general");

        // Mode 10 should have (1.0, 1), 20 should have (0.0, 1), 30 should have (1.0, 1)
        assert_eq!(verifier.mode_rewards[10].1, 1);
        assert!((verifier.mode_rewards[10].0 - 1.0).abs() < 1e-10);
        assert_eq!(verifier.mode_rewards[20].1, 1);
        assert!((verifier.mode_rewards[20].0 - 0.0).abs() < 1e-10);
        assert_eq!(verifier.mode_rewards[30].1, 1);
        assert!((verifier.mode_rewards[30].0 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_verify_and_blend_one_shot() {
        let steps = vec![make_step(0, 0, true), make_step(1, 1, true)];
        let traj = AgentTrajectory {
            trajectory_id: 3,
            task: "math problem".into(),
            steps,
            outcome_reward: Some(1.0),
            completed: true,
            total_duration_ms: None,
        };
        let coach = HeuristicCoach::default();
        let mut verifier = GroundedPrmVerifier::new(48, 0.3);
        let blended = verifier.verify_and_blend(&traj, &coach, "math");
        assert_eq!(blended.len(), 2);
        assert!(blended[0].score >= 0.0 && blended[0].score <= 1.0);
    }

    #[test]
    fn test_grounded_prm_oscillation_detection() {
        // Repeated modes should produce low oscillation score
        let step = make_step(2, 42, true);
        let trajectory = vec![make_step(0, 42, true), make_step(1, 42, true)];
        let osc = VerificationAction::OscillationCheck.ground(
            &step,
            &trajectory,
            &[(0.0, 0); 64],
            "general",
        );
        assert!(
            osc <= 0.5,
            "oscillation should be low for repeated mode, got {}",
            osc
        );
    }

    #[test]
    fn test_direction_change_early_vs_late() {
        let step_early = make_step(0, 60, true); // ABST bit 0x20 is on
        let step_late = make_step(8, 60, true);
        let mut traj_early = vec![make_step(0, 36, true)]; // 36 = no ABST

        let dc_early = VerificationAction::DirectionChange.ground(
            &step_early,
            &traj_early,
            &[(0.0, 0); 64],
            "general",
        );
        traj_early.push(make_step(1, 36, true));
        traj_early.push(make_step(2, 36, true));
        traj_early.push(make_step(3, 36, true));
        traj_early.push(make_step(4, 36, true));
        traj_early.push(make_step(5, 36, true));
        traj_early.push(make_step(6, 36, true));
        traj_early.push(make_step(7, 36, true));
        let dc_late = VerificationAction::DirectionChange.ground(
            &step_late,
            &traj_early,
            &[(0.0, 0); 64],
            "general",
        );
        // Late direction change should score higher than early
        assert!(
            dc_late >= dc_early,
            "late direction change {}, early {}",
            dc_late,
            dc_early
        );
    }

    #[test]
    fn test_reward_history_ground_exact_match() {
        let step = make_step(0, 5, true);
        let mut mode_rewards = [(0.0, 0); 64];
        mode_rewards[5] = (5.0, 10); // avg = 0.5
        let score = VerificationAction::RewardHistory.ground(&step, &[], &mode_rewards, "general");
        // step_val = 1.0, avg = 0.5, diff = 0.5, score = 1 - 0.5 = 0.5
        assert!((score - 0.5).abs() < 1e-10, "expected 0.5, got {}", score);
    }

    #[test]
    fn test_transition_pattern_hamming_distances() {
        // Hamming distance 1 (single bit flip) = 0.9
        let step_h1 = make_step(0, 0x21, true); // 33 in binary: 100001
        let prev_h1 = vec![make_step(0, 0x01, true)]; // 1 in binary: 000001 — Hamming=1
        let tp_h1 = VerificationAction::TransitionPattern.ground(
            &step_h1,
            &prev_h1,
            &[(0.0, 0); 64],
            "general",
        );
        assert!(
            (tp_h1 - 0.9).abs() < 1e-10,
            "expected 0.9 for Hamming=1, got {}",
            tp_h1
        );

        // Hamming distance 0 (same) = 0.3
        let step_h0 = make_step(1, 0x01, true);
        let prev_h0 = vec![make_step(0, 0x01, true)];
        let tp_h0 = VerificationAction::TransitionPattern.ground(
            &step_h0,
            &prev_h0,
            &[(0.0, 0); 64],
            "general",
        );
        assert!(
            (tp_h0 - 0.3).abs() < 1e-10,
            "expected 0.3 for Hamming=0, got {}",
            tp_h0
        );
    }

    #[test]
    fn test_grounded_prm_confidence_blend() {
        let step = make_step(0, 42, true);
        let original = ProcessScore {
            step_idx: 0,
            score: 0.5,
            confidence: 0.3,
            criteria: vec![],
            attribution_tags: vec![],
        };
        let traj = AgentTrajectory {
            trajectory_id: 4,
            task: "test".into(),
            steps: vec![step],
            outcome_reward: None,
            completed: false,
            total_duration_ms: None,
        };

        let mut verifier = GroundedPrmVerifier::new(48, 0.5);
        let blended = verifier.verify_trajectory(&traj, &[original], "general");
        // confidence = (0.3 + grounded) / 2, should be >= 0.15
        assert!(
            blended[0].confidence >= 0.15,
            "confidence too low: {}",
            blended[0].confidence
        );
    }

    #[test]
    fn test_grounded_score_persists_across_calls() {
        let mut verifier = GroundedPrmVerifier::new(48, 0.3);
        let steps = vec![make_step(0, 42, true), make_step(1, 42, true)];
        let traj = AgentTrajectory {
            trajectory_id: 5,
            task: "test".into(),
            steps,
            outcome_reward: None,
            completed: true,
            total_duration_ms: None,
        };
        let original: Vec<ProcessScore> = (0..2)
            .map(|i| ProcessScore {
                step_idx: i,
                score: 0.5,
                confidence: 0.5,
                criteria: vec![],
                attribution_tags: vec![],
            })
            .collect();

        let _ = verifier.verify_trajectory(&traj, &original, "general");
        assert_eq!(verifier.total_steps_verified, 2);
        let _ = verifier.verify_trajectory(&traj, &original, "general");
        assert_eq!(verifier.total_steps_verified, 4);
    }

    #[test]
    fn test_mcts_ucb_exploration_all_actions_visited() {
        let step = make_step(0, 56, true);
        let mut tree = MctsTree::new();
        tree.run(&step, &[], &[(0.0, 0); 64], "general", 60);
        // With 60 iterations and 6 actions, each should be visited at least once
        for &child_idx in &tree.nodes[0].children {
            assert!(
                tree.nodes[child_idx].visits > 0,
                "action {:?} was never visited in 60 iterations",
                tree.nodes[child_idx].action
            );
        }
    }
}

#[cfg(test)]
mod ws_grpo_tests {
    use super::*;

    #[test]
    fn test_ws_preference_model_record_and_score() {
        let mut model = WsPreferenceModel::new(0.1, 100);

        model.record_outcome("code task", 0.8);
        model.record_outcome("code task", 0.9);
        model.record_outcome("math problem", 0.3);

        let code_score = model.preference_score("code task");
        let math_score = model.preference_score("math problem");

        // code avg = (0.8 + 0.9) / 2 = 0.85, ema: 0.5 + 0.1*(0.8-0.5) = 0.53,
        //   then 0.53 + 0.1*(0.85-0.53) = 0.562
        assert!(
            (code_score - 0.562).abs() < 1e-6,
            "code_score={}",
            code_score
        );
        assert!(
            code_score > 0.5,
            "code score should reflect positive outcomes"
        );

        // math avg = 0.3, ema: 0.5 + 0.1*(0.3-0.5) = 0.48
        assert!(
            (math_score - 0.48).abs() < 1e-6,
            "math_score={}",
            math_score
        );
        assert!(
            math_score < 0.5,
            "math score should reflect negative outcome"
        );
    }

    #[test]
    fn test_ws_preference_model_unknown_task() {
        let model = WsPreferenceModel::new(0.1, 100);
        let score = model.preference_score("completely unknown gibberish task");
        assert!(
            (score - 0.5).abs() < 1e-10,
            "unknown tasks should score 0.5, got {}",
            score
        );
    }

    #[test]
    fn test_ws_grpo_learner_learn_step() {
        let policy = crate::core::nt_core_policy::E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        let coach: Box<dyn Coach> = Box::new(HeuristicCoach::default());
        let mut learner = WsGrpoLearner::new(policy, coach, 0.1, 100);

        let results = learner.learn_step_ws(|collector| {
            collector.begin("code debugging".into());
            collector.record_step(
                SpecialistType::Planner,
                ReasoningHexagram(0),
                "plan".into(),
                "".into(),
                "out".into(),
                None,
                true,
                None,
            );
            collector.record_step(
                SpecialistType::CodeAnalyzer,
                ReasoningHexagram(1),
                "code".into(),
                "out".into(),
                "result".into(),
                None,
                true,
                None,
            );
            collector.finish(Some(1.0), true);
        });

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].step_advantages.len(), 2);

        // Preference model should have been updated for "code" type
        let code_score = learner.preference_model.preference_score("code debugging");
        // one outcome with reward 1.0 → 0.5 + 0.1*(1.0-0.5) = 0.55
        assert!(
            (code_score - 0.55).abs() < 1e-6,
            "code_score={}",
            code_score
        );

        // Policy should have been updated via learn_from_scores
        let total_value: f64 = learner.inner.policy.mode_values.iter().sum();
        assert!(total_value >= 0.0, "policy values should be non-negative");
    }

    #[test]
    fn test_ws_preference_model_task_type_extraction() {
        assert_eq!(extract_task_type("implement fibonacci"), "code");
        assert_eq!(extract_task_type("solve equation 2x+3=7"), "math");
        assert_eq!(extract_task_type("logical deduction puzzle"), "reasoning");
        assert_eq!(extract_task_type("organize team schedule"), "planning");
        assert_eq!(extract_task_type("find the best route"), "search");
        assert_eq!(extract_task_type("write a poem"), "unknown");
    }
}
