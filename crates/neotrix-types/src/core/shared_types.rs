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
        Self {
            state: BreakerState::Closed,
            failure_count: 0,
            failure_threshold,
            cooldown: Duration::from_secs(cooldown_secs),
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

    pub fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_state_change = Some(Instant::now());
        if self.failure_count >= self.failure_threshold {
            self.state = BreakerState::Open;
        } else {
            self.state = BreakerState::HalfOpen;
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
