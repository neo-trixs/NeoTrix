use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// ML 预测模型 — 预测请求延迟和成功率
pub struct MLPredictor {
    /// 历史数据: provider -> (latencies, successes)
    data: RwLock<HashMap<String, ProviderStats>>,
}

#[derive(Debug, Clone)]
struct ProviderStats {
    latencies: Vec<(Duration, bool)>,
    last_update: Instant,
}

#[derive(Debug, Clone)]
pub struct LatencyPrediction {
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub confidence: f64,
}

impl MLPredictor {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }

    pub fn record(&self, provider: &str, latency: Duration, success: bool) {
        let mut data = self.data.write().unwrap();
        let stats = data.entry(provider.to_string()).or_insert_with(|| ProviderStats {
            latencies: Vec::new(),
            last_update: Instant::now(),
        });
        stats.latencies.push((latency, success));
        if stats.latencies.len() > 1000 {
            stats.latencies.drain(0..500);
        }
        stats.last_update = Instant::now();
    }

    pub fn predict_latency(&self, provider: &str) -> Option<LatencyPrediction> {
        let data = self.data.read().unwrap();
        let stats = data.get(provider)?;

        let mut sorted: Vec<Duration> = stats
            .latencies
            .iter()
            .map(|(d, _)| *d)
            .collect();
        sorted.sort();

        if sorted.is_empty() {
            return None;
        }

        let len = sorted.len();
        let p50 = sorted[len / 2];
        let p95 = sorted[(len as f64 * 0.95) as usize];
        let p99 = sorted[((len as f64 * 0.99) as usize).min(len - 1)];

        let success_count = stats.latencies.iter().filter(|(_, s)| *s).count();
        let confidence = success_count as f64 / len as f64;

        Some(LatencyPrediction {
            p50,
            p95,
            p99,
            confidence,
        })
    }

    pub fn predict_success_rate(&self, provider: &str) -> f64 {
        let data = self.data.read().unwrap();
        match data.get(provider) {
            Some(stats) if !stats.latencies.is_empty() => {
                stats.latencies.iter().filter(|(_, s)| *s).count() as f64
                    / stats.latencies.len() as f64
            }
            _ => 0.5,
        }
    }

    pub fn get_providers(&self) -> Vec<String> {
        self.data.read().unwrap().keys().cloned().collect()
    }
}

impl Default for MLPredictor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prediction() {
        let predictor = MLPredictor::new();
        for i in 0..100 {
            predictor.record(
                "fast",
                Duration::from_millis(100 + i),
                true,
            );
        }
        let pred = predictor.predict_latency("fast").unwrap();
        assert!(pred.p50 > Duration::from_millis(100));
        assert!(pred.p95 > pred.p50);
        assert!(pred.confidence > 0.9);
    }

    #[test]
    fn test_success_rate() {
        let predictor = MLPredictor::new();
        for _ in 0..80 {
            predictor.record("reliable", Duration::from_millis(100), true);
        }
        for _ in 0..20 {
            predictor.record("reliable", Duration::from_millis(200), false);
        }
        let rate = predictor.predict_success_rate("reliable");
        assert!((rate - 0.8).abs() < 0.01);
    }
}
