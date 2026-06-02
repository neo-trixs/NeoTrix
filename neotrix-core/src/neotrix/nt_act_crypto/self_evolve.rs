use super::opportunity::StrategyStats;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BacktestResult {
    pub strategy: String,
    pub total_runs: u64,
    pub success_rate: f64,
    pub avg_profit_usd: f64,
    pub total_profit_usd: f64,
    pub roi_pct: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown_pct: f64,
}

#[derive(Clone, Debug)]
pub struct AdaptiveConfig {
    pub exploration_rate: f64,
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub min_confidence: f64,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            exploration_rate: 0.15,
            learning_rate: 0.1,
            discount_factor: 0.95,
            min_confidence: 0.3,
        }
    }
}

pub struct SelfEvolver {
    backtest_history: Vec<BacktestResult>,
    config: AdaptiveConfig,
    strategy_q: HashMap<String, f64>,
    strategy_counts: HashMap<String, u64>,
}

impl SelfEvolver {
    pub fn new() -> Self {
        Self {
            backtest_history: Vec::new(),
            config: AdaptiveConfig::default(),
            strategy_q: HashMap::new(),
            strategy_counts: HashMap::new(),
        }
    }

    pub fn with_config(config: AdaptiveConfig) -> Self {
        Self {
            backtest_history: Vec::new(),
            config,
            strategy_q: HashMap::new(),
            strategy_counts: HashMap::new(),
        }
    }

    pub fn backtest(&mut self, strategy: &str, stats: &StrategyStats) -> BacktestResult {
        let runs = stats.attempts.max(1) as u64;
        let success_rate = stats.success_rate();
        let total_profit = stats.net_profit();
        let avg_profit = if runs > 0 { total_profit / runs as f64 } else { 0.0 };
        let roi = if stats.total_gas_usd > 0.0 {
            (total_profit / stats.total_gas_usd) * 100.0
        } else {
            stats.successes as f64 * 10.0
        };

        let sharpe = if roi > 0.0 { roi / (100.0 - success_rate * 100.0).max(0.1) } else { 0.0 };
        let dd = (1.0 - success_rate) * 100.0;

        let result = BacktestResult {
            strategy: strategy.to_string(),
            total_runs: runs,
            success_rate,
            avg_profit_usd: avg_profit,
            total_profit_usd: total_profit,
            roi_pct: roi,
            sharpe_ratio: sharpe,
            max_drawdown_pct: dd,
        };

        self.backtest_history.push(result.clone());
        self.update_q_value(strategy, roi);
        result
    }

    fn update_q_value(&mut self, strategy: &str, reward: f64) {
        let count = self.strategy_counts.get(strategy).copied().unwrap_or(0);
        let lr = self.config.learning_rate / (1.0 + count as f64 * 0.01);
        let current = self.strategy_q.get(strategy).copied().unwrap_or(0.0);
        let new_q = current + lr * (reward - current);
        self.strategy_q.insert(strategy.to_string(), new_q);
        *self.strategy_counts.entry(strategy.to_string()).or_insert(0) += 1;
    }

    pub fn select_strategy(&self, strategies: &[String]) -> Option<String> {
        if strategies.is_empty() {
            return None;
        }

        if rand::random::<f64>() < self.config.exploration_rate {
            let idx = rand::random::<usize>() % strategies.len();
            return Some(strategies[idx].clone());
        }

        strategies
            .iter()
            .map(|s| {
                let q = self.strategy_q.get(s).copied().unwrap_or(0.0);
                (s, q)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(s, _)| s.clone())
    }

    pub fn q_value(&self, strategy: &str) -> f64 {
        self.strategy_q.get(strategy).copied().unwrap_or(0.0)
    }

    pub fn best_strategy(&self) -> Option<(String, f64)> {
        self.strategy_q
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, v)| (k.clone(), *v))
    }

    pub fn backtest_history(&self) -> &[BacktestResult] {
        &self.backtest_history
    }

    pub fn insight_report(&self) -> Vec<String> {
        let mut lines = Vec::new();

        if self.backtest_history.is_empty() {
            lines.push("📊 尚无回测数据".into());
            return lines;
        }

        lines.push("📊 自进化分析报告".into());
        lines.push(format!("  Q-Table: {} 策略", self.strategy_q.len()));
        lines.push(format!("  探索率: {:.1}%", self.config.exploration_rate * 100.0));
        lines.push(format!("  学习率: {:.2}", self.config.learning_rate));

        if let Some((best, q)) = self.best_strategy() {
            lines.push(format!("  最佳策略: {} (Q={:.2})", best, q));
        }

        for result in self.backtest_history.iter().rev().take(5) {
            lines.push(format!(
                "  {}: SR={:.0}% ROI={:.0}% Sharpe={:.2}",
                result.strategy, result.success_rate * 100.0, result.roi_pct, result.sharpe_ratio,
            ));
        }

        lines
    }

    pub fn decay_exploration(&mut self, factor: f64) {
        self.config.exploration_rate = (self.config.exploration_rate * factor).max(0.01);
    }

    pub fn config(&self) -> &AdaptiveConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut AdaptiveConfig {
        &mut self.config
    }
}

impl Default for SelfEvolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_stats() -> StrategyStats {
        StrategyStats {
            opportunity_type: super::super::OpportunityType::FaucetClaim,
            attempts: 20,
            successes: 15,
            total_value_usd: 500.0,
            total_gas_usd: 50.0,
        }
    }

    #[test]
    fn test_self_evolver_new() {
        let evolver = SelfEvolver::new();
        assert!(evolver.backtest_history().is_empty());
        assert!((evolver.config.exploration_rate - 0.15).abs() < 0.01);
    }

    #[test]
    fn test_backtest() {
        let mut evolver = SelfEvolver::new();
        let result = evolver.backtest("FaucetClaim", &sample_stats());
        assert_eq!(result.strategy, "FaucetClaim");
        assert_eq!(result.total_runs, 20);
        assert!((result.success_rate - 0.75).abs() < 0.01);
        assert!((result.total_profit_usd - 450.0).abs() < 0.01);
    }

    #[test]
    fn test_q_value_update() {
        let mut evolver = SelfEvolver::new();
        evolver.backtest("FaucetClaim", &sample_stats());
        assert!(evolver.q_value("FaucetClaim") > 0.0);
    }

    #[test]
    fn test_select_strategy_greedy() {
        let mut evolver = SelfEvolver::new();
        evolver.config.exploration_rate = 0.0;
        evolver.backtest("StrategyA", &sample_stats());
        let stats_b = StrategyStats {
            opportunity_type: super::super::OpportunityType::FaucetClaim,
            attempts: 10,
            successes: 2,
            total_value_usd: 50.0,
            total_gas_usd: 30.0,
        };
        evolver.backtest("StrategyB", &stats_b);

        let selected = evolver.select_strategy(&["StrategyA".into(), "StrategyB".into()]);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap(), "StrategyA");
    }

    #[test]
    fn test_best_strategy() {
        let mut evolver = SelfEvolver::new();
        assert!(evolver.best_strategy().is_none());

        evolver.backtest("FaucetClaim", &sample_stats());
        assert!(evolver.best_strategy().is_some());
    }

    #[test]
    fn test_insight_report_empty() {
        let evolver = SelfEvolver::new();
        let report = evolver.insight_report();
        assert!(!report.is_empty());
        assert!(report[0].contains("无回测数据"));
    }

    #[test]
    fn test_insight_report_with_data() {
        let mut evolver = SelfEvolver::new();
        evolver.backtest("FaucetClaim", &sample_stats());
        let report = evolver.insight_report();
        assert!(report.iter().any(|l| l.contains("FaucetClaim")));
    }

    #[test]
    fn test_decay_exploration() {
        let mut evolver = SelfEvolver::new();
        evolver.decay_exploration(0.5);
        assert!((evolver.config.exploration_rate - 0.075).abs() < 0.001);
    }

    #[test]
    fn test_select_strategy_empty() {
        let evolver = SelfEvolver::new();
        let selected = evolver.select_strategy(&[]);
        assert!(selected.is_none());
    }

    #[test]
    fn test_config_mut() {
        let mut evolver = SelfEvolver::new();
        let cfg = evolver.config_mut();
        cfg.min_confidence = 0.8;
        assert!((evolver.config.min_confidence - 0.8).abs() < 0.01);
    }
}
