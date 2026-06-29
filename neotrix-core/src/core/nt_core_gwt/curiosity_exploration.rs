use rand::Rng;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct CuriosityConfig {
    pub exploration_bonus: f64,
    pub curiosity_decay: f64,
    pub info_gain_threshold: f64,
    pub max_exploration_steps: usize,
    pub diversity_weight: f64,
    pub bandit_alpha: f64,
    pub bandit_beta: f64,
}

impl Default for CuriosityConfig {
    fn default() -> Self {
        Self {
            exploration_bonus: 0.3,
            curiosity_decay: 0.9,
            info_gain_threshold: 0.1,
            max_exploration_steps: 10,
            diversity_weight: 0.2,
            bandit_alpha: 2.0,
            bandit_beta: 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GapType {
    KnowledgeGap,
    SkillGap,
    UncertaintyGap,
    ContradictionGap,
    NoveltyGap,
}

#[derive(Debug, Clone)]
pub struct KnowledgeGap {
    pub id: u64,
    pub domain: String,
    pub gap_type: GapType,
    pub predicted_info_gain: f64,
    pub curiosity_signal: f64,
    pub is_open: bool,
}

#[derive(Debug, Clone)]
pub struct CuriosityExploration {
    pub config: CuriosityConfig,
    pub novelty_scores: HashMap<String, f64>,
    pub exploration_budget: f64,
    pub total_explorations: usize,
    pub successful_explorations: usize,
    pub knowledge_gaps: Vec<KnowledgeGap>,
    pub bandit_history: Vec<(String, bool, f64)>,
    pub exploration_trajectory: VecDeque<String>,
}

impl CuriosityExploration {
    pub fn new(config: CuriosityConfig) -> Self {
        Self {
            config,
            novelty_scores: HashMap::new(),
            exploration_budget: 1.0,
            total_explorations: 0,
            successful_explorations: 0,
            knowledge_gaps: Vec::new(),
            bandit_history: Vec::new(),
            exploration_trajectory: VecDeque::new(),
        }
    }

    pub fn compute_curiosity_signal(&self, prediction_error: f64, novelty: f64) -> f64 {
        let info_gain = prediction_error.max(0.0);
        let curiosity = 0.6 * info_gain + 0.4 * novelty;
        curiosity.clamp(0.0, 1.0)
    }

    pub fn identify_gaps(&mut self, domains: &[String]) -> Vec<u64> {
        let mut ids = Vec::new();
        let mut next_id = self.knowledge_gaps.len() as u64 + 1;

        for domain in domains {
            let novelty = self.novelty_of(domain);
            if novelty > self.config.info_gain_threshold {
                let predicted_gain = self.predicted_information_gain(domain);
                let curiosity = self.compute_curiosity_signal(0.5, novelty);
                let gap_type = if novelty > 0.8 {
                    GapType::NoveltyGap
                } else if predicted_gain > 0.6 {
                    GapType::KnowledgeGap
                } else {
                    GapType::UncertaintyGap
                };
                let gap = KnowledgeGap {
                    id: next_id,
                    domain: domain.clone(),
                    gap_type,
                    predicted_info_gain: predicted_gain,
                    curiosity_signal: curiosity,
                    is_open: true,
                };
                ids.push(gap.id);
                self.knowledge_gaps.push(gap);
                next_id += 1;
            }
        }
        ids
    }

    pub fn select_exploration(&mut self, actions: &[String]) -> Option<String> {
        if actions.is_empty() {
            return None;
        }
        if self.exploration_budget <= 0.0 {
            return None;
        }
        let open_gaps: Vec<&KnowledgeGap> =
            self.knowledge_gaps.iter().filter(|g| g.is_open).collect();
        if !open_gaps.is_empty() {
            let best = open_gaps
                .iter()
                .max_by(|a, b| {
                    a.curiosity_signal
                        .partial_cmp(&b.curiosity_signal)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();
            if actions.contains(&best.domain) {
                return Some(best.domain.clone());
            }
        }
        if !actions.is_empty() {
            let recent: std::collections::HashSet<&str> = self
                .exploration_trajectory
                .iter()
                .map(|s| s.as_str())
                .collect();
            for action in actions {
                if !recent.contains(action.as_str()) {
                    return Some(action.clone());
                }
            }
            let idx = self.thompson_sample(
                &self
                    .bandit_history
                    .iter()
                    .filter_map(|(a, s, r)| {
                        if actions.contains(a) {
                            Some((if *s { r + 1.0 } else { 0.5 }, 1.0))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            );
            if idx < actions.len() {
                return Some(actions[idx].clone());
            }
            return Some(actions[0].clone());
        }
        None
    }

    pub fn update_novelty(&mut self, domain: &str) {
        let entry = self.novelty_scores.entry(domain.to_string()).or_insert(1.0);
        *entry = 1.0;
    }

    pub fn novelty_of(&self, domain: &str) -> f64 {
        self.novelty_scores.get(domain).copied().unwrap_or(1.0)
    }

    pub fn decay_novelty(&mut self) {
        for val in self.novelty_scores.values_mut() {
            *val *= self.config.curiosity_decay;
        }
        self.novelty_scores.retain(|_, v| *v > 0.01);
    }

    pub fn predicted_information_gain(&self, action: &str) -> f64 {
        let novelty = self.novelty_of(action);
        let recency_penalty = if !self.exploration_trajectory.is_empty() {
            let count = self
                .exploration_trajectory
                .iter()
                .filter(|s| s.as_str() == action)
                .count();
            1.0 / (count as f64 + 1.0)
        } else {
            1.0
        };
        let prior_entropy = -novelty * novelty.ln().max(-10.0);
        let posterior_entropy = -recency_penalty * recency_penalty.ln().max(-10.0);
        let gain = prior_entropy - posterior_entropy;
        (gain * 0.5 + novelty * 0.5).clamp(0.0, 1.0)
    }

    pub fn actual_information_gain(&self, prior_entropy: f64, posterior_entropy: f64) -> f64 {
        let gain = prior_entropy - posterior_entropy;
        gain.max(0.0)
    }

    pub fn exploration_bonus(&self, action: &str) -> f64 {
        let novelty = self.novelty_of(action);
        let visit_count = self
            .bandit_history
            .iter()
            .filter(|(a, _, _)| a == action)
            .count() as f64;
        let bonus = self.config.exploration_bonus / (visit_count + 1.0).sqrt();
        bonus * (1.0 + novelty * self.config.diversity_weight)
    }

    pub fn shaped_reward(&self, action: &str, base_reward: f64) -> f64 {
        let bonus = self.exploration_bonus(action);
        let novelty_bonus = self.novelty_of(action) * self.config.diversity_weight;
        base_reward + bonus + novelty_bonus
    }

    pub fn thompson_sample(&self, action_successes: &[(f64, f64)]) -> usize {
        if action_successes.is_empty() {
            return 0;
        }
        let mut rng = rand::thread_rng();
        let mut best_idx = 0;
        let mut best_sample = f64::NEG_INFINITY;
        for (i, &(alpha, beta)) in action_successes.iter().enumerate() {
            let a = (alpha + self.config.bandit_alpha).max(0.01);
            let b = (beta + self.config.bandit_beta).max(0.01);
            let sample = sample_beta(&mut rng, a, b);
            if sample > best_sample {
                best_sample = sample;
                best_idx = i;
            }
        }
        best_idx
    }

    pub fn record_bandit_outcome(&mut self, action: &str, success: bool, reward: f64) {
        self.total_explorations += 1;
        if success {
            self.successful_explorations += 1;
        }
        self.exploration_budget = (self.exploration_budget - 0.1).max(0.0);
        if success {
            let replenish = reward * 0.15;
            self.exploration_budget = (self.exploration_budget + replenish).min(1.0);
        }
        self.bandit_history
            .push((action.to_string(), success, reward));
        self.exploration_trajectory.push_back(action.to_string());
        if self.exploration_trajectory.len() > 20 {
            self.exploration_trajectory.pop_front();
        }
        if let Some(gap) = self
            .knowledge_gaps
            .iter_mut()
            .find(|g| g.domain == action && g.is_open)
        {
            if success {
                gap.is_open = false;
            }
        }
    }

    pub fn stats(&self) -> CuriosityStats {
        let open_gaps = self.knowledge_gaps.iter().filter(|g| g.is_open).count();
        let avg_info_gain = if self.bandit_history.is_empty() {
            0.0
        } else {
            self.bandit_history.iter().map(|(_, _, r)| *r).sum::<f64>()
                / self.bandit_history.len() as f64
        };
        let exploration_success_rate = if self.total_explorations > 0 {
            self.successful_explorations as f64 / self.total_explorations as f64
        } else {
            0.0
        };
        CuriosityStats {
            total_explorations: self.total_explorations,
            successful_explorations: self.successful_explorations,
            open_gaps,
            avg_info_gain,
            exploration_success_rate,
        }
    }
}

fn sample_beta(rng: &mut impl Rng, alpha: f64, beta: f64) -> f64 {
    let alpha = alpha.max(0.01);
    let beta = beta.max(0.01);
    let x = rng.gen::<f64>();
    let y = rng.gen::<f64>();
    let u = x.powf(1.0 / alpha);
    let v = y.powf(1.0 / beta);
    let sum = u + v;
    if sum > 0.0 {
        u / sum
    } else {
        0.5
    }
}

#[derive(Debug, Clone)]
pub struct CuriosityStats {
    pub total_explorations: usize,
    pub successful_explorations: usize,
    pub open_gaps: usize,
    pub avg_info_gain: f64,
    pub exploration_success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> CuriosityConfig {
        CuriosityConfig::default()
    }

    #[test]
    fn test_new_exploration_defaults() {
        let ce = CuriosityExploration::new(default_config());
        assert_eq!(ce.total_explorations, 0);
        assert_eq!(ce.successful_explorations, 0);
        assert_eq!(ce.knowledge_gaps.len(), 0);
        assert!((ce.exploration_budget - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_compute_curiosity_signal_zero_input() {
        let ce = CuriosityExploration::new(default_config());
        let signal = ce.compute_curiosity_signal(0.0, 0.0);
        assert!((signal - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_compute_curiosity_signal_max_input() {
        let ce = CuriosityExploration::new(default_config());
        let signal = ce.compute_curiosity_signal(1.0, 1.0);
        assert!((signal - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_compute_curiosity_signal_mixed() {
        let ce = CuriosityExploration::new(default_config());
        let signal = ce.compute_curiosity_signal(0.5, 0.5);
        let expected = 0.6 * 0.5 + 0.4 * 0.5;
        assert!((signal - expected).abs() < 1e-9);
    }

    #[test]
    fn test_identify_gaps_creates_gaps() {
        let mut ce = CuriosityExploration::new(default_config());
        let domains: Vec<String> = vec!["math".into(), "physics".into()];
        let ids = ce.identify_gaps(&domains);
        assert_eq!(ids.len(), 2);
        assert_eq!(ce.knowledge_gaps.len(), 2);
        assert!(ce.knowledge_gaps.iter().all(|g| g.is_open));
    }

    #[test]
    fn test_identify_gaps_below_threshold() {
        let mut ce = CuriosityExploration::new(default_config());
        ce.novelty_scores.insert("known".into(), 0.01);
        let domains: Vec<String> = vec!["known".into()];
        let ids = ce.identify_gaps(&domains);
        assert!(ids.is_empty());
    }

    #[test]
    fn test_novelty_update_and_decay() {
        let mut ce = CuriosityExploration::new(default_config());
        ce.update_novelty("math");
        assert!((ce.novelty_of("math") - 1.0).abs() < 1e-9);
        ce.decay_novelty();
        let expected = 1.0 * 0.9;
        assert!((ce.novelty_of("math") - expected).abs() < 1e-9);
    }

    #[test]
    fn test_novelty_of_unknown_domain() {
        let ce = CuriosityExploration::new(default_config());
        assert!((ce.novelty_of("unknown") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_decay_novelty_removes_stale() {
        let mut ce = CuriosityExploration::new(default_config());
        ce.novelty_scores.insert("stale".into(), 0.005);
        ce.decay_novelty();
        assert!(ce.novelty_of("stale") < 0.01);
    }

    #[test]
    fn test_predicted_information_gain_high_novelty() {
        let ce = CuriosityExploration::new(default_config());
        let gain = ce.predicted_information_gain("novel_domain");
        assert!(gain > 0.0);
        assert!(gain <= 1.0);
    }

    #[test]
    fn test_predicted_information_gain_low_novelty() {
        let mut ce = CuriosityExploration::new(default_config());
        ce.novelty_scores.insert("boring".into(), 0.05);
        let gain = ce.predicted_information_gain("boring");
        assert!(gain < 0.5);
    }

    #[test]
    fn test_actual_information_gain() {
        let ce = CuriosityExploration::new(default_config());
        let gain = ce.actual_information_gain(1.0, 0.2);
        assert!((gain - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_actual_information_gain_negative_clamped() {
        let ce = CuriosityExploration::new(default_config());
        let gain = ce.actual_information_gain(0.2, 1.0);
        assert!((gain - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_exploration_bonus_decreases_with_visits() {
        let mut ce = CuriosityExploration::new(default_config());
        let first = ce.exploration_bonus("test");
        ce.record_bandit_outcome("test", true, 0.5);
        let second = ce.exploration_bonus("test");
        assert!(second < first);
    }

    #[test]
    fn test_shaped_reward_includes_bonus() {
        let ce = CuriosityExploration::new(default_config());
        let reward = ce.shaped_reward("new_action", 0.5);
        assert!(reward > 0.5);
    }

    #[test]
    fn test_select_exploration_empty_actions() {
        let mut ce = CuriosityExploration::new(default_config());
        assert!(ce.select_exploration(&[]).is_none());
    }

    #[test]
    fn test_select_exploration_returns_gap_domain() {
        let mut ce = CuriosityExploration::new(default_config());
        ce.novelty_scores.insert("math".into(), 0.9);
        let domains: Vec<String> = vec!["math".into()];
        ce.identify_gaps(&domains);
        let selected = ce.select_exploration(&["math".into(), "physics".into()]);
        assert_eq!(selected, Some("math".into()));
    }

    #[test]
    fn test_select_exploration_diverse_when_no_gaps() {
        let mut ce = CuriosityExploration::new(default_config());
        ce.exploration_budget = 1.0;
        let actions: Vec<String> = vec!["a".into(), "b".into()];
        let selected = ce.select_exploration(&actions);
        assert!(selected.is_some());
    }

    #[test]
    fn test_select_exploration_none_when_budget_exhausted() {
        let mut ce = CuriosityExploration::new(default_config());
        ce.exploration_budget = 0.0;
        assert!(ce.select_exploration(&["anything".into()]).is_none());
    }

    #[test]
    fn test_record_bandit_outcome_increments_counters() {
        let mut ce = CuriosityExploration::new(default_config());
        assert_eq!(ce.total_explorations, 0);
        assert_eq!(ce.successful_explorations, 0);
        ce.record_bandit_outcome("test_action", true, 0.8);
        assert_eq!(ce.total_explorations, 1);
        assert_eq!(ce.successful_explorations, 1);
    }

    #[test]
    fn test_record_bandit_outcome_failure() {
        let mut ce = CuriosityExploration::new(default_config());
        ce.record_bandit_outcome("fail_action", false, 0.0);
        assert_eq!(ce.total_explorations, 1);
        assert_eq!(ce.successful_explorations, 0);
    }

    #[test]
    fn test_record_bandit_outcome_closes_gap_on_success() {
        let mut ce = CuriosityExploration::new(default_config());
        ce.novelty_scores.insert("gap_domain".into(), 0.9);
        ce.identify_gaps(&["gap_domain".into()]);
        assert!(ce.knowledge_gaps[0].is_open);
        ce.record_bandit_outcome("gap_domain", true, 0.9);
        assert!(!ce.knowledge_gaps[0].is_open);
    }

    #[test]
    fn test_record_bandit_outcome_trajectory_bounded() {
        let mut ce = CuriosityExploration::new(default_config());
        for i in 0..25 {
            ce.record_bandit_outcome(&format!("action_{}", i), true, 0.5);
        }
        assert_eq!(ce.exploration_trajectory.len(), 20);
    }

    #[test]
    fn test_stats_empty() {
        let ce = CuriosityExploration::new(default_config());
        let stats = ce.stats();
        assert_eq!(stats.total_explorations, 0);
        assert_eq!(stats.open_gaps, 0);
        assert!((stats.exploration_success_rate - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_stats_after_explorations() {
        let mut ce = CuriosityExploration::new(default_config());
        ce.record_bandit_outcome("a", true, 0.8);
        ce.record_bandit_outcome("b", false, 0.0);
        ce.record_bandit_outcome("c", true, 0.6);
        let stats = ce.stats();
        assert_eq!(stats.total_explorations, 3);
        assert_eq!(stats.successful_explorations, 2);
        assert!((stats.exploration_success_rate - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_thompson_sample_returns_valid_index() {
        let ce = CuriosityExploration::new(default_config());
        let successes = vec![(1.0, 1.0), (5.0, 1.0), (1.0, 5.0)];
        let idx = ce.thompson_sample(&successes);
        assert!(idx < 3);
    }

    #[test]
    fn test_thompson_sample_empty() {
        let ce = CuriosityExploration::new(default_config());
        assert_eq!(ce.thompson_sample(&[]), 0);
    }

    #[test]
    fn test_exploration_budget_replenished_on_success() {
        let mut ce = CuriosityExploration::new(default_config());
        ce.exploration_budget = 0.1;
        ce.record_bandit_outcome("a", true, 1.0);
        assert!(ce.exploration_budget > 0.1);
    }

    #[test]
    fn test_curiosity_config_default_sane() {
        let cfg = CuriosityConfig::default();
        assert!((cfg.exploration_bonus - 0.3).abs() < 1e-9);
        assert!((cfg.curiosity_decay - 0.9).abs() < 1e-9);
        assert!((cfg.info_gain_threshold - 0.1).abs() < 1e-9);
        assert_eq!(cfg.max_exploration_steps, 10);
    }
}
