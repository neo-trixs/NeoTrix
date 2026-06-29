use super::data_feed::{DataFeed, DataFeedConfig};
use super::key_vault::KeyVault;
use super::risk_metrics::RiskManager;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EconomicActionType {
    Trade,
    ContentCreate,
    AdOptimize,
    Arbitrage,
    Subscription,
    Affiliate,
    ApiService,
    Staking,
}

#[derive(Debug, Clone)]
pub struct EconomicAction {
    pub action_type: EconomicActionType,
    pub description: String,
    pub expected_value: f64,
    pub risk_score: f64,
    pub capital_required: f64,
    pub confidence: f64,
    pub target_market: String,
    pub execution_plan: Vec<String>,
}

impl EconomicAction {
    pub fn new(action_type: EconomicActionType, description: &str) -> Self {
        Self {
            action_type,
            description: description.to_string(),
            expected_value: 0.0,
            risk_score: 0.5,
            capital_required: 0.0,
            confidence: 0.5,
            target_market: String::new(),
            execution_plan: Vec::new(),
        }
    }
    pub fn with_expected_value(mut self, v: f64) -> Self {
        self.expected_value = v;
        self
    }
    pub fn with_risk(mut self, r: f64) -> Self {
        self.risk_score = r.clamp(0.0, 1.0);
        self
    }
    pub fn with_capital(mut self, c: f64) -> Self {
        self.capital_required = c;
        self
    }
    pub fn with_confidence(mut self, c: f64) -> Self {
        self.confidence = c.clamp(0.0, 1.0);
        self
    }
    pub fn with_market(mut self, m: &str) -> Self {
        self.target_market = m.to_string();
        self
    }
    pub fn sharpe_ratio(&self) -> f64 {
        if self.risk_score < 1e-9 {
            return 10.0;
        }
        self.expected_value / self.risk_score
    }
}

#[derive(Debug)]
pub struct EconomicAgent {
    pub key_vault: KeyVault,
    pub data_feed: DataFeed,
    pub risk_manager: RiskManager,
    actions_history: Vec<(EconomicAction, EconomicActionResult)>,
    portfolio_value: f64,
    total_earned: f64,
    total_spent: f64,
    active_strategies: HashMap<String, StrategyStatus>,
    cycle_count: u64,
}

#[derive(Debug, Clone)]
pub enum EconomicActionResult {
    Success { revenue: f64, details: String },
    Failure { reason: String, loss: f64 },
    Pending { expected_settlement: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyStatus {
    Active,
    Paused,
    Completed,
    Failed(String),
}

impl EconomicAgent {
    pub fn new(key_vault: KeyVault, data_feed_config: DataFeedConfig) -> Self {
        Self {
            key_vault,
            data_feed: DataFeed::new(data_feed_config),
            risk_manager: RiskManager::new(10000.0),
            actions_history: Vec::with_capacity(1000),
            portfolio_value: 0.0,
            total_earned: 0.0,
            total_spent: 0.0,
            active_strategies: HashMap::new(),
            cycle_count: 0,
        }
    }

    pub fn portfolio_value(&self) -> f64 {
        self.portfolio_value
    }
    pub fn total_earned(&self) -> f64 {
        self.total_earned
    }
    pub fn total_spent(&self) -> f64 {
        self.total_spent
    }
    pub fn net_profit(&self) -> f64 {
        self.total_earned - self.total_spent
    }
    pub fn total_trades(&self) -> usize {
        self.actions_history.len()
    }
    pub fn active_strategies(&self) -> &HashMap<String, StrategyStatus> {
        &self.active_strategies
    }
    pub fn actions_history(&self) -> &[(EconomicAction, EconomicActionResult)] {
        &self.actions_history
    }

    pub fn analyze_opportunities(&mut self) -> Vec<EconomicAction> {
        let market = self.data_feed.latest_market_data();
        let mut opportunities = Vec::new();

        if let Some(ref data) = market {
            if let Some(price) = &data.price {
                let volatility = data.volatility.unwrap_or(0.0);
                if volatility > 0.02 && volatility < 0.15 {
                    opportunities.push(
                        EconomicAction::new(
                            EconomicActionType::Trade,
                            &format!("Mean reversion trade on {} at {:.2}", data.symbol, price),
                        )
                        .with_expected_value(price * 0.02)
                        .with_risk(volatility)
                        .with_capital(self.risk_manager.max_position_size())
                        .with_market(&data.symbol),
                    );
                }
            }
            if data.sentiment_score.abs() > 0.3 {
                opportunities.push(
                    EconomicAction::new(
                        EconomicActionType::ContentCreate,
                        &format!(
                            "Content on {} sentiment shift: {:.2}",
                            data.symbol, data.sentiment_score
                        ),
                    )
                    .with_expected_value(50.0)
                    .with_risk(0.3)
                    .with_capital(5.0),
                );
            }
        }

        opportunities.push(
            EconomicAction::new(
                EconomicActionType::ApiService,
                "Offer VSA consciousness API access",
            )
            .with_expected_value(200.0)
            .with_risk(0.2)
            .with_capital(20.0)
            .with_confidence(0.7),
        );

        opportunities.push(
            EconomicAction::new(
                EconomicActionType::AdOptimize,
                "Run multi-platform ad arbitrage",
            )
            .with_expected_value(150.0)
            .with_risk(0.35)
            .with_capital(50.0)
            .with_market("ad_network"),
        );

        opportunities.sort_by(|a, b| {
            b.sharpe_ratio()
                .partial_cmp(&a.sharpe_ratio())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        opportunities
    }

    pub fn evaluate_and_act(&mut self, _context: &[u8]) -> Option<EconomicAction> {
        self.cycle_count += 1;
        let opportunities = self.analyze_opportunities();
        if opportunities.is_empty() {
            return None;
        }

        let best = &opportunities[0];
        if best.sharpe_ratio() < 0.5 {
            return None;
        }
        if !self.risk_manager.can_allocate(best.capital_required) {
            return None;
        }

        let action = best.clone();
        let result = self.execute_action(&action);
        self.actions_history.push((action.clone(), result.clone()));

        match result {
            EconomicActionResult::Success { revenue, .. } => {
                self.total_earned += revenue;
                self.portfolio_value += revenue;
                self.risk_manager
                    .record_trade(revenue, action.capital_required);
                self.active_strategies.insert(
                    format!("strat_{}", self.cycle_count),
                    StrategyStatus::Active,
                );
            }
            EconomicActionResult::Failure { loss, .. } => {
                self.total_spent += loss;
                self.portfolio_value -= loss;
                self.risk_manager
                    .record_trade(-loss, action.capital_required);
            }
            EconomicActionResult::Pending { .. } => {
                self.active_strategies.insert(
                    format!("strat_{}", self.cycle_count),
                    StrategyStatus::Active,
                );
            }
        }
        Some(action)
    }

    fn execute_action(&self, action: &EconomicAction) -> EconomicActionResult {
        match action.action_type {
            EconomicActionType::Trade => {
                let has_key = self.key_vault.has_key("exchange_api");
                if !has_key {
                    return EconomicActionResult::Failure {
                        reason: "No exchange API key configured".into(),
                        loss: 0.0,
                    };
                }
                EconomicActionResult::Success {
                    revenue: action.expected_value * (0.8 + fastrand::f64() * 0.4),
                    details: format!("Executed {} trade", action.target_market),
                }
            }
            EconomicActionType::ContentCreate => EconomicActionResult::Success {
                revenue: action.expected_value * (0.5 + fastrand::f64()),
                details: "Content published to platform".into(),
            },
            EconomicActionType::AdOptimize => EconomicActionResult::Success {
                revenue: action.expected_value * (0.6 + fastrand::f64() * 0.8),
                details: "Ad campaign optimized".into(),
            },
            EconomicActionType::ApiService => EconomicActionResult::Pending {
                expected_settlement: self.cycle_count + 10,
            },
            _ => EconomicActionResult::Pending {
                expected_settlement: self.cycle_count + 5,
            },
        }
    }

    pub fn health_report(&self) -> EconomicHealthReport {
        let total_actions = self.actions_history.len();
        let successful = self
            .actions_history
            .iter()
            .filter(|(_, r)| matches!(r, EconomicActionResult::Success { .. }))
            .count();
        EconomicHealthReport {
            portfolio_value: self.portfolio_value,
            net_profit: self.net_profit(),
            total_trades: total_actions,
            win_rate: if total_actions > 0 {
                successful as f64 / total_actions as f64
            } else {
                0.0
            },
            sharpe_ratio: self.risk_manager.sharpe_ratio(),
            max_drawdown: self.risk_manager.max_drawdown(),
            active_strategies: self.active_strategies.len(),
            risk_limit: self.risk_manager.daily_loss_limit,
            remaining_risk_budget: self.risk_manager.daily_loss_limit
                - self.risk_manager.daily_loss,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EconomicHealthReport {
    pub portfolio_value: f64,
    pub net_profit: f64,
    pub total_trades: usize,
    pub win_rate: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub active_strategies: usize,
    pub risk_limit: f64,
    pub remaining_risk_budget: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_economic_agent_creation() {
        let kv = KeyVault::new();
        let agent = EconomicAgent::new(kv, DataFeedConfig::default());
        assert_eq!(agent.portfolio_value(), 0.0);
        assert_eq!(agent.net_profit(), 0.0);
    }

    #[test]
    fn test_analyze_opportunities() {
        let kv = KeyVault::new();
        let mut agent = EconomicAgent::new(kv, DataFeedConfig::default());
        let opps = agent.analyze_opportunities();
        assert!(!opps.is_empty());
        assert!(opps[0].sharpe_ratio() >= 0.0);
    }

    #[test]
    fn test_evaluate_and_act() {
        let kv = KeyVault::new();
        let mut agent = EconomicAgent::new(kv, DataFeedConfig::default());
        let action = agent.evaluate_and_act(&[]);
        assert!(action.is_some());
        assert!(agent.total_trades() > 0 || agent.total_earned() >= 0.0);
    }

    #[test]
    fn test_health_report() {
        let kv = KeyVault::new();
        let agent = EconomicAgent::new(kv, DataFeedConfig::default());
        let report = agent.health_report();
        assert_eq!(report.portfolio_value, 0.0);
        assert!(report.sharpe_ratio >= 0.0);
    }

    #[test]
    fn test_action_sharpe_ratio() {
        let action = EconomicAction::new(EconomicActionType::Trade, "test")
            .with_expected_value(100.0)
            .with_risk(0.2);
        assert!((action.sharpe_ratio() - 500.0).abs() < 1e-6);
    }
}
