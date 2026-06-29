#![forbid(unsafe_code)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub struct VsaMarketSnapshot {
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: u64,
}

impl VsaMarketSnapshot {
    pub fn encode_vsa(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);
        let data = [
            self.open as u64,
            self.high as u64,
            self.low as u64,
            self.close as u64,
            self.volume as u64,
            self.timestamp,
        ];
        for v in data {
            let bytes = v.to_le_bytes();
            buf.extend_from_slice(&bytes);
        }
        buf
    }
}

#[derive(Debug, Clone)]
pub struct MarketHistoryRingBuffer {
    capacity: usize,
    inner: VecDeque<VsaMarketSnapshot>,
}

impl MarketHistoryRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            inner: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, snapshot: VsaMarketSnapshot) {
        if self.inner.len() >= self.capacity {
            self.inner.pop_front();
        }
        self.inner.push_back(snapshot);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn recent(&self, n: usize) -> Vec<&VsaMarketSnapshot> {
        let n = n.min(self.inner.len());
        self.inner.iter().rev().take(n).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuantDataSource {
    YFinance(String, u32),
}

pub struct QuantDataIngestion {
    sources: Vec<QuantDataSource>,
    buffers: HashMap<String, MarketHistoryRingBuffer>,
}

impl QuantDataIngestion {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            buffers: HashMap::new(),
        }
    }

    pub fn add_source(&mut self, source: QuantDataSource) {
        self.sources.push(source);
    }

    pub fn ingest_tick(&mut self, symbol: &str, price: f64, volume: f64) -> VsaMarketSnapshot {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let buf = self
            .buffers
            .entry(symbol.to_string())
            .or_insert_with(|| MarketHistoryRingBuffer::new(1000));
        let prev = buf.recent(1).first().cloned();
        let (open, high, low) = match prev {
            Some(p) => (p.close, p.close.max(price), p.close.min(price)),
            None => (price, price, price),
        };
        let snap = VsaMarketSnapshot {
            symbol: symbol.to_string(),
            open,
            high,
            low,
            close: price,
            volume,
            timestamp: now,
        };
        buf.push(snap.clone());
        snap
    }

    pub fn recent_snapshots(&self, symbol: &str, n: usize) -> Vec<&VsaMarketSnapshot> {
        self.buffers
            .get(symbol)
            .map(|b| b.recent(n))
            .unwrap_or_default()
    }

    pub fn regime_classifier(&self, snapshot: &VsaMarketSnapshot) -> &'static str {
        let range = snapshot.high - snapshot.low;
        let avg = (snapshot.open + snapshot.close) / 2.0;
        if avg == 0.0 {
            return "unknown";
        }
        let volatility = range / avg;
        if range < 0.01 * avg {
            "range"
        } else if snapshot.close > snapshot.open && volatility > 0.02 {
            "bull"
        } else if snapshot.close < snapshot.open && volatility > 0.02 {
            "bear"
        } else {
            "range"
        }
    }

    pub fn source_count(&self) -> usize {
        self.sources.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingestion_creates_snapshot() {
        let mut engine = QuantDataIngestion::new();
        let snap = engine.ingest_tick("AAPL", 150.0, 1_000_000.0);
        assert_eq!(snap.symbol, "AAPL");
        assert_eq!(snap.close, 150.0);
        assert!(snap.timestamp > 0);
    }

    #[test]
    fn test_recent_snapshots_returns_n() {
        let mut engine = QuantDataIngestion::new();
        for i in 0..10 {
            engine.ingest_tick("AAPL", 150.0 + i as f64, 1_000_000.0);
        }
        let recent = engine.recent_snapshots("AAPL", 5);
        assert_eq!(recent.len(), 5);
    }

    #[test]
    fn test_regime_classifier_bull() {
        let mut engine = QuantDataIngestion::new();
        let _snap = engine.ingest_tick("TEST", 101.0, 1000.0);
        // second tick higher = possible bull
        let snap2 = engine.ingest_tick("TEST", 105.0, 1000.0);
        let regime = engine.regime_classifier(&snap2);
        assert!(!regime.is_empty());
    }

    #[test]
    fn test_ring_buffer_eviction() {
        let mut buf = MarketHistoryRingBuffer::new(5);
        for i in 0..10 {
            buf.push(VsaMarketSnapshot {
                symbol: "T".into(),
                open: i as f64,
                high: i as f64,
                low: i as f64,
                close: i as f64,
                volume: 0.0,
                timestamp: i as u64,
            });
        }
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.recent(10)[4].close, 9.0);
    }

    #[test]
    fn test_empty_ring_buffer() {
        let buf = MarketHistoryRingBuffer::new(10);
        assert!(buf.is_empty());
        assert_eq!(buf.recent(5).len(), 0);
    }

    #[test]
    fn test_add_source() {
        let mut engine = QuantDataIngestion::new();
        engine.add_source(QuantDataSource::YFinance("AAPL".into(), 60));
        assert_eq!(engine.source_count(), 1);
    }

    #[test]
    fn test_vsa_snapshot_encode() {
        let snap = VsaMarketSnapshot {
            symbol: "T".into(),
            open: 1.0,
            high: 2.0,
            low: 1.0,
            close: 1.5,
            volume: 1000.0,
            timestamp: 100,
        };
        let encoded = snap.encode_vsa();
        assert_eq!(encoded.len(), 48);
    }

    #[test]
    fn test_regime_unknown_on_zero_avg() {
        let snap = VsaMarketSnapshot {
            symbol: "T".into(),
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0.0,
            timestamp: 0,
        };
        let engine = QuantDataIngestion::new();
        assert_eq!(engine.regime_classifier(&snap), "unknown");
    }

    #[test]
    fn test_recent_empty_symbol() {
        let engine = QuantDataIngestion::new();
        assert!(engine.recent_snapshots("NONEXIST", 5).is_empty());
    }
}
