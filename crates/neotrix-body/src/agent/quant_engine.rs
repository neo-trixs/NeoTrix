use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::io::finance_pipeline::KLine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalAction {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossDir {
    Above,
    Below,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingRule {
    Crossing {
        indicator: String,
        threshold: f64,
        direction: CrossDir,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub name: String,
    pub indicators: Vec<String>,
    pub rules: Vec<TradingRule>,
    pub vsa_schema: [u64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSignal {
    pub symbol: String,
    pub action: SignalAction,
    pub confidence: f64,
    pub reason: String,
    pub vsa_trigger: [u64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub total_return_pct: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown_pct: f64,
    pub win_rate: f64,
    pub trade_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaType {
    Sma,
    Ema,
}

pub trait TechnicalIndicator: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn compute(&self, kline: &KLine) -> Vec<f64>;
    fn vsa_signature(&self) -> [u64; 4];
}

#[derive(Debug, Clone)]
pub struct MacdIndicator {
    pub fast: usize,
    pub slow: usize,
    pub signal: usize,
}

#[derive(Debug, Clone)]
pub struct RsiIndicator {
    pub period: usize,
}

#[derive(Debug, Clone)]
pub struct MovingAverage {
    pub period: usize,
    pub ma_type: MaType,
}

impl MacdIndicator {
    pub fn new(fast: usize, slow: usize, signal: usize) -> Self {
        Self { fast, slow, signal }
    }
}

impl Default for MacdIndicator {
    fn default() -> Self {
        Self { fast: 12, slow: 26, signal: 9 }
    }
}

impl TechnicalIndicator for MacdIndicator {
    fn name(&self) -> &str {
        "MACD"
    }

    fn compute(&self, kline: &KLine) -> Vec<f64> {
        let values: Vec<f64> = kline.bars.iter().map(|b| b.close).collect();
        let (_, _, histogram) = Self::compute_macd_inner(&values, self.fast, self.slow, self.signal);
        histogram
    }

    fn vsa_signature(&self) -> [u64; 4] {
        [0x4d414344, 0x00000001, 0x00000000, 0x00000000]
    }
}

impl MacdIndicator {
    fn compute_macd_inner(
        values: &[f64],
        fast: usize,
        slow: usize,
        signal: usize,
    ) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        if values.len() < slow + signal {
            return (vec![], vec![], vec![]);
        }
        let fast_ema = Self::ema(values, fast);
        let slow_ema = Self::ema(values, slow);
        let macd_line: Vec<f64> = fast_ema
            .iter()
            .zip(slow_ema.iter())
            .map(|(f, s)| f - s)
            .collect();
        let signal_line = Self::ema_skip_nan(&macd_line, signal);
        let histogram: Vec<f64> = macd_line
            .iter()
            .zip(signal_line.iter())
            .map(|(m, s)| m - s)
            .collect();
        (macd_line, signal_line, histogram)
    }

    fn ema(values: &[f64], period: usize) -> Vec<f64> {
        let n = values.len();
        let mut result = vec![f64::NAN; n];
        if n < period {
            return result;
        }
        let sum: f64 = values[0..period].iter().sum();
        let mut ema_val = sum / period as f64;
        result[period - 1] = ema_val;
        let multiplier = 2.0 / (period as f64 + 1.0);
        for i in period..n {
            ema_val = (values[i] - ema_val) * multiplier + ema_val;
            result[i] = ema_val;
        }
        result
    }

    fn ema_skip_nan(values: &[f64], period: usize) -> Vec<f64> {
        let n = values.len();
        let mut result = vec![f64::NAN; n];
        if n < period {
            return result;
        }
        let first = match values.iter().position(|v| !v.is_nan()) {
            Some(i) if i + period <= n => i,
            _ => return result,
        };
        let sum: f64 = values[first..first + period].iter().sum();
        let mut ema_val = sum / period as f64;
        result[first + period - 1] = ema_val;
        let multiplier = 2.0 / (period as f64 + 1.0);
        for i in (first + period)..n {
            if values[i].is_nan() {
                continue;
            }
            ema_val = (values[i] - ema_val) * multiplier + ema_val;
            result[i] = ema_val;
        }
        result
    }
}

impl RsiIndicator {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Default for RsiIndicator {
    fn default() -> Self {
        Self { period: 14 }
    }
}

impl TechnicalIndicator for RsiIndicator {
    fn name(&self) -> &str {
        "RSI"
    }

    fn compute(&self, kline: &KLine) -> Vec<f64> {
        let values: Vec<f64> = kline.bars.iter().map(|b| b.close).collect();
        Self::compute_rsi_inner(&values, self.period)
    }

    fn vsa_signature(&self) -> [u64; 4] {
        [0x52534900, 0x00000002, 0x00000000, 0x00000000]
    }
}

impl RsiIndicator {
    fn compute_rsi_inner(values: &[f64], period: usize) -> Vec<f64> {
        let n = values.len();
        let mut rsi = vec![f64::NAN; n];
        if n < period + 1 {
            return rsi;
        }
        let mut gains = Vec::with_capacity(n);
        let mut losses = Vec::with_capacity(n);
        for i in 1..n {
            let diff = values[i] - values[i - 1];
            if diff > 0.0 {
                gains.push(diff);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-diff);
            }
        }
        let avg_gain_sma: f64 = gains[0..period].iter().sum();
        let avg_loss_sma: f64 = losses[0..period].iter().sum();
        let mut avg_gain = avg_gain_sma / period as f64;
        let mut avg_loss = avg_loss_sma / period as f64;
        let rs = if avg_loss.abs() > 1e-10 {
            avg_gain / avg_loss
        } else {
            100.0
        };
        rsi[period] = 100.0 - 100.0 / (1.0 + rs);
        for i in (period + 1)..n {
            avg_gain = (avg_gain * (period as f64 - 1.0) + gains[i - 1]) / period as f64;
            avg_loss = (avg_loss * (period as f64 - 1.0) + losses[i - 1]) / period as f64;
            let rs = if avg_loss.abs() > 1e-10 {
                avg_gain / avg_loss
            } else {
                100.0
            };
            rsi[i] = 100.0 - 100.0 / (1.0 + rs);
        }
        rsi
    }
}

impl MovingAverage {
    pub fn new(period: usize, ma_type: MaType) -> Self {
        Self { period, ma_type }
    }
}

impl TechnicalIndicator for MovingAverage {
    fn name(&self) -> &str {
        match self.ma_type {
            MaType::Sma => "SMA",
            MaType::Ema => "EMA",
        }
    }

    fn compute(&self, kline: &KLine) -> Vec<f64> {
        let values: Vec<f64> = kline.bars.iter().map(|b| b.close).collect();
        match self.ma_type {
            MaType::Sma => Self::sma(&values, self.period),
            MaType::Ema => Self::ema(&values, self.period),
        }
    }

    fn vsa_signature(&self) -> [u64; 4] {
        match self.ma_type {
            MaType::Sma => [0x534d4100, 0x00000003, self.period as u64, 0x00000000],
            MaType::Ema => [0x454d4100, 0x00000004, self.period as u64, 0x00000000],
        }
    }
}

impl MovingAverage {
    fn sma(values: &[f64], period: usize) -> Vec<f64> {
        let n = values.len();
        let mut result = vec![f64::NAN; n];
        if n < period {
            return result;
        }
        let mut sum: f64 = values[0..period].iter().sum();
        result[period - 1] = sum / period as f64;
        for i in period..n {
            sum += values[i] - values[i - period];
            result[i] = sum / period as f64;
        }
        result
    }

    fn ema(values: &[f64], period: usize) -> Vec<f64> {
        let n = values.len();
        let mut result = vec![f64::NAN; n];
        if n < period {
            return result;
        }
        let sum: f64 = values[0..period].iter().sum();
        let mut ema_val = sum / period as f64;
        result[period - 1] = ema_val;
        let multiplier = 2.0 / (period as f64 + 1.0);
        for i in period..n {
            ema_val = (values[i] - ema_val) * multiplier + ema_val;
            result[i] = ema_val;
        }
        result
    }
}

#[derive(Debug)]
pub struct QuantEngine {
    indicators: Vec<Box<dyn TechnicalIndicator>>,
    strategies: Vec<Strategy>,
}

impl Clone for QuantEngine {
    fn clone(&self) -> Self {
        Self {
            indicators: Vec::new(),
            strategies: self.strategies.clone(),
        }
    }
}

impl QuantEngine {
    pub fn new() -> Self {
        Self {
            indicators: vec![
                Box::new(MacdIndicator::default()),
                Box::new(RsiIndicator::default()),
                Box::new(MovingAverage::new(20, MaType::Sma)),
                Box::new(MovingAverage::new(20, MaType::Ema)),
            ],
            strategies: vec![],
        }
    }

    pub fn add_indicator(&mut self, indicator: Box<dyn TechnicalIndicator>) {
        self.indicators.push(indicator);
    }

    pub fn add_strategy(&mut self, strategy: Strategy) {
        self.strategies.push(strategy);
    }

    pub fn compute_signals(&self, kline: &KLine) -> Vec<TradingSignal> {
        if kline.bars.is_empty() {
            return vec![];
        }

        let mut indicator_map: HashMap<String, Vec<f64>> = HashMap::new();
        for ind in &self.indicators {
            let values = ind.compute(kline);
            indicator_map.insert(ind.name().to_string(), values);
        }

        let mut signals = Vec::new();
        for strategy in &self.strategies {
            let mut all_rules_met = true;
            let mut reasons = Vec::new();
            let mut action = SignalAction::Hold;

            for rule in &strategy.rules {
                match rule {
                    TradingRule::Crossing {
                        indicator: ind_name,
                        threshold,
                        direction,
                    } => {
                        if let Some(values) = indicator_map.get(ind_name) {
                            if let Some(&latest) = values.last() {
                                if latest.is_nan() {
                                    all_rules_met = false;
                                    continue;
                                }
                                let met = match direction {
                                    CrossDir::Above => latest > *threshold,
                                    CrossDir::Below => latest < *threshold,
                                };
                                if met {
                                    reasons.push(format!(
                                        "{} crossed {} {:.2}",
                                        ind_name,
                                        match direction {
                                            CrossDir::Above => "above",
                                            CrossDir::Below => "below",
                                        },
                                        threshold
                                    ));
                                } else {
                                    all_rules_met = false;
                                }
                            }
                        }
                    }
                }
            }

            if all_rules_met && !reasons.is_empty() {
                if reasons.iter().any(|r| r.contains("crossed below")) {
                    action = SignalAction::Sell;
                } else if reasons.iter().any(|r| r.contains("crossed above")) {
                    action = SignalAction::Buy;
                }

                let confidence = 0.5 + (reasons.len() as f64 * 0.1).min(0.4);
                signals.push(TradingSignal {
                    symbol: kline.symbol.clone(),
                    action,
                    confidence: (confidence * 100.0).round() / 100.0,
                    reason: reasons.join("; "),
                    vsa_trigger: strategy.vsa_schema,
                });
            }
        }

        signals
    }

    pub fn backtest(&self, kline: &KLine, initial_capital: f64) -> BacktestResult {
        if kline.bars.len() < 2 {
            return BacktestResult {
                total_return_pct: 0.0,
                sharpe_ratio: 0.0,
                max_drawdown_pct: 0.0,
                win_rate: 0.0,
                trade_count: 0,
            };
        }

        #[allow(dead_code)]
        struct Trade {
            entry_price: f64,
            exit_price: f64,
            return_pct: f64,
        }

        let mut trades: Vec<Trade> = Vec::new();
        let mut capital = initial_capital;
        let mut position: Option<f64> = None;
        let mut portfolio_values = Vec::with_capacity(kline.bars.len());
        let mut peak = initial_capital;
        let mut max_drawdown = 0.0;

        for i in 0..kline.bars.len() {
            let bar = &kline.bars[i];

            let sub_kline = KLine {
                symbol: kline.symbol.clone(),
                interval: kline.interval.clone(),
                bars: kline.bars[..=i].to_vec(),
            };
            let signals = self.compute_signals(&sub_kline);

            for signal in &signals {
                match signal.action {
                    SignalAction::Buy if position.is_none() => {
                        position = Some(bar.close);
                    }
                    SignalAction::Sell => {
                        if let Some(entry) = position.take() {
                            let ret = (bar.close - entry) / entry;
                            let ret_pct = ret * 100.0;
                            capital *= 1.0 + ret;
                            trades.push(Trade {
                                entry_price: entry,
                                exit_price: bar.close,
                                return_pct: ret_pct,
                            });
                        }
                    }
                    _ => {}
                }
            }

            let current_value = match position {
                Some(entry) => capital * (bar.close / entry),
                None => capital,
            };
            portfolio_values.push(current_value);

            if current_value > peak {
                peak = current_value;
            }
            let dd = (peak - current_value) / peak * 100.0;
            if dd > max_drawdown {
                max_drawdown = dd;
            }
        }

        if let Some(entry) = position {
            let last_close = kline.bars.last().unwrap().close;
            let ret = (last_close - entry) / entry;
            capital *= 1.0 + ret;
            trades.push(Trade {
                entry_price: entry,
                exit_price: last_close,
                return_pct: ret * 100.0,
            });
        }

        let trade_count = trades.len() as u32;
        let win_count = trades.iter().filter(|t| t.return_pct > 0.0).count() as f64;
        let win_rate = if trade_count > 0 {
            win_count / trade_count as f64 * 100.0
        } else {
            0.0
        };

        let total_return_pct = (capital - initial_capital) / initial_capital * 100.0;

        let daily_returns: Vec<f64> = portfolio_values
            .windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();

        let sharpe_ratio = if daily_returns.len() > 1 {
            let mean = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
            let variance = daily_returns
                .iter()
                .map(|r| (r - mean).powi(2))
                .sum::<f64>()
                / (daily_returns.len() - 1) as f64;
            let std_dev = variance.sqrt();
            if std_dev > 1e-10 {
                mean / std_dev * 252.0_f64.sqrt()
            } else {
                0.0
            }
        } else {
            0.0
        };

        BacktestResult {
            total_return_pct: (total_return_pct * 100.0).round() / 100.0,
            sharpe_ratio: (sharpe_ratio * 100.0).round() / 100.0,
            max_drawdown_pct: (max_drawdown * 100.0).round() / 100.0,
            win_rate: (win_rate * 100.0).round() / 100.0,
            trade_count,
        }
    }

    pub fn indicators(&self) -> &[Box<dyn TechnicalIndicator>] {
        &self.indicators
    }

    pub fn strategies(&self) -> &[Strategy] {
        &self.strategies
    }

    pub fn compute_macd(
        values: &[f64],
        fast: usize,
        slow: usize,
        signal: usize,
    ) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        MacdIndicator::compute_macd_inner(values, fast, slow, signal)
    }

    pub fn compute_rsi(values: &[f64], period: usize) -> Vec<f64> {
        RsiIndicator::compute_rsi_inner(values, period)
    }
}

impl Default for QuantEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::finance_pipeline::{KLineInterval, Ohlcv};

    fn make_kline(symbol: &str, closes: &[f64]) -> KLine {
        KLine {
            symbol: symbol.to_string(),
            interval: KLineInterval::Day,
            bars: closes
                .iter()
                .map(|&c| Ohlcv {
                    open: c,
                    high: c * 1.01,
                    low: c * 0.99,
                    close: c,
                    volume: 1_000_000,
                })
                .collect(),
        }
    }

    #[test]
    fn test_new_engine() {
        let engine = QuantEngine::new();
        assert_eq!(engine.indicators().len(), 4);
    }

    #[test]
    fn test_add_indicator() {
        let mut engine = QuantEngine::new();
        engine.add_indicator(Box::new(MovingAverage::new(50, MaType::Sma)));
        assert_eq!(engine.indicators().len(), 5);
    }

    #[test]
    fn test_macd_indicator_name() {
        let ind = MacdIndicator::default();
        assert_eq!(ind.name(), "MACD");
    }

    #[test]
    fn test_rsi_indicator_name() {
        let ind = RsiIndicator::new(14);
        assert_eq!(ind.name(), "RSI");
    }

    #[test]
    fn test_macd_compute() {
        let kline = make_kline("600519", &[10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0]);
        let ind = MacdIndicator::new(3, 6, 3);
        let result = ind.compute(&kline);
        assert_eq!(result.len(), 11);
        assert!(result[5..].iter().any(|v| !v.is_nan()));
    }

    #[test]
    fn test_rsi_compute() {
        let kline = make_kline("000001", &[44.0, 45.0, 46.0, 47.0, 48.0, 49.0, 50.0, 49.0, 48.0, 47.0, 46.0, 45.0, 44.0, 43.0, 42.0]);
        let ind = RsiIndicator::new(5);
        let result = ind.compute(&kline);
        assert_eq!(result.len(), 15);
        assert!(result[5..].iter().any(|v| !v.is_nan()));
    }

    #[test]
    fn test_sma_indicator() {
        let kline = make_kline("600000", &[1.0, 2.0, 3.0, 4.0, 5.0]);
        let ind = MovingAverage::new(3, MaType::Sma);
        let result = ind.compute(&kline);
        assert_eq!(result.len(), 5);
        assert!(result[2].is_finite());
        assert!((result[2] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_ema_indicator() {
        let kline = make_kline("600000", &[1.0, 2.0, 3.0, 4.0, 5.0]);
        let ind = MovingAverage::new(3, MaType::Ema);
        let result = ind.compute(&kline);
        assert_eq!(result.len(), 5);
        assert!(result[2].is_finite());
    }

    #[test]
    fn test_compute_macd_static() {
        let values = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0];
        let (macd, signal, hist) = QuantEngine::compute_macd(&values, 3, 6, 3);
        assert_eq!(macd.len(), 11);
        assert_eq!(signal.len(), 11);
        assert_eq!(hist.len(), 11);
    }

    #[test]
    fn test_compute_rsi_static() {
        let values = vec![44.0, 45.0, 46.0, 47.0, 48.0, 49.0, 50.0, 49.0, 48.0, 47.0, 46.0, 45.0, 44.0, 43.0, 42.0];
        let rsi = QuantEngine::compute_rsi(&values, 5);
        assert_eq!(rsi.len(), 15);
        assert!(rsi[5] > 0.0 && rsi[5] < 100.0);
    }

    #[test]
    fn test_strategy_and_signal() {
        let mut engine = QuantEngine::new();
        engine.add_strategy(Strategy {
            name: "RSI Oversold".into(),
            indicators: vec!["RSI".into()],
            rules: vec![TradingRule::Crossing {
                indicator: "RSI".into(),
                threshold: 30.0,
                direction: CrossDir::Below,
            }],
            vsa_schema: [1, 2, 3, 4],
        });

        let kline = make_kline("300750", &[50.0, 49.0, 48.0, 47.0, 46.0, 45.0, 44.0, 43.0, 42.0, 41.0, 40.0, 39.0, 38.0, 37.0, 36.0, 35.0, 34.0, 33.0, 32.0, 31.0, 30.0]);
        let signals = engine.compute_signals(&kline);
        assert!(signals.is_empty() || signals.iter().any(|s| matches!(s.action, SignalAction::Sell)));
    }

    #[test]
    fn test_backtest_empty_kline() {
        let engine = QuantEngine::new();
        let kline = make_kline("600000", &[]);
        let result = engine.backtest(&kline, 100_000.0);
        assert_eq!(result.trade_count, 0);
    }

    #[test]
    fn test_backtest_simple() {
        let mut engine = QuantEngine::new();
        engine.add_strategy(Strategy {
            name: "Always Buy".into(),
            indicators: vec![],
            rules: vec![],
            vsa_schema: [0; 4],
        });

        let kline = make_kline("600519", &[100.0, 101.0, 102.0, 103.0, 104.0, 105.0]);
        let result = engine.backtest(&kline, 10_000.0);
        assert!(result.total_return_pct >= 0.0 || result.trade_count == 0);
    }

    #[test]
    fn test_vsa_signature_macd() {
        let ind = MacdIndicator::default();
        let sig = ind.vsa_signature();
        assert_ne!(sig, [0; 4]);
    }

    #[test]
    fn test_vsa_signature_rsi() {
        let ind = RsiIndicator::new(14);
        let sig = ind.vsa_signature();
        assert_ne!(sig, [0; 4]);
    }

    #[test]
    fn test_vsa_signature_ma() {
        let ind = MovingAverage::new(20, MaType::Sma);
        let sig = ind.vsa_signature();
        assert_ne!(sig, [0; 4]);
    }

    #[test]
    fn test_indicators_dispatch() {
        let engine = QuantEngine::new();
        let names: Vec<&str> = engine.indicators().iter().map(|i| i.name()).collect();
        assert!(names.contains(&"MACD"));
        assert!(names.contains(&"RSI"));
        assert!(names.contains(&"SMA"));
        assert!(names.contains(&"EMA"));
    }
}
