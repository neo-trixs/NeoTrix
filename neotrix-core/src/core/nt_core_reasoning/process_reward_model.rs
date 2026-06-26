use std::collections::{HashMap, VecDeque};

#[allow(unused_imports)]
use super::vsa_blackboard::{ExpertType, Hypothesis, VsaBlackboard};
use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

#[derive(Debug, Clone)]
pub struct PrmConfig {
    pub max_steps: usize,
    pub completion_reward: f64,
    pub penalty_per_step: f64,
    pub reward_discount: f64,
    pub hierarchical_levels: usize,
    pub min_step_reward: f64,
    pub max_step_reward: f64,
}

impl Default for PrmConfig {
    fn default() -> Self {
        Self {
            max_steps: 50,
            completion_reward: 1.0,
            penalty_per_step: -0.05,
            reward_discount: 0.95,
            hierarchical_levels: 3,
            min_step_reward: -1.0,
            max_step_reward: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub step_id: u64,
    pub hypothesis_id: Option<u64>,
    pub content: String,
    pub step_type: StepType,
    pub pre_reward: f64,
    pub post_reward: f64,
    pub process_score: f64,
    pub outcome_score: f64,
}

impl ReasoningStep {
    pub fn new(step_id: u64, content: String, step_type: StepType) -> Self {
        Self {
            step_id,
            hypothesis_id: None,
            content,
            step_type,
            pre_reward: 0.0,
            post_reward: 0.0,
            process_score: 0.0,
            outcome_score: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepType {
    Decompose,
    Infer,
    Retrieve,
    Verify,
    Synthesize,
    Abduce,
    Analogize,
    Counterfactual,
    Summarize,
    Meta,
}

impl StepType {
    pub fn all() -> Vec<StepType> {
        vec![
            StepType::Decompose,
            StepType::Infer,
            StepType::Retrieve,
            StepType::Verify,
            StepType::Synthesize,
            StepType::Abduce,
            StepType::Analogize,
            StepType::Counterfactual,
            StepType::Summarize,
            StepType::Meta,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            StepType::Decompose => "Decompose",
            StepType::Infer => "Infer",
            StepType::Retrieve => "Retrieve",
            StepType::Verify => "Verify",
            StepType::Synthesize => "Synthesize",
            StepType::Abduce => "Abduce",
            StepType::Analogize => "Analogize",
            StepType::Counterfactual => "Counterfactual",
            StepType::Summarize => "Summarize",
            StepType::Meta => "Meta",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrmStats {
    pub total_steps: usize,
    pub trajectories_completed: usize,
    pub avg_step_reward: f64,
    pub avg_chunk_reward: f64,
    pub avg_trajectory_reward: f64,
    pub reward_variance: f64,
    pub step_type_distribution: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct ProcessRewardModel {
    pub config: PrmConfig,
    pub steps: Vec<ReasoningStep>,
    pub next_step_id: u64,
    pub trajectories: Vec<Vec<u64>>,
    pub hierarchical_scores: Vec<Vec<f64>>,
    pub reward_history: VecDeque<f64>,
}

impl ProcessRewardModel {
    pub fn new(config: PrmConfig) -> Self {
        Self {
            config,
            steps: Vec::new(),
            next_step_id: 1,
            trajectories: Vec::new(),
            hierarchical_scores: Vec::new(),
            reward_history: VecDeque::new(),
        }
    }

    pub fn add_step(&mut self, step: ReasoningStep) -> u64 {
        if self.steps.len() >= self.config.max_steps {
            return 0;
        }
        let id = self.next_step_id;
        self.next_step_id += 1;
        let mut s = step;
        s.step_id = id;
        self.steps.push(s);
        id
    }

    pub fn evaluate_step(&mut self, step_id: u64) -> f64 {
        let idx = match self.steps.iter().position(|s| s.step_id == step_id) {
            Some(i) => i,
            None => return 0.0,
        };

        let step = &self.steps[idx];
        let quality = self.step_quality_score(step);
        let coherence = self.coherence_score(idx);
        let advance = step.outcome_score;

        let raw = 0.4 * quality + 0.3 * coherence + 0.3 * advance;
        let reward = raw.clamp(self.config.min_step_reward, self.config.max_step_reward);

        self.steps[idx].pre_reward = reward;
        self.steps[idx].process_score = quality;

        let discounted = reward * self.config.reward_discount;
        self.steps[idx].post_reward = discounted;
        self.reward_history.push_back(discounted);
        if self.reward_history.len() > self.config.max_steps * 2 {
            self.reward_history.pop_front();
        }

        discounted
    }

    pub fn final_reward(&mut self, success: bool) -> f64 {
        let n = self.steps.len();
        if n == 0 {
            return 0.0;
        }

        let total_penalty = self.config.penalty_per_step * n as f64;
        let completion = if success {
            self.config.completion_reward
        } else {
            0.0
        };

        let trajectory = self.trajectory_score();
        let avg_step = self.steps.iter().map(|s| s.post_reward).sum::<f64>() / n as f64;

        let reward = 0.5 * completion + 0.3 * trajectory + 0.2 * avg_step + total_penalty;
        let clamped = reward.clamp(self.config.min_step_reward, self.config.max_step_reward);

        self.trajectories
            .push(self.steps.iter().map(|s| s.step_id).collect());

        let mut hier_scores = Vec::new();
        hier_scores.push(self.steps.iter().map(|s| s.post_reward).collect());
        if n >= 3 {
            let chunk = self.chunk_score(0, n);
            hier_scores.push(vec![chunk; n]);
        }
        hier_scores.push(vec![trajectory; n]);
        self.hierarchical_scores = hier_scores;

        clamped
    }

    pub fn chunk_score(&self, start: usize, end: usize) -> f64 {
        if start >= end || end > self.steps.len() {
            return 0.0;
        }
        let chunk: &[ReasoningStep] = &self.steps[start..end];
        if chunk.is_empty() {
            return 0.0;
        }

        let avg_process = chunk.iter().map(|s| s.process_score).sum::<f64>() / chunk.len() as f64;
        let avg_outcome = chunk.iter().map(|s| s.outcome_score).sum::<f64>() / chunk.len() as f64;
        let step_type_diversity = {
            let types: std::collections::HashSet<StepType> =
                chunk.iter().map(|s| s.step_type).collect();
            types.len() as f64 / StepType::all().len() as f64
        };

        0.5 * avg_process + 0.3 * avg_outcome + 0.2 * step_type_diversity
    }

    pub fn trajectory_score(&self) -> f64 {
        if self.steps.is_empty() {
            return 0.0;
        }

        let n = self.steps.len();
        let n_chunks = (n + 4) / 5;
        let mut chunk_scores = Vec::new();

        for i in 0..n_chunks {
            let start = i * 5;
            let end = (start + 5).min(n);
            chunk_scores.push(self.chunk_score(start, end));
        }

        let avg_chunk = chunk_scores.iter().sum::<f64>() / chunk_scores.len() as f64;

        let monotonicity = {
            let mut increases = 0;
            for i in 1..n {
                if self.steps[i].outcome_score > self.steps[i - 1].outcome_score {
                    increases += 1;
                }
            }
            increases as f64 / n as f64
        };

        let end_weight = if n >= 3 {
            let last = &self.steps[n - 1];
            let second_last = &self.steps[n - 2];
            0.6 * last.outcome_score + 0.4 * second_last.outcome_score
        } else if n == 1 {
            self.steps[0].outcome_score
        } else {
            0.5 * self.steps[n - 1].outcome_score + 0.5 * self.steps[n - 2].outcome_score
        };

        0.5 * avg_chunk + 0.2 * monotonicity + 0.3 * end_weight
    }

    pub fn shaped_reward(&self, step_id: u64, next_state_value: f64) -> f64 {
        let idx = match self.steps.iter().position(|s| s.step_id == step_id) {
            Some(i) => i,
            None => return 0.0,
        };

        let step = &self.steps[idx];
        let current_potential = step.pre_reward;
        let shaping = self.config.reward_discount * next_state_value - current_potential;
        step.post_reward + shaping
    }

    pub fn vsa_coherence_reward(&self, step: &ReasoningStep, vsa_state: &[u8]) -> f64 {
        let step_vsa = QuantizedVSA::seeded_random(step.step_id.wrapping_mul(17), VSA_DIM);
        let sim = QuantizedVSA::similarity(&step_vsa, vsa_state);
        let coherence_bonus = if sim > 0.6 { sim * 0.3 } else { 0.0 };
        (sim + coherence_bonus).clamp(0.0, 1.0)
    }

    pub fn step_quality_score(&self, step: &ReasoningStep) -> f64 {
        let length_score = {
            let words = step.content.split_whitespace().count();
            if words < 3 {
                0.2
            } else if words > 100 {
                0.5
            } else {
                0.6 + 0.4 * (words as f64 / 50.0).min(1.0)
            }
        };

        let evidence_hint = {
            if step.hypothesis_id.is_some() {
                0.2
            } else {
                0.0
            }
        };

        let type_bonus = match step.step_type {
            StepType::Verify => 0.15,
            StepType::Synthesize => 0.1,
            StepType::Infer => 0.05,
            StepType::Meta => 0.1,
            StepType::Counterfactual => 0.05,
            _ => 0.0,
        };

        let has_question_mark = step.content.contains('?') as i32 as f64 * (-0.1);
        let has_numbers = step.content.chars().any(|c| c.is_ascii_digit()) as i32 as f64 * 0.1;

        let raw = length_score + evidence_hint + type_bonus + has_question_mark + has_numbers;
        raw.clamp(self.config.min_step_reward, self.config.max_step_reward)
    }

    fn coherence_score(&self, idx: usize) -> f64 {
        if idx == 0 || self.steps.is_empty() {
            return 1.0;
        }

        let current = &self.steps[idx];
        let prev = &self.steps[idx - 1];

        let type_transition = match (prev.step_type, current.step_type) {
            (StepType::Decompose, StepType::Infer) => 1.0,
            (StepType::Infer, StepType::Verify) => 1.0,
            (StepType::Infer, StepType::Synthesize) => 1.0,
            (StepType::Retrieve, StepType::Infer) => 1.0,
            (StepType::Verify, StepType::Synthesize) => 1.0,
            (StepType::Verify, StepType::Infer) => 0.8,
            (StepType::Synthesize, StepType::Summarize) => 1.0,
            (StepType::Abduce, StepType::Verify) => 1.0,
            (StepType::Analogize, StepType::Infer) => 0.9,
            (StepType::Counterfactual, StepType::Infer) => 0.8,
            (StepType::Summarize, StepType::Meta) => 1.0,
            (a, b) if a == b => 1.0,
            _ => 0.5,
        };

        let outcome_trend = if idx >= 2 {
            let prev_outcome = prev.outcome_score;
            let prev2_outcome = self.steps[idx - 2].outcome_score;
            if prev_outcome >= prev2_outcome {
                1.0
            } else {
                0.6
            }
        } else {
            1.0
        };

        let process_stability = {
            let window = 3.min(idx + 1);
            let scores: Vec<f64> = self.steps[idx + 1 - window..=idx]
                .iter()
                .map(|s| s.process_score)
                .collect();
            let mean = scores.iter().sum::<f64>() / scores.len() as f64;
            let variance =
                scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / scores.len() as f64;
            (-variance * 5.0).exp()
        };

        0.5 * type_transition + 0.3 * outcome_trend + 0.2 * process_stability
    }

    pub fn stats(&self) -> PrmStats {
        let n = self.steps.len();
        if n == 0 {
            return PrmStats {
                total_steps: 0,
                trajectories_completed: self.trajectories.len(),
                avg_step_reward: 0.0,
                avg_chunk_reward: 0.0,
                avg_trajectory_reward: 0.0,
                reward_variance: 0.0,
                step_type_distribution: HashMap::new(),
            };
        }

        let avg_step = self.steps.iter().map(|s| s.post_reward).sum::<f64>() / n as f64;
        let variance = self
            .steps
            .iter()
            .map(|s| (s.post_reward - avg_step).powi(2))
            .sum::<f64>()
            / n as f64;

        let n_chunks = (n + 4) / 5;
        let mut chunk_scores = Vec::new();
        for i in 0..n_chunks {
            let start = i * 5;
            let end = (start + 5).min(n);
            chunk_scores.push(self.chunk_score(start, end));
        }
        let avg_chunk = if chunk_scores.is_empty() {
            0.0
        } else {
            chunk_scores.iter().sum::<f64>() / chunk_scores.len() as f64
        };

        let avg_traj = if self.trajectories.is_empty() {
            self.trajectory_score()
        } else {
            self.trajectory_score()
        };

        let mut dist: HashMap<String, usize> = HashMap::new();
        for step in &self.steps {
            *dist.entry(step.step_type.name().to_string()).or_insert(0) += 1;
        }

        PrmStats {
            total_steps: n,
            trajectories_completed: self.trajectories.len(),
            avg_step_reward: avg_step,
            avg_chunk_reward: avg_chunk,
            avg_trajectory_reward: avg_traj,
            reward_variance: variance,
            step_type_distribution: dist,
        }
    }

    pub fn weakest_steps(&self, n: usize) -> Vec<&ReasoningStep> {
        let mut sorted: Vec<&ReasoningStep> = self.steps.iter().collect();
        sorted.sort_by(|a, b| {
            a.process_score
                .partial_cmp(&b.process_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(n.min(sorted.len()));
        sorted
    }

    pub fn strongest_steps(&self, n: usize) -> Vec<&ReasoningStep> {
        let mut sorted: Vec<&ReasoningStep> = self.steps.iter().collect();
        sorted.sort_by(|a, b| {
            b.process_score
                .partial_cmp(&a.process_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(n.min(sorted.len()));
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> PrmConfig {
        PrmConfig::default()
    }

    fn make_step(
        id: u64,
        content: &str,
        st: StepType,
        process: f64,
        outcome: f64,
    ) -> ReasoningStep {
        ReasoningStep {
            step_id: id,
            hypothesis_id: None,
            content: content.to_string(),
            step_type: st,
            pre_reward: 0.0,
            post_reward: 0.0,
            process_score: process,
            outcome_score: outcome,
        }
    }

    #[test]
    fn test_new_prm_has_empty_state() {
        let prm = ProcessRewardModel::new(default_config());
        assert!(prm.steps.is_empty());
        assert_eq!(prm.next_step_id, 1);
        assert!(prm.trajectories.is_empty());
    }

    #[test]
    fn test_default_config_values() {
        let cfg = PrmConfig::default();
        assert_eq!(cfg.max_steps, 50);
        assert_eq!(cfg.completion_reward, 1.0);
        assert_eq!(cfg.penalty_per_step, -0.05);
        assert_eq!(cfg.reward_discount, 0.95);
        assert_eq!(cfg.hierarchical_levels, 3);
        assert_eq!(cfg.min_step_reward, -1.0);
        assert_eq!(cfg.max_step_reward, 1.0);
    }

    #[test]
    fn test_add_step_returns_increasing_ids() {
        let mut prm = ProcessRewardModel::new(default_config());
        let s1 = ReasoningStep::new(0, "first step".into(), StepType::Infer);
        let s2 = ReasoningStep::new(0, "second step".into(), StepType::Verify);

        let id1 = prm.add_step(s1);
        let id2 = prm.add_step(s2);

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(prm.steps.len(), 2);
    }

    #[test]
    fn test_add_step_respects_max_steps() {
        let mut cfg = default_config();
        cfg.max_steps = 2;
        let mut prm = ProcessRewardModel::new(cfg);

        let s1 = ReasoningStep::new(0, "a".into(), StepType::Infer);
        let s2 = ReasoningStep::new(0, "b".into(), StepType::Infer);
        let s3 = ReasoningStep::new(0, "c".into(), StepType::Infer);

        prm.add_step(s1);
        prm.add_step(s2);
        let id3 = prm.add_step(s3);

        assert_eq!(id3, 0);
        assert_eq!(prm.steps.len(), 2);
    }

    #[test]
    fn test_evaluate_step_sets_rewards() {
        let mut prm = ProcessRewardModel::new(default_config());
        let id = prm.add_step(make_step(0, "test reasoning", StepType::Infer, 0.0, 0.5));

        let reward = prm.evaluate_step(id);
        let step = &prm.steps[0];

        assert!(reward > 0.0);
        assert!(step.pre_reward > 0.0);
        assert!(step.post_reward > 0.0);
        assert!(step.process_score > 0.0);
        assert!(prm.reward_history.len() == 1);
    }

    #[test]
    fn test_evaluate_step_unknown_id_returns_zero() {
        let mut prm = ProcessRewardModel::new(default_config());
        assert_eq!(prm.evaluate_step(999), 0.0);
    }

    #[test]
    fn test_final_reward_success_vs_failure() {
        let mut prm = ProcessRewardModel::new(default_config());
        prm.add_step(make_step(0, "step a", StepType::Infer, 0.5, 0.3));
        prm.add_step(make_step(0, "step b", StepType::Infer, 0.6, 0.6));
        prm.add_step(make_step(0, "step c", StepType::Synthesize, 0.8, 0.9));

        for step in prm.steps.clone() {
            prm.evaluate_step(step.step_id);
        }

        let success_reward = prm.final_reward(true);
        let failure_reward = prm.final_reward(false);

        assert!(success_reward > failure_reward);
        assert_eq!(prm.trajectories.len(), 2);
    }

    #[test]
    fn test_chunk_score_bounds() {
        let prm = ProcessRewardModel::new(default_config());
        assert_eq!(prm.chunk_score(0, 0), 0.0);
        assert_eq!(prm.chunk_score(5, 3), 0.0);
        assert_eq!(prm.chunk_score(0, 1), 0.0);
    }

    #[test]
    fn test_chunk_score_computes() {
        let mut prm = ProcessRewardModel::new(default_config());
        prm.add_step(make_step(0, "s1", StepType::Decompose, 0.7, 0.5));
        prm.add_step(make_step(0, "s2", StepType::Infer, 0.8, 0.6));
        prm.add_step(make_step(0, "s3", StepType::Verify, 0.9, 0.7));

        let score = prm.chunk_score(0, 3);
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_trajectory_score_empty() {
        let prm = ProcessRewardModel::new(default_config());
        assert_eq!(prm.trajectory_score(), 0.0);
    }

    #[test]
    fn test_trajectory_score_increases_with_good_steps() {
        let mut prm = ProcessRewardModel::new(default_config());
        prm.add_step(make_step(0, "s1", StepType::Decompose, 0.3, 0.2));
        prm.add_step(make_step(0, "s2", StepType::Infer, 0.5, 0.4));
        prm.add_step(make_step(0, "s3", StepType::Infer, 0.7, 0.6));
        prm.add_step(make_step(0, "s4", StepType::Verify, 0.8, 0.8));
        prm.add_step(make_step(0, "s5", StepType::Synthesize, 0.9, 0.95));

        let score = prm.trajectory_score();
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_shaped_reward_unknown_step() {
        let prm = ProcessRewardModel::new(default_config());
        assert_eq!(prm.shaped_reward(999, 0.5), 0.0);
    }

    #[test]
    fn test_shaped_reward_potential_based() {
        let mut prm = ProcessRewardModel::new(default_config());
        let id = prm.add_step(make_step(0, "step", StepType::Infer, 0.5, 0.5));
        prm.evaluate_step(id);

        let shaped = prm.shaped_reward(id, 0.9);
        assert!(shaped.is_finite());
    }

    #[test]
    fn test_vsa_coherence_reward() {
        let prm = ProcessRewardModel::new(default_config());
        let step = make_step(1, "test", StepType::Infer, 0.5, 0.5);
        let vsa_state = QuantizedVSA::random_binary();

        let reward = prm.vsa_coherence_reward(&step, &vsa_state);
        assert!(reward >= 0.0 && reward <= 1.0);
    }

    #[test]
    fn test_step_quality_score_content_length() {
        let prm = ProcessRewardModel::new(default_config());

        let short = make_step(0, "hi", StepType::Infer, 0.0, 0.0);
        let long = make_step(0, &"word ".repeat(60), StepType::Infer, 0.0, 0.0);

        let score_short = prm.step_quality_score(&short);
        let score_long = prm.step_quality_score(&long);

        assert!(score_short >= 0.0);
        assert!(score_long >= 0.0);
    }

    #[test]
    fn test_step_quality_score_verify_bonus() {
        let prm = ProcessRewardModel::new(default_config());
        let step = make_step(
            0,
            "verification with numbers 42",
            StepType::Verify,
            0.0,
            0.0,
        );
        let score = prm.step_quality_score(&step);

        let plain = make_step(0, "plain content", StepType::Decompose, 0.0, 0.0);
        let plain_score = prm.step_quality_score(&plain);

        assert!(score > plain_score);
    }

    #[test]
    fn test_stats_empty() {
        let prm = ProcessRewardModel::new(default_config());
        let stats = prm.stats();
        assert_eq!(stats.total_steps, 0);
        assert_eq!(stats.trajectories_completed, 0);
    }

    #[test]
    fn test_stats_tracks_distribution() {
        let mut prm = ProcessRewardModel::new(default_config());
        prm.add_step(make_step(0, "a", StepType::Infer, 0.5, 0.5));
        prm.add_step(make_step(0, "b", StepType::Verify, 0.5, 0.5));
        prm.add_step(make_step(0, "c", StepType::Infer, 0.5, 0.5));

        let stats = prm.stats();
        assert_eq!(stats.total_steps, 3);
        assert_eq!(*stats.step_type_distribution.get("Infer").unwrap(), 2);
        assert_eq!(*stats.step_type_distribution.get("Verify").unwrap(), 1);
    }

    #[test]
    fn test_weakest_and_strongest_steps() {
        let mut prm = ProcessRewardModel::new(default_config());
        prm.add_step(make_step(0, "low", StepType::Infer, 0.2, 0.0));
        prm.add_step(make_step(0, "mid", StepType::Infer, 0.5, 0.0));
        prm.add_step(make_step(0, "high", StepType::Infer, 0.9, 0.0));

        for step in prm.steps.clone() {
            prm.evaluate_step(step.step_id);
        }

        let weakest = prm.weakest_steps(2);
        let strongest = prm.strongest_steps(1);

        assert_eq!(weakest.len(), 2);
        assert!(weakest[0].process_score <= weakest[1].process_score);
        assert_eq!(strongest.len(), 1);
        assert_eq!(strongest[0].step_id, 3);
    }

    #[test]
    fn test_step_type_name() {
        assert_eq!(StepType::Decompose.name(), "Decompose");
        assert_eq!(StepType::Meta.name(), "Meta");
        assert_eq!(StepType::Counterfactual.name(), "Counterfactual");
    }

    #[test]
    fn test_step_type_all_includes_ten() {
        assert_eq!(StepType::all().len(), 10);
    }

    #[test]
    fn test_final_reward_empty_trajectories() {
        let mut prm = ProcessRewardModel::new(default_config());
        assert_eq!(prm.final_reward(true), 0.0);
        assert_eq!(prm.final_reward(false), 0.0);
    }
}
