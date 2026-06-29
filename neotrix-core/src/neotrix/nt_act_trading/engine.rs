use crate::core::nt_core_trading::risk_manager::{RiskConfig, RiskManager};
use crate::core::nt_core_trading::signal_generator::{MarketRegime, SignalGenerator};
use crate::core::nt_core_trading::types::{AssetClass, OHLCVBar, PortfolioSummary, TradingSignal};

#[derive(Debug, Clone)]
pub struct TradingEngine {
    pub signal_generator: SignalGenerator,
    pub risk_manager: RiskManager,
    pub portfolio: PortfolioSummary,
    pub symbol: String,
    pub asset_class: AssetClass,
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub total_trades: u64,
    pub winning_trades: u64,
}

impl TradingEngine {
    pub fn new(symbol: &str, asset_class: AssetClass, initial_equity: f64) -> Self {
        let config = RiskConfig::for_asset_class(asset_class);
        Self {
            signal_generator: SignalGenerator::new(500),
            risk_manager: RiskManager::new(initial_equity, config),
            portfolio: PortfolioSummary::new(initial_equity),
            symbol: symbol.to_string(),
            asset_class,
            win_rate: 0.5,
            avg_win: 0.02,
            avg_loss: 0.015,
            total_trades: 0,
            winning_trades: 0,
        }
    }

    pub fn ingest_bar(&mut self, bar: OHLCVBar) {
        self.signal_generator.ingest(bar);
    }

    pub fn ingest_bars(&mut self, bars: &[OHLCVBar]) {
        for bar in bars {
            self.ingest_bar(bar.clone());
        }
    }

    pub fn detect_regime(&mut self) -> Option<MarketRegime> {
        let bars = self.signal_generator.bars_slice();
        if bars.len() < 50 {
            return None;
        }
        let regime = MarketRegime::detect(&bars);
        self.risk_manager.set_regime(regime.clone());
        Some(regime)
    }

    pub fn generate_signal(&self) -> Option<TradingSignal> {
        self.signal_generator.generate_signal(None, None, None)
    }

    pub fn update_equity(&mut self, equity: f64) {
        self.risk_manager.update_equity(equity);
        self.portfolio.total_value = equity;
    }

    pub fn record_trade_result(&mut self, pnl_pct: f64) {
        self.total_trades += 1;
        if pnl_pct > 0.0 {
            self.winning_trades += 1;
        }
        self.win_rate = if self.total_trades > 0 {
            self.winning_trades as f64 / self.total_trades as f64
        } else {
            0.5
        };
        let window = 20.min(self.total_trades as usize);
        if window > 0 {
            let mult = 1.0 / window as f64;
            self.avg_win = self.avg_win * (1.0 - mult) + pnl_pct.max(0.0) * mult;
            self.avg_loss = self.avg_loss * (1.0 - mult) + pnl_pct.min(0.0).abs() * mult;
        }
        self.risk_manager.drawdown.record_trade(pnl_pct);
    }

    pub fn position_size(&mut self, confidence: f64) -> f64 {
        let equity = self.portfolio.total_value;
        let base = self.risk_manager.position_size(
            equity,
            confidence,
            self.win_rate,
            self.avg_win,
            self.avg_loss,
        );
        self.risk_manager.apply_regime_adjustment(base)
    }

    pub fn can_trade(&self) -> bool {
        self.risk_manager.can_trade()
    }

    pub fn status_report(&self) -> String {
        let regime = match self.risk_manager.regime {
            Some(ref r) => format!(
                "{:?}/{:?} strength={:.2}",
                r.trend, r.volatility, r.strength
            ),
            None => "unknown".to_string(),
        };
        format!(
            "TradingEngine[{}] regime={} equity={:.2} win_rate={:.2} trades={} paused={} dd={:.2}% signal_gen={}",
            self.symbol,
            regime,
            self.portfolio.total_value,
            self.win_rate,
            self.total_trades,
            self.risk_manager.drawdown.is_paused,
            self.risk_manager.drawdown.current_drawdown * 100.0,
            self.signal_generator.bars.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_trading::types::{AssetClass, OHLCVBar, Timeframe, TradeSide};

    fn make_bar(close: f64, i: usize) -> OHLCVBar {
        OHLCVBar {
            symbol: "BTC/USD".into(),
            asset_class: AssetClass::Crypto,
            timeframe: Timeframe::Hour1,
            open: close - 0.5,
            high: close + 1.0,
            low: close - 1.0,
            close,
            volume: 1000.0,
            timestamp: i as u64 * 3600,
        }
    }

    fn make_bullish_bars(count: usize) -> Vec<OHLCVBar> {
        (0..count)
            .map(|i| {
                let base = 100.0 + (i as f64) * 0.5;
                make_bar(base, i)
            })
            .collect()
    }

    #[test]
    fn test_trading_engine_new() {
        let engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        assert_eq!(engine.symbol, "BTC/USD");
        assert_eq!(engine.asset_class, AssetClass::Crypto);
        assert!((engine.portfolio.total_value - 10000.0).abs() < 1e-6);
        assert!((engine.win_rate - 0.5).abs() < 1e-6);
        assert_eq!(engine.total_trades, 0);
        assert!(engine.can_trade());
    }

    #[test]
    fn test_trading_engine_new_stock_defaults() {
        let engine = TradingEngine::new("AAPL", AssetClass::Stock, 50000.0);
        assert_eq!(engine.symbol, "AAPL");
        assert!((engine.portfolio.total_value - 50000.0).abs() < 1e-6);
    }

    #[test]
    fn test_trading_engine_ingest_bars() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        let bars = make_bullish_bars(30);
        engine.ingest_bars(&bars);
        assert_eq!(engine.signal_generator.bars.len(), 30);
    }

    #[test]
    fn test_trading_engine_generate_signal_with_enough_data() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        engine.ingest_bars(&make_bullish_bars(100));
        let signal = engine.generate_signal();
        assert!(signal.is_some(), "should generate signal with 100 bars");
        if let Some(sig) = signal {
            assert_eq!(sig.symbol, "BTC/USD");
        }
    }

    #[test]
    fn test_trading_engine_generate_signal_without_data_returns_none() {
        let engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        let signal = engine.generate_signal();
        assert!(signal.is_none(), "should return None with no bars");
    }

    #[test]
    fn test_trading_engine_detect_regime_with_enough_data() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        engine.ingest_bars(&make_bullish_bars(210));
        let regime = engine.detect_regime();
        assert!(regime.is_some(), "should detect regime with 210 bars");
        assert_eq!(
            regime.unwrap().trend,
            crate::core::nt_core_trading::signal_generator::TrendDirection::Bullish
        );
    }

    #[test]
    fn test_trading_engine_detect_regime_too_few_bars() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        engine.ingest_bars(&make_bullish_bars(30));
        let regime = engine.detect_regime();
        assert!(regime.is_none(), "should return None with < 50 bars");
    }

    #[test]
    fn test_trading_engine_update_equity() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        engine.update_equity(12000.0);
        assert!((engine.portfolio.total_value - 12000.0).abs() < 1e-6);
    }

    #[test]
    fn test_trading_engine_record_trade_result_winning() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        engine.record_trade_result(0.05);
        assert_eq!(engine.total_trades, 1);
        assert_eq!(engine.winning_trades, 1);
        assert!((engine.win_rate - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_trading_engine_record_trade_result_losing() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        engine.record_trade_result(-0.03);
        assert_eq!(engine.total_trades, 1);
        assert_eq!(engine.winning_trades, 0);
        // avg_loss updated to 0.03 (abs)
        assert!((engine.avg_loss - 0.03).abs() < 1e-6);
    }

    #[test]
    fn test_trading_engine_record_trade_result_updates_win_rate() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        engine.record_trade_result(0.05);
        engine.record_trade_result(-0.03);
        engine.record_trade_result(0.02);
        assert_eq!(engine.total_trades, 3);
        assert_eq!(engine.winning_trades, 2);
        assert!((engine.win_rate - 2.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_trading_engine_record_trade_ema_smoothing() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        // avg_win starts at 0.02
        for _ in 0..20 {
            engine.record_trade_result(0.04);
        }
        // After 20 wins, avg_win should be close to 0.04
        assert!(
            (engine.avg_win - 0.04).abs() < 0.005,
            "avg_win={} should converge to 0.04",
            engine.avg_win
        );
    }

    #[test]
    fn test_trading_engine_position_size() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        engine.ingest_bars(&make_bullish_bars(210));
        engine.detect_regime();
        let size = engine.position_size(0.8);
        // Crypto uses kelly_fraction=0.20, max_position_size_pct=0.10
        // kelly with 0.5 win_rate, 0.02 avg_win, 0.015 avg_loss
        // kelly = (0.5 * 1.333 - 0.5) / 1.333 = (0.667 - 0.5) / 1.333 = 0.125
        // conf_adj = 0.125 * 0.8 = 0.1 = min(0.1, 0.1) = 0.1
        // base = 10000 * 0.1 = 1000
        // regime adj for normal vol = 1.0
        assert!((size - 1000.0).abs() < 50.0, "size={} expected ~1000", size);
    }

    #[test]
    fn test_trading_engine_position_size_zero_when_cannot_trade() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        engine.risk_manager.drawdown.is_paused = true;
        let size = engine.position_size(0.8);
        assert!((size - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_trading_engine_can_trade_true_by_default() {
        let engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        assert!(engine.can_trade());
    }

    #[test]
    fn test_trading_engine_can_trade_false_when_paused() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        engine.risk_manager.drawdown.is_paused = true;
        assert!(!engine.can_trade());
    }

    #[test]
    fn test_trading_engine_status_report() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        engine.ingest_bars(&make_bullish_bars(60));
        let report = engine.status_report();
        assert!(report.contains("BTC/USD"));
        assert!(report.contains("equity=10000.00"));
    }

    #[test]
    fn test_trading_engine_detect_regime_sets_risk_manager_regime() {
        let mut engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        engine.ingest_bars(&make_bullish_bars(210));
        engine.detect_regime();
        assert!(
            engine.risk_manager.regime.is_some(),
            "regime should propagate to risk_manager"
        );
    }

    #[test]
    fn test_trading_engine_no_data_does_not_panic() {
        let engine = TradingEngine::new("BTC/USD", AssetClass::Crypto, 10000.0);
        // calling methods with no data should never panic
        let _ = engine.can_trade();
        let _ = engine.generate_signal();
        let _ = engine.status_report();
    }
}
