use super::*;
use serde::{Deserialize, Serialize};
/// Step-GRPO configuration for overthinking reduction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StepGrpoConfig {
    /// Base reward for each correct step (default: 1.0)
    pub step_reward: f64,
    /// Penalty per step beyond optimal length (default: -0.1 per extra step)
    pub overthinking_penalty: f64,
    /// Reward for completing the task (default: 2.0)
    pub completion_bonus: f64,
    /// Optimal number of steps for the task type (default: 3)
    pub optimal_steps: usize,
    /// Maximum steps before heavy penalty (default: 10)
    pub max_steps: usize,
    /// Whether to apply length normalization (default: true)
    pub length_normalize: bool,
    /// GRPO clipping epsilon (default: 0.2)
    pub epsilon: f64,
}

impl Default for StepGrpoConfig {
    fn default() -> Self {
        Self {
            step_reward: 1.0,
            overthinking_penalty: -0.1,
            completion_bonus: 2.0,
            optimal_steps: 3,
            max_steps: 10,
            length_normalize: true,
            epsilon: 0.2,
        }
    }
}

/// Per-step reward with overthinking penalty
#[derive(Debug, Clone)]
pub struct StepReward {
    pub step_idx: usize,
    /// Base reward for this step
    pub base_reward: f64,
    /// Overthinking penalty applied
    pub overthinking_penalty: f64,
    /// Whether this step completed the task
    pub is_completion: bool,
    /// Final reward = base_reward + overthinking_penalty + completion_bonus
    pub final_reward: f64,
}

impl StepReward {
    pub fn new(step_idx: usize, base: f64) -> Self {
        Self {
            step_idx,
            base_reward: base,
            overthinking_penalty: 0.0,
            is_completion: false,
            final_reward: base,
        }
    }
}

/// Compute step-level rewards with overthinking penalty.
///
/// Steps beyond `optimal_steps` receive increasing penalties.
/// Steps beyond `max_steps` receive heavy penalties.
/// The completion step gets a bonus.
///
/// Returns (step_rewards, total_tokens_saved_estimate)
pub fn compute_step_rewards(
    trajectory: &AgentTrajectory,
    config: &StepGrpoConfig,
) -> (Vec<StepReward>, usize) {
    let total_steps = trajectory.steps.len();
    let mut rewards = Vec::with_capacity(total_steps);
    let mut tokens_saved = 0usize;

    for (i, step) in trajectory.steps.iter().enumerate() {
        let base = if step.success {
            config.step_reward
        } else {
            0.0
        };
        let mut reward = StepReward::new(i, base);

        if i >= config.optimal_steps {
            let extra = i - config.optimal_steps + 1;
            let penalty = if extra > (config.max_steps - config.optimal_steps) {
                -0.5 * (extra as f64)
            } else {
                config.overthinking_penalty * (extra as f64)
            };
            reward.overthinking_penalty = penalty;
            tokens_saved += 1;
        }

        if i == total_steps - 1 && step.success && trajectory.completed {
            reward.is_completion = true;
            reward.final_reward = base + reward.overthinking_penalty + config.completion_bonus;
        } else {
            reward.final_reward = base + reward.overthinking_penalty;
        }

        if config.length_normalize {
            reward.final_reward /= total_steps.max(1) as f64;
        }

        rewards.push(reward);
    }

    (rewards, tokens_saved)
}

/// Compute Step-GRPO advantages from step rewards.
///
/// Normalizes rewards within a group:
///   advantage_t = (reward_t - μ_group) / (σ_group + ε)
pub fn compute_step_advantages(
    all_rewards: &[Vec<StepReward>],
    _config: &StepGrpoConfig,
) -> Vec<Vec<f64>> {
    let all_final: Vec<f64> = all_rewards
        .iter()
        .flat_map(|rewards| rewards.iter().map(|r| r.final_reward))
        .collect();

    let mu = all_final.iter().sum::<f64>() / (all_final.len() as f64).max(1.0);
    let variance =
        all_final.iter().map(|r| (r - mu).powi(2)).sum::<f64>() / (all_final.len() as f64).max(1.0);
    let sigma = variance.sqrt().max(1e-8);

    all_rewards
        .iter()
        .map(|rewards| {
            rewards
                .iter()
                .map(|r| (r.final_reward - mu) / sigma)
                .collect()
        })
        .collect()
}

/// Estimate token savings from Step-GRPO compared to baseline
pub fn estimate_token_savings(all_rewards: &[Vec<StepReward>]) -> (usize, f64) {
    let total_saved: usize = all_rewards
        .iter()
        .flat_map(|r| r.iter())
        .map(|r| if r.overthinking_penalty < 0.0 { 1 } else { 0 })
        .sum();
    let total_steps: usize = all_rewards.iter().map(|r| r.len()).sum();
    let savings_ratio = if total_steps > 0 {
        total_saved as f64 / total_steps as f64
    } else {
        0.0
    };
    (total_saved, savings_ratio)
}

/// Step-GRPO report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepGrpoReport {
    pub config: StepGrpoConfig,
    pub trajectory_count: usize,
    pub total_steps: usize,
    pub total_tokens_saved: usize,
    pub savings_ratio: f64,
    pub avg_steps_per_task: f64,
}

/// Step-GRPO enhanced learner that reduces overthinking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepGrpoLearner {
    pub config: StepGrpoConfig,
    pub collector: TrajectoryCollector,
    pub reports: Vec<StepGrpoReport>,
}

impl StepGrpoLearner {
    pub fn new(config: StepGrpoConfig) -> Self {
        Self {
            config,
            collector: TrajectoryCollector::new(),
            reports: Vec::new(),
        }
    }

    /// Run Step-GRPO on collected trajectories and return (rewards, advantages, report)
    pub fn evaluate<F>(
        &mut self,
        collect_fn: F,
    ) -> (Vec<Vec<StepReward>>, Vec<Vec<f64>>, StepGrpoReport)
    where
        F: FnOnce(&mut TrajectoryCollector),
    {
        collect_fn(&mut self.collector);
        let trajectories: Vec<AgentTrajectory> = self.collector.collected.drain(..).collect();
        let total_steps: usize = trajectories.iter().map(|t| t.steps.len()).sum();
        let count = trajectories.len();

        let all_rewards: Vec<Vec<StepReward>> = trajectories
            .iter()
            .map(|t| compute_step_rewards(t, &self.config))
            .map(|(r, _)| r)
            .collect();

        let (total_saved, savings_ratio) = estimate_token_savings(&all_rewards);
        let advantages = compute_step_advantages(&all_rewards, &self.config);

        let report = StepGrpoReport {
            config: self.config,
            trajectory_count: count,
            total_steps,
            total_tokens_saved: total_saved,
            savings_ratio,
            avg_steps_per_task: if count > 0 {
                total_steps as f64 / count as f64
            } else {
                0.0
            },
        };
        self.reports.push(report.clone());

        (all_rewards, advantages, report)
    }
}

#[cfg(test)]
mod step_grpo_tests {
    use super::*;

    fn make_step_grpo_traj(
        id: u64,
        task: &str,
        n_steps: usize,
        all_success: bool,
        completed: bool,
    ) -> AgentTrajectory {
        let mut traj = AgentTrajectory::new(id, task.to_string());
        for i in 0..n_steps {
            traj.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "grpo_step".into(),
                input: "in".into(),
                output: "out".into(),
                duration_ms: None,
                success: all_success,
                external_reward: None,
            });
        }
        traj.completed = completed;
        traj.outcome_reward = if completed { Some(1.0) } else { None };
        traj
    }

    #[test]
    fn test_step_grpo_config_defaults() {
        let cfg = StepGrpoConfig::default();
        assert_eq!(cfg.step_reward, 1.0);
        assert_eq!(cfg.overthinking_penalty, -0.1);
        assert_eq!(cfg.completion_bonus, 2.0);
        assert_eq!(cfg.optimal_steps, 3);
        assert_eq!(cfg.max_steps, 10);
        assert!(cfg.length_normalize);
        assert_eq!(cfg.epsilon, 0.2);
    }

    #[test]
    fn test_compute_step_rewards_basic() {
        let cfg = StepGrpoConfig::default();
        let traj = make_step_grpo_traj(1, "simple", 3, true, true);
        let (rewards, saved) = compute_step_rewards(&traj, &cfg);

        assert_eq!(rewards.len(), 3);
        assert_eq!(saved, 0);
        // All steps succeed, length=3 = optimal, no overthinking penalty
        // length_normalize = true → divide by 3
        for r in &rewards {
            assert!(r.overthinking_penalty >= 0.0 || r.overthinking_penalty == 0.0);
        }
        // Last step gets completion bonus before normalization
        assert!(rewards[2].is_completion);
    }

    #[test]
    fn test_compute_step_rewards_overthinking_penalty() {
        let cfg = StepGrpoConfig {
            optimal_steps: 2,
            length_normalize: false,
            ..Default::default()
        };
        // 5 steps, optimal is 2 → steps 2,3,4 get penalties
        let traj = make_step_grpo_traj(1, "overthink", 5, true, true);
        let (rewards, saved) = compute_step_rewards(&traj, &cfg);

        assert_eq!(rewards.len(), 5);
        assert!(saved > 0);
        // Steps 0 and 1: no penalty
        assert_eq!(rewards[0].overthinking_penalty, 0.0);
        assert_eq!(rewards[1].overthinking_penalty, 0.0);
        // Step 2: extra = 1 → penalty = -0.1
        assert_eq!(rewards[2].overthinking_penalty, -0.1);
        // Step 3: extra = 2 → penalty = -0.2
        assert_eq!(rewards[3].overthinking_penalty, -0.2);
        // Step 4: extra = 3 → penalty = -0.3
        assert!((rewards[4].overthinking_penalty - (-0.3)).abs() < 1e-12);
    }

    #[test]
    fn test_compute_step_rewards_completion_bonus() {
        let cfg = StepGrpoConfig {
            length_normalize: false,
            ..Default::default()
        };
        let mut traj = make_step_grpo_traj(1, "bonus", 2, true, true);
        // Make completion explicit
        traj.completed = true;

        let (rewards, _) = compute_step_rewards(&traj, &cfg);
        // Last (index 1) step gets completion bonus = 2.0
        // base=1.0, penalty=0.0, bonus=2.0 → final=3.0
        assert!(rewards[1].is_completion);
        assert!((rewards[1].final_reward - 3.0).abs() < 1e-6);

        // Non-last step does not get completion bonus
        assert!(!rewards[0].is_completion);
        assert!((rewards[0].final_reward - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_step_advantages_basic() {
        let cfg = StepGrpoConfig::default();
        let traj1 = make_step_grpo_traj(1, "good", 2, true, true);
        let traj2 = make_step_grpo_traj(2, "bad", 2, false, false);

        let r1 = compute_step_rewards(&traj1, &cfg).0;
        let r2 = compute_step_rewards(&traj2, &cfg).0;

        let advantages = compute_step_advantages(&[r1, r2], &cfg);
        assert_eq!(advantages.len(), 2);
        assert_eq!(advantages[0].len(), 2);
        assert_eq!(advantages[1].len(), 2);

        // Good trajectory should have some positive advantages
        for &a in &advantages[0] {
            assert!(!a.is_nan());
        }
        // Bad trajectory should have some negative or lower advantages
        for &a in &advantages[1] {
            assert!(!a.is_nan());
        }
    }

    #[test]
    fn test_compute_step_advantages_group_normalization() {
        let cfg = StepGrpoConfig::default();
        // Three trajectories: high, medium, low
        let high = make_step_grpo_traj(1, "high", 2, true, true);
        let mid = make_step_grpo_traj(2, "mid", 2, true, false);
        let low = make_step_grpo_traj(3, "low", 2, false, false);

        let rh = compute_step_rewards(&high, &cfg).0;
        let rm = compute_step_rewards(&mid, &cfg).0;
        let rl = compute_step_rewards(&low, &cfg).0;

        let advantages = compute_step_advantages(&[rh, rm, rl], &cfg);
        assert_eq!(advantages.len(), 3);

        // All advantages should be normalized around 0
        let all: Vec<f64> = advantages.iter().flat_map(|v| v.iter().copied()).collect();
        let mean = all.iter().sum::<f64>() / all.len() as f64;
        assert!(
            mean.abs() < 1e-6,
            "group-normalized advantages should have mean near 0"
        );
    }

    #[test]
    fn test_estimate_token_savings() {
        let cfg = StepGrpoConfig {
            optimal_steps: 1,
            length_normalize: false,
            ..Default::default()
        };
        // 3-step trajectory where optimal is 1
        let traj = make_step_grpo_traj(1, "verbose", 3, true, true);
        let (rewards, _) = compute_step_rewards(&traj, &cfg);

        let (saved, ratio) = estimate_token_savings(&[rewards]);
        assert!(saved > 0);
        assert!(ratio > 0.0);
    }

    #[test]
    fn test_step_grpo_learner_evaluate() {
        let cfg = StepGrpoConfig::default();
        let mut learner = StepGrpoLearner::new(cfg);

        let (rewards, advantages, report) = learner.evaluate(|collector| {
            collector.begin("eval_test".into());
            collector.record_step(
                SpecialistType::Planner,
                ReasoningHexagram(0),
                "plan".into(),
                "".into(),
                "out1".into(),
                None,
                true,
                None,
            );
            collector.record_step(
                SpecialistType::CodeAnalyzer,
                ReasoningHexagram(1),
                "code".into(),
                "out1".into(),
                "out2".into(),
                None,
                true,
                None,
            );
            collector.record_step(
                SpecialistType::MetaCognitionAnalyst,
                ReasoningHexagram(2),
                "verify".into(),
                "out2".into(),
                "verified".into(),
                None,
                true,
                Some(1.0),
            );
            collector.finish(Some(1.0), true);
        });

        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].len(), 3);
        assert_eq!(advantages.len(), 1);
        assert_eq!(advantages[0].len(), 3);
        assert_eq!(report.trajectory_count, 1);
        assert_eq!(report.total_steps, 3);
    }

    #[test]
    fn test_step_grpo_report_generation() {
        let cfg = StepGrpoConfig::default();
        let mut learner = StepGrpoLearner::new(cfg);

        learner.evaluate(|collector| {
            collector.begin("report_test1".into());
            collector.record_step(
                SpecialistType::Planner,
                ReasoningHexagram(0),
                "step1".into(),
                "".into(),
                "out".into(),
                None,
                true,
                None,
            );
            collector.finish(Some(1.0), true);
        });

        learner.evaluate(|collector| {
            collector.begin("report_test2".into());
            collector.record_step(
                SpecialistType::Planner,
                ReasoningHexagram(0),
                "step1".into(),
                "".into(),
                "out".into(),
                None,
                true,
                None,
            );
            collector.record_step(
                SpecialistType::CodeAnalyzer,
                ReasoningHexagram(1),
                "step2".into(),
                "out".into(),
                "out2".into(),
                None,
                true,
                None,
            );
            collector.finish(Some(1.0), true);
        });

        assert_eq!(learner.reports.len(), 2);
        let r2 = &learner.reports[1];
        assert_eq!(r2.trajectory_count, 1);
        assert_eq!(r2.total_steps, 2);
        assert!(r2.avg_steps_per_task > 0.0);
    }

    #[test]
    fn test_compute_step_rewards_max_steps_heavy_penalty() {
        let cfg = StepGrpoConfig {
            optimal_steps: 2,
            max_steps: 4,
            length_normalize: false,
            ..Default::default()
        };
        // 6 steps, max_steps=4, optimal=2
        // Steps 0,1: no penalty
        // Step 2: extra=1, within max range (max-opt=2), penalty=-0.1
        // Step 3: extra=2, within max range, penalty=-0.2
        // Step 4: extra=3 > 2 → heavy penalty = -0.5*3 = -1.5
        // Step 5: extra=4 > 2 → heavy penalty = -0.5*4 = -2.0
        let traj = make_step_grpo_traj(1, "too_long", 6, true, true);
        let (rewards, _) = compute_step_rewards(&traj, &cfg);

        assert_eq!(rewards.len(), 6);
        // Steps 0,1: no penalty
        assert_eq!(rewards[0].overthinking_penalty, 0.0);
        assert_eq!(rewards[1].overthinking_penalty, 0.0);
        // Step 2: within max range
        assert_eq!(rewards[2].overthinking_penalty, -0.1);
        // Step 3: within max range
        assert_eq!(rewards[3].overthinking_penalty, -0.2);
        // Step 4: heavy penalty
        assert_eq!(rewards[4].overthinking_penalty, -1.5);
        // Step 5: heavy penalty
        assert_eq!(rewards[5].overthinking_penalty, -2.0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// WS-GRPO: Weakly Supervised GRPO with Preference Model
// ═══════════════════════════════════════════════════════════════════


/// Extract a task type from a task description by keyword matching.
///
/// Known types: code, math, reasoning, planning, search.
/// Falls back to "unknown" when no keywords match.
pub(super) fn extract_task_type(task: &str) -> String {
    let lower = task.to_lowercase();
    if lower.contains("code")
        || lower.contains("program")
        || lower.contains("implement")
        || lower.contains("function")
        || lower.contains("algorithm")
        || lower.contains("debug")
    {
        return "code".to_string();
    }
    if lower.contains("math")
        || lower.contains("equation")
        || lower.contains("calculate")
        || lower.contains("numerical")
        || lower.contains("arithmetic")
    {
        return "math".to_string();
    }
    if lower.contains("reason")
        || lower.contains("logic")
        || lower.contains("deduce")
        || lower.contains("infer")
        || lower.contains("syllogism")
    {
        return "reasoning".to_string();
    }
    if lower.contains("plan")
        || lower.contains("schedule")
        || lower.contains("organize")
        || lower.contains("strategy")
        || lower.contains("arrange")
    {
        return "planning".to_string();
    }
    if lower.contains("search")
        || lower.contains("find")
        || lower.contains("lookup")
        || lower.contains("retrieve")
        || lower.contains("query")
    {
        return "search".to_string();
    }
    "unknown".to_string()
}
