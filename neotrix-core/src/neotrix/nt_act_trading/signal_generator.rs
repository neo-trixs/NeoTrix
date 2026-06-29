use crate::core::nt_core_trading::types::{
    AssetClass, OHLCVBar, SignalFusion, TradeSide, TradingSignal,
};
use std::collections::VecDeque;

const TECHNICAL_WEIGHT: f64 = 0.35;
const SENTIMENT_WEIGHT: f64 = 0.25;
const TRUTH_WEIGHT: f64 = 0.20;
const ONCHAIN_WEIGHT: f64 = 0.20;

#[derive(Debug, Clone)]
pub struct TechnicalIndicators {
    pub sma_20: Option<f64>,
    pub sma_50: Option<f64>,
    pub sma_200: Option<f64>,
    pub ema_12: Option<f64>,
    pub ema_26: Option<f64>,
    pub rsi_14: Option<f64>,
    pub macd_line: Option<f64>,
    pub macd_signal: Option<f64>,
    pub macd_histogram: Option<f64>,
    pub bollinger_upper: Option<f64>,
    pub bollinger_middle: Option<f64>,
    pub bollinger_lower: Option<f64>,
    pub atr_14: Option<f64>,
    pub volume_sma_20: Option<f64>,
}

impl TechnicalIndicators {
    pub fn compute(bars: &[OHLCVBar]) -> Self {
        let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
        let highs: Vec<f64> = bars.iter().map(|b| b.high).collect();
        let lows: Vec<f64> = bars.iter().map(|b| b.low).collect();
        let volumes: Vec<f64> = bars.iter().map(|b| b.volume).collect();
        let n = closes.len();

        Self {
            sma_20: if n >= 20 {
                Some(sma(&closes, 20))
            } else {
                None
            },
            sma_50: if n >= 50 {
                Some(sma(&closes, 50))
            } else {
                None
            },
            sma_200: if n >= 200 {
                Some(sma(&closes, 200))
            } else {
                None
            },
            ema_12: if n >= 12 {
                Some(ema(&closes, 12))
            } else {
                None
            },
            ema_26: if n >= 26 {
                Some(ema(&closes, 26))
            } else {
                None
            },
            rsi_14: if n >= 15 {
                Some(rsi(&closes, 14))
            } else {
                None
            },
            macd_line: if n >= 26 {
                Some(macd_line(&closes))
            } else {
                None
            },
            macd_signal: if n >= 26 {
                Some(macd_signal(&closes))
            } else {
                None
            },
            macd_histogram: if n >= 26 {
                Some(macd_line(&closes) - macd_signal(&closes))
            } else {
                None
            },
            bollinger_upper: if n >= 20 {
                Some(bollinger_bands(&closes, 20, 2.0).0)
            } else {
                None
            },
            bollinger_middle: if n >= 20 {
                Some(sma(&closes, 20))
            } else {
                None
            },
            bollinger_lower: if n >= 20 {
                Some(bollinger_bands(&closes, 20, 2.0).1)
            } else {
                None
            },
            atr_14: if n >= 15 {
                Some(atr(&highs, &lows, &closes, 14))
            } else {
                None
            },
            volume_sma_20: if n >= 20 {
                Some(sma(&volumes, 20))
            } else {
                None
            },
        }
    }

    pub fn score(&self) -> f64 {
        let mut score = 0.0;
        let mut count = 0u32;

        if let (Some(sma20), Some(sma50)) = (self.sma_20, self.sma_50) {
            if sma20 > sma50 {
                score += 1.0;
            } else {
                score -= 0.5;
            }
            count += 1;
        }
        if let (Some(sma50), Some(sma200)) = (self.sma_50, self.sma_200) {
            if sma50 > sma200 {
                score += 1.0;
            } else {
                score -= 0.5;
            }
            count += 1;
        }
        if let (Some(ema12), Some(ema26)) = (self.ema_12, self.ema_26) {
            if ema12 > ema26 {
                score += 1.0;
            } else {
                score -= 0.5;
            }
            count += 1;
        }
        if let Some(rsi) = self.rsi_14 {
            if rsi < 30.0 {
                score += 1.5;
            } else if rsi > 70.0 {
                score -= 1.0;
            } else if rsi > 50.0 {
                score += 0.3;
            } else {
                score -= 0.2;
            }
            count += 1;
        }
        if let (Some(macd), Some(signal)) = (self.macd_line, self.macd_signal) {
            if macd > signal {
                score += 1.0;
            } else {
                score -= 0.5;
            }
            count += 1;
        }
        if count == 0 {
            return 0.5;
        }
        let normalized = 0.5 + (score / count as f64) * 0.3;
        normalized.clamp(0.0, 1.0)
    }
}

fn sma(data: &[f64], period: usize) -> f64 {
    let len = data.len();
    if len < period {
        return 0.0;
    }
    data[len - period..].iter().sum::<f64>() / period as f64
}

fn ema(data: &[f64], period: usize) -> f64 {
    let len = data.len();
    if len < period {
        return data.last().copied().unwrap_or(0.0);
    }
    let k = 2.0 / (period as f64 + 1.0);
    let mut val = data[len - period..].iter().sum::<f64>() / period as f64;
    for i in (len - period + 1)..len {
        val = data[i] * k + val * (1.0 - k);
    }
    val
}

fn rsi(data: &[f64], period: usize) -> f64 {
    let len = data.len();
    if len < period + 1 {
        return 50.0;
    }
    let mut gains = 0.0;
    let mut losses = 0.0;
    for i in (len - period - 1)..(len - 1) {
        let diff = data[i + 1] - data[i];
        if diff > 0.0 {
            gains += diff;
        } else {
            losses -= diff;
        }
    }
    let period_f = period as f64;
    let avg_gain = gains / period_f;
    let avg_loss = losses / period_f;
    if avg_loss == 0.0 {
        return 100.0;
    }
    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

fn macd_line(data: &[f64]) -> f64 {
    ema(data, 12) - ema(data, 26)
}

fn macd_signal(data: &[f64]) -> f64 {
    let len = data.len();
    let macd_vals: Vec<f64> = (9..len)
        .map(|i| ema(&data[..=i], 12) - ema(&data[..=i], 26))
        .collect();
    ema(&macd_vals, 9)
}

fn bollinger_bands(data: &[f64], period: usize, stddev: f64) -> (f64, f64) {
    let mean = sma(data, period);
    let len = data.len();
    let variance: f64 = data[len - period..]
        .iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>()
        / period as f64;
    let std = variance.sqrt();
    (mean + stddev * std, mean - stddev * std)
}

fn atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> f64 {
    let len = highs.len();
    if len < period + 1 {
        return 0.0;
    }
    let tr_values: Vec<f64> = (len - period..len)
        .map(|i| {
            let hl = highs[i] - lows[i];
            let hc = (highs[i] - closes[i - 1]).abs();
            let lc = (lows[i] - closes[i - 1]).abs();
            hl.max(hc).max(lc)
        })
        .collect();
    sma(&tr_values, period)
}

#[derive(Debug, Clone)]
pub struct SentimentSignal {
    pub bullish_score: f64,
    pub bearish_score: f64,
    pub volume_anomaly: f64,
    pub news_sentiment: f64,
    pub social_sentiment: f64,
}

impl SentimentSignal {
    pub fn score(&self) -> f64 {
        let raw = self.bullish_score - self.bearish_score
            + self.news_sentiment * 0.3
            + self.social_sentiment * 0.2
            + self.volume_anomaly * 0.1;
        (raw + 1.0) / 2.0
    }
}

#[derive(Debug, Clone)]
pub struct OnchainSignal {
    pub exchange_flows: f64,
    pub whale_activity: f64,
    pub network_growth: f64,
    pub staking_yield: f64,
}

impl OnchainSignal {
    pub fn score(&self) -> f64 {
        let raw = self.exchange_flows * 0.4
            + self.whale_activity * 0.3
            + self.network_growth * 0.2
            + self.staking_yield * 0.1;
        (raw + 1.0) / 2.0
    }
}

#[derive(Debug, Clone)]
pub struct FusedMarketSignal {
    pub symbol: String,
    pub asset_class: AssetClass,
    pub technical_score: f64,
    pub sentiment_score: f64,
    pub truth_validated_score: f64,
    pub onchain_score: f64,
    pub fused: f64,
    pub confidence: f64,
    pub divergence_warning: bool,
}

impl FusedMarketSignal {
    pub fn new(symbol: &str, asset_class: AssetClass) -> Self {
        Self {
            symbol: symbol.to_string(),
            asset_class,
            technical_score: 0.5,
            sentiment_score: 0.5,
            truth_validated_score: 0.5,
            onchain_score: 0.5,
            fused: 0.5,
            confidence: 0.0,
            divergence_warning: false,
        }
    }

    pub fn with_technical(mut self, tech: &TechnicalIndicators) -> Self {
        self.technical_score = tech.score();
        self
    }

    pub fn with_sentiment(mut self, sent: &SentimentSignal) -> Self {
        self.sentiment_score = sent.score();
        self
    }

    pub fn with_truth_validation(mut self, validated: f64) -> Self {
        self.truth_validated_score = validated.clamp(0.0, 1.0);
        self
    }

    pub fn with_onchain(mut self, onchain: &OnchainSignal) -> Self {
        self.onchain_score = onchain.score();
        self
    }

    pub fn fuse(&mut self) {
        let scores = [
            self.technical_score,
            self.sentiment_score,
            self.truth_validated_score,
            self.onchain_score,
        ];
        let weights = [
            TECHNICAL_WEIGHT,
            SENTIMENT_WEIGHT,
            TRUTH_WEIGHT,
            ONCHAIN_WEIGHT,
        ];
        self.fused = scores.iter().zip(weights.iter()).map(|(s, w)| s * w).sum();

        let divergence =
            scores.iter().map(|s| (s - self.fused).abs()).sum::<f64>() / scores.len() as f64;
        self.divergence_warning = divergence > 0.25;

        let agreement = 1.0 - divergence;
        self.confidence = self.fused * agreement;
    }

    pub fn to_signal(&self) -> TradingSignal {
        let side = if self.fused > 0.55 {
            TradeSide::Long
        } else if self.fused < 0.45 {
            TradeSide::Short
        } else {
            TradeSide::Neutral
        };

        let mut signal = TradingSignal::new(&self.symbol, side, self.confidence, 0.0)
            .with_source("Technical", TECHNICAL_WEIGHT, self.technical_score)
            .with_source("Sentiment", SENTIMENT_WEIGHT, self.sentiment_score)
            .with_source("TruthValidation", TRUTH_WEIGHT, self.truth_validated_score)
            .with_source("OnChain", ONCHAIN_WEIGHT, self.onchain_score);

        if self.divergence_warning {
            let msg = format!(
                "divergence detected: tech={:.2} sent={:.2} truth={:.2} onchain={:.2}",
                self.technical_score,
                self.sentiment_score,
                self.truth_validated_score,
                self.onchain_score
            );
            signal = signal.with_reasoning(&msg);
        }

        signal
    }
}

#[derive(Debug, Clone)]
pub struct MarketRegime {
    pub trend: TrendDirection,
    pub volatility: VolatilityLevel,
    pub strength: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Sideways,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VolatilityLevel {
    Low,
    Normal,
    High,
    Extreme,
}

impl MarketRegime {
    pub fn detect(bars: &[OHLCVBar]) -> Self {
        let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
        let n = closes.len();
        if n < 50 {
            return Self {
                trend: TrendDirection::Sideways,
                volatility: VolatilityLevel::Normal,
                strength: 0.5,
            };
        }

        let sma50 = sma(&closes, 50);
        let sma200 = if n >= 200 { sma(&closes, 200) } else { sma50 };
        let current = closes[n - 1];
        let sma20 = sma(&closes, 20);

        let trend = if current > sma50 && sma50 > sma200 {
            TrendDirection::Bullish
        } else if current < sma50 && sma50 < sma200 {
            TrendDirection::Bearish
        } else {
            TrendDirection::Sideways
        };

        let highs: Vec<f64> = bars.iter().map(|b| b.high).collect();
        let lows: Vec<f64> = bars.iter().map(|b| b.low).collect();
        let atr_val = if n >= 15 {
            atr(&highs, &lows, &closes, 14)
        } else {
            0.0
        };
        let avg_price = sma(&closes, 20);

        let vol_ratio = if avg_price > 0.0 {
            atr_val / avg_price
        } else {
            0.0
        };
        let volatility = if vol_ratio > 0.05 {
            VolatilityLevel::Extreme
        } else if vol_ratio > 0.03 {
            VolatilityLevel::High
        } else if vol_ratio > 0.01 {
            VolatilityLevel::Normal
        } else {
            VolatilityLevel::Low
        };

        let slope = if n >= 50 {
            // Rate of change of SMA20 relative to SMA50 (trend momentum)
            (sma20 - sma50) / sma50.abs().max(0.001)
        } else if n >= 20 {
            (sma20 - closes[0]) / closes[0].abs().max(0.001) / n as f64
        } else {
            0.0
        };
        let strength =
            (slope.abs() * 5.0 + (1.0 - vol_ratio * 10.0).clamp(0.0, 1.0) * 0.5).clamp(0.0, 1.0);

        Self {
            trend,
            volatility,
            strength,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignalGenerator {
    pub bars: VecDeque<OHLCVBar>,
    pub max_bars: usize,
}

impl SignalGenerator {
    pub fn new(max_bars: usize) -> Self {
        Self {
            bars: VecDeque::with_capacity(max_bars),
            max_bars,
        }
    }

    pub fn ingest(&mut self, bar: OHLCVBar) {
        while self.bars.len() >= self.max_bars {
            self.bars.pop_front();
        }
        self.bars.push_back(bar);
    }

    pub fn ingest_slice(&mut self, new_bars: &[OHLCVBar]) {
        for bar in new_bars {
            self.ingest(bar.clone());
        }
    }

    pub fn bars_slice(&self) -> Vec<OHLCVBar> {
        self.bars.iter().cloned().collect()
    }

    pub fn generate_signal(
        &self,
        sentiment: Option<&SentimentSignal>,
        truth_score: Option<f64>,
        onchain: Option<&OnchainSignal>,
    ) -> Option<TradingSignal> {
        let bars = self.bars_slice();
        if bars.len() < 20 {
            return None;
        }

        let tech = TechnicalIndicators::compute(&bars);
        let mut fused = FusedMarketSignal::new(
            &bars[bars.len() - 1].symbol,
            bars[bars.len() - 1].asset_class,
        );
        fused = fused.with_technical(&tech);

        if let Some(sent) = sentiment {
            fused = fused.with_sentiment(sent);
        }
        if let Some(validated) = truth_score {
            fused = fused.with_truth_validation(validated);
        }
        if let Some(oc) = onchain {
            fused = fused.with_onchain(oc);
        }

        fused.fuse();
        Some(fused.to_signal())
    }

    pub fn generate_fused(
        &self,
        sentiment: Option<&SentimentSignal>,
        truth_score: Option<f64>,
        onchain: Option<&OnchainSignal>,
    ) -> Option<SignalFusion> {
        let signal = self.generate_signal(sentiment, truth_score, onchain)?;
        let bars = self.bars_slice();
        if bars.is_empty() {
            return None;
        }
        Some(SignalFusion::fuse(
            &bars[bars.len() - 1].symbol,
            vec![signal],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_trading::types::{AssetClass, OHLCVBar, Timeframe, TradeSide};

    fn make_bars(closes: &[f64]) -> Vec<OHLCVBar> {
        closes
            .iter()
            .enumerate()
            .map(|(i, &c)| OHLCVBar {
                symbol: "BTC/USD".into(),
                asset_class: AssetClass::Crypto,
                timeframe: Timeframe::Hour1,
                open: c - 0.5,
                high: c + 1.0,
                low: c - 1.0,
                close: c,
                volume: 1000.0,
                timestamp: i as u64 * 3600,
            })
            .collect()
    }

    fn make_bullish_bars(count: usize) -> Vec<OHLCVBar> {
        (0..count)
            .map(|i| {
                let base = 100.0 + (i as f64) * 0.5;
                OHLCVBar {
                    symbol: "BTC/USD".into(),
                    asset_class: AssetClass::Crypto,
                    timeframe: Timeframe::Hour1,
                    open: base - 0.2,
                    high: base + 0.5,
                    low: base - 0.5,
                    close: base,
                    volume: 1000.0,
                    timestamp: i as u64 * 3600,
                }
            })
            .collect()
    }

    fn make_bearish_bars(count: usize) -> Vec<OHLCVBar> {
        (0..count)
            .map(|i| {
                let base = 200.0 - (i as f64) * 0.5;
                OHLCVBar {
                    symbol: "BTC/USD".into(),
                    asset_class: AssetClass::Crypto,
                    timeframe: Timeframe::Hour1,
                    open: base + 0.2,
                    high: base + 0.5,
                    low: base - 0.5,
                    close: base,
                    volume: 1000.0,
                    timestamp: i as u64 * 3600,
                }
            })
            .collect()
    }

    fn make_sideways_bars(count: usize) -> Vec<OHLCVBar> {
        (0..count)
            .map(|i| {
                let osc = (i as f64 * 0.3).sin() * 5.0;
                let base = 100.0 + osc;
                OHLCVBar {
                    symbol: "BTC/USD".into(),
                    asset_class: AssetClass::Crypto,
                    timeframe: Timeframe::Hour1,
                    open: base - 0.2,
                    high: base + 0.5,
                    low: base - 0.5,
                    close: base,
                    volume: 1000.0,
                    timestamp: i as u64 * 3600,
                }
            })
            .collect()
    }

    // --- TechnicalIndicators tests ---

    #[test]
    fn test_technical_indicators_compute_returns_all_fields() {
        let bars = make_bullish_bars(210);
        let ti = TechnicalIndicators::compute(&bars);
        assert!(ti.sma_20.is_some(), "sma_20 should be Some with 210 bars");
        assert!(ti.sma_50.is_some(), "sma_50 should be Some with 210 bars");
        assert!(ti.sma_200.is_some(), "sma_200 should be Some with 210 bars");
        assert!(ti.ema_12.is_some());
        assert!(ti.ema_26.is_some());
        assert!(ti.rsi_14.is_some());
        assert!(ti.macd_line.is_some());
        assert!(ti.macd_signal.is_some());
        assert!(ti.bollinger_upper.is_some());
        assert!(ti.bollinger_middle.is_some());
        assert!(ti.bollinger_lower.is_some());
        assert!(ti.atr_14.is_some());
        assert!(ti.volume_sma_20.is_some());
    }

    #[test]
    fn test_technical_indicators_compute_few_bars_returns_none() {
        let bars = make_bullish_bars(10);
        let ti = TechnicalIndicators::compute(&bars);
        assert!(ti.sma_20.is_none(), "sma_20 should be None with 10 bars");
        assert!(ti.sma_50.is_none());
        assert!(ti.sma_200.is_none());
    }

    #[test]
    fn test_technical_indicators_score_bullish_range() {
        let bars = make_bullish_bars(210);
        let ti = TechnicalIndicators::compute(&bars);
        let score = ti.score();
        assert!(
            score > 0.5,
            "bullish bars should score > 0.5, got {}",
            score
        );
        assert!(score <= 1.0);
    }

    #[test]
    fn test_technical_indicators_score_bearish_range() {
        let bars = make_bearish_bars(210);
        let ti = TechnicalIndicators::compute(&bars);
        let score = ti.score();
        assert!(
            score < 0.5,
            "bearish bars should score < 0.5, got {}",
            score
        );
        assert!(score >= 0.0);
    }

    #[test]
    fn test_technical_indicators_score_too_few_bars_default() {
        let bars = make_bullish_bars(5);
        let ti = TechnicalIndicators::compute(&bars);
        let score = ti.score();
        assert!(
            (score - 0.5).abs() < 1e-6,
            "few bars should default to 0.5, got {}",
            score
        );
    }

    #[test]
    fn test_technical_indicators_sma_values() {
        let closes: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let bars = make_bars(&closes);
        let ti = TechnicalIndicators::compute(&bars);
        let sma20 = ti.sma_20.unwrap();
        // Last 20 closes: indices 10..29
        let expected: f64 = (10..30).map(|i| 100.0 + i as f64).sum::<f64>() / 20.0;
        assert!(
            (sma20 - expected).abs() < 1e-6,
            "sma20={} expected={}",
            sma20,
            expected
        );
    }

    // --- MarketRegime tests ---

    #[test]
    fn test_market_regime_bullish() {
        let bars = make_bullish_bars(210);
        let regime = MarketRegime::detect(&bars);
        assert_eq!(regime.trend, TrendDirection::Bullish);
    }

    #[test]
    fn test_market_regime_bearish() {
        let bars = make_bearish_bars(210);
        let regime = MarketRegime::detect(&bars);
        assert_eq!(regime.trend, TrendDirection::Bearish);
    }

    #[test]
    fn test_market_regime_sideways() {
        let bars = make_sideways_bars(210);
        let regime = MarketRegime::detect(&bars);
        assert_eq!(regime.trend, TrendDirection::Sideways);
    }

    #[test]
    fn test_market_regime_too_few_bars_defaults_sideways() {
        let bars = make_bullish_bars(30);
        let regime = MarketRegime::detect(&bars);
        assert_eq!(regime.trend, TrendDirection::Sideways);
    }

    #[test]
    fn test_market_regime_volatility_low() {
        // Very tight range — low volatility
        let bars: Vec<OHLCVBar> = (0..210)
            .map(|i| OHLCVBar {
                symbol: "BTC/USD".into(),
                asset_class: AssetClass::Crypto,
                timeframe: Timeframe::Hour1,
                open: 100.0,
                high: 100.1,
                low: 99.9,
                close: 100.0,
                volume: 1000.0,
                timestamp: i as u64 * 3600,
            })
            .collect();
        let regime = MarketRegime::detect(&bars);
        assert_eq!(regime.volatility, VolatilityLevel::Low);
    }

    #[test]
    fn test_market_regime_volatility_extreme() {
        // Wide range — extreme volatility
        let bars: Vec<OHLCVBar> = (0..210)
            .map(|i| {
                let base = 100.0 + (i as f64 % 10.0 - 5.0) * 5.0;
                OHLCVBar {
                    symbol: "BTC/USD".into(),
                    asset_class: AssetClass::Crypto,
                    timeframe: Timeframe::Hour1,
                    open: base,
                    high: base + 10.0,
                    low: base - 10.0,
                    close: base + 1.0,
                    volume: 1000.0,
                    timestamp: i as u64 * 3600,
                }
            })
            .collect();
        let regime = MarketRegime::detect(&bars);
        assert_eq!(regime.volatility, VolatilityLevel::Extreme);
    }

    // --- SignalGenerator tests ---

    #[test]
    fn test_signal_generator_ingest_and_generate() {
        let mut gen = SignalGenerator::new(500);
        let bars = make_bullish_bars(100);
        gen.ingest_slice(&bars);
        assert_eq!(gen.bars.len(), 100);
        let signal = gen.generate_signal(None, None, None);
        assert!(signal.is_some(), "should generate signal with 100 bars");
        let sig = signal.unwrap();
        assert_eq!(sig.symbol, "BTC/USD");
    }

    #[test]
    fn test_signal_generator_too_few_bars_returns_none() {
        let mut gen = SignalGenerator::new(500);
        let bars = make_bullish_bars(10);
        gen.ingest_slice(&bars);
        let signal = gen.generate_signal(None, None, None);
        assert!(signal.is_none(), "should return None with < 20 bars");
    }

    #[test]
    fn test_signal_generator_max_bars_respected() {
        let mut gen = SignalGenerator::new(50);
        let bars = make_bullish_bars(100);
        gen.ingest_slice(&bars);
        assert_eq!(gen.bars.len(), 50);
    }

    // --- FusedMarketSignal tests ---

    #[test]
    fn test_fused_market_signal_fuse_no_divergence() {
        let bars = make_bullish_bars(210);
        let tech = TechnicalIndicators::compute(&bars);
        let mut fused = FusedMarketSignal::new("BTC/USD", AssetClass::Crypto);
        fused = fused.with_technical(&tech);
        fused.fuse();
        assert!(fused.fused >= 0.0 && fused.fused <= 1.0);
        assert!(fused.confidence >= 0.0 && fused.confidence <= 1.0);
    }

    #[test]
    fn test_fused_market_signal_to_signal_long() {
        let mut fused = FusedMarketSignal::new("BTC/USD", AssetClass::Crypto);
        fused.technical_score = 0.9;
        fused.sentiment_score = 0.8;
        fused.truth_validated_score = 0.7;
        fused.onchain_score = 0.6;
        fused.fuse();
        let signal = fused.to_signal();
        assert_eq!(signal.side, TradeSide::Long);
    }

    #[test]
    fn test_fused_market_signal_to_signal_short() {
        let mut fused = FusedMarketSignal::new("BTC/USD", AssetClass::Crypto);
        fused.technical_score = 0.2;
        fused.sentiment_score = 0.3;
        fused.truth_validated_score = 0.4;
        fused.onchain_score = 0.1;
        fused.fuse();
        let signal = fused.to_signal();
        assert_eq!(signal.side, TradeSide::Short);
    }

    #[test]
    fn test_fused_market_signal_to_signal_neutral() {
        let mut fused = FusedMarketSignal::new("BTC/USD", AssetClass::Crypto);
        fused.technical_score = 0.5;
        fused.sentiment_score = 0.5;
        fused.truth_validated_score = 0.5;
        fused.onchain_score = 0.5;
        fused.fuse();
        let signal = fused.to_signal();
        assert_eq!(signal.side, TradeSide::Neutral);
    }

    #[test]
    fn test_fused_market_signal_divergence_warning() {
        let mut fused = FusedMarketSignal::new("BTC/USD", AssetClass::Crypto);
        fused.technical_score = 0.9;
        fused.sentiment_score = 0.1;
        fused.truth_validated_score = 0.9;
        fused.onchain_score = 0.1;
        fused.fuse();
        assert!(fused.divergence_warning);
    }

    // --- SentimentSignal & OnchainSignal tests ---

    #[test]
    fn test_sentiment_signal_score() {
        let s = SentimentSignal {
            bullish_score: 0.7,
            bearish_score: 0.2,
            volume_anomaly: 0.3,
            news_sentiment: 0.6,
            social_sentiment: 0.5,
        };
        let score = s.score();
        assert!(score >= 0.0 && score <= 1.0, "score={} out of range", score);
    }

    #[test]
    fn test_onchain_signal_score() {
        let s = OnchainSignal {
            exchange_flows: 0.2,
            whale_activity: 0.6,
            network_growth: 0.7,
            staking_yield: 0.3,
        };
        let score = s.score();
        assert!(score >= 0.0 && score <= 1.0, "score={} out of range", score);
    }
}
