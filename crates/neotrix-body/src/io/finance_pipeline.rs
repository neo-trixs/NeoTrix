use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};
use rand::Rng;

use crate::io::IoResult;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarketType {
    AShare,
    HK,
    US,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub name: String,
    pub market: MarketType,
    pub priority: u8,
    pub rate_limit_per_min: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
    pub volume: u64,
    pub timestamp_ms: u64,
    pub market: MarketType,
    pub vsa_fingerprint: [u64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KLineInterval {
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ohlcv {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KLine {
    pub symbol: String,
    pub interval: KLineInterval,
    pub bars: Vec<Ohlcv>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialReport {
    pub revenue: f64,
    pub profit: f64,
    pub eps: f64,
    pub pe_ratio: f64,
    pub pb_ratio: f64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    pub symbol: String,
    pub name: String,
    pub market: MarketType,
    pub sector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketIndex {
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOverview {
    pub indices: Vec<MarketIndex>,
    pub total_volume: u64,
    pub advances: u32,
    pub declines: u32,
    pub unchanged: u32,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CachedData {
    pub data: String,
    pub fetched_ms: u64,
}

#[derive(Debug, Clone)]
pub struct FinancePipeline {
    data_sources: Vec<DataSource>,
    cache: HashMap<String, CachedData>,
}

impl Default for MarketType {
    fn default() -> Self {
        Self::AShare
    }
}

fn symbol_hash(symbol: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    symbol.hash(&mut hasher);
    hasher.finish()
}

fn seeded_rng(symbol: &str, seed: u64) -> impl Rng {
    use rand::SeedableRng;
    rand::rngs::StdRng::seed_from_u64(symbol_hash(symbol).wrapping_add(seed))
}

impl FinancePipeline {
    pub fn new() -> Self {
        Self {
            data_sources: vec![
                DataSource {
                    name: "sina".into(),
                    market: MarketType::AShare,
                    priority: 1,
                    rate_limit_per_min: 100,
                },
                DataSource {
                    name: "eastmoney".into(),
                    market: MarketType::AShare,
                    priority: 2,
                    rate_limit_per_min: 200,
                },
            ],
            cache: HashMap::new(),
        }
    }

    pub fn get_quote(&self, symbol: &str, market: MarketType) -> IoResult<StockQuote> {
        let h = symbol_hash(symbol);
        let mut rng = seeded_rng(symbol, 42);
        let base_price = match market {
            MarketType::AShare => 10.0 + (h % 200) as f64 * 0.5,
            MarketType::HK => 50.0 + (h % 100) as f64 * 1.0,
            MarketType::US => 20.0 + (h % 400) as f64 * 0.25,
        };
        let price = base_price + rng.gen_range(-0.05..0.05) * base_price;
        let change_pct = (price - base_price) / base_price * 100.0;
        let volume = 100_000 + (h % 10_000) as u64 * 100;
        let vsa = Self::compute_vsa(symbol, price);

        let names: HashMap<&str, &str> = [
            ("600000", "浦发银行"),
            ("600519", "贵州茅台"),
            ("000001", "平安银行"),
            ("300750", "宁德时代"),
            ("00700", "腾讯控股"),
            ("AAPL", "Apple Inc."),
        ]
        .iter()
        .cloned()
        .collect();

        let name = names
            .get(symbol)
            .copied()
            .unwrap_or("未知")
            .to_string();

        Ok(StockQuote {
            symbol: symbol.to_string(),
            name,
            price: (price * 100.0).round() / 100.0,
            change_pct: (change_pct * 100.0).round() / 100.0,
            volume,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            market,
            vsa_fingerprint: vsa,
        })
    }

    pub fn get_kline(
        &self,
        symbol: &str,
        interval: KLineInterval,
        days: u32,
    ) -> IoResult<KLine> {
        let mut rng = seeded_rng(symbol, 99);
        let base_price = 10.0 + (symbol_hash(symbol) % 200) as f64 * 0.5;
        let bar_count = match interval {
            KLineInterval::Day => days as usize,
            KLineInterval::Week => std::cmp::max(1, (days / 7) as usize),
            KLineInterval::Month => std::cmp::max(1, (days / 30) as usize),
        };

        let mut bars = Vec::with_capacity(bar_count);
        let mut price = base_price;

        for _ in 0..bar_count {
            let change = rng.gen_range(-0.04..0.04);
            let open = price;
            let close = price * (1.0 + change);
            let (low, high) = if open < close {
                (open * (1.0 - rng.gen_range(0.0..0.02)), close * (1.0 + rng.gen_range(0.0..0.02)))
            } else {
                (close * (1.0 - rng.gen_range(0.0..0.02)), open * (1.0 + rng.gen_range(0.0..0.02)))
            };
            let volume = 500_000 + rng.gen_range(0..2_000_000u64);

            bars.push(Ohlcv {
                open: (open * 100.0).round() / 100.0,
                high: (high * 100.0).round() / 100.0,
                low: (low * 100.0).round() / 100.0,
                close: (close * 100.0).round() / 100.0,
                volume,
            });

            price = close;
        }

        Ok(KLine {
            symbol: symbol.to_string(),
            interval,
            bars,
        })
    }

    pub fn get_financials(&self, symbol: &str) -> IoResult<FinancialReport> {
        let h = symbol_hash(symbol);
        let revenue = 1_000_000_000.0 + (h % 1000) as f64 * 10_000_000.0;
        let profit_margin = 0.05 + (h % 20) as f64 * 0.01;
        let profit = revenue * profit_margin;
        let shares = 1_000_000_000.0 + (h % 500) as f64 * 10_000_000.0;
        let eps = profit / shares;
        let price = 10.0 + (h % 200) as f64 * 0.5;
        let pe_ratio = if eps.abs() > 1e-10 { price / eps } else { 0.0 };
        let book_value = revenue * 0.6;
        let pb_ratio = if book_value.abs() > 1e-10 {
            (price * shares) / book_value
        } else {
            0.0
        };

        Ok(FinancialReport {
            revenue: (revenue * 100.0).round() / 100.0,
            profit: (profit * 100.0).round() / 100.0,
            eps: (eps * 10000.0).round() / 10000.0,
            pe_ratio: (pe_ratio * 100.0).round() / 100.0,
            pb_ratio: (pb_ratio * 100.0).round() / 100.0,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        })
    }

    pub fn search_stocks(&self, keyword: &str) -> Vec<StockInfo> {
        let pool = vec![
            StockInfo { symbol: "600000".into(), name: "浦发银行".into(), market: MarketType::AShare, sector: "金融".into() },
            StockInfo { symbol: "600519".into(), name: "贵州茅台".into(), market: MarketType::AShare, sector: "消费".into() },
            StockInfo { symbol: "000001".into(), name: "平安银行".into(), market: MarketType::AShare, sector: "金融".into() },
            StockInfo { symbol: "300750".into(), name: "宁德时代".into(), market: MarketType::AShare, sector: "新能源".into() },
            StockInfo { symbol: "00700".into(), name: "腾讯控股".into(), market: MarketType::HK, sector: "科技".into() },
            StockInfo { symbol: "AAPL".into(), name: "Apple Inc.".into(), market: MarketType::US, sector: "科技".into() },
            StockInfo { symbol: "MSFT".into(), name: "Microsoft Corp.".into(), market: MarketType::US, sector: "科技".into() },
            StockInfo { symbol: "000333".into(), name: "美的集团".into(), market: MarketType::AShare, sector: "家电".into() },
            StockInfo { symbol: "601318".into(), name: "中国平安".into(), market: MarketType::AShare, sector: "金融".into() },
            StockInfo { symbol: "600036".into(), name: "招商银行".into(), market: MarketType::AShare, sector: "金融".into() },
        ];

        let lower = keyword.to_lowercase();
        pool.into_iter()
            .filter(|s| {
                s.symbol.to_lowercase().contains(&lower)
                    || s.name.to_lowercase().contains(&lower)
                    || s.sector.to_lowercase().contains(&lower)
            })
            .collect()
    }

    pub fn get_market_overview(&self) -> MarketOverview {
        let mut rng = rand::thread_rng();
        MarketOverview {
            indices: vec![
                MarketIndex {
                    name: "上证指数".into(),
                    price: 3100.0 + rng.gen_range(-50.0..50.0),
                    change_pct: rng.gen_range(-1.5..1.5),
                },
                MarketIndex {
                    name: "深证成指".into(),
                    price: 9500.0 + rng.gen_range(-200.0..200.0),
                    change_pct: rng.gen_range(-2.0..2.0),
                },
                MarketIndex {
                    name: "创业板指".into(),
                    price: 1900.0 + rng.gen_range(-50.0..50.0),
                    change_pct: rng.gen_range(-2.5..2.5),
                },
                MarketIndex {
                    name: "恒生指数".into(),
                    price: 17500.0 + rng.gen_range(-300.0..300.0),
                    change_pct: rng.gen_range(-1.8..1.8),
                },
            ],
            total_volume: 50_000_000_000 + rng.gen_range(0..20_000_000_000u64),
            advances: 1500 + rng.gen_range(0..500),
            declines: 1200 + rng.gen_range(0..400),
            unchanged: 300 + rng.gen_range(0..100),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    pub fn compute_vsa(symbol: &str, price: f64) -> [u64; 4] {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        symbol.hash(&mut hasher);
        let h1 = hasher.finish();

        let price_bits = price.to_bits();
        let h2 = (price_bits as u64).wrapping_mul(6_364_136_223_846_793_005);
        let h3 = h1.wrapping_add(h2).wrapping_mul(1_442_695_040_888_963_407);
        let h4 = h2.wrapping_sub(h1) ^ 0x9e3779b97f4a7c15;

        [h1, h2, h3, h4]
    }

    pub fn sources(&self) -> &[DataSource] {
        &self.data_sources
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for FinancePipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pipeline() {
        let fp = FinancePipeline::new();
        assert_eq!(fp.sources().len(), 2);
        assert_eq!(fp.sources()[0].name, "sina");
    }

    #[test]
    fn test_get_quote_ashare() {
        let fp = FinancePipeline::new();
        let quote = fp.get_quote("600519", MarketType::AShare).unwrap();
        assert_eq!(quote.symbol, "600519");
        assert_eq!(quote.market, MarketType::AShare);
        assert!(quote.price > 0.0);
        assert!(quote.vsa_fingerprint.iter().all(|&v| v != 0));
    }

    #[test]
    fn test_get_quote_hk() {
        let fp = FinancePipeline::new();
        let quote = fp.get_quote("00700", MarketType::HK).unwrap();
        assert_eq!(quote.symbol, "00700");
        assert_eq!(quote.market, MarketType::HK);
    }

    #[test]
    fn test_get_quote_us() {
        let fp = FinancePipeline::new();
        let quote = fp.get_quote("AAPL", MarketType::US).unwrap();
        assert_eq!(quote.symbol, "AAPL");
        assert_eq!(quote.market, MarketType::US);
    }

    #[test]
    fn test_get_kline_day() {
        let fp = FinancePipeline::new();
        let kline = fp.get_kline("600519", KLineInterval::Day, 5).unwrap();
        assert_eq!(kline.symbol, "600519");
        assert_eq!(kline.bars.len(), 5);
        assert!(kline.bars[0].close > 0.0);
    }

    #[test]
    fn test_get_kline_week() {
        let fp = FinancePipeline::new();
        let kline = fp.get_kline("000001", KLineInterval::Week, 30).unwrap();
        assert!(!kline.bars.is_empty());
        assert!(kline.bars.len() <= 5);
    }

    #[test]
    fn test_get_kline_month() {
        let fp = FinancePipeline::new();
        let kline = fp.get_kline("300750", KLineInterval::Month, 365).unwrap();
        assert!(!kline.bars.is_empty());
        assert!(kline.bars.len() <= 13);
    }

    #[test]
    fn test_get_financials() {
        let fp = FinancePipeline::new();
        let fin = fp.get_financials("600519").unwrap();
        assert!(fin.revenue > 0.0);
        assert!(fin.eps > 0.0);
        assert!(fin.pe_ratio > 0.0);
    }

    #[test]
    fn test_search_stocks() {
        let fp = FinancePipeline::new();
        let results = fp.search_stocks("金融");
        assert!(!results.is_empty());
        assert!(results.iter().all(|s| s.sector == "金融"));
    }

    #[test]
    fn test_search_stocks_by_symbol() {
        let fp = FinancePipeline::new();
        let results = fp.search_stocks("600519");
        assert!(results.iter().any(|s| s.symbol == "600519"));
    }

    #[test]
    fn test_search_stocks_empty() {
        let fp = FinancePipeline::new();
        let results = fp.search_stocks("NONEXISTENT_ZZZ");
        assert!(results.is_empty());
    }

    #[test]
    fn test_market_overview() {
        let fp = FinancePipeline::new();
        let overview = fp.get_market_overview();
        assert_eq!(overview.indices.len(), 4);
        assert!(overview.total_volume > 0);
        assert!(overview.advances > 0);
        assert!(overview.declines > 0);
    }

    #[test]
    fn test_compute_vsa_deterministic() {
        let v1 = FinancePipeline::compute_vsa("600519", 150.0);
        let v2 = FinancePipeline::compute_vsa("600519", 150.0);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_compute_vsa_different_symbols() {
        let v1 = FinancePipeline::compute_vsa("600519", 150.0);
        let v2 = FinancePipeline::compute_vsa("000001", 150.0);
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_cached_data_starts_empty() {
        let fp = FinancePipeline::new();
        assert_eq!(fp.cache_size(), 0);
    }

    #[test]
    fn test_quote_name_known() {
        let fp = FinancePipeline::new();
        let quote = fp.get_quote("600519", MarketType::AShare).unwrap();
        assert_eq!(quote.name, "贵州茅台");
    }
}
