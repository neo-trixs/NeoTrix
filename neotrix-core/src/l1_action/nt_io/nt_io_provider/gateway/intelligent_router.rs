use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;
use std::time::{Duration, Instant};

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
struct RouteDecision {
    provider: String,
    complexity: f64,
    latency: Duration,
    success: bool,
    timestamp: Instant,
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
            .unwrap()
            .insert(weight.provider.clone(), weight);
    }

    pub fn route(&self, profile: &RequestProfile) -> Option<String> {
        let weights = self.weights.read().unwrap();
        let history = self.history.read().unwrap();

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
        let mut history = self.history.write().unwrap();
        if history.len() >= self.max_history {
            history.pop_front();
        }
        history.push_back(decision);
    }

    pub fn update_weight(&self, provider: &str, success: bool, latency: Duration) {
        let mut weights = self.weights.write().unwrap();
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
