#[derive(Debug, Clone)]
pub struct RiskManager {
    pub max_position_size_pct: f64,
    pub daily_loss_limit: f64,
    pub max_drawdown_pct: f64,
    pub max_leverage: f64,
    pub capital_base: f64,
    pub daily_loss: f64,
    pub peak_portfolio: f64,
    pub current_drawdown: f64,
    pub trade_history: Vec<TradeRecord>,
    kill_switch_engaged: bool,
}

#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub pnl: f64,
    pub capital_used: f64,
    pub timestamp: u64,
    pub trade_count: u64,
}

impl RiskManager {
    pub fn new(capital_base: f64) -> Self {
        Self {
            max_position_size_pct: 0.02,
            daily_loss_limit: capital_base * 0.05,
            max_drawdown_pct: 0.20,
            max_leverage: 1.0,
            capital_base,
            daily_loss: 0.0,
            peak_portfolio: capital_base,
            current_drawdown: 0.0,
            trade_history: Vec::with_capacity(1000),
            kill_switch_engaged: false,
        }
    }

    pub fn max_position_size(&self) -> f64 {
        self.capital_base * self.max_position_size_pct
    }

    pub fn can_allocate(&self, amount: f64) -> bool {
        if self.kill_switch_engaged {
            return false;
        }
        if amount > self.max_position_size() {
            return false;
        }
        if self.daily_loss >= self.daily_loss_limit {
            return false;
        }
        if self.current_drawdown >= self.max_drawdown_pct {
            return false;
        }
        true
    }

    pub fn record_trade(&mut self, pnl: f64, capital_used: f64) {
        if pnl < 0.0 {
            self.daily_loss += pnl.abs();
        }
        let new_portfolio =
            self.capital_base + self.trade_history.iter().map(|t| t.pnl).sum::<f64>() + pnl;
        if new_portfolio > self.peak_portfolio {
            self.peak_portfolio = new_portfolio;
        }
        self.current_drawdown = if self.peak_portfolio > 0.0 {
            (self.peak_portfolio - new_portfolio) / self.peak_portfolio
        } else {
            0.0
        };

        self.trade_history.push(TradeRecord {
            pnl,
            capital_used,
            timestamp: 0,
            trade_count: self.trade_history.len() as u64 + 1,
        });

        if self.daily_loss >= self.daily_loss_limit
            || self.current_drawdown >= self.max_drawdown_pct
        {
            self.kill_switch_engaged = true;
        }
    }

    pub fn sharpe_ratio(&self) -> f64 {
        if self.trade_history.len() < 2 {
            return 0.0;
        }
        let returns: Vec<f64> = self
            .trade_history
            .iter()
            .map(|t| {
                if t.capital_used > 0.0 {
                    t.pnl / t.capital_used
                } else {
                    0.0
                }
            })
            .collect();
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance =
            returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();
        if std_dev < 1e-9 {
            return 10.0;
        }
        mean / std_dev * (252.0_f64).sqrt()
    }

    pub fn max_drawdown(&self) -> f64 {
        self.current_drawdown
    }

    pub fn reset_daily_loss(&mut self) {
        self.daily_loss = 0.0;
    }

    pub fn reset_kill_switch(&mut self) {
        self.kill_switch_engaged = false;
    }

    pub fn is_kill_switch_engaged(&self) -> bool {
        self.kill_switch_engaged
    }

    pub fn var_95(&self) -> f64 {
        if self.trade_history.len() < 20 {
            return self.max_position_size() * 0.05;
        }
        let mut pnls: Vec<f64> = self.trade_history.iter().map(|t| t.pnl).collect();
        pnls.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = (pnls.len() as f64 * 0.05) as usize;
        pnls[idx.min(pnls.len() - 1)].abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let rm = RiskManager::new(10000.0);
        assert_eq!(rm.max_position_size(), 200.0);
        assert!(!rm.is_kill_switch_engaged());
    }

    #[test]
    fn test_can_allocate_within_limits() {
        let rm = RiskManager::new(10000.0);
        assert!(rm.can_allocate(100.0));
        assert!(!rm.can_allocate(500.0));
    }

    #[test]
    fn test_kill_switch_on_daily_loss() {
        let mut rm = RiskManager::new(10000.0);
        rm.daily_loss_limit = 500.0;
        rm.record_trade(-300.0, 1000.0);
        assert!(!rm.is_kill_switch_engaged());
        rm.record_trade(-300.0, 1000.0);
        assert!(rm.is_kill_switch_engaged());
        assert!(!rm.can_allocate(100.0));
    }

    #[test]
    fn test_kill_switch_on_drawdown() {
        let mut rm = RiskManager::new(10000.0);
        rm.max_drawdown_pct = 0.10;
        rm.record_trade(-2000.0, 5000.0);
        assert!(rm.is_kill_switch_engaged());
    }

    #[test]
    fn test_sharpe_ratio() {
        let mut rm = RiskManager::new(10000.0);
        for _ in 0..10 {
            rm.record_trade(100.0, 1000.0);
        }
        assert!(rm.sharpe_ratio() > 0.0);
    }

    #[test]
    fn test_var_95_calculation() {
        let mut rm = RiskManager::new(10000.0);
        for _ in 0..30 {
            rm.record_trade(-50.0, 1000.0);
        }
        let var = rm.var_95();
        assert!(var > 0.0);
    }

    #[test]
    fn test_reset_functions() {
        let mut rm = RiskManager::new(10000.0);
        rm.record_trade(-5000.0, 10000.0);
        assert!(rm.is_kill_switch_engaged());
        rm.reset_kill_switch();
        assert!(!rm.is_kill_switch_engaged());
    }
}
