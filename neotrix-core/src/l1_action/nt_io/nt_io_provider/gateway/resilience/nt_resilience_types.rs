//! 共享类型 — 网关韧性策略共用的纯数据类型 (熔断 / 恢复 / 检测 / 质量 / 池健康)。
use std::time::{Duration, Instant};

use serde::Serialize;

/// 熔断器状态
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}
#[derive(Debug, Clone)]
pub struct AnomalyConfig {
    pub window_size: usize,
    pub z_score_threshold: f64,
    pub min_samples: usize,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            window_size: 100,
            z_score_threshold: 2.5,
            min_samples: 10,
        }
    }
}
#[derive(Debug, Clone)]
pub struct AnomalyAlert {
    pub provider: String,
    pub metric: String,
    pub value: f64,
    pub z_score: f64,
    pub threshold: f64,
    pub timestamp: Instant,
}
/// 自动恢复配置
#[derive(Debug, Clone)]
pub struct AutoRecoveryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
    pub circuit_breaker_threshold: u32,
    pub recovery_check_interval: Duration,
}

impl Default for AutoRecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
            circuit_breaker_threshold: 5,
            recovery_check_interval: Duration::from_secs(60),
        }
    }
}
/// Provider 健康状态
#[derive(Debug, Clone, PartialEq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Recovering,
    CircuitOpen,
}
/// Provider 指标快照
pub struct ProviderMetric {
    pub provider_id: String,
    pub latency_ms: u64,
    pub cost_per_token: f64,
    pub success_rate: f64,
    pub quality_score: f64,
    pub timestamp: i64,
}
/// 漂移检测结果
#[derive(Debug, Clone, PartialEq)]
pub enum DriftStatus {
    Normal,
    MildDrift {
        metric: String,
        deviation: f64,
    },
    SevereDrift {
        metric: String,
        deviation: f64,
        suggested_alternative: String,
    },
}
#[derive(Debug, Clone)]
pub struct ResponseQualityScore {
    pub coherence: f64,
    pub relevance: f64,
    pub completeness: f64,
}
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct PoolHealthReport {
    pub total: usize,
    pub free: usize,
    pub paid: usize,
    pub local: usize,
    pub model_locked: usize,
    pub sufficient: bool,
    pub min_free: usize,
}
