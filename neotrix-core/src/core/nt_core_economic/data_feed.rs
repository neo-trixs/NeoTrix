use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct DataFeedConfig {
    pub exchange_api_url: String,
    pub news_api_key: Option<String>,
    pub ad_network_api_key: Option<String>,
    pub polling_interval_ms: u64,
    pub max_history: usize,
}

impl Default for DataFeedConfig {
    fn default() -> Self {
        Self {
            exchange_api_url: "https://api.example.com/v1".into(),
            news_api_key: None,
            ad_network_api_key: None,
            polling_interval_ms: 60_000,
            max_history: 1000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub price: Option<f64>,
    pub volume: Option<f64>,
    pub volatility: Option<f64>,
    pub bid_ask_spread: Option<f64>,
    pub sentiment_score: f64,
    pub timestamp: u64,
    pub news_headlines: Vec<String>,
    pub market_regime: MarketRegime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketRegime {
    Bull,
    Bear,
    Sideways,
    Volatile,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DataFeed {
    config: DataFeedConfig,
    history: VecDeque<MarketData>,
    #[allow(dead_code)]
    last_update: u64,
}

impl DataFeed {
    pub fn new(config: DataFeedConfig) -> Self {
        Self {
            config,
            history: VecDeque::with_capacity(1000),
            last_update: 0,
        }
    }

    pub fn latest_market_data(&self) -> Option<MarketData> {
        self.history.back().cloned()
    }

    pub fn ingest_market_data(&mut self, data: MarketData) {
        if self.history.len() >= self.config.max_history {
            self.history.pop_front();
        }
        self.history.push_back(data);
    }

    pub fn recent_data(&self, n: usize) -> Vec<&MarketData> {
        let n = n.min(self.history.len());
        self.history.iter().rev().take(n).collect()
    }

    pub fn average_price(&self, symbol: &str, n: usize) -> Option<f64> {
        let relevant: Vec<f64> = self
            .history
            .iter()
            .rev()
            .take(n)
            .filter(|d| d.symbol == symbol)
            .filter_map(|d| d.price)
            .collect();
        if relevant.is_empty() {
            return None;
        }
        Some(relevant.iter().sum::<f64>() / relevant.len() as f64)
    }

    pub fn price_trend(&self, symbol: &str, n: usize) -> Option<f64> {
        let prices: Vec<f64> = self
            .history
            .iter()
            .rev()
            .take(n)
            .filter(|d| d.symbol == symbol)
            .filter_map(|d| d.price)
            .collect();
        if prices.len() < 2 {
            return None;
        }
        Some((prices[0] - prices[prices.len() - 1]) / prices[prices.len() - 1])
    }

    pub fn max_volatility(&self, n: usize) -> f64 {
        self.history
            .iter()
            .rev()
            .take(n)
            .filter_map(|d| d.volatility)
            .fold(0.0_f64, f64::max)
    }

    pub fn average_sentiment(&self, n: usize) -> f64 {
        let scores: Vec<f64> = self
            .history
            .iter()
            .rev()
            .take(n)
            .filter_map(|d| Some(d.sentiment_score))
            .collect();
        if scores.is_empty() {
            return 0.0;
        }
        scores.iter().sum::<f64>() / scores.len() as f64
    }

    pub fn config(&self) -> &DataFeedConfig {
        &self.config
    }
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data(symbol: &str, price: f64, sentiment: f64) -> MarketData {
        MarketData {
            symbol: symbol.into(),
            price: Some(price),
            volume: Some(1000.0),
            volatility: Some(0.05),
            bid_ask_spread: Some(0.01),
            sentiment_score: sentiment,
            timestamp: 1000,
            news_headlines: vec!["Market update".into()],
            market_regime: MarketRegime::Sideways,
        }
    }

    #[test]
    fn test_ingest_and_retrieve() {
        let mut feed = DataFeed::new(DataFeedConfig::default());
        feed.ingest_market_data(sample_data("BTC", 50000.0, 0.2));
        let latest = feed.latest_market_data();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().symbol, "BTC");
    }

    #[test]
    fn test_average_price() {
        let mut feed = DataFeed::new(DataFeedConfig::default());
        feed.ingest_market_data(sample_data("ETH", 3000.0, 0.1));
        feed.ingest_market_data(sample_data("ETH", 3100.0, 0.0));
        feed.ingest_market_data(sample_data("ETH", 3200.0, -0.1));
        let avg = feed.average_price("ETH", 3);
        assert!(avg.is_some());
        assert!((avg.unwrap() - 3100.0).abs() < 1.0);
    }

    #[test]
    fn test_price_trend_positive() {
        let mut feed = DataFeed::new(DataFeedConfig::default());
        feed.ingest_market_data(sample_data("SOL", 100.0, 0.0));
        feed.ingest_market_data(sample_data("SOL", 110.0, 0.0));
        feed.ingest_market_data(sample_data("SOL", 120.0, 0.0));
        let trend = feed.price_trend("SOL", 3);
        assert!(trend.is_some());
        assert!(trend.unwrap() > 0.0);
    }

    #[test]
    fn test_empty_feed() {
        let feed = DataFeed::new(DataFeedConfig::default());
        assert!(feed.latest_market_data().is_none());
        assert_eq!(feed.recent_data(10).len(), 0);
    }

    #[test]
    fn test_max_volatility() {
        let mut feed = DataFeed::new(DataFeedConfig::default());
        feed.ingest_market_data(sample_data("TEST", 100.0, 0.0));
        assert!(feed.max_volatility(10) > 0.0);
    }
}
