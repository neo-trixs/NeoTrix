#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;
use std::time::{Duration, Instant};

// ═══════════════════════════════════════════════════════════════════
// ML Predictor — 预测请求延迟和成功率
// ═══════════════════════════════════════════════════════════════════

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
        let mut data = self.data.write().unwrap_or_else(|e| e.into_inner());
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
        let data = self.data.read().unwrap_or_else(|e| e.into_inner());
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
        let data = self.data.read().unwrap_or_else(|e| e.into_inner());
        match data.get(provider) {
            Some(stats) if !stats.latencies.is_empty() => {
                stats.latencies.iter().filter(|(_, s)| *s).count() as f64
                    / stats.latencies.len() as f64
            }
            _ => 0.5,
        }
    }

    pub fn get_providers(&self) -> Vec<String> {
        self.data.read().unwrap_or_else(|e| e.into_inner()).keys().cloned().collect()
    }
}

impl Default for MLPredictor {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Intelligent Router — 基于请求特征选择最优 provider
// ═══════════════════════════════════════════════════════════════════

/// 请求特征 — 用于智能路由决策
#[derive(Debug, Clone)]
pub struct RequestProfile {
    pub complexity: f64,
    pub token_estimate: u32,
    pub latency_sla: Duration,
    pub requires_streaming: bool,
    pub cost_budget: Option<f64>,
}

/// Provider 路由权重
#[derive(Debug, Clone)]
pub struct ProviderWeight {
    pub provider: String,
    pub weight: f64,
    pub avg_latency: Duration,
    pub error_rate: f64,
    pub cost_per_1k: f64,
}

/// 智能路由器 — 基于请求特征选择最优 provider
pub struct IntelligentRouter {
    weights: RwLock<HashMap<String, ProviderWeight>>,
    history: RwLock<VecDeque<RouteDecision>>,
    max_history: usize,
}

#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub provider: String,
    pub complexity: f64,
    pub latency: Duration,
    pub success: bool,
    pub timestamp: Instant,
}

impl IntelligentRouter {
    pub fn new() -> Self {
        Self {
            weights: RwLock::new(HashMap::new()),
            history: RwLock::new(VecDeque::new()),
            max_history: 1000,
        }
    }

    pub fn register_provider(&self, weight: ProviderWeight) {
        self.weights
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(weight.provider.clone(), weight);
    }

    pub fn route(&self, profile: &RequestProfile) -> Option<String> {
        let weights = self.weights.read().unwrap_or_else(|e| e.into_inner());
        let history = self.history.read().unwrap_or_else(|e| e.into_inner());

        let mut best: Option<(&str, f64)> = None;

        for (name, pw) in weights.iter() {
            let mut score = 100.0;

            // 复杂度匹配: 高复杂度 → 高性能 provider
            if profile.complexity > 0.8 {
                score -= pw.avg_latency.as_millis() as f64 * 0.1;
            }

            // 成本约束
            if let Some(budget) = profile.cost_budget {
                let est_cost = (profile.token_estimate as f64 / 1000.0) * pw.cost_per_1k;
                if est_cost > budget {
                    score -= 50.0;
                }
            }

            // 历史成功率
            let recent: Vec<_> = history
                .iter()
                .filter(|d| d.provider == *name)
                .rev()
                .take(10)
                .collect();
            if !recent.is_empty() {
                let success_rate =
                    recent.iter().filter(|d| d.success).count() as f64 / recent.len() as f64;
                score += success_rate * 20.0;
            }

            // 错误率惩罚
            score -= pw.error_rate * 30.0;

            match best {
                None => best = Some((name, score)),
                Some((_, best_score)) if score > best_score => {
                    best = Some((name, score));
                }
                _ => {}
            }
        }

        best.map(|(name, _)| name.to_string())
    }

    pub fn record_decision(&self, decision: RouteDecision) {
        let mut history = self.history.write().unwrap_or_else(|e| e.into_inner());
        if history.len() >= self.max_history {
            history.pop_front();
        }
        history.push_back(decision);
    }

    pub fn update_weight(&self, provider: &str, success: bool, latency: Duration) {
        let mut weights = self.weights.write().unwrap_or_else(|e| e.into_inner());
        if let Some(pw) = weights.get_mut(provider) {
            let alpha = 0.1;
            let lat_ms = latency.as_millis() as f64;
            pw.avg_latency =
                Duration::from_millis((alpha * lat_ms + (1.0 - alpha) * pw.avg_latency.as_millis() as f64) as u64);
            if success {
                pw.error_rate *= 0.95;
            } else {
                pw.error_rate = (pw.error_rate + 0.1).min(1.0);
            }
        }
    }
}

impl Default for IntelligentRouter {
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

    #[test]
    fn test_route_by_complexity() {
        let router = IntelligentRouter::new();
        router.register_provider(ProviderWeight {
            provider: "fast".into(),
            weight: 1.0,
            avg_latency: Duration::from_millis(100),
            error_rate: 0.01,
            cost_per_1k: 0.01,
        });
        router.register_provider(ProviderWeight {
            provider: "slow".into(),
            weight: 1.0,
            avg_latency: Duration::from_millis(500),
            error_rate: 0.05,
            cost_per_1k: 0.005,
        });

        let profile = RequestProfile {
            complexity: 0.9,
            token_estimate: 500,
            latency_sla: Duration::from_secs(5),
            requires_streaming: false,
            cost_budget: None,
        };

        let result = router.route(&profile).unwrap();
        assert_eq!(result, "fast");
    }

    #[test]
    fn test_route_cost_constraint() {
        let router = IntelligentRouter::new();
        router.register_provider(ProviderWeight {
            provider: "expensive".into(),
            weight: 1.0,
            avg_latency: Duration::from_millis(100),
            error_rate: 0.01,
            cost_per_1k: 0.1,
        });
        router.register_provider(ProviderWeight {
            provider: "cheap".into(),
            weight: 1.0,
            avg_latency: Duration::from_millis(200),
            error_rate: 0.01,
            cost_per_1k: 0.005,
        });

        let profile = RequestProfile {
            complexity: 0.5,
            token_estimate: 1000,
            latency_sla: Duration::from_secs(10),
            requires_streaming: false,
            cost_budget: Some(0.003),
        };

        let result = router.route(&profile).unwrap();
        assert_eq!(result, "cheap");
    }
}
