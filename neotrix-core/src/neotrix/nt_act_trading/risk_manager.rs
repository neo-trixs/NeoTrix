use crate::core::nt_core_trading::signal_generator::MarketRegime;
use crate::core::nt_core_trading::types::{
    AssetClass, OHLCVBar, PortfolioSummary, Position, TradeSide,
};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub max_position_size_pct: f64,
    pub max_portfolio_risk_pct: f64,
    pub max_drawdown_pct: f64,
    pub max_leverage: f64,
    pub kelly_fraction: f64,
    pub min_confidence: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub max_positions_per_asset: usize,
    pub max_correlation: f64,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_position_size_pct: 0.20,
            max_portfolio_risk_pct: 0.02,
            max_drawdown_pct: 0.15,
            max_leverage: 1.0,
            kelly_fraction: 0.25,
            min_confidence: 0.55,
            stop_loss_pct: 0.05,
            take_profit_pct: 0.15,
            max_positions_per_asset: 3,
            max_correlation: 0.70,
        }
    }
}

impl RiskConfig {
    pub fn conservative() -> Self {
        Self {
            max_position_size_pct: 0.10,
            max_portfolio_risk_pct: 0.01,
            max_drawdown_pct: 0.10,
            max_leverage: 1.0,
            kelly_fraction: 0.15,
            min_confidence: 0.65,
            stop_loss_pct: 0.03,
            take_profit_pct: 0.10,
            max_positions_per_asset: 2,
            max_correlation: 0.50,
        }
    }

    pub fn aggressive() -> Self {
        Self {
            max_position_size_pct: 0.35,
            max_portfolio_risk_pct: 0.04,
            max_drawdown_pct: 0.25,
            max_leverage: 2.0,
            kelly_fraction: 0.50,
            min_confidence: 0.45,
            stop_loss_pct: 0.08,
            take_profit_pct: 0.25,
            max_positions_per_asset: 5,
            max_correlation: 0.85,
        }
    }

    pub fn for_asset_class(asset: AssetClass) -> Self {
        match asset {
            AssetClass::Crypto => Self {
                max_position_size_pct: 0.10,
                max_portfolio_risk_pct: 0.03,
                max_drawdown_pct: 0.30,
                max_leverage: 1.0,
                kelly_fraction: 0.20,
                min_confidence: 0.60,
                stop_loss_pct: 0.07,
                take_profit_pct: 0.20,
                max_positions_per_asset: 2,
                max_correlation: 0.60,
            },
            AssetClass::Stock => Self::default(),
            AssetClass::Forex => Self {
                max_position_size_pct: 0.15,
                max_portfolio_risk_pct: 0.015,
                max_drawdown_pct: 0.12,
                max_leverage: 5.0,
                kelly_fraction: 0.20,
                min_confidence: 0.60,
                stop_loss_pct: 0.02,
                take_profit_pct: 0.06,
                max_positions_per_asset: 3,
                max_correlation: 0.70,
            },
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KellyCalculator;

impl KellyCalculator {
    pub fn fraction(win_rate: f64, avg_win: f64, avg_loss: f64) -> f64 {
        if avg_loss <= 0.0 || win_rate <= 0.0 || win_rate >= 1.0 {
            return 0.0;
        }
        let b = avg_win / avg_loss;
        let p = win_rate;
        let q = 1.0 - p;
        let kelly = (p * b - q) / b;
        kelly.clamp(0.0, 0.95)
    }

    pub fn fractional_kelly(win_rate: f64, avg_win: f64, avg_loss: f64, fraction: f64) -> f64 {
        Self::fraction(win_rate, avg_win, avg_loss) * fraction.clamp(0.0, 1.0)
    }

    pub fn half_kelly(win_rate: f64, avg_win: f64, avg_loss: f64) -> f64 {
        Self::fractional_kelly(win_rate, avg_win, avg_loss, 0.5)
    }

    pub fn quarter_kelly(win_rate: f64, avg_win: f64, avg_loss: f64) -> f64 {
        Self::fractional_kelly(win_rate, avg_win, avg_loss, 0.25)
    }
}

#[derive(Debug, Clone)]
pub struct VarCalculator {
    pub returns: VecDeque<f64>,
    pub max_samples: usize,
}

impl VarCalculator {
    pub fn new(max_samples: usize) -> Self {
        Self {
            returns: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn record(&mut self, return_pct: f64) {
        while self.returns.len() >= self.max_samples {
            self.returns.pop_front();
        }
        self.returns.push_back(return_pct);
    }

    pub fn update_from_bars(&mut self, bars: &[OHLCVBar]) {
        for window in bars.windows(2) {
            let prev_close = window[0].close;
            let curr_close = window[1].close;
            if prev_close > 0.0 {
                self.record((curr_close - prev_close) / prev_close);
            }
        }
    }

    pub fn var_historical(&self, percentile: f64) -> f64 {
        if self.returns.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f64> = self.returns.iter().copied().collect();
        sorted.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((1.0 - percentile) * sorted.len() as f64) as usize;
        let idx = idx.min(sorted.len().saturating_sub(1));
        sorted[idx]
    }

    pub fn var_95(&self) -> f64 {
        self.var_historical(0.95)
    }

    pub fn var_99(&self) -> f64 {
        self.var_historical(0.99)
    }

    pub fn cvar_95(&self) -> f64 {
        self.cvar(0.95)
    }

    pub fn cvar(&self, percentile: f64) -> f64 {
        if self.returns.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f64> = self.returns.iter().copied().collect();
        sorted.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let threshold_idx = ((1.0 - percentile) * sorted.len() as f64) as usize;
        let threshold_idx = threshold_idx.min(sorted.len().saturating_sub(1));
        let tail: Vec<&f64> = sorted.iter().take(threshold_idx + 1).collect();
        if tail.is_empty() {
            return 0.0;
        }
        tail.iter().copied().sum::<f64>() / tail.len() as f64
    }

    pub fn sharpe_ratio(&self, risk_free_rate: f64) -> f64 {
        if self.returns.len() < 2 {
            return 0.0;
        }
        let n = self.returns.len() as f64;
        let mean_return: f64 = self.returns.iter().sum::<f64>() / n;
        let variance: f64 = self
            .returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>()
            / (n - 1.0);
        let std_dev = variance.sqrt();
        if std_dev <= 0.0 {
            return 0.0;
        }
        let excess = mean_return - risk_free_rate;
        let annualized = excess / std_dev * (252.0_f64.sqrt());
        annualized
    }

    pub fn max_drawdown(&self) -> f64 {
        if self.returns.is_empty() {
            return 0.0;
        }
        let mut peak = 0.0_f64;
        let mut cumulative = 1.0_f64;
        let mut max_dd = 0.0_f64;
        for r in &self.returns {
            cumulative *= 1.0 + r;
            if cumulative > peak {
                peak = cumulative;
            }
            let dd = (cumulative - peak) / peak;
            if dd < max_dd {
                max_dd = dd;
            }
        }
        max_dd
    }

    pub fn calmar_ratio(&self, risk_free_rate: f64) -> f64 {
        let max_dd = self.max_drawdown().abs();
        if max_dd <= 0.0 {
            return 0.0;
        }
        let n = self.returns.len() as f64;
        let mean_return: f64 = self.returns.iter().sum::<f64>() / n;
        let annualized_return = mean_return * 252.0;
        let excess = annualized_return - risk_free_rate;
        excess / max_dd
    }
}

#[derive(Debug, Clone)]
pub struct DrawdownMonitor {
    pub peak_value: f64,
    pub current_drawdown: f64,
    pub max_drawdown_observed: f64,
    pub consecutive_losses: u32,
    pub is_paused: bool,
    pub pause_threshold: f64,
    pub max_consecutive_losses: u32,
}

impl DrawdownMonitor {
    pub fn new(initial_value: f64) -> Self {
        Self {
            peak_value: initial_value,
            current_drawdown: 0.0,
            max_drawdown_observed: 0.0,
            consecutive_losses: 0,
            is_paused: false,
            pause_threshold: 0.10,
            max_consecutive_losses: 5,
        }
    }

    pub fn update(&mut self, current_value: f64) {
        if current_value > self.peak_value {
            self.peak_value = current_value;
        }
        self.current_drawdown = if self.peak_value > 0.0 {
            (self.peak_value - current_value) / self.peak_value
        } else {
            0.0
        };
        if self.current_drawdown > self.max_drawdown_observed {
            self.max_drawdown_observed = self.current_drawdown;
        }
        if current_value < self.peak_value * (1.0 - self.pause_threshold) {
            self.is_paused = true;
        }
    }

    pub fn record_trade(&mut self, pnl_pct: f64) {
        if pnl_pct < 0.0 {
            self.consecutive_losses += 1;
            if self.consecutive_losses >= self.max_consecutive_losses {
                self.is_paused = true;
            }
        } else {
            self.consecutive_losses = 0;
        }
    }

    pub fn resume(&mut self) {
        self.is_paused = false;
        self.consecutive_losses = 0;
    }

    pub fn status_summary(&self) -> String {
        if self.is_paused {
            format!(
                "PAUSED | DD={:.1}% peak={:.1}% consec_loss={}",
                self.current_drawdown * 100.0,
                self.max_drawdown_observed * 100.0,
                self.consecutive_losses
            )
        } else {
            format!(
                "ACTIVE | DD={:.1}% peak={:.1}% consec_loss={}",
                self.current_drawdown * 100.0,
                self.max_drawdown_observed * 100.0,
                self.consecutive_losses
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct PositionSizer;

impl PositionSizer {
    pub fn risk_dollar(account_equity: f64, risk_pct: f64, stop_loss_pct: f64) -> f64 {
        if stop_loss_pct <= 0.0 {
            return 0.0;
        }
        let risk_amount = account_equity * risk_pct;
        risk_amount / stop_loss_pct
    }

    pub fn kelly_sized(
        account_equity: f64,
        confidence: f64,
        win_rate: f64,
        avg_win: f64,
        avg_loss: f64,
        kelly_fraction: f64,
        max_position_pct: f64,
    ) -> f64 {
        let kelly = KellyCalculator::fractional_kelly(win_rate, avg_win, avg_loss, kelly_fraction);
        let conf_adjusted = kelly * confidence;
        let pct_of_equity = conf_adjusted.min(max_position_pct).max(0.0);
        account_equity * pct_of_equity
    }

    pub fn volatility_adjusted(
        account_equity: f64,
        atr: f64,
        price: f64,
        risk_pct: f64,
        volatility_mult: f64,
    ) -> f64 {
        if price <= 0.0 || atr <= 0.0 {
            return 0.0;
        }
        let base_size = account_equity * risk_pct / price;
        let vol_ratio = atr / price;
        let adjustment = if vol_ratio > 0.0 {
            (0.02 / vol_ratio).clamp(0.5, 2.0) * volatility_mult
        } else {
            volatility_mult
        };
        base_size * adjustment
    }
}

#[derive(Debug, Clone)]
pub struct RiskManager {
    pub config: RiskConfig,
    pub var_calc: VarCalculator,
    pub drawdown: DrawdownMonitor,
    pub regime: Option<MarketRegime>,
    pub peak_equity: f64,
    pub history: VecDeque<RiskSnapshot>,
    pub max_history: usize,
}

#[derive(Debug, Clone)]
pub struct RiskSnapshot {
    pub equity: f64,
    pub var_95: f64,
    pub var_99: f64,
    pub max_dd: f64,
    pub sharpe: f64,
    pub position_count: usize,
    pub is_paused: bool,
    pub timestamp: u64,
}

impl RiskManager {
    pub fn new(initial_equity: f64, config: RiskConfig) -> Self {
        Self {
            config,
            var_calc: VarCalculator::new(500),
            drawdown: DrawdownMonitor::new(initial_equity),
            regime: None,
            peak_equity: initial_equity,
            history: VecDeque::with_capacity(100),
            max_history: 100,
        }
    }

    pub fn update_equity(&mut self, equity: f64) {
        if equity > self.peak_equity {
            self.peak_equity = equity;
        }
        self.drawdown.update(equity);
        self.var_calc
            .record((equity - self.peak_equity) / self.peak_equity.max(0.01));
    }

    pub fn set_regime(&mut self, regime: MarketRegime) {
        self.regime = Some(regime);
    }

    pub fn can_trade(&self) -> bool {
        if self.drawdown.is_paused {
            return false;
        }
        if self.drawdown.current_drawdown > self.config.max_drawdown_pct {
            return false;
        }
        true
    }

    pub fn validate_signal(&self, side: TradeSide, confidence: f64) -> Option<String> {
        if confidence < self.config.min_confidence {
            return Some(format!(
                "confidence {:.2} < min {:.2}",
                confidence, self.config.min_confidence
            ));
        }
        if !self.can_trade() {
            return Some(format!(
                "risk manager blocks trade: {}",
                self.drawdown.status_summary()
            ));
        }
        if let Some(ref regime) = self.regime {
            if side == TradeSide::Long
                && regime.trend
                    == crate::core::nt_core_trading::signal_generator::TrendDirection::Bearish
                && regime.strength > 0.7
            {
                return Some(format!(
                    "strong bearish regime ({:.2}) blocks long",
                    regime.strength
                ));
            }
            if side == TradeSide::Short
                && regime.trend
                    == crate::core::nt_core_trading::signal_generator::TrendDirection::Bullish
                && regime.strength > 0.7
            {
                return Some(format!(
                    "strong bullish regime ({:.2}) blocks short",
                    regime.strength
                ));
            }
        }
        None
    }

    pub fn position_size(
        &self,
        equity: f64,
        confidence: f64,
        win_rate: f64,
        avg_win: f64,
        avg_loss: f64,
    ) -> f64 {
        if !self.can_trade() {
            return 0.0;
        }
        PositionSizer::kelly_sized(
            equity,
            confidence,
            win_rate,
            avg_win,
            avg_loss,
            self.config.kelly_fraction,
            self.config.max_position_size_pct,
        )
    }

    pub fn apply_regime_adjustment(&self, base_size: f64) -> f64 {
        match self.regime {
            Some(ref regime) => {
                let vol_factor = match regime.volatility {
                    crate::core::nt_core_trading::signal_generator::VolatilityLevel::Extreme => {
                        0.25
                    }
                    crate::core::nt_core_trading::signal_generator::VolatilityLevel::High => 0.50,
                    crate::core::nt_core_trading::signal_generator::VolatilityLevel::Normal => 1.0,
                    crate::core::nt_core_trading::signal_generator::VolatilityLevel::Low => 1.25,
                };
                base_size * vol_factor
            }
            None => base_size,
        }
    }

    pub fn snapshot(&self, equity: f64, positions: &[Position]) -> RiskSnapshot {
        RiskSnapshot {
            equity,
            var_95: self.var_calc.var_95(),
            var_99: self.var_calc.var_99(),
            max_dd: self.drawdown.max_drawdown_observed,
            sharpe: self.var_calc.sharpe_ratio(0.02),
            position_count: positions.len(),
            is_paused: self.drawdown.is_paused,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn record_snapshot(&mut self, equity: f64, positions: &[Position]) {
        let snap = self.snapshot(equity, positions);
        while self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(snap);
    }

    pub fn status_report(&self, portfolio: &PortfolioSummary) -> String {
        format!(
            "Risk status: paused={} dd={:.1}% max_dd={:.1}% VaR95={:.2}% VaR99={:.2}% Sharpe={:.2} portfolio={:.2} positions={}",
            self.drawdown.is_paused,
            self.drawdown.current_drawdown * 100.0,
            self.drawdown.max_drawdown_observed * 100.0,
            self.var_calc.var_95() * 100.0,
            self.var_calc.var_99() * 100.0,
            self.var_calc.sharpe_ratio(0.02),
            portfolio.total_value,
            portfolio.positions.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_trading::signal_generator::{
        MarketRegime, TrendDirection, VolatilityLevel,
    };
    use crate::core::nt_core_trading::types::{AssetClass, OHLCVBar, Timeframe, TradeSide};

    // --- RiskConfig tests ---

    #[test]
    fn test_risk_config_defaults() {
        let cfg = RiskConfig::default();
        assert!((cfg.max_position_size_pct - 0.20).abs() < 1e-6);
        assert!((cfg.max_drawdown_pct - 0.15).abs() < 1e-6);
        assert!((cfg.kelly_fraction - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_risk_config_conservative() {
        let cfg = RiskConfig::conservative();
        assert!((cfg.kelly_fraction - 0.15).abs() < 1e-6);
        assert!((cfg.max_position_size_pct - 0.10).abs() < 1e-6);
    }

    #[test]
    fn test_risk_config_aggressive() {
        let cfg = RiskConfig::aggressive();
        assert!((cfg.kelly_fraction - 0.50).abs() < 1e-6);
        assert!((cfg.max_position_size_pct - 0.35).abs() < 1e-6);
    }

    #[test]
    fn test_risk_config_for_crypto() {
        let cfg = RiskConfig::for_asset_class(AssetClass::Crypto);
        assert!((cfg.max_position_size_pct - 0.10).abs() < 1e-6);
        assert!((cfg.max_drawdown_pct - 0.30).abs() < 1e-6);
    }

    #[test]
    fn test_risk_config_for_forex() {
        let cfg = RiskConfig::for_asset_class(AssetClass::Forex);
        assert!((cfg.max_leverage - 5.0).abs() < 1e-6);
    }

    // --- KellyCalculator tests ---

    #[test]
    fn test_kelly_fraction_winning_strategy() {
        // 60% win rate, 2:1 reward:risk
        let k = KellyCalculator::fraction(0.6, 0.02, 0.01);
        // kelly = (0.6 * 2 - 0.4) / 2 = (1.2 - 0.4) / 2 = 0.4
        assert!((k - 0.4).abs() < 1e-6, "kelly={} expected=0.4", k);
    }

    #[test]
    fn test_kelly_fraction_losing_strategy_returns_zero() {
        // 30% win rate, even reward:risk
        let k = KellyCalculator::fraction(0.3, 0.01, 0.01);
        assert!((k - 0.0).abs() < 1e-6, "kelly={} expected=0.0", k);
    }

    #[test]
    fn test_kelly_fraction_zero_avg_loss_returns_zero() {
        let k = KellyCalculator::fraction(0.6, 0.02, 0.0);
        assert!((k - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_kelly_fraction_perfect_win_rate_returns_zero() {
        let k = KellyCalculator::fraction(1.0, 0.02, 0.01);
        assert!((k - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_kelly_fraction_clamped_at_095() {
        let k = KellyCalculator::fraction(0.99, 100.0, 0.01);
        assert!(k <= 0.95, "kelly={} should be clamped at 0.95", k);
    }

    #[test]
    fn test_half_kelly() {
        let k = KellyCalculator::half_kelly(0.6, 0.02, 0.01);
        assert!((k - 0.2).abs() < 1e-6, "half_kelly={} expected=0.2", k);
    }

    #[test]
    fn test_quarter_kelly() {
        let k = KellyCalculator::quarter_kelly(0.6, 0.02, 0.01);
        assert!((k - 0.1).abs() < 1e-6, "quarter_kelly={} expected=0.1", k);
    }

    // --- DrawdownMonitor tests ---

    #[test]
    fn test_drawdown_monitor_new_active() {
        let dm = DrawdownMonitor::new(10000.0);
        assert!(!dm.is_paused);
        assert!((dm.current_drawdown - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_drawdown_monitor_updates_peak() {
        let mut dm = DrawdownMonitor::new(10000.0);
        dm.update(11000.0);
        assert!((dm.peak_value - 11000.0).abs() < 1e-6);
        assert!((dm.current_drawdown - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_drawdown_monitor_pauses_at_threshold() {
        let mut dm = DrawdownMonitor::new(10000.0);
        dm.pause_threshold = 0.05;
        dm.update(9400.0);
        assert!(dm.is_paused, "should pause when below 95% of peak");
    }

    #[test]
    fn test_drawdown_monitor_consecutive_losses_blocks() {
        let mut dm = DrawdownMonitor::new(10000.0);
        dm.max_consecutive_losses = 3;
        dm.record_trade(-0.01);
        dm.record_trade(-0.02);
        dm.record_trade(-0.015);
        assert!(dm.is_paused);
        assert_eq!(dm.consecutive_losses, 3);
    }

    #[test]
    fn test_drawdown_monitor_win_resets_consecutive_losses() {
        let mut dm = DrawdownMonitor::new(10000.0);
        dm.record_trade(-0.01);
        dm.record_trade(-0.02);
        dm.record_trade(0.03);
        assert!(!dm.is_paused);
        assert_eq!(dm.consecutive_losses, 0);
    }

    #[test]
    fn test_drawdown_monitor_resume() {
        let mut dm = DrawdownMonitor::new(10000.0);
        dm.is_paused = true;
        dm.consecutive_losses = 5;
        dm.resume();
        assert!(!dm.is_paused);
        assert_eq!(dm.consecutive_losses, 0);
    }

    #[test]
    fn test_drawdown_monitor_status_summary_paused() {
        let mut dm = DrawdownMonitor::new(10000.0);
        dm.is_paused = true;
        assert!(dm.status_summary().contains("PAUSED"));
    }

    #[test]
    fn test_drawdown_monitor_status_summary_active() {
        let dm = DrawdownMonitor::new(10000.0);
        assert!(dm.status_summary().contains("ACTIVE"));
    }

    // --- PositionSizer tests ---

    #[test]
    fn test_position_sizer_kelly_capped() {
        let size = PositionSizer::kelly_sized(10000.0, 1.0, 0.6, 0.02, 0.01, 0.25, 0.20);
        // kelly=0.4, fraction=0.25 => 0.1, conf=1.0 => 0.1, min(0.1, 0.2) => 0.1
        // size = 10000 * 0.1 = 1000
        assert!((size - 1000.0).abs() < 1e-6, "size={} expected=1000", size);
    }

    #[test]
    fn test_position_sizer_kelly_respects_max_position_pct() {
        let size = PositionSizer::kelly_sized(10000.0, 1.0, 0.8, 0.05, 0.01, 1.0, 0.05);
        // kelly*fraction=large, but max=0.05 => 10000*0.05=500
        assert!((size - 500.0).abs() < 1e-6, "size={} expected=500", size);
    }

    #[test]
    fn test_position_sizer_volatility_adjusted() {
        let size = PositionSizer::volatility_adjusted(10000.0, 2.0, 100.0, 0.02, 1.0);
        assert!(size > 0.0);
    }

    #[test]
    fn test_position_sizer_volatility_adjusted_zero_price() {
        let size = PositionSizer::volatility_adjusted(10000.0, 2.0, 0.0, 0.02, 1.0);
        assert!((size - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_risk_dollar() {
        let size = PositionSizer::risk_dollar(10000.0, 0.02, 0.05);
        // risk_amount = 10000 * 0.02 = 200, size = 200 / 0.05 = 4000
        assert!((size - 4000.0).abs() < 1e-6, "size={} expected=4000", size);
    }

    // --- VaR Calculator tests ---

    #[test]
    fn test_var_calculator_empty_returns_zero() {
        let vc = VarCalculator::new(500);
        assert!((vc.var_95() - 0.0).abs() < 1e-6);
        assert!((vc.var_99() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_var_calculator_historical() {
        let mut vc = VarCalculator::new(500);
        // Add 100 returns with mean ~0, mostly positive with some losses
        for i in 0..100 {
            let r = if i < 5 { -0.03 } else { 0.01 };
            vc.record(r);
        }
        let var95 = vc.var_95();
        assert!(var95 < 0.0, "VaR(95) should be negative, got {}", var95);
    }

    #[test]
    fn test_var_calculator_update_from_bars() {
        let bars: Vec<OHLCVBar> = (0..50)
            .map(|i| OHLCVBar {
                symbol: "BTC/USD".into(),
                asset_class: AssetClass::Crypto,
                timeframe: Timeframe::Hour1,
                open: 100.0 + i as f64,
                high: 101.0 + i as f64,
                low: 99.0 + i as f64,
                close: 100.0 + i as f64,
                volume: 1000.0,
                timestamp: i as u64 * 3600,
            })
            .collect();
        let mut vc = VarCalculator::new(500);
        vc.update_from_bars(&bars);
        assert!(
            vc.returns.len() == 49,
            "should have 49 returns from 50 bars"
        );
    }

    #[test]
    fn test_sharpe_ratio() {
        let mut vc = VarCalculator::new(500);
        for _ in 0..252 {
            vc.record(0.001);
        }
        let sharpe = vc.sharpe_ratio(0.02);
        assert!(sharpe > 0.0, "sharpe={} should be positive", sharpe);
    }

    #[test]
    fn test_sharpe_ratio_few_samples_returns_zero() {
        let vc = VarCalculator::new(500);
        assert!((vc.sharpe_ratio(0.02) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_drawdown() {
        let mut vc = VarCalculator::new(500);
        vc.record(0.1);
        vc.record(-0.2);
        vc.record(0.05);
        let mdd = vc.max_drawdown();
        assert!(mdd < 0.0, "max_drawdown={} should be negative", mdd);
    }

    // --- RiskManager tests ---

    #[test]
    fn test_risk_manager_new() {
        let rm = RiskManager::new(10000.0, RiskConfig::default());
        assert!(rm.can_trade());
    }

    #[test]
    fn test_risk_manager_validate_signal_below_min_confidence() {
        let rm = RiskManager::new(10000.0, RiskConfig::default());
        let result = rm.validate_signal(TradeSide::Long, 0.3);
        assert!(result.is_some(), "should reject low confidence");
        assert!(result.unwrap().contains("confidence"));
    }

    #[test]
    fn test_risk_manager_validate_signal_above_min_confidence() {
        let rm = RiskManager::new(10000.0, RiskConfig::default());
        let result = rm.validate_signal(TradeSide::Long, 0.8);
        assert!(result.is_none(), "should accept high confidence");
    }

    #[test]
    fn test_risk_manager_validate_signal_blocks_when_drawdown_exceeded() {
        let mut rm = RiskManager::new(10000.0, RiskConfig::default());
        rm.drawdown.current_drawdown = 0.20; // exceeds max_drawdown_pct of 0.15
        let result = rm.validate_signal(TradeSide::Long, 0.8);
        assert!(result.is_some(), "should block when drawdown exceeded");
    }

    #[test]
    fn test_risk_manager_validate_signal_blocks_when_paused() {
        let mut rm = RiskManager::new(10000.0, RiskConfig::default());
        rm.drawdown.is_paused = true;
        let result = rm.validate_signal(TradeSide::Long, 0.8);
        assert!(result.is_some(), "should block when paused");
    }

    #[test]
    fn test_risk_manager_blocks_long_in_strong_bearish() {
        let mut rm = RiskManager::new(10000.0, RiskConfig::default());
        let regime = MarketRegime {
            trend: TrendDirection::Bearish,
            volatility: VolatilityLevel::Normal,
            strength: 0.85,
        };
        rm.set_regime(regime);
        let result = rm.validate_signal(TradeSide::Long, 0.8);
        assert!(result.is_some(), "should block long in strong bearish");
    }

    #[test]
    fn test_risk_manager_blocks_short_in_strong_bullish() {
        let mut rm = RiskManager::new(10000.0, RiskConfig::default());
        let regime = MarketRegime {
            trend: TrendDirection::Bullish,
            volatility: VolatilityLevel::Normal,
            strength: 0.85,
        };
        rm.set_regime(regime);
        let result = rm.validate_signal(TradeSide::Short, 0.8);
        assert!(result.is_some(), "should block short in strong bullish");
    }

    #[test]
    fn test_risk_manager_allows_short_in_bearish() {
        let mut rm = RiskManager::new(10000.0, RiskConfig::default());
        let regime = MarketRegime {
            trend: TrendDirection::Bearish,
            volatility: VolatilityLevel::Normal,
            strength: 0.85,
        };
        rm.set_regime(regime);
        let result = rm.validate_signal(TradeSide::Short, 0.8);
        assert!(result.is_none(), "should allow short in bearish");
    }

    #[test]
    fn test_risk_manager_position_size_zero_when_cannot_trade() {
        let mut rm = RiskManager::new(10000.0, RiskConfig::default());
        rm.drawdown.is_paused = true;
        let size = rm.position_size(10000.0, 0.8, 0.6, 0.02, 0.01);
        assert!((size - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_risk_manager_position_size_positive_when_can_trade() {
        let rm = RiskManager::new(10000.0, RiskConfig::default());
        let size = rm.position_size(10000.0, 0.8, 0.6, 0.02, 0.01);
        assert!(size > 0.0, "size={} should be positive", size);
    }

    #[test]
    fn test_risk_manager_regime_adjustment_extreme_vol() {
        let mut rm = RiskManager::new(10000.0, RiskConfig::default());
        let regime = MarketRegime {
            trend: TrendDirection::Sideways,
            volatility: VolatilityLevel::Extreme,
            strength: 0.5,
        };
        rm.set_regime(regime);
        let adjusted = rm.apply_regime_adjustment(1000.0);
        assert!(
            (adjusted - 250.0).abs() < 1e-6,
            "adjusted={} expected=250",
            adjusted
        );
    }

    #[test]
    fn test_risk_manager_regime_adjustment_low_vol() {
        let mut rm = RiskManager::new(10000.0, RiskConfig::default());
        let regime = MarketRegime {
            trend: TrendDirection::Sideways,
            volatility: VolatilityLevel::Low,
            strength: 0.5,
        };
        rm.set_regime(regime);
        let adjusted = rm.apply_regime_adjustment(1000.0);
        assert!(
            (adjusted - 1250.0).abs() < 1e-6,
            "adjusted={} expected=1250",
            adjusted
        );
    }

    #[test]
    fn test_risk_manager_snapshot() {
        let rm = RiskManager::new(10000.0, RiskConfig::default());
        let snap = rm.snapshot(10000.0, &[]);
        assert!((snap.equity - 10000.0).abs() < 1e-6);
        assert!(snap.var_95 == 0.0);
    }

    #[test]
    fn test_risk_manager_update_equity_tracks_peak() {
        let mut rm = RiskManager::new(10000.0, RiskConfig::default());
        rm.update_equity(12000.0);
        assert!((rm.peak_equity - 12000.0).abs() < 1e-6);
    }

    #[test]
    fn test_risk_manager_status_report() {
        let mut rm = RiskManager::new(10000.0, RiskConfig::default());
        let portfolio = PortfolioSummary::new(10000.0);
        let report = rm.status_report(&portfolio);
        assert!(report.contains("Risk status"));
    }
}
