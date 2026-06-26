use std::collections::{HashMap, VecDeque};

use super::dead_end_detector::DeadEndType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReasoningStrategy {
    Analogical,
    Causal,
    MultiHop,
    Deductive,
    Inductive,
    Abductive,
    Counterfactual,
    MctsTreeSearch,
    Decomposition,
    MetaReflection,
}

impl ReasoningStrategy {
    pub fn all() -> Vec<ReasoningStrategy> {
        vec![
            ReasoningStrategy::Analogical,
            ReasoningStrategy::Causal,
            ReasoningStrategy::MultiHop,
            ReasoningStrategy::Deductive,
            ReasoningStrategy::Inductive,
            ReasoningStrategy::Abductive,
            ReasoningStrategy::Counterfactual,
            ReasoningStrategy::MctsTreeSearch,
            ReasoningStrategy::Decomposition,
            ReasoningStrategy::MetaReflection,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            ReasoningStrategy::Analogical => "analogical",
            ReasoningStrategy::Causal => "causal",
            ReasoningStrategy::MultiHop => "multi_hop",
            ReasoningStrategy::Deductive => "deductive",
            ReasoningStrategy::Inductive => "inductive",
            ReasoningStrategy::Abductive => "abductive",
            ReasoningStrategy::Counterfactual => "counterfactual",
            ReasoningStrategy::MctsTreeSearch => "mcts_tree_search",
            ReasoningStrategy::Decomposition => "decomposition",
            ReasoningStrategy::MetaReflection => "meta_reflection",
        }
    }
}

#[derive(Debug)]
pub struct StrategyConfig {
    pub max_failures_before_switch: usize,
    pub performance_window: usize,
    pub switch_cost_penalty: f64,
    pub exploration_rate: f64,
    pub min_samples_per_strategy: usize,
    pub recovery_strategy: ReasoningStrategy,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            max_failures_before_switch: 3,
            performance_window: 10,
            switch_cost_penalty: -0.1,
            exploration_rate: 0.1,
            min_samples_per_strategy: 2,
            recovery_strategy: ReasoningStrategy::Decomposition,
        }
    }
}

#[derive(Debug)]
pub struct StrategyPerformance {
    pub strategy: ReasoningStrategy,
    pub attempts: usize,
    pub successes: usize,
    pub avg_confidence: f64,
    pub avg_reward: f64,
    pub recent_rewards: VecDeque<f64>,
    pub failure_patterns: Vec<DeadEndType>,
    pub is_starving: bool,
}

impl StrategyPerformance {
    fn new(strategy: ReasoningStrategy) -> Self {
        Self {
            strategy,
            attempts: 0,
            successes: 0,
            avg_confidence: 0.0,
            avg_reward: 0.0,
            recent_rewards: VecDeque::with_capacity(10),
            failure_patterns: Vec::new(),
            is_starving: true,
        }
    }

    fn success_rate(&self) -> f64 {
        if self.attempts == 0 {
            return 0.0;
        }
        self.successes as f64 / self.attempts as f64
    }

    fn record_outcome(&mut self, reward: f64, success: bool) {
        self.attempts += 1;
        if success {
            self.successes += 1;
        }
        let n = self.attempts as f64;
        self.avg_reward = self.avg_reward + (reward - self.avg_reward) / n;
        self.recent_rewards.push_back(reward);
        if self.recent_rewards.len() > 10 {
            self.recent_rewards.pop_front();
        }
        self.is_starving = self.attempts < 2;
    }

    fn record_failure_pattern(&mut self, dead_end: DeadEndType) {
        self.failure_patterns.push(dead_end);
        if self.failure_patterns.len() > 20 {
            self.failure_patterns.remove(0);
        }
    }

    #[allow(dead_code)]
    fn dominant_failure(&self) -> Option<DeadEndType> {
        if self.failure_patterns.is_empty() {
            return None;
        }
        let mut counts: HashMap<DeadEndType, usize> = HashMap::new();
        for &p in &self.failure_patterns {
            *counts.entry(p).or_insert(0) += 1;
        }
        counts.into_iter().max_by_key(|&(_, c)| c).map(|(t, _)| t)
    }
}

#[derive(Debug)]
pub struct SelfHealingSelector {
    pub config: StrategyConfig,
    pub performances: HashMap<ReasoningStrategy, StrategyPerformance>,
    pub current_strategy: ReasoningStrategy,
    pub strategy_history: Vec<(ReasoningStrategy, f64, bool)>,
    pub switch_count: usize,
    pub last_switch_step: usize,
    pub total_steps: usize,
    pub recovery_count: usize,
}

impl SelfHealingSelector {
    pub fn new(config: StrategyConfig) -> Self {
        let mut performances = HashMap::new();
        for s in ReasoningStrategy::all() {
            performances.insert(s, StrategyPerformance::new(s));
        }
        Self {
            config,
            performances,
            current_strategy: ReasoningStrategy::Decomposition,
            strategy_history: Vec::new(),
            switch_count: 0,
            last_switch_step: 0,
            total_steps: 0,
            recovery_count: 0,
        }
    }

    pub fn select_strategy(&mut self, problem_domain: &str) -> ReasoningStrategy {
        self.total_steps += 1;
        let _ = problem_domain;

        if self.should_switch() {
            if let Some(best) = self.best_performing() {
                self.switch_strategy(best);
                return self.current_strategy;
            }
        }

        let r = rand::random::<f64>();
        if r < self.config.exploration_rate {
            let starving = self.detect_starvation();
            if !starving.is_empty() {
                let idx = (rand::random::<f64>() * starving.len() as f64) as usize;
                let chosen = starving[idx.min(starving.len() - 1)];
                if chosen != self.current_strategy {
                    self.switch_strategy(chosen);
                }
                return self.current_strategy;
            }
            let all = ReasoningStrategy::all();
            let idx = (rand::random::<f64>() * all.len() as f64) as usize;
            let chosen = all[idx.min(all.len() - 1)];
            if chosen != self.current_strategy {
                self.switch_strategy(chosen);
            }
            return self.current_strategy;
        }

        let mut best_score = f64::NEG_INFINITY;
        let mut best_strat = self.current_strategy;
        for perf in self.performances.values() {
            let score = self.ucb_score(perf);
            if score > best_score {
                best_score = score;
                best_strat = perf.strategy;
            }
        }
        if best_strat != self.current_strategy {
            self.switch_strategy(best_strat);
        }
        self.current_strategy
    }

    pub fn record_outcome(&mut self, strategy: ReasoningStrategy, reward: f64, success: bool) {
        if let Some(perf) = self.performances.get_mut(&strategy) {
            perf.record_outcome(reward, success);
        }
        self.strategy_history.push((strategy, reward, success));
    }

    pub fn record_failure_pattern(&mut self, strategy: ReasoningStrategy, dead_end: DeadEndType) {
        if let Some(perf) = self.performances.get_mut(&strategy) {
            perf.record_failure_pattern(dead_end);
        }
    }

    fn ucb_score(&self, perf: &StrategyPerformance) -> f64 {
        if perf.attempts == 0 {
            return f64::MAX;
        }
        let exploitation = perf.avg_reward;
        let total_attempts: usize = self.performances.values().map(|p| p.attempts).sum();
        let exploration = (2.0 * (total_attempts as f64).ln() / perf.attempts as f64).sqrt();
        exploitation + exploration
    }

    fn best_performing(&self) -> Option<ReasoningStrategy> {
        self.performances
            .values()
            .filter(|p| p.attempts >= self.config.min_samples_per_strategy)
            .max_by(|a, b| {
                a.avg_reward
                    .partial_cmp(&b.avg_reward)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.strategy)
    }

    fn should_switch(&self) -> bool {
        let current = match self.performances.get(&self.current_strategy) {
            Some(p) => p,
            None => return false,
        };
        if current.attempts < self.config.min_samples_per_strategy {
            return false;
        }
        if self.total_steps - self.last_switch_step < current.attempts / 2 {
            return false;
        }
        let window_len = self
            .config
            .performance_window
            .min(current.recent_rewards.len());
        if window_len < 2 {
            return false;
        }
        let recent: Vec<&f64> = current
            .recent_rewards
            .iter()
            .rev()
            .take(window_len)
            .collect();
        recent.len() >= self.config.max_failures_before_switch
            && recent.iter().all(|&&r| r <= 0.0)
            && current.success_rate() < 0.3
    }

    fn switch_strategy(&mut self, new: ReasoningStrategy) {
        self.current_strategy = new;
        self.switch_count += 1;
        self.last_switch_step = self.total_steps;
    }

    fn detect_starvation(&self) -> Vec<ReasoningStrategy> {
        self.performances
            .values()
            .filter(|p| p.is_starving)
            .map(|p| p.strategy)
            .collect()
    }

    pub fn heal(&mut self, failed_strategy: ReasoningStrategy) -> ReasoningStrategy {
        let recovery = self.recovery_strategies();
        self.recovery_count += 1;
        let target = recovery
            .into_iter()
            .find(|&s| s != failed_strategy && s != self.current_strategy)
            .unwrap_or(self.config.recovery_strategy);
        self.switch_strategy(target);
        self.current_strategy
    }

    pub fn recovery_strategies(&self) -> Vec<ReasoningStrategy> {
        let mut ranked: Vec<&StrategyPerformance> = self
            .performances
            .values()
            .filter(|p| p.attempts > 0)
            .collect();
        ranked.sort_by(|a, b| {
            b.avg_reward
                .partial_cmp(&a.avg_reward)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked.into_iter().map(|p| p.strategy).collect()
    }

    pub fn update_exploration_rate(&mut self) {
        let total: usize = self.performances.values().map(|p| p.attempts).sum();
        if total < 20 {
            self.config.exploration_rate = 0.2;
        } else if total < 50 {
            self.config.exploration_rate = 0.15;
        } else if total < 100 {
            self.config.exploration_rate = 0.1;
        } else {
            self.config.exploration_rate = 0.05;
        }
    }

    pub fn performance_summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "SelfHealingSelector | current: {} | switches: {} | recovery: {} | steps: {}",
            self.current_strategy.name(),
            self.switch_count,
            self.recovery_count,
            self.total_steps,
        ));
        let mut sorted: Vec<&StrategyPerformance> = self.performances.values().collect();
        sorted.sort_by(|a, b| {
            b.avg_reward
                .partial_cmp(&a.avg_reward)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for perf in sorted {
            lines.push(format!(
                "  {}: attempts={} success_rate={:.2} avg_reward={:.3} starving={}",
                perf.strategy.name(),
                perf.attempts,
                perf.success_rate(),
                perf.avg_reward,
                perf.is_starving,
            ));
        }
        lines.join("\n")
    }

    pub fn stats(&self) -> SelectorStats {
        let top = self.best_performing();
        let coverage = self
            .performances
            .values()
            .filter(|p| p.attempts > 0)
            .count();
        let recovery_rate = if self.switch_count > 0 {
            self.recovery_count as f64 / self.switch_count as f64
        } else {
            0.0
        };
        SelectorStats {
            current_strategy: self.current_strategy,
            switch_count: self.switch_count,
            total_steps: self.total_steps,
            recovery_rate,
            strategy_coverage: coverage,
            exploration_rate: self.config.exploration_rate,
            top_strategy: top,
        }
    }

    pub fn best_strategies(&self, n: usize) -> Vec<ReasoningStrategy> {
        let mut sorted: Vec<&StrategyPerformance> = self
            .performances
            .values()
            .filter(|p| p.attempts >= self.config.min_samples_per_strategy)
            .collect();
        sorted.sort_by(|a, b| {
            b.avg_reward
                .partial_cmp(&a.avg_reward)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(n).map(|p| p.strategy).collect()
    }
}

#[derive(Debug, Clone)]
pub struct SelectorStats {
    pub current_strategy: ReasoningStrategy,
    pub switch_count: usize,
    pub total_steps: usize,
    pub recovery_rate: f64,
    pub strategy_coverage: usize,
    pub exploration_rate: f64,
    pub top_strategy: Option<ReasoningStrategy>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = StrategyConfig::default();
        assert_eq!(cfg.max_failures_before_switch, 3);
        assert_eq!(cfg.performance_window, 10);
        assert_eq!(cfg.exploration_rate, 0.1);
        assert_eq!(cfg.recovery_strategy, ReasoningStrategy::Decomposition);
    }

    #[test]
    fn test_new_selector_all_strategies_tracked() {
        let sel = SelfHealingSelector::new(StrategyConfig::default());
        assert_eq!(sel.performances.len(), 10);
        assert!(sel
            .performances
            .contains_key(&ReasoningStrategy::Analogical));
        assert!(sel
            .performances
            .contains_key(&ReasoningStrategy::MetaReflection));
    }

    #[test]
    fn test_initial_strategy_is_decomposition() {
        let sel = SelfHealingSelector::new(StrategyConfig::default());
        assert_eq!(sel.current_strategy, ReasoningStrategy::Decomposition);
    }

    #[test]
    fn test_record_outcome_updates_performance() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        sel.record_outcome(ReasoningStrategy::Causal, 0.8, true);
        let perf = sel.performances.get(&ReasoningStrategy::Causal).unwrap();
        assert_eq!(perf.attempts, 1);
        assert_eq!(perf.successes, 1);
        assert!((perf.avg_reward - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_record_outcome_multiple_updates() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        sel.record_outcome(ReasoningStrategy::Analogical, 0.5, true);
        sel.record_outcome(ReasoningStrategy::Analogical, 0.3, false);
        let perf = sel
            .performances
            .get(&ReasoningStrategy::Analogical)
            .unwrap();
        assert_eq!(perf.attempts, 2);
        assert_eq!(perf.successes, 1);
        assert!((perf.avg_reward - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_record_failure_pattern() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        sel.record_failure_pattern(ReasoningStrategy::MultiHop, DeadEndType::Loop);
        sel.record_failure_pattern(ReasoningStrategy::MultiHop, DeadEndType::Loop);
        sel.record_failure_pattern(ReasoningStrategy::MultiHop, DeadEndType::Divergence);
        let perf = sel.performances.get(&ReasoningStrategy::MultiHop).unwrap();
        assert_eq!(perf.failure_patterns.len(), 3);
        assert_eq!(perf.dominant_failure(), Some(DeadEndType::Loop));
    }

    #[test]
    fn test_strategy_history_length() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        sel.record_outcome(ReasoningStrategy::Deductive, 1.0, true);
        sel.record_outcome(ReasoningStrategy::Inductive, 0.0, false);
        assert_eq!(sel.strategy_history.len(), 2);
    }

    #[test]
    fn test_ucb_score_returns_finite() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        sel.record_outcome(ReasoningStrategy::Abductive, 0.9, true);
        sel.record_outcome(ReasoningStrategy::Abductive, 0.7, true);
        let perf = sel.performances.get(&ReasoningStrategy::Abductive).unwrap();
        let score = sel.ucb_score(perf);
        assert!(score.is_finite());
        assert!(score > 0.0);
    }

    #[test]
    fn test_select_strategy_returns_valid_strategy() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        let strategy = sel.select_strategy("test");
        assert!(ReasoningStrategy::all().contains(&strategy));
    }

    #[test]
    fn test_switch_strategy_changes_current() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        assert_eq!(sel.current_strategy, ReasoningStrategy::Decomposition);
        sel.switch_strategy(ReasoningStrategy::Causal);
        assert_eq!(sel.current_strategy, ReasoningStrategy::Causal);
        assert_eq!(sel.switch_count, 1);
    }

    #[test]
    fn test_heal_returns_recovery_strategy() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        sel.record_outcome(ReasoningStrategy::Analogical, 0.9, true);
        sel.record_outcome(ReasoningStrategy::Analogical, 0.8, true);
        let healed = sel.heal(ReasoningStrategy::Inductive);
        assert!(ReasoningStrategy::all().contains(&healed));
        assert_eq!(sel.recovery_count, 1);
    }

    #[test]
    fn test_recovery_strategies_ranked_by_performance() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        sel.record_outcome(ReasoningStrategy::Causal, 0.9, true);
        sel.record_outcome(ReasoningStrategy::Causal, 0.8, true);
        sel.record_outcome(ReasoningStrategy::Analogical, 0.1, false);
        sel.record_outcome(ReasoningStrategy::Analogical, 0.2, false);
        let ranked = sel.recovery_strategies();
        assert_eq!(ranked[0], ReasoningStrategy::Causal);
    }

    #[test]
    fn test_starvation_detection() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        let starving = sel.detect_starvation();
        assert_eq!(starving.len(), 10);
        sel.record_outcome(ReasoningStrategy::Causal, 0.5, true);
        sel.record_outcome(ReasoningStrategy::Causal, 0.6, true);
        let starving = sel.detect_starvation();
        assert!(!starving.contains(&ReasoningStrategy::Causal));
    }

    #[test]
    fn test_best_strategies_empty_when_insufficient_samples() {
        let sel = SelfHealingSelector::new(StrategyConfig::default());
        let best = sel.best_strategies(3);
        assert!(best.is_empty());
    }

    #[test]
    fn test_best_strategies_returns_top_n() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        sel.record_outcome(ReasoningStrategy::Causal, 0.9, true);
        sel.record_outcome(ReasoningStrategy::Causal, 0.8, true);
        sel.record_outcome(ReasoningStrategy::Analogical, 0.7, true);
        sel.record_outcome(ReasoningStrategy::Analogical, 0.6, true);
        sel.record_outcome(ReasoningStrategy::Deductive, 0.1, false);
        sel.record_outcome(ReasoningStrategy::Deductive, 0.2, false);
        let best = sel.best_strategies(2);
        assert_eq!(best.len(), 2);
        assert_eq!(best[0], ReasoningStrategy::Causal);
        assert_eq!(best[1], ReasoningStrategy::Analogical);
    }

    #[test]
    fn test_update_exploration_rate_decreases_over_time() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        for _ in 0..5 {
            for s in ReasoningStrategy::all() {
                sel.record_outcome(s, 0.5, true);
            }
        }
        sel.update_exploration_rate();
        assert!((sel.config.exploration_rate - 0.15).abs() < 1e-6);
        for _ in 0..100 {
            for s in ReasoningStrategy::all() {
                sel.record_outcome(s, 0.5, true);
            }
        }
        sel.update_exploration_rate();
        assert!((sel.config.exploration_rate - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_stats_consistency() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        sel.record_outcome(ReasoningStrategy::Causal, 0.9, true);
        sel.record_outcome(ReasoningStrategy::Causal, 0.8, true);
        sel.record_outcome(ReasoningStrategy::Analogical, 0.1, false);
        let stats = sel.stats();
        assert_eq!(stats.total_steps, 0);
        assert_eq!(stats.switch_count, 0);
        assert_eq!(stats.strategy_coverage, 2);
        assert_eq!(stats.current_strategy, ReasoningStrategy::Decomposition);
    }

    #[test]
    fn test_should_switch_false_when_insufficient_samples() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        sel.current_strategy = ReasoningStrategy::Causal;
        sel.record_outcome(ReasoningStrategy::Causal, 0.0, false);
        assert!(!sel.should_switch());
    }

    #[test]
    fn test_ucb_score_untried_is_infinite() {
        let sel = SelfHealingSelector::new(StrategyConfig::default());
        let perf = sel
            .performances
            .get(&ReasoningStrategy::MctsTreeSearch)
            .unwrap();
        assert_eq!(sel.ucb_score(perf), f64::MAX);
    }

    #[test]
    fn test_performance_summary_contains_all_strategies() {
        let sel = SelfHealingSelector::new(StrategyConfig::default());
        let summary = sel.performance_summary();
        for s in ReasoningStrategy::all() {
            assert!(summary.contains(s.name()));
        }
    }

    #[test]
    fn test_select_strategy_explores_starving_strategies() {
        let mut sel = SelfHealingSelector::new(StrategyConfig {
            exploration_rate: 1.0,
            ..StrategyConfig::default()
        });
        sel.record_outcome(ReasoningStrategy::Analogical, 0.1, false);
        sel.record_outcome(ReasoningStrategy::Analogical, 0.2, false);
        sel.record_outcome(ReasoningStrategy::Analogical, 0.3, false);
        let _strategy = sel.select_strategy("test");
    }

    #[test]
    fn test_heal_falls_back_to_config_recovery() {
        let mut sel = SelfHealingSelector::new(StrategyConfig::default());
        let healed = sel.heal(ReasoningStrategy::Decomposition);
        assert_eq!(healed, sel.config.recovery_strategy);
    }
}
