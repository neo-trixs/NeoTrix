//! Unified shared types for all NT-* domains.
use std::fmt;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    High,
    Error,
    Medium,
    Warning,
    Low,
    Info,
    Informational,
    Pass,
}

impl Severity {
    pub fn from_score(score: f64) -> Self {
        if score >= 9.0 { Severity::Critical }
        else if score >= 7.0 { Severity::High }
        else if score >= 5.0 { Severity::Medium }
        else if score >= 3.0 { Severity::Low }
        else if score >= 0.1 { Severity::Info }
        else { Severity::Informational }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Critical => "Critical",
            Severity::High => "High",
            Severity::Error => "Error",
            Severity::Medium => "Medium",
            Severity::Warning => "Warning",
            Severity::Low => "Low",
            Severity::Info => "Info",
            Severity::Informational => "Informational",
            Severity::Pass => "Pass",
        }
    }
    pub fn numeric(&self) -> u8 {
        match self {
            Severity::Critical => 8, Severity::High => 7, Severity::Error => 6,
            Severity::Medium => 5, Severity::Warning => 4, Severity::Low => 3,
            Severity::Info => 2, Severity::Informational => 1, Severity::Pass => 0,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NtDomain {
    NtCore, NtMind, NtMemory, NtWorld, NtAct, NtIo, NtShield,
    NtMeta, NtRepair, NtGovernance, NtNexus, NtPhysical, NtFeel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus { Healthy, Degraded, Unhealthy, Unknown }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskState { Pending, Running, Completed, Failed, Cancelled, Timeout }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrendDirection { Rising, Stable, Falling, Volatile }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Canonical circuit breaker shared across NT-* domains.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub state: BreakerState,
    pub failure_count: u64,
    pub failure_threshold: u64,
    pub cooldown: Duration,
    pub last_state_change: Option<Instant>,
    pub half_open_probes_used: u64,
    pub half_open_max_probes: u64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, cooldown_secs: u64) -> Self {
        Self::new_with_cooldown(failure_threshold, Duration::from_secs(cooldown_secs))
    }

    /// Construct with an explicit cooldown duration.
    ///
    /// Added 2026-09-28: `new` takes **whole seconds**, so a sub-second cooldown
    /// was inexpressible. Two tests in `nt_core_observer_error` were written
    /// against a millisecond semantic the API never had (sleep 2ms against
    /// `new(1, 1)`) and could never pass. Rather than weaken them into no
    /// longer exercising the Open -> HalfOpen transition, the API now accepts
    /// the granularity callers want. Purely additive: `new` delegates here.
    pub fn new_with_cooldown(failure_threshold: u64, cooldown: Duration) -> Self {
        Self {
            state: BreakerState::Closed,
            failure_count: 0,
            failure_threshold,
            cooldown,
            last_state_change: None,
            half_open_probes_used: 0,
            half_open_max_probes: 3,
        }
    }

    pub fn is_available(&self) -> bool {
        match self.state {
            BreakerState::Closed => true,
            BreakerState::Open => {
                if let Some(t) = self.last_state_change {
                    t.elapsed() >= self.cooldown
                } else {
                    true
                }
            }
            BreakerState::HalfOpen => self.half_open_probes_used < self.half_open_max_probes,
        }
    }

    pub fn on_success(&mut self) {
        self.state = BreakerState::Closed;
        self.failure_count = 0;
        self.half_open_probes_used = 0;
    }

    /// 尝试占用一次调用配额 —— **会执行 Open -> HalfOpen 状态转换**。
    ///
    /// 2026-09-28 修三个相互关联的缺陷（均由测试暴露, 均为真 bug 而非测试写错）:
    ///
    /// 1. `is_available(&self)` 是纯查询: 冷却期已过只返回 true, **不转换状态**。
    ///    于是 state 永远是 Open, 调用方随后 on_success 仍按 Open 走 ——
    ///    状态机与查询结果不一致。
    /// 2. `half_open_probes_used` **从未自增**, 只被读和清零, 于是
    ///    `probes_used < max_probes` 恒为 `0 < 3`, HalfOpen 探针配额形同虚设。
    /// 3. 未达阈值的失败会降级到 HalfOpen（已由 on_failure 上方注释记录并修正）。
    ///
    /// 语义: Closed 恒放行且不计配额; Open 冷却期未到则拒绝, 已到则转入
    /// HalfOpen 并**消耗一个探针**; HalfOpen 按剩余探针放行。
    pub fn try_acquire(&mut self) -> bool {
        // 首次实现把「Open 且冷却未到」也放行了 —— 因为末尾的 else 兜底返回 true。
        // 熔断器的定义就是冷却期内必须拒绝, 故这里按状态逐一分派, 不留兜底。
        match self.state {
            BreakerState::Closed => true,
            BreakerState::Open => {
                let cooled = self
                    .last_state_change
                    .map(|t| t.elapsed() >= self.cooldown)
                    .unwrap_or(true);
                if !cooled {
                    return false;
                }
                self.state = BreakerState::HalfOpen;
                self.half_open_probes_used = 0;
                self.consume_probe()
            }
            BreakerState::HalfOpen => self.consume_probe(),
        }
    }

    /// 消耗一个 HalfOpen 探针配额。
    fn consume_probe(&mut self) -> bool {
        if self.half_open_probes_used < self.half_open_max_probes {
            self.half_open_probes_used += 1;
            true
        } else {
            false
        }
    }

    /// Record a failed call.
    ///
    /// Below `failure_threshold` the breaker stays `Closed` — a transient
    /// failure must not demote an available dependency into a probe-limited
    /// `HalfOpen` (which would also silently un-block a `force_open()` breaker
    /// on its very next failure). `HalfOpen` is only ever entered *after* an
    /// `Open` cooldown elapses; crossing the threshold trips to `Open`.
    pub fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_state_change = Some(Instant::now());
        if self.failure_count >= self.failure_threshold {
            self.state = BreakerState::Open;
        }
    }

    pub fn is_open(&self) -> bool {
        matches!(self.state, BreakerState::Open)
    }

    pub fn reset(&mut self) {
        self.state = BreakerState::Closed;
        self.failure_count = 0;
        self.half_open_probes_used = 0;
    }

    pub fn force_open(&mut self) {
        self.state = BreakerState::Open;
        self.last_state_change = Some(Instant::now());
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, 60)
    }
}
