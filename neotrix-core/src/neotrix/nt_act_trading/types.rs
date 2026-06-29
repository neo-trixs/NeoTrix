use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TradeSide {
    Long,
    Short,
    Neutral,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderType {
    Market,
    Limit { price: f64 },
    Stop { stop_price: f64 },
    StopLimit { stop_price: f64, limit_price: f64 },
    TrailingStop { trail_percent: f64 },
}

impl PartialEq for OrderType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (OrderType::Market, OrderType::Market) => true,
            (OrderType::Limit { price: a }, OrderType::Limit { price: b }) => {
                a.to_bits() == b.to_bits()
            }
            (OrderType::Stop { stop_price: a }, OrderType::Stop { stop_price: b }) => {
                a.to_bits() == b.to_bits()
            }
            (
                OrderType::StopLimit {
                    stop_price: a,
                    limit_price: c,
                },
                OrderType::StopLimit {
                    stop_price: b,
                    limit_price: d,
                },
            ) => a.to_bits() == b.to_bits() && c.to_bits() == d.to_bits(),
            (
                OrderType::TrailingStop { trail_percent: a },
                OrderType::TrailingStop { trail_percent: b },
            ) => a.to_bits() == b.to_bits(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timeframe {
    Tick,
    Minute1,
    Minute5,
    Minute15,
    Minute30,
    Hour1,
    Hour4,
    Day1,
    Week1,
    Month1,
}

impl Timeframe {
    pub fn seconds(&self) -> u64 {
        match self {
            Timeframe::Tick => 0,
            Timeframe::Minute1 => 60,
            Timeframe::Minute5 => 300,
            Timeframe::Minute15 => 900,
            Timeframe::Minute30 => 1800,
            Timeframe::Hour1 => 3600,
            Timeframe::Hour4 => 14400,
            Timeframe::Day1 => 86400,
            Timeframe::Week1 => 604800,
            Timeframe::Month1 => 2592000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetClass {
    Stock,
    Crypto,
    Forex,
    Commodity,
    Bond,
    Derivative,
    Index,
}

#[derive(Debug, Clone)]
pub struct OHLCVBar {
    pub symbol: String,
    pub asset_class: AssetClass,
    pub timeframe: Timeframe,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct Ticker {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume_24h: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub side: TradeSide,
    pub size: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub timestamp: u64,
}

impl Position {
    pub fn markup_pnl(&mut self, current: f64) {
        self.current_price = current;
        self.unrealized_pnl = match self.side {
            TradeSide::Long => (current - self.entry_price) * self.size,
            TradeSide::Short => (self.entry_price - current) * self.size,
            TradeSide::Neutral => 0.0,
        };
    }

    pub fn pnl_percent(&self) -> f64 {
        if self.entry_price == 0.0 {
            return 0.0;
        }
        match self.side {
            TradeSide::Long => (self.current_price - self.entry_price) / self.entry_price,
            TradeSide::Short => (self.entry_price - self.current_price) / self.entry_price,
            TradeSide::Neutral => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignalSource {
    pub name: String,
    pub weight: f64,
    pub contribution: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TradingSignal {
    pub symbol: String,
    pub side: TradeSide,
    pub confidence: f64,
    pub price: f64,
    pub timeframe: Timeframe,
    pub sources: Vec<SignalSource>,
    pub reasoning: String,
    pub timestamp: u64,
    pub expires_at: u64,
}

impl TradingSignal {
    pub fn new(symbol: &str, side: TradeSide, confidence: f64, price: f64) -> Self {
        let now = unix_now();
        Self {
            symbol: symbol.to_string(),
            side,
            confidence: confidence.clamp(0.0, 1.0),
            price,
            timeframe: Timeframe::Hour1,
            sources: Vec::new(),
            reasoning: String::new(),
            timestamp: now,
            expires_at: now + 3600,
        }
    }

    pub fn with_source(mut self, name: &str, weight: f64, contribution: f64) -> Self {
        self.sources.push(SignalSource {
            name: name.to_string(),
            weight,
            contribution,
            metadata: HashMap::new(),
        });
        self
    }

    pub fn with_timeframe(mut self, tf: Timeframe) -> Self {
        let now = self.timestamp;
        self.timeframe = tf;
        self.expires_at = now + tf.seconds();
        self
    }

    pub fn with_reasoning(mut self, r: &str) -> Self {
        self.reasoning = r.to_string();
        self
    }

    pub fn weighted_confidence(&self) -> f64 {
        if self.sources.is_empty() {
            return self.confidence;
        }
        let total_weight: f64 = self.sources.iter().map(|s| s.weight).sum();
        if total_weight == 0.0 {
            return self.confidence;
        }
        let weighted: f64 = self.sources.iter().map(|s| s.contribution * s.weight).sum();
        weighted / total_weight
    }

    pub fn is_expired(&self) -> bool {
        unix_now() > self.expires_at
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone)]
pub struct SignalFusion {
    pub symbol: String,
    pub signals: Vec<TradingSignal>,
    pub fused_side: TradeSide,
    pub fused_confidence: f64,
    pub consensus_ratio: f64,
    pub disagreement: f64,
}

impl SignalFusion {
    pub fn fuse(symbol: &str, signals: Vec<TradingSignal>) -> Self {
        let non_expired: Vec<_> = signals.into_iter().filter(|s| !s.is_expired()).collect();
        let total = non_expired.len() as f64;
        if total == 0.0 {
            return Self {
                symbol: symbol.to_string(),
                signals: non_expired,
                fused_side: TradeSide::Neutral,
                fused_confidence: 0.0,
                consensus_ratio: 0.0,
                disagreement: 0.0,
            };
        }

        let longs = non_expired
            .iter()
            .filter(|s| s.side == TradeSide::Long)
            .count() as f64;
        let shorts = non_expired
            .iter()
            .filter(|s| s.side == TradeSide::Short)
            .count() as f64;

        let side = if longs > shorts {
            TradeSide::Long
        } else if shorts > longs {
            TradeSide::Short
        } else {
            let long_conf: f64 = non_expired
                .iter()
                .filter(|s| s.side == TradeSide::Long)
                .map(|s| s.weighted_confidence())
                .sum();
            let short_conf: f64 = non_expired
                .iter()
                .filter(|s| s.side == TradeSide::Short)
                .map(|s| s.weighted_confidence())
                .sum();
            if long_conf >= short_conf {
                TradeSide::Long
            } else {
                TradeSide::Short
            }
        };

        let side_signals: Vec<_> = non_expired.iter().filter(|s| s.side == side).collect();
        let fused_conf = if side_signals.is_empty() {
            0.0
        } else {
            side_signals
                .iter()
                .map(|s| s.weighted_confidence())
                .sum::<f64>()
                / side_signals.len() as f64
        };

        let max_possible = (longs.max(shorts) + 0.5 * (total - longs.max(shorts))) / total;
        let consensus_ratio = max_possible;
        let disagreement = 1.0 - (longs - shorts).abs() / total;

        Self {
            symbol: symbol.to_string(),
            signals: non_expired,
            fused_side: side,
            fused_confidence: fused_conf,
            consensus_ratio,
            disagreement,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PortfolioSummary {
    pub total_value: f64,
    pub cash: f64,
    pub positions: Vec<Position>,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub total_pnl: f64,
    pub total_pnl_percent: f64,
    pub diversification_score: f64,
    pub timestamp: u64,
}

impl PortfolioSummary {
    pub fn new(cash: f64) -> Self {
        Self {
            total_value: cash,
            cash,
            positions: Vec::new(),
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            total_pnl: 0.0,
            total_pnl_percent: 0.0,
            diversification_score: 1.0,
            timestamp: unix_now(),
        }
    }

    pub fn from_positions(cash: f64, positions: Vec<Position>) -> Self {
        let unrealized: f64 = positions.iter().map(|p| p.unrealized_pnl).sum();
        let realized: f64 = positions.iter().map(|p| p.realized_pnl).sum();
        let position_value: f64 = positions.iter().map(|p| p.current_price * p.size).sum();
        let total = cash + position_value;
        let total_cost: f64 = positions.iter().map(|p| p.entry_price * p.size).sum();
        let pnl_pct = if total_cost > 0.0 {
            (total - cash - total_cost) / total_cost
        } else {
            0.0
        };
        let n = positions.len();
        let divers = if n <= 1 {
            1.0
        } else {
            let max_alloc = positions
                .iter()
                .map(|p| (p.current_price * p.size) / total.max(1.0))
                .fold(0.0f64, f64::max);
            1.0 - max_alloc
        };

        Self {
            total_value: total,
            cash,
            positions,
            unrealized_pnl: unrealized,
            realized_pnl: realized,
            total_pnl: unrealized + realized,
            total_pnl_percent: pnl_pct,
            diversification_score: divers,
            timestamp: unix_now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signal(symbol: &str, side: TradeSide, confidence: f64) -> TradingSignal {
        TradingSignal::new(symbol, side, confidence, 100.0)
    }

    #[test]
    fn test_market_signal_creation_long() {
        let s = make_signal("BTC/USD", TradeSide::Long, 0.8);
        assert_eq!(s.symbol, "BTC/USD");
        assert_eq!(s.side, TradeSide::Long);
        assert!((s.confidence - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_market_signal_creation_short() {
        let s = make_signal("ETH/USD", TradeSide::Short, 0.65);
        assert_eq!(s.side, TradeSide::Short);
        assert!((s.confidence - 0.65).abs() < 1e-6);
    }

    #[test]
    fn test_signal_confidence_clamped_above() {
        let s = make_signal("XRP/USD", TradeSide::Long, 1.5);
        assert!((s.confidence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_signal_confidence_clamped_below() {
        let s = make_signal("XRP/USD", TradeSide::Short, -0.5);
        assert!((s.confidence - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_signal_fusion_majority_long_wins() {
        let signals = vec![
            make_signal("BTC/USD", TradeSide::Long, 0.7),
            make_signal("BTC/USD", TradeSide::Long, 0.6),
            make_signal("BTC/USD", TradeSide::Short, 0.8),
        ];
        let fused = SignalFusion::fuse("BTC/USD", signals);
        assert_eq!(fused.fused_side, TradeSide::Long);
        assert!(fused.consensus_ratio > 0.5);
    }

    #[test]
    fn test_signal_fusion_majority_short_wins() {
        let signals = vec![
            make_signal("BTC/USD", TradeSide::Short, 0.75),
            make_signal("BTC/USD", TradeSide::Short, 0.6),
            make_signal("BTC/USD", TradeSide::Long, 0.7),
        ];
        let fused = SignalFusion::fuse("BTC/USD", signals);
        assert_eq!(fused.fused_side, TradeSide::Short);
    }

    #[test]
    fn test_signal_fusion_tie_higher_confidence_wins() {
        let signals = vec![
            make_signal("BTC/USD", TradeSide::Long, 0.9),
            make_signal("BTC/USD", TradeSide::Short, 0.3),
        ];
        let fused = SignalFusion::fuse("BTC/USD", signals);
        assert_eq!(fused.fused_side, TradeSide::Long);
    }

    #[test]
    fn test_signal_fusion_tie_lower_confidence_loses() {
        let signals = vec![
            make_signal("BTC/USD", TradeSide::Long, 0.2),
            make_signal("BTC/USD", TradeSide::Short, 0.85),
        ];
        let fused = SignalFusion::fuse("BTC/USD", signals);
        assert_eq!(fused.fused_side, TradeSide::Short);
    }

    #[test]
    fn test_signal_fusion_disagreement_drops_consensus() {
        let signals = vec![
            make_signal("BTC/USD", TradeSide::Long, 0.9),
            make_signal("BTC/USD", TradeSide::Short, 0.9),
        ];
        let fused = SignalFusion::fuse("BTC/USD", signals);
        assert!(
            fused.disagreement > 0.4,
            "disagreement={} should be high",
            fused.disagreement
        );
        assert!(fused.consensus_ratio < 0.75);
    }

    #[test]
    fn test_signal_fusion_all_agree_high_consensus() {
        let signals = vec![
            make_signal("BTC/USD", TradeSide::Long, 0.8),
            make_signal("BTC/USD", TradeSide::Long, 0.7),
            make_signal("BTC/USD", TradeSide::Long, 0.9),
        ];
        let fused = SignalFusion::fuse("BTC/USD", signals);
        assert_eq!(fused.fused_side, TradeSide::Long);
        assert!(fused.consensus_ratio > 0.8);
        assert!(fused.disagreement < 0.1);
    }

    #[test]
    fn test_signal_fusion_empty_returns_neutral() {
        let fused = SignalFusion::fuse("BTC/USD", vec![]);
        assert_eq!(fused.fused_side, TradeSide::Neutral);
        assert!((fused.fused_confidence - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_trading_signal_weighted_confidence_equal_weights() {
        let s = TradingSignal::new("BTC/USD", TradeSide::Long, 0.5, 100.0)
            .with_source("src1", 1.0, 0.8)
            .with_source("src2", 1.0, 0.6);
        let wc = s.weighted_confidence();
        assert!((wc - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_trading_signal_weighted_confidence_unequal_weights() {
        let s = TradingSignal::new("BTC/USD", TradeSide::Long, 0.5, 100.0)
            .with_source("heavy", 3.0, 0.9)
            .with_source("light", 1.0, 0.5);
        let wc = s.weighted_confidence();
        assert!((wc - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_trading_signal_weighted_confidence_no_sources() {
        let s = TradingSignal::new("BTC/USD", TradeSide::Long, 0.7, 100.0);
        assert!((s.weighted_confidence() - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_trading_signal_is_expired() {
        let mut s = TradingSignal::new("BTC/USD", TradeSide::Long, 0.5, 100.0);
        s.expires_at = 0;
        assert!(s.is_expired());
    }

    #[test]
    fn test_trading_signal_not_expired() {
        let s = TradingSignal::new("BTC/USD", TradeSide::Long, 0.5, 100.0);
        assert!(!s.is_expired());
    }

    #[test]
    fn test_trading_signal_with_timeframe() {
        let s = TradingSignal::new("BTC/USD", TradeSide::Long, 0.5, 100.0)
            .with_timeframe(Timeframe::Day1);
        assert_eq!(s.timeframe, Timeframe::Day1);
        assert_eq!(s.expires_at - s.timestamp, Timeframe::Day1.seconds());
    }

    #[test]
    fn test_position_markup_pnl_long() {
        let mut p = Position {
            symbol: "BTC/USD".into(),
            side: TradeSide::Long,
            size: 2.0,
            entry_price: 100.0,
            current_price: 100.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            timestamp: 0,
        };
        p.markup_pnl(110.0);
        assert!((p.unrealized_pnl - 20.0).abs() < 1e-6);
        assert!((p.pnl_percent() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_position_markup_pnl_short() {
        let mut p = Position {
            symbol: "BTC/USD".into(),
            side: TradeSide::Short,
            size: 1.0,
            entry_price: 100.0,
            current_price: 100.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            timestamp: 0,
        };
        p.markup_pnl(90.0);
        assert!((p.unrealized_pnl - 10.0).abs() < 1e-6);
        assert!((p.pnl_percent() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_position_pnl_percent_neutral() {
        let p = Position {
            symbol: "BTC/USD".into(),
            side: TradeSide::Neutral,
            size: 0.0,
            entry_price: 0.0,
            current_price: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            timestamp: 0,
        };
        assert!((p.pnl_percent() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_portfolio_summary_new() {
        let ps = PortfolioSummary::new(10000.0);
        assert!((ps.cash - 10000.0).abs() < 1e-6);
        assert!((ps.total_value - 10000.0).abs() < 1e-6);
        assert!(ps.positions.is_empty());
    }

    #[test]
    fn test_portfolio_summary_from_positions() {
        let positions = vec![Position {
            symbol: "BTC/USD".into(),
            side: TradeSide::Long,
            size: 1.0,
            entry_price: 100.0,
            current_price: 110.0,
            unrealized_pnl: 10.0,
            realized_pnl: 0.0,
            timestamp: 0,
        }];
        let ps = PortfolioSummary::from_positions(1000.0, positions);
        assert!((ps.total_value - 1110.0).abs() < 1e-6);
        assert!((ps.unrealized_pnl - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_timeframe_seconds() {
        assert_eq!(Timeframe::Tick.seconds(), 0);
        assert_eq!(Timeframe::Minute1.seconds(), 60);
        assert_eq!(Timeframe::Hour1.seconds(), 3600);
        assert_eq!(Timeframe::Day1.seconds(), 86400);
        assert_eq!(Timeframe::Week1.seconds(), 604800);
        assert_eq!(Timeframe::Month1.seconds(), 2592000);
    }
}
