use super::super::circuit_breaker::CircuitBreaker;
use super::super::provider_catalog::{CommunicationProfile, ProviderCategory};
use super::super::rate_limiter::RateLimiter;

#[derive(Debug)]
pub struct ProviderState {
    pub circuit_breaker: CircuitBreaker,
    pub rate_limiter: RateLimiter,
    pub success_ema: f64,
    pub latency_window: Vec<f64>,
    pub total_calls: u64,
    pub total_errors: u64,
    pub total_tokens: u64,
    pub cost_per_1k_tokens: f64,
    pub is_free: bool,
    pub category: ProviderCategory,
}

impl ProviderState {
    pub fn new(is_free: bool, category: ProviderCategory) -> Self {
        Self {
            circuit_breaker: CircuitBreaker::default(),
            rate_limiter: RateLimiter::default(),
            success_ema: 0.8,
            latency_window: Vec::with_capacity(100),
            total_calls: 0,
            total_errors: 0,
            total_tokens: 0,
            cost_per_1k_tokens: if is_free { 0.0 } else { 0.01 },
            is_free,
            category,
        }
    }

    pub fn is_available(&self) -> bool {
        self.circuit_breaker.is_available()
    }

    /// 配额耗尽标记 — 记录 provider 处于配额耗尽状态 (freellmapi/aimux 模式)。
    /// 配额耗尽 (quota/credit 耗尽) 与瞬时限速 (429) 语义不同: 重试无益, 应剔除该 provider
    /// 直至配额恢复, 而不是反复重试同一个耗尽账户。
    pub fn mark_quota_exhausted(&mut self) {
        self.circuit_breaker.force_open();
        self.total_errors += 1;
    }

    /// 配额耗尽后是否已过冷却期可再次尝试 (配额恢复探测)
    pub fn quota_recovery_elapsed(&self) -> bool {
        self.circuit_breaker.cooldown_elapsed()
    }

    pub fn composite_score(&self) -> f64 {
        let health = self.circuit_breaker.health_penalty();
        if health <= 0.0 {
            return 0.0;
        }
        let latency_factor = {
            let mut sorted = self.latency_window.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let p95 = sorted
                .get((sorted.len() as f64 * 0.95) as usize)
                .copied()
                .unwrap_or(1000.0);
            if p95 > 0.0 {
                1.0 / (p95 / 1000.0).max(0.1)
            } else {
                0.5
            }
        };
        let cost_factor = if self.cost_per_1k_tokens > 0.0 {
            1.0 / (self.cost_per_1k_tokens * 10.0 + 1.0)
        } else {
            1.0
        };
        // 分类优先级: Local (+0.3) > Proxy (+0.15) > Cloud (+0.0)
        let category_boost = match self.category {
            ProviderCategory::Local => 0.3,
            ProviderCategory::Proxy => 0.15,
            ProviderCategory::Cloud => 0.0,
        };
        (self.success_ema.powi(2) * latency_factor * cost_factor * health) + category_boost
    }

    pub fn record_success(&mut self, latency_ms: f64, tokens: u32) {
        self.circuit_breaker.on_success();
        self.total_calls += 1;
        self.total_tokens += tokens as u64;
        self.success_ema = self.success_ema * 0.9 + 1.0 * 0.1;
        self.latency_window.push(latency_ms);
        if self.latency_window.len() > 100 {
            self.latency_window.remove(0);
        }
    }

    pub fn record_failure(&mut self, latency_ms: f64) {
        self.circuit_breaker.on_failure();
        self.total_calls += 1;
        self.total_errors += 1;
        self.success_ema = self.success_ema * 0.9 + 0.0 * 0.1;
        self.latency_window.push(latency_ms);
        if self.latency_window.len() > 100 {
            self.latency_window.remove(0);
        }
    }
}

/// An event fired after each provider call attempt.
#[derive(Debug, Clone)]
pub struct CallEvent {
    pub provider_name: String,
    pub success: bool,
    pub latency_ms: f64,
    pub tokens: u32,
    pub model: String,
    pub attempt_phase: AttemptPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptPhase {
    Normal,
    AggressiveRetry,
}

pub type CallObserver = std::sync::Arc<dyn Fn(CallEvent) + Send + Sync>;

/// 子网格: 由同一安全画像的 provider 组成的小循环通信单元 (子母阵基本单元)
/// 每个子网格包含满足特定 CommunicationProfile 的 provider 组合
#[derive(Debug, Clone)]
pub struct SubGrid {
    /// 子网格名称 (如 "anonymous-local", "proxied-cloud", "tor-anonymous")
    pub name: String,
    /// 安全画像: 该子网格满足的通信安全级别
    pub security_profile: CommunicationProfile,
    /// 包含的 provider 名称列表
    pub provider_names: Vec<String>,
    /// 创建时间
    pub created_at: std::time::SystemTime,
    /// 健康状态: 调用成功/失败次数与延迟 (子母阵反馈回路)
    pub health: SubGridHealth,
}

/// 子网格运行时健康状态 — 反馈回路 (D21 外部观察 + D30 行为化)
/// 通过 nt_core_telemetry 记录, 支持画像动态降级 (Gap 4)
#[derive(Debug, Clone, Default)]
pub struct SubGridHealth {
    /// 总调用次数
    pub call_count: u64,
    /// 成功次数
    pub success_count: u64,
    /// 错误次数
    pub error_count: u64,
    /// 累计延迟 (ms)
    pub total_latency_ms: u64,
    /// 最近一次调用时间
    pub last_used: Option<std::time::SystemTime>,
}

impl SubGridHealth {
    pub fn record_call(&mut self, success: bool, latency_ms: u64) {
        self.call_count += 1;
        if success {
            self.success_count += 1;
        } else {
            self.error_count += 1;
        }
        self.total_latency_ms += latency_ms;
        self.last_used = Some(std::time::SystemTime::now());
    }

    /// 成功率 (0.0-1.0), 无调用时返回 1.0 (未知视为健康)
    pub fn success_rate(&self) -> f64 {
        if self.call_count == 0 {
            return 1.0;
        }
        self.success_count as f64 / self.call_count as f64
    }

    /// 平均延迟 (ms)
    pub fn avg_latency_ms(&self) -> f64 {
        if self.call_count == 0 {
            return 0.0;
        }
        self.total_latency_ms as f64 / self.call_count as f64
    }

    /// 是否健康: 成功率 >= 0.8 或样本太少 (< 5)
    pub fn is_healthy(&self) -> bool {
        self.call_count < 5 || self.success_rate() >= 0.8
    }
}

impl SubGrid {
    /// 创建新子网格
    pub fn new(
        name: String,
        security_profile: CommunicationProfile,
        provider_names: Vec<String>,
    ) -> Self {
        Self {
            name,
            security_profile,
            provider_names,
            created_at: std::time::SystemTime::now(),
            health: SubGridHealth::default(),
        }
    }

    /// 检查子网格是否满足要求的安全级别
    pub fn meets_profile(&self, required: CommunicationProfile) -> bool {
        self.security_profile.meets(required)
    }
}