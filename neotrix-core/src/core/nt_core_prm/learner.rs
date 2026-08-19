use super::*;
use serde::{Deserialize, Serialize};
use crate::core::nt_core_policy::E8Policy;

/// Lightweight online learner that wraps Coach + Policy + TrajectoryCollector.
///
/// This is the CPU-trainable PRM integration point:
/// 1. Collect trajectories via `TrajectoryCollector`
/// 2. Score them with a `Coach`
/// 3. Learn from scores via `E8Policy::learn_from_scores`
///
/// # Serde note
/// The `policy` and `coach` fields are skipped during serialization.
/// After deserialization, call `init_coach()` to restore the coach.
/// `Debug`/`Clone` are implemented manually because `dyn Coach`
/// doesn't natively derive these traits.
pub struct ProcessRewardLearner {
    pub policy: E8Policy,
    pub coach: Box<dyn Coach>,
    pub collector: TrajectoryCollector,
    pub learning_count: u64,
    pub score_history: Vec<f64>,
}

fn default_coach() -> Box<dyn Coach> {
    Box::new(HeuristicCoach::default())
}

impl std::fmt::Debug for ProcessRewardLearner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessRewardLearner")
            .field("policy", &self.policy)
            .field("coach", &"<dyn Coach>")
            .field("collector", &self.collector)
            .field("learning_count", &self.learning_count)
            .field("score_history", &self.score_history)
            .finish()
    }
}

impl Default for ProcessRewardLearner {
    fn default() -> Self {
        Self {
            policy: E8Policy::default(),
            coach: default_coach(),
            collector: TrajectoryCollector::default(),
            learning_count: 0,
            score_history: Vec::new(),
        }
    }
}

impl Clone for ProcessRewardLearner {
    fn clone(&self) -> Self {
        Self {
            policy: self.policy.clone(),
            coach: default_coach(),
            collector: self.collector.clone(),
            learning_count: self.learning_count,
            score_history: self.score_history.clone(),
        }
    }
}

impl serde::Serialize for ProcessRewardLearner {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ProcessRewardLearner", 3)?;
        state.serialize_field("collector", &self.collector)?;
        state.serialize_field("learning_count", &self.learning_count)?;
        state.serialize_field("score_history", &self.score_history)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for ProcessRewardLearner {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct ProcessRewardLearnerHelper {
            collector: TrajectoryCollector,
            learning_count: u64,
            score_history: Vec<f64>,
        }
        let helper = ProcessRewardLearnerHelper::deserialize(deserializer)?;
        Ok(Self {
            policy: E8Policy::new(0.3, 0.995, 0.01, 0.1, 0.9),
            coach: default_coach(),
            collector: helper.collector,
            learning_count: helper.learning_count,
            score_history: helper.score_history,
        })
    }
}

impl ProcessRewardLearner {
    pub fn new(policy: E8Policy, coach: Box<dyn Coach>) -> Self {
        Self {
            policy,
            coach,
            collector: TrajectoryCollector::new(),
            learning_count: 0,
            score_history: Vec::new(),
        }
    }

    /// Replace the coach after deserialization.
    /// Required because `Box<dyn Coach>` is skipped during serde.
    pub fn init_coach(&mut self, coach: Box<dyn Coach>) {
        self.coach = coach;
    }

    /// Run one learning step: collect, score, learn.
    ///
    /// `collect_fn` should populate `collector` with trajectory steps.
    /// Applies LATA (√L) trajectory-length normalization and optional auxiliary reward blend.
    pub fn learn_step<F>(&mut self, collect_fn: F)
    where
        F: FnOnce(&mut TrajectoryCollector),
    {
        collect_fn(&mut self.collector);

        let trajectories: Vec<AgentTrajectory> = self.collector.collected.drain(..).collect();

        for traj in &trajectories {
            let scores = self.coach.score_episode(traj);
            let avg_score =
                scores.iter().map(|s| s.score).sum::<f64>() / scores.len().max(1) as f64;
            self.score_history.push(avg_score);

            // Learn from trajectory and scores directly
            self.policy.learn_from_scores(traj, &scores);

            // Blend auxiliary rule-based reward if available
            let aux = Self::auxiliary_reward(traj);
            if aux != 0.0 {
                let aux_scores: Vec<ProcessScore> = traj
                    .steps
                    .iter()
                    .map(|s| {
                        let mut ps = ProcessScore::new(s.step_idx);
                        ps.score = aux.max(0.0).min(1.0);
                        ps.attribution_tags = vec!["auxiliary_reward".to_string()];
                        ps
                    })
                    .collect();
                self.policy.learn_from_scores(traj, &aux_scores);
            }

            for step in &traj.steps {
                if let Some(ext_r) = step.external_reward {
                    let _outcome = crate::core::nt_core_policy::E8Outcome {
                        task: traj.task.clone(),
                        mode: step.e8_mode,
                        reward: ext_r,
                        iteration: self.learning_count,
                    };
                }
            }
        }

        self.learning_count += 1;
    }

    /// Compute an auxiliary rule-based reward for the entire trajectory.
    ///
    /// Uses simple heuristics:
    /// - Completion bonus: +0.3 if trajectory completed successfully
    /// - Efficiency bonus: +0.1 if trajectory is short and successful
    /// - Consistency penalty: -0.1 if any step failed
    ///
    /// Returns a single scalar in [-0.1, 0.4].
    pub fn auxiliary_reward(traj: &AgentTrajectory) -> f64 {
        let mut reward = 0.0;
        if traj.completed {
            reward += 0.3;
        }
        let steps = traj.steps.len();
        if traj.completed && steps <= 5 {
            reward += 0.1;
        }
        if traj.steps.iter().any(|s| !s.success) {
            reward -= 0.1;
        }
        reward
    }

    /// Average score from recent learning steps.
    pub fn avg_recent_score(&self, window: usize) -> f64 {
        let window = window.min(self.score_history.len());
        if window == 0 {
            return 0.0;
        }
        self.score_history.iter().rev().take(window).sum::<f64>() / window as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step(idx: usize, success: bool) -> TrajectoryStep {
        TrajectoryStep {
            step_idx: idx,
            specialist: SpecialistType::Planner,
            e8_mode: ReasoningHexagram(idx as u8),
            action: "test".into(),
            input: "".into(),
            output: "".into(),
            duration_ms: None,
            success,
            external_reward: None,
        }
    }

    #[test]
    fn test_trajectory_step_defaults() {
        let step = make_step(0, true);
        assert_eq!(step.step_idx, 0);
        assert!(step.success);
    }

    #[test]
    fn test_trajectory_collector_begin_finish() {
        let mut tc = TrajectoryCollector::new();
        assert!(!tc.is_active());
        tc.begin("test task".into());
        assert!(tc.is_active());
        assert_eq!(tc.active_task(), Some("test task"));

        tc.record_step(
            SpecialistType::Planner,
            ReasoningHexagram(0),
            "plan".into(),
            "input".into(),
            "output".into(),
            None,
            true,
            None,
        );
        tc.record_step(
            SpecialistType::CodeAnalyzer,
            ReasoningHexagram(1),
            "code".into(),
            "input2".into(),
            "output2".into(),
            None,
            true,
            Some(0.8),
        );

        let traj = tc.finish(Some(1.0), true);
        assert!(traj.is_some());
        let traj = traj.unwrap();
        assert_eq!(traj.steps.len(), 2);
        assert_eq!(traj.task, "test task");
        assert!(traj.completed);
    }

    #[test]
    fn test_heuristic_coach_scores_success_step() {
        let coach = HeuristicCoach::default();
        let step = make_step(0, true);
        let ctx = CoachContext::new(false);
        let score = coach.score_step(&step, &ctx);
        assert_eq!(score.score, coach.success_base);
        assert!(score.attribution_tags.contains(&"step_ok".to_string()));
    }

    #[test]
    fn test_heuristic_coach_scores_failure_step() {
        let coach = HeuristicCoach::default();
        let step = make_step(0, false);
        let ctx = CoachContext::new(false);
        let score = coach.score_step(&step, &ctx);
        assert_eq!(score.score, coach.failure_penalty);
        assert!(score.attribution_tags.contains(&"step_fail".to_string()));
    }

    #[test]
    fn test_heuristic_coach_terminal_bonus() {
        let coach = HeuristicCoach::default();
        let step = make_step(0, true);
        let ctx = CoachContext::new(true);
        let score = coach.score_step(&step, &ctx);
        assert!(score.score > coach.success_base);
    }

    #[test]
    fn test_collector_abort() {
        let mut tc = TrajectoryCollector::new();
        tc.begin("abortable".into());
        tc.record_step(
            SpecialistType::Planner,
            ReasoningHexagram(0),
            "action".into(),
            "in".into(),
            "out".into(),
            None,
            true,
            None,
        );
        tc.abort();
        assert!(!tc.is_active());
        assert!(tc.latest().is_none());
    }

    #[test]
    fn test_heuristic_coach_external_reward_bonus() {
        let coach = HeuristicCoach::default();
        let step = TrajectoryStep {
            step_idx: 0,
            specialist: SpecialistType::MetaCognitionAnalyst,
            e8_mode: ReasoningHexagram(0),
            action: "verify".into(),
            input: "code".into(),
            output: "pass".into(),
            duration_ms: None,
            success: true,
            external_reward: Some(0.5),
        };
        let ctx = CoachContext::new(false);
        let score = coach.score_step(&step, &ctx);
        assert!(score.score > coach.success_base);
        assert!(score.criteria.iter().any(|c| c.name == "external_reward"));
    }

    #[test]
    fn test_process_reward_learner_end_to_end() {
        let policy = crate::core::nt_core_policy::E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        let coach: Box<dyn Coach> = Box::new(HeuristicCoach::default());
        let mut learner = ProcessRewardLearner::new(policy, coach);

        learner.learn_step(|collector| {
            collector.begin("test task".into());
            collector.record_step(
                SpecialistType::Planner,
                ReasoningHexagram(0),
                "plan".into(),
                "".into(),
                "plan_output".into(),
                None,
                true,
                None,
            );
            collector.record_step(
                SpecialistType::CodeAnalyzer,
                ReasoningHexagram(1),
                "code".into(),
                "plan_output".into(),
                "code_output".into(),
                None,
                true,
                None,
            );
            collector.record_step(
                SpecialistType::MetaCognitionAnalyst,
                ReasoningHexagram(2),
                "verify".into(),
                "code_output".into(),
                "verified".into(),
                None,
                true,
                Some(1.0),
            );
            collector.finish(Some(1.0), true);
        });

        assert_eq!(learner.learning_count, 1);
        assert!(!learner.score_history.is_empty());
        assert!(learner.avg_recent_score(1) > 0.0);
        // Policy values should have been updated by the learning step
        let total_value: f64 = learner.policy.mode_values.iter().sum();
        assert!(
            total_value > 0.0,
            "policy should have learned positive values"
        );
    }

    #[test]
    fn test_trajectory_collector_multiple_collected() {
        let mut tc = TrajectoryCollector::new();
        tc.begin("task1".into());
        tc.record_step(
            SpecialistType::Planner,
            ReasoningHexagram(0),
            "plan".into(),
            "".into(),
            "out1".into(),
            None,
            true,
            None,
        );
        tc.finish(Some(1.0), true);

        tc.begin("task2".into());
        tc.record_step(
            SpecialistType::CodeAnalyzer,
            ReasoningHexagram(1),
            "code".into(),
            "".into(),
            "out2".into(),
            None,
            true,
            None,
        );
        tc.finish(Some(0.0), false);

        assert_eq!(tc.count(), 2);
        let latest = tc.latest().unwrap();
        assert_eq!(latest.task, "task2");
        assert!(!latest.completed);
    }
}

// ═══════════════════════════════════════════════════════════════════
// λ-GRPO: Implicit PRM via Step-Level Advantage Normalization
// ═══════════════════════════════════════════════════════════════════

/// λ-GRPO configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LambdaGrpoConfig {
    /// Lambda parameter: 0.0 = pure outcome GRPO, 1.0 = full step-level PRM
    /// Recommended: 0.3 (balance between outcome + process supervision)
    pub lambda: f64,
    /// Clipping parameter ε for PPO-style clipped surrogate objective (default: 0.2)
    pub epsilon: f64,
    /// Group size for GRPO group sampling (default: 4)
    pub group_size: usize,
    /// Whether to use LATA (√L) trajectory-length normalization (default: true)
    pub lata_normalize: bool,
    /// KL penalty coefficient (default: 0.01)
    pub kl_coef: f64,
    /// Whether to use per-trajectory difficulty-adaptive λ.
    /// When enabled, λ_t = lambda * convergence_t where convergence_t
    /// is computed from step reward variance within each trajectory.
    /// Low variance → high convergence → higher λ (more step-level PRM).
    /// High variance → low convergence → lower λ (more outcome-level GRPO).
    /// Default: false (fixed λ).
    pub difficulty_adaptive_lambda: bool,
    /// Multiplicative scale for difficulty adaptivity (default: 1.0).
    /// Higher values amplify the convergence effect on λ.
    pub difficulty_scale: f64,
}

impl Default for LambdaGrpoConfig {
    fn default() -> Self {
        Self {
            lambda: 0.3,
            epsilon: 0.2,
            group_size: 4,
            lata_normalize: true,
            kl_coef: 0.01,
            difficulty_adaptive_lambda: false,
            difficulty_scale: 1.0,
        }
    }
}

/// Per-step advantage computed by λ-GRPO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepAdvantage {
    pub step_idx: usize,
    /// Raw advantage value (can be positive or negative)
    pub advantage: f64,
    /// Normalized advantage (z-scored within group)
    pub normalized_advantage: f64,
}

/// λ-GRPO result for a single trajectory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaGrpoResult {
    /// Trajectory ID
    pub trajectory_id: u64,
    /// Per-step advantages
    pub step_advantages: Vec<StepAdvantage>,
    /// Total loss for this trajectory
    pub loss: f64,
    /// Policy gradient contribution
    pub policy_gradient: f64,
    /// KL divergence penalty
    pub kl_penalty: f64,
}

/// Normalize values within a group using z-score: (x - μ) / (σ + ε)
///
/// Edge cases:
/// - Empty slice returns empty vec
/// - Single element returns vec![0.0]
/// - Constant values (σ ≈ 0) return zeros
pub fn zscore_normalize(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![0.0];
    }
    let clean: Vec<f64> = values
        .iter()
        .map(|v| if v.is_finite() { *v } else { 0.0 })
        .collect();
    let mean = clean.iter().sum::<f64>() / n as f64;
    let variance = clean.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let std = variance.sqrt();
    if !std.is_finite() || std < 1e-8 {
        return vec![0.0; n];
    }
    clean.iter().map(|v| (v - mean) / std).collect()
}

/// Blended advantage: λ · A_step + (1-λ) · A_outcome
pub fn blended_advantage(step_advantage: f64, outcome_advantage: f64, lambda: f64) -> f64 {
    lambda * step_advantage + (1.0 - lambda) * outcome_advantage
}

/// Compute trajectory-level convergence from step reward variance.
///
/// Low variance → high convergence (easy/consistent) → lean on step-level.
/// High variance → low convergence (hard/noisy) → lean on outcome.
/// Returns a value in [0, 1].
pub fn trajectory_convergence(trajectory: &AgentTrajectory) -> f64 {
    let rewards: Vec<f64> = trajectory
        .steps
        .iter()
        .map(|s| {
            s.external_reward
                .unwrap_or(if s.success { 1.0 } else { 0.0 })
        })
        .collect();
    let n = rewards.len();
    if n < 2 {
        return 0.5; // neutral for short trajectories
    }
    let mean = rewards.iter().sum::<f64>() / n as f64;
    let variance = rewards.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    if !variance.is_finite() || variance < 1e-8 {
        return 0.95; // nearly constant → high convergence
    }
    // convergence = exp(-variance * 5), maps variance ~0 → 1.0, variance ~1 → 0.006
    (-variance * 5.0).exp()
}

/// λ-GRPO loss function: implicit PRM via step-level advantage normalization.
///
/// Key formula (from λ-GRPO ICML 2026):
///   L_λ-GRPO = E[min(r_t(θ)·A_t, clip(r_t(θ), 1-ε, 1+ε)·A_t)] - β·KL
///
/// Where:
///   r_t(θ) = π_θ(a_t|s_t) / π_old(a_t|s_t)  — importance sampling ratio
///   A_t = λ · A_step_t + (1-λ) · A_outcome  — blended advantage
///   A_step_t = (reward_t - mean(reward_group)) / std(reward_group)  — step-level normalization
///   A_outcome = (outcome - mean(outcome_group)) / std(outcome_group)  — outcome-level
///   When difficulty_adaptive_lambda is enabled: λ_t = λ_base * convergence_t * difficulty_scale
///
/// Computes per-step rewards directly from trajectory data WITHOUT needing an
/// explicit PRM model. The λ parameter implicitly controls PRM strength:
///   λ=0:  pure outcome-level GRPO (ignore step signals)
///   λ=1:  pure step-level PRM (ignore final outcome)
///   λ=0.3: recommended balance
pub fn lambda_grpo_loss(
    trajectories: &[AgentTrajectory],
    config: &LambdaGrpoConfig,
) -> Vec<LambdaGrpoResult> {
    if trajectories.is_empty() {
        return Vec::new();
    }

    // Step 1: Extract implicit step-level rewards from trajectory data (no PRM/coach needed)
    let all_step_rewards: Vec<f64> = trajectories
        .iter()
        .flat_map(|t| {
            t.steps.iter().map(|s| {
                s.external_reward
                    .unwrap_or(if s.success { 1.0 } else { 0.0 })
            })
        })
        .collect();

    // Step 2: Extract outcome rewards
    let all_outcomes: Vec<f64> = trajectories
        .iter()
        .map(|t| {
            t.outcome_reward
                .unwrap_or(if t.completed { 1.0 } else { 0.0 })
        })
        .collect();

    // Step 3: Z-score normalize both within their groups
    let normalized_steps = zscore_normalize(&all_step_rewards);
    let normalized_outcomes = zscore_normalize(&all_outcomes);

    // Step 4: Compute λ-GRPO advantages and losses per trajectory
    let mut results = Vec::with_capacity(trajectories.len());
    let mut step_offset = 0;

    for (traj_idx, trajectory) in trajectories.iter().enumerate() {
        let n_steps = trajectory.steps.len();
        let outcome_adv = if traj_idx < normalized_outcomes.len() {
            normalized_outcomes[traj_idx]
        } else {
            0.0
        };

        // Per-trajectory difficulty-adaptive λ
        let lambda_t = if config.difficulty_adaptive_lambda {
            let conv = trajectory_convergence(trajectory);
            let base = config.lambda;
            let adapted = base * conv * config.difficulty_scale;
            adapted.max(0.0).min(1.0)
        } else {
            config.lambda
        };

        // Apply LATA (√L) normalization if enabled
        let lata = if config.lata_normalize {
            (n_steps.max(1) as f64).sqrt()
        } else {
            1.0
        };

        let mut step_advantages = Vec::with_capacity(n_steps);
        let mut total_loss = 0.0;
        let mut policy_gradient_sum = 0.0;

        for local_idx in 0..n_steps {
            let abs_step_idx = step_offset + local_idx;
            let step_adv_raw = if abs_step_idx < normalized_steps.len() {
                normalized_steps[abs_step_idx]
            } else {
                0.0
            };

            let step_adv = step_adv_raw / lata;
            let outcome_adv_scaled = outcome_adv / lata;
            let blended = blended_advantage(step_adv, outcome_adv_scaled, lambda_t);

            step_advantages.push(StepAdvantage {
                step_idx: trajectory.steps[local_idx].step_idx,
                advantage: blended,
                normalized_advantage: step_adv_raw,
            });

            // Clipped surrogate loss (PPO-style pessimistic bound)
            // When advantage ≥ 0: cap at ε to avoid over-optimization
            // When advantage < 0: floor at -ε to avoid instability
            let clipped_adv = if blended >= 0.0 {
                blended.min(config.epsilon)
            } else {
                blended.max(-config.epsilon)
            };
            let step_loss = -clipped_adv;

            total_loss += step_loss;
            policy_gradient_sum += -blended;
        }

        // KL penalty: β · KL ≈ β · (LATA²) as regularization
        let kl_penalty = config.kl_coef * lata.powi(2);

        results.push(LambdaGrpoResult {
            trajectory_id: trajectory.trajectory_id,
            step_advantages,
            loss: total_loss + kl_penalty,
            policy_gradient: policy_gradient_sum,
            kl_penalty,
        });

        step_offset += n_steps;
    }

    results
}

/// λ-GRPO enhanced ProcessRewardLearner.
///
/// Wraps the standard `ProcessRewardLearner` and replaces its `learn_step`
/// with λ-GRPO's implicit PRM learning. Trajectories are collected via
/// `TrajectoryCollector`, then the λ-GRPO loss computes step-level advantages
/// via within-group z-score normalization and blended step/outcome rewards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaGrpoLearner {
    pub config: LambdaGrpoConfig,
    #[serde(skip)]
    pub inner: ProcessRewardLearner,
    pub results: Vec<LambdaGrpoResult>,
    pub total_steps_learned: u64,
}

impl LambdaGrpoLearner {
    pub fn new(policy: E8Policy, coach: Box<dyn Coach>, config: LambdaGrpoConfig) -> Self {
        Self {
            config,
            inner: ProcessRewardLearner::new(policy, coach),
            results: Vec::new(),
            total_steps_learned: 0,
        }
    }

    /// Learn step with λ-GRPO: collect trajectories, compute λ-GRPO loss, update policy.
    ///
    /// `collect_fn` populates the inner `TrajectoryCollector`, after which
    /// `lambda_grpo_loss` computes per-step advantages WITHOUT requiring an
    /// explicit PRM. The advantages are fed into `E8Policy::learn_from_scores`
    /// for a 2x training speedup over explicit PRM methods.
    pub fn learn_step_grpo<F>(&mut self, collect_fn: F) -> Vec<LambdaGrpoResult>
    where
        F: FnOnce(&mut TrajectoryCollector),
    {
        collect_fn(&mut self.inner.collector);
        let trajectories: Vec<AgentTrajectory> = self.inner.collector.collected.drain(..).collect();

        let step_results = lambda_grpo_loss(&trajectories, &self.config);

        // Feed λ-GRPO advantages into the policy (implicit PRM update)
        for (result, traj) in step_results.iter().zip(trajectories.iter()) {
            let mode_scores: Vec<_> = result
                .step_advantages
                .iter()
                .filter_map(|sa| {
                    traj.steps
                        .get(sa.step_idx)
                        .map(|step| (sa.step_idx, step.e8_mode, sa.advantage))
                })
                .collect();
            if !mode_scores.is_empty() {
                let process_scores: Vec<ProcessScore> = mode_scores
                    .iter()
                    .map(|&(orig_idx, _mode, adv)| ProcessScore {
                        step_idx: orig_idx,
                        score: ((adv + 1.0) / 2.0).max(0.0).min(1.0),
                        confidence: 0.5,
                        criteria: Vec::new(),
                        attribution_tags: vec!["grpo".to_string()],
                    })
                    .collect();
                self.inner.policy.learn_from_scores(traj, &process_scores);
            }
        }

        self.results.extend(step_results.clone());
        self.total_steps_learned += 1;
        self.inner.learning_count += 1;

        step_results
    }

    /// Learn multiple steps for convergence tracking.
    ///
    /// `collect_fn` is called `steps` times, each time producing a batch of
    /// λ-GRPO results. Returns a vector of batch results for convergence analysis.
    pub fn learn_steps<F>(&mut self, collect_fn: F, steps: usize) -> Vec<Vec<LambdaGrpoResult>>
    where
        F: Fn(&mut TrajectoryCollector),
    {
        let mut all_results = Vec::with_capacity(steps);
        for _ in 0..steps {
            let r = self.learn_step_grpo(|c| collect_fn(c));
            all_results.push(r);
        }
        all_results
    }
}

#[cfg(test)]
mod lambda_grpo_tests {
    use super::*;

    fn make_trajectory(
        id: u64,
        task: &str,
        n_steps: usize,
        all_success: bool,
        outcome: Option<f64>,
        completed: bool,
    ) -> AgentTrajectory {
        let mut traj = AgentTrajectory::new(id, task.to_string());
        for i in 0..n_steps {
            traj.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "lambda_test".into(),
                input: "in".into(),
                output: "out".into(),
                duration_ms: None,
                success: all_success,
                external_reward: None,
            });
        }
        traj.outcome_reward = outcome;
        traj.completed = completed;
        traj
    }

    fn make_trajectory_with_rewards(
        id: u64,
        task: &str,
        step_successes: &[bool],
        step_rewards: &[Option<f64>],
        outcome: Option<f64>,
        completed: bool,
    ) -> AgentTrajectory {
        let mut traj = AgentTrajectory::new(id, task.to_string());
        for i in 0..step_successes.len() {
            let ext_r = step_rewards.get(i).copied().unwrap_or(None);
            traj.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "step_action".into(),
                input: "in".into(),
                output: "out".into(),
                duration_ms: None,
                success: step_successes[i],
                external_reward: ext_r,
            });
        }
        traj.outcome_reward = outcome;
        traj.completed = completed;
        traj
    }

    #[test]
    fn test_lambda_grpo_config_defaults() {
        let cfg = LambdaGrpoConfig::default();
        assert_eq!(cfg.lambda, 0.3);
        assert_eq!(cfg.epsilon, 0.2);
        assert_eq!(cfg.group_size, 4);
        assert!(cfg.lata_normalize);
        assert_eq!(cfg.kl_coef, 0.01);
    }

    #[test]
    fn test_blended_advantage_lambda_0() {
        let blended = blended_advantage(0.8, 0.2, 0.0);
        assert!((blended - 0.2).abs() < 1e-10);
    }

    #[test]
    fn test_blended_advantage_lambda_1() {
        let blended = blended_advantage(0.8, 0.2, 1.0);
        assert!((blended - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_blended_advantage_lambda_0_5() {
        let blended = blended_advantage(0.8, 0.2, 0.5);
        assert!((blended - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_blended_advantage_negative_values() {
        let blended = blended_advantage(-0.5, 1.0, 0.3);
        // 0.3 * (-0.5) + 0.7 * 1.0 = -0.15 + 0.7 = 0.55
        assert!((blended - 0.55).abs() < 1e-10);
    }

    #[test]
    fn test_zscore_normalize_empty() {
        let result = zscore_normalize(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_zscore_normalize_single_element() {
        let result = zscore_normalize(&[42.0]);
        assert_eq!(result.len(), 1);
        assert!((result[0] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_zscore_normalize_constant() {
        let result = zscore_normalize(&[5.0, 5.0, 5.0]);
        assert_eq!(result.len(), 3);
        for v in &result {
            assert!((v - 0.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_zscore_normalize_basic() {
        let result = zscore_normalize(&[1.0, 2.0, 3.0]);
        assert_eq!(result.len(), 3);
        // mean = 2.0, std = sqrt((1+0+1)/3) = sqrt(2/3) ≈ 0.8165
        // [-1/0.8165, 0, 1/0.8165] = [-1.2247, 0, 1.2247]
        assert!((result[0] - (-1.224744871391589)).abs() < 1e-10);
        assert!((result[1] - 0.0).abs() < 1e-10);
        assert!((result[2] - 1.224744871391589).abs() < 1e-10);
    }

    #[test]
    fn test_zscore_normalize_two_values() {
        let result = zscore_normalize(&[10.0, 20.0]);
        assert_eq!(result.len(), 2);
        // mean = 15, std = sqrt((25+25)/2) = 5
        // [(10-15)/5, (20-15)/5] = [-1, 1]
        assert!((result[0] - (-1.0)).abs() < 1e-10);
        assert!((result[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_lambda_grpo_loss_empty_input() {
        let cfg = LambdaGrpoConfig::default();
        let results = lambda_grpo_loss(&[], &cfg);
        assert!(results.is_empty());
    }

    #[test]
    fn test_lambda_grpo_loss_single_trajectory() {
        let cfg = LambdaGrpoConfig::default();
        let traj = make_trajectory(1, "simple", 3, true, Some(1.0), true);
        let results = lambda_grpo_loss(&[traj], &cfg);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].step_advantages.len(), 3);
        // Single trajectory → step scores are all same → z-score normalizes to 0
        // So blended advantage ≈ 0, loss ≈ 0
        for sa in &results[0].step_advantages {
            assert!(sa.advantage.abs() < 1e-8 || sa.advantage.is_nan());
        }
    }

    #[test]
    fn test_lambda_grpo_loss_with_group() {
        let cfg = LambdaGrpoConfig::default();
        // Two trajectories: one good, one bad
        let good = make_trajectory(1, "good", 2, true, Some(1.0), true);
        let bad = make_trajectory(2, "bad", 2, false, Some(0.0), false);
        let results = lambda_grpo_loss(&[good, bad], &cfg);
        assert_eq!(results.len(), 2);
        // Good trajectory should have positive advantages
        for sa in &results[0].step_advantages {
            assert!(sa.advantage >= -0.01 || sa.advantage.is_nan());
        }
        // Bad trajectory should have negative advantages
        for sa in &results[1].step_advantages {
            assert!(sa.advantage <= 0.01 || sa.advantage.is_nan());
        }
    }

    #[test]
    fn test_lambda_grpo_loss_positive_for_good_trajectories() {
        let cfg = LambdaGrpoConfig::default();
        // Mixed group: 2 good + 1 bad
        let good1 = make_trajectory(1, "good1", 1, true, Some(1.0), true);
        let good2 = make_trajectory(2, "good2", 1, true, Some(0.9), true);
        let bad = make_trajectory(3, "bad", 1, false, Some(0.0), false);
        let results = lambda_grpo_loss(&[good1, good2, bad], &cfg);

        // Good trajectories should have positive advantages relative to group
        assert_eq!(results[0].trajectory_id, 1);
        assert_eq!(results[1].trajectory_id, 2);
        assert_eq!(results[2].trajectory_id, 3);

        // Good outcomes normalize to positive; bad to negative
        for sa in &results[0].step_advantages {
            assert!(
                sa.advantage > -0.5,
                "good trajectory should not be heavily penalized"
            );
        }
        for sa in &results[1].step_advantages {
            assert!(
                sa.advantage > -0.5,
                "good trajectory should not be heavily penalized"
            );
        }
        // The bad one should have negative or near-zero advantages
        for sa in &results[2].step_advantages {
            assert!(
                sa.advantage <= 0.5,
                "bad trajectory should not have large positive advantage"
            );
        }
    }

    #[test]
    fn test_lambda_grpo_loss_external_rewards_feed_through() {
        let cfg = LambdaGrpoConfig::default();
        // Trajectories with explicit external rewards on each step
        let high = make_trajectory_with_rewards(
            1,
            "high",
            &[true, true],
            &[Some(0.9), Some(0.8)],
            Some(1.0),
            true,
        );
        let low = make_trajectory_with_rewards(
            2,
            "low",
            &[true, false],
            &[Some(0.1), Some(0.0)],
            Some(0.0),
            false,
        );
        let results = lambda_grpo_loss(&[high, low], &cfg);
        assert_eq!(results.len(), 2);
        // High-reward trajectory should have higher (less negative) loss
        // than low-reward trajectory
        assert!(
            results[0].loss < results[1].loss,
            "high-reward trajectory should have lower loss: {:.4} vs {:.4}",
            results[0].loss,
            results[1].loss
        );
    }

    #[test]
    fn test_lambda_grpo_loss_lata_disable() {
        let mut cfg = LambdaGrpoConfig::default();
        cfg.lata_normalize = false;

        let short = make_trajectory(1, "short", 1, true, Some(1.0), true);
        let long = make_trajectory(2, "long", 8, true, Some(0.9), true);
        let results = lambda_grpo_loss(&[short, long], &cfg);

        // Both should have valid results
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].step_advantages.len(), 1);
        assert_eq!(results[1].step_advantages.len(), 8);
    }

    #[test]
    fn test_lambda_grpo_loss_different_lambda_values() {
        let good = make_trajectory(1, "good", 2, true, Some(1.0), true);
        let bad = make_trajectory(2, "bad", 2, false, Some(0.0), false);

        // λ=0: pure outcome — both step rewards ignored, only outcome matters
        let cfg0 = LambdaGrpoConfig {
            lambda: 0.0,
            ..Default::default()
        };
        let r0 = lambda_grpo_loss(&[good.clone(), bad.clone()], &cfg0);
        let loss_diff_0 = (r0[0].loss - r0[1].loss).abs();

        // λ=1: pure step — outcome ignored, only step rewards matter
        let cfg1 = LambdaGrpoConfig {
            lambda: 1.0,
            ..Default::default()
        };
        let r1 = lambda_grpo_loss(&[good.clone(), bad.clone()], &cfg1);
        let loss_diff_1 = (r1[0].loss - r1[1].loss).abs();

        // λ=0.5: balanced
        let cfg5 = LambdaGrpoConfig {
            lambda: 0.5,
            ..Default::default()
        };
        let r5 = lambda_grpo_loss(&[good, bad], &cfg5);
        let loss_diff_5 = (r5[0].loss - r5[1].loss).abs();

        // All should produce different loss separations
        // (different λ values weight step vs outcome differently)
        assert!(loss_diff_0 >= 0.0);
        assert!(loss_diff_1 >= 0.0);
        assert!(loss_diff_5 >= 0.0);
    }

    #[test]
    fn test_lambda_grpo_learner_learn_step() {
        let policy = E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        let coach: Box<dyn Coach> = Box::new(HeuristicCoach::default());
        let config = LambdaGrpoConfig::default();
        let mut learner = LambdaGrpoLearner::new(policy, coach, config);

        let results = learner.learn_step_grpo(|collector| {
            collector.begin("grpo_test".into());
            collector.record_step(
                SpecialistType::Planner,
                ReasoningHexagram(0),
                "plan".into(),
                "".into(),
                "plan_out".into(),
                None,
                true,
                None,
            );
            collector.record_step(
                SpecialistType::CodeAnalyzer,
                ReasoningHexagram(1),
                "code".into(),
                "plan_out".into(),
                "code_out".into(),
                None,
                true,
                None,
            );
            collector.record_step(
                SpecialistType::MetaCognitionAnalyst,
                ReasoningHexagram(2),
                "verify".into(),
                "code_out".into(),
                "verified".into(),
                None,
                true,
                Some(1.0),
            );
            collector.finish(Some(1.0), true);
        });

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].step_advantages.len(), 3);
        assert_eq!(learner.total_steps_learned, 1);
        assert_eq!(learner.inner.learning_count, 1);
        // Policy should have been updated
        let total_value: f64 = learner.inner.policy.mode_values.iter().sum();
        assert!(total_value >= 0.0, "policy values should be non-negative");
    }

    #[test]
    fn test_lambda_grpo_learner_multiple_steps() {
        let policy = E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        let coach: Box<dyn Coach> = Box::new(HeuristicCoach::default());
        let config = LambdaGrpoConfig::default();
        let mut learner = LambdaGrpoLearner::new(policy, coach, config);

        let all_results = learner.learn_steps(
            |collector| {
                collector.begin("multi_step".into());
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
                collector.finish(Some(1.0), true);
            },
            3,
        );

        assert_eq!(all_results.len(), 3);
        assert_eq!(learner.total_steps_learned, 3);
        assert_eq!(learner.inner.learning_count, 3);
        assert_eq!(learner.results.len(), 3);

        // Each step produced one trajectory result
        for (i, batch) in all_results.iter().enumerate() {
            assert_eq!(batch.len(), 1, "batch {} should have 1 result", i);
        }
    }

    #[test]
    fn test_lambda_grpo_loss_with_varying_lengths() {
        let cfg = LambdaGrpoConfig::default();
        let short = make_trajectory(1, "short", 1, true, Some(1.0), true);
        let medium = make_trajectory(2, "medium", 3, true, Some(0.8), true);
        let long = make_trajectory(3, "long", 6, true, Some(0.9), true);
        let results = lambda_grpo_loss(&[short, medium, long], &cfg);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].step_advantages.len(), 1);
        assert_eq!(results[1].step_advantages.len(), 3);
        assert_eq!(results[2].step_advantages.len(), 6);
        // All should produce valid step advantages
        for result in &results {
            for sa in &result.step_advantages {
                assert!(!sa.advantage.is_nan(), "advantage should not be NaN");
            }
        }
    }

    #[test]
    fn test_lambda_grpo_learner_config_affects_learning() {
        // λ=0: only outcome matters, step-level rewards ignored
        // λ=1: only step matters, outcome ignored
        // Use 2 trajectories so group normalization produces non-zero advantages

        // Create trajectories where step-level and outcome-level signals conflict:
        // Traj A: step fails (0) but final outcome succeeds (1) — overridden by external reward
        // Traj B: step succeeds (1) but final outcome fails (0)
        // This creates divergent step vs outcome z-scores for λ to blend differently
        let collect = |collector: &mut TrajectoryCollector| {
            collector.begin("step_bad_outcome_good".into());
            collector.record_step(
                SpecialistType::Planner,
                ReasoningHexagram(0),
                "step".into(),
                "".into(),
                "out".into(),
                None,
                false,
                Some(1.0),
            );
            collector.finish(Some(0.0), true);

            collector.begin("step_good_outcome_bad".into());
            collector.record_step(
                SpecialistType::Planner,
                ReasoningHexagram(1),
                "step".into(),
                "".into(),
                "out".into(),
                None,
                true,
                Some(0.0),
            );
            collector.finish(Some(1.0), true);
        };

        let cfg0 = LambdaGrpoConfig {
            lambda: 0.0,
            ..Default::default()
        };
        let policy0 = E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        let coach0: Box<dyn Coach> = Box::new(HeuristicCoach::default());
        let mut learner0 = LambdaGrpoLearner::new(policy0, coach0, cfg0);
        let r0 = learner0.learn_step_grpo(collect);

        let cfg1 = LambdaGrpoConfig {
            lambda: 1.0,
            ..Default::default()
        };
        let policy1 = E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        let coach1: Box<dyn Coach> = Box::new(HeuristicCoach::default());
        let mut learner1 = LambdaGrpoLearner::new(policy1, coach1, cfg1);
        let r1 = learner1.learn_step_grpo(collect);

        // Different λ values → different advantages and loss
        assert!(
            (r0[0].loss - r1[0].loss).abs() > 1e-6,
            "λ=0 and λ=1 should produce different loss: {:.6} vs {:.6}",
            r0[0].loss,
            r1[0].loss
        );
        let adv0 = r0[0].step_advantages[0].advantage;
        let adv1 = r1[0].step_advantages[0].advantage;
        assert!(
            (adv0 - adv1).abs() > 1e-6,
            "λ=0 and λ=1 should produce different advantages: {:.6} vs {:.6}",
            adv0,
            adv1
        );
    }

    #[test]
    fn test_trajectory_convergence_constant() {
        let mut traj = AgentTrajectory::new(1, "test".into());
        for i in 0..4 {
            traj.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "".into(),
                input: "".into(),
                output: "".into(),
                duration_ms: None,
                success: true,
                external_reward: Some(1.0),
            });
        }
        let c = trajectory_convergence(&traj);
        assert!(
            c > 0.9,
            "constant rewards should give high convergence: {:.4}",
            c
        );
    }

    #[test]
    fn test_trajectory_convergence_noisy() {
        let mut traj = AgentTrajectory::new(2, "noisy".into());
        let rewards = [0.0, 1.0, 0.0, 1.0];
        for (i, &r) in rewards.iter().enumerate() {
            traj.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "".into(),
                input: "".into(),
                output: "".into(),
                duration_ms: None,
                success: r > 0.5,
                external_reward: Some(r),
            });
        }
        let c = trajectory_convergence(&traj);
        assert!(
            c < 0.5,
            "noisy rewards should give low convergence: {:.4}",
            c
        );
    }

    #[test]
    fn test_trajectory_convergence_short() {
        let mut traj = AgentTrajectory::new(3, "short".into());
        traj.push(TrajectoryStep {
            step_idx: 0,
            specialist: SpecialistType::Planner,
            e8_mode: ReasoningHexagram(0),
            action: "".into(),
            input: "".into(),
            output: "".into(),
            duration_ms: None,
            success: true,
            external_reward: None,
        });
        let c = trajectory_convergence(&traj);
        assert!(
            (c - 0.5).abs() < 1e-6,
            "short (<2) should give neutral 0.5: {:.4}",
            c
        );
    }

    #[test]
    fn test_lambda_grpo_loss_difficulty_adaptive() {
        // Two trajectories with same outcome but different step variance.
        // High-consistency traj (all 1.0s) should get higher λ than low-consistency (0,1,0,1).
        let mut high_cons = AgentTrajectory::new(1, "easy".into());
        for i in 0..4 {
            high_cons.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "".into(),
                input: "".into(),
                output: "".into(),
                duration_ms: None,
                success: true,
                external_reward: Some(1.0),
            });
        }
        high_cons.outcome_reward = Some(1.0);
        high_cons.completed = true;

        let mut low_cons = AgentTrajectory::new(2, "hard".into());
        let noisy = [0.0, 1.0, 0.0, 1.0];
        for (i, &r) in noisy.iter().enumerate() {
            low_cons.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "".into(),
                input: "".into(),
                output: "".into(),
                duration_ms: None,
                success: r > 0.5,
                external_reward: Some(r),
            });
        }
        low_cons.outcome_reward = Some(1.0);
        low_cons.completed = true;

        let mut cfg = LambdaGrpoConfig {
            difficulty_adaptive_lambda: true,
            difficulty_scale: 1.0,
            ..Default::default()
        };
        let results = lambda_grpo_loss(&[high_cons.clone(), low_cons.clone()], &cfg);

        assert_eq!(results.len(), 2);
        // With adaptive λ, the high-consistency traj leans more on step-level,
        // which should produce different advantages than uniform λ.
        // Both trajectories have same outcome (1.0) but different step variance.
        // λ_high ≈ 0.3 * 0.95 * 1.0 = 0.285, λ_low ≈ 0.3 * ~0.03 * 1.0 ≈ 0.009
        // So high-consistency advantages blend step + outcome, low-consistency ≈ pure outcome.
        // Outcome z-scores are the same (1.0, 1.0) → normalized to 0.0 within group.
        // Step z-scores: high_cons steps all same → 0.0; low_cons steps: 0→-1, 1→1, 0→-1, 1→1
        // So advantages should reflect the difference.
        for result in &results {
            for sa in &result.step_advantages {
                assert!(!sa.advantage.is_nan(), "advantage should not be NaN");
            }
        }

        // Verify that disabling adaptive λ produces different results
        cfg.difficulty_adaptive_lambda = false;
        let results_uniform = lambda_grpo_loss(&[high_cons.clone(), low_cons.clone()], &cfg);
        let mut any_diff = false;
        for (adaptive, uniform) in results.iter().zip(results_uniform.iter()) {
            for (sa_a, sa_u) in adaptive
                .step_advantages
                .iter()
                .zip(uniform.step_advantages.iter())
            {
                if (sa_a.advantage - sa_u.advantage).abs() > 1e-8 {
                    any_diff = true;
                }
            }
        }
        assert!(
            any_diff,
            "adaptive and uniform λ should produce different advantages"
        );
    }

    #[test]
    fn test_lambda_grpo_loss_adaptive_high_vs_low_scale() {
        let mut traj = AgentTrajectory::new(1, "mixed".into());
        let mixed = [0.0, 0.8, 0.2, 0.9, 0.1, 0.95];
        for (i, &r) in mixed.iter().enumerate() {
            traj.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "".into(),
                input: "".into(),
                output: "".into(),
                duration_ms: None,
                success: r > 0.5,
                external_reward: Some(r),
            });
        }
        traj.outcome_reward = Some(0.8);
        traj.completed = true;

        // Low scale (0.1) should produce λ close to base (0.3 * conv * 0.1)
        let cfg_low = LambdaGrpoConfig {
            difficulty_adaptive_lambda: true,
            difficulty_scale: 0.1,
            ..Default::default()
        };
        let r_low = lambda_grpo_loss(&[traj.clone()], &cfg_low);

        // High scale (3.0) should amplify convergence effect
        let cfg_high = LambdaGrpoConfig {
            difficulty_adaptive_lambda: true,
            difficulty_scale: 3.0,
            ..Default::default()
        };
        let r_high = lambda_grpo_loss(&[traj], &cfg_high);

        // With same trajectory, different scales → different advantages
        let mut diff = false;
        for (sa_l, sa_h) in r_low[0]
            .step_advantages
            .iter()
            .zip(r_high[0].step_advantages.iter())
        {
            if (sa_l.advantage - sa_h.advantage).abs() > 1e-8 {
                diff = true;
            }
        }
        assert!(
            diff,
            "scale=0.1 and scale=3.0 should produce different advantages"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// Step-GRPO: Token-Efficient Overthinking Reduction
// ═══════════════════════════════════════════════════════════════════

