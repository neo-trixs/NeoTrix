use std::time::{Duration, Instant};

use super::state::ProviderState;

/// G: market-wisdom 路由 + Auto Exacto 周期重估。
/// 维护每 provider 的市场权重 (success_rate / composite_score / avg_latency 加权混合),
/// `re_evaluate()` 每隔 N 分钟 (默认 5min, 可经构造器注入短间隔) 重算一次权重;
/// `route()` 按权重返回最佳 provider index。
#[derive(Debug)]
pub struct MarketRouter {
    interval: Duration,
    last_eval: Instant,
    weights: Vec<f64>,
    eval_count: u64,
}

impl MarketRouter {
    /// 默认重估间隔: 5 分钟
    pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(300);

    pub fn new() -> Self {
        Self::with_interval(Self::DEFAULT_INTERVAL)
    }

    /// 可配置重估间隔 (测试注入短间隔)
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            interval,
            last_eval: Instant::now(),
            weights: Vec::new(),
            eval_count: 0,
        }
    }

    /// 周期重估: 距上次重估超过 `interval` (或首次) 时重算全部权重。
    /// 返回是否真的重估了。
    pub fn re_evaluate(&mut self, providers: &[&ProviderState]) -> bool {
        let due = self.weights.is_empty() || self.last_eval.elapsed() >= self.interval;
        if !due {
            return false;
        }
        self.weights = providers.iter().map(|s| market_weight(s)).collect();
        self.last_eval = Instant::now();
        self.eval_count += 1;
        true
    }

    /// 返回最佳 provider index (不可用 / 权重 <= 0 者跳过); 无可用返回 None
    pub fn route(&mut self, providers: &[&mut ProviderState]) -> Option<usize> {
        if providers.is_empty() {
            return None;
        }
        let refs: Vec<&ProviderState> = providers.iter().map(|p| &**p).collect();
        let _ = self.re_evaluate(&refs);
        drop(refs);
        let mut best: Option<(usize, f64)> = None;
        for (i, p) in providers.iter().enumerate() {
            let w = self
                .weights
                .get(i)
                .copied()
                .unwrap_or_else(|| market_weight(p));
            if w <= 0.0 {
                continue;
            }
            if best.map(|(_, bw)| w > bw).unwrap_or(true) {
                best = Some((i, w));
            }
        }
        best.map(|(i, _)| i)
    }

    pub fn weights(&self) -> &[f64] {
        &self.weights
    }

    pub fn eval_count(&self) -> u64 {
        self.eval_count
    }
}

impl Default for MarketRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// 单 provider 的市场权重: success_rate + composite_score + avg_latency 加权混合
fn market_weight(s: &ProviderState) -> f64 {
    if !s.is_available() {
        return 0.0;
    }
    let success = s.success_ema.clamp(0.0, 1.0);
    // avg latency factor: 越低越优 (无样本时视为最优 1.0)
    let avg_latency = if s.latency_window.is_empty() {
        0.0
    } else {
        s.latency_window.iter().sum::<f64>() / s.latency_window.len() as f64
    };
    let latency_factor = if avg_latency > 0.0 {
        (1.0 / (avg_latency / 1000.0).max(0.1)).min(1.0)
    } else {
        1.0
    };
    // composite_score 归一化到 ~[0,1]
    let composite_factor = (s.composite_score() / 1.5).clamp(0.0, 1.0);
    success * 0.4 + composite_factor * 0.4 + latency_factor * 0.2
}