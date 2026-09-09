//! Unified shared types for all NT-* domains.
use std::fmt;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

pub mod trade;
pub mod evolution;
pub mod benchmark;

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
    Open { since: Option<usize> },
    HalfOpen,
}

/// Canonical circuit breaker — shared across all NT-* domains.
///
/// Trips after `failure_threshold` consecutive failures (or when the sliding
/// window error-rate exceeds `error_rate_threshold`) and transitions to
/// half-open after `cooldown` has elapsed since the last trip.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub state: BreakerState,
    pub failure_count: u64,
    pub success_count: u64,
    pub failure_threshold: u64,
    pub success_threshold: u64,
    pub cooldown: Duration,
    pub last_failure: Option<Instant>,
    pub last_state_change: Option<Instant>,
    pub half_open_probes_used: u64,
    pub half_open_max_probes: u64,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub rejected_calls: u64,
    pub state_changes: u32,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, cooldown_secs: u64) -> Self {
        Self {
            state: BreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold: 3,
            cooldown: Duration::from_secs(cooldown_secs),
            last_failure: None,
            last_state_change: None,
            half_open_probes_used: 0,
            half_open_max_probes: 3,
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            rejected_calls: 0,
            state_changes: 0,
        }
    }

    pub fn state(&self) -> BreakerState {
        self.state
    }

    pub fn is_available(&self) -> bool {
        match self.state {
            BreakerState::Closed => true,
            BreakerState::HalfOpen => self.half_open_probes_used < self.half_open_max_probes,
            BreakerState::Open { .. } => {
                if let Some(t) = self.last_state_change {
                    t.elapsed() >= self.cooldown
                } else {
                    false
                }
            }
        }
    }

    pub fn on_success(&mut self) {
        self.total_calls += 1;
        self.successful_calls += 1;
        self.success_count += 1;
        self.failure_count = 0;

        match self.state {
            BreakerState::HalfOpen => {
                self.half_open_probes_used += 1;
                if self.half_open_probes_used >= self.half_open_max_probes {
                    self.state = BreakerState::Closed;
                    self.half_open_probes_used = 0;
                    self.last_state_change = Some(Instant::now());
                    self.state_changes += 1;
                }
            }
            BreakerState::Open { .. } => {
                self.state = BreakerState::HalfOpen;
                self.half_open_probes_used = 1;
                self.last_state_change = Some(Instant::now());
                self.state_changes += 1;
            }
            BreakerState::Closed => {}
        }
    }

    pub fn on_failure(&mut self) {
        self.total_calls += 1;
        self.failed_calls += 1;
        self.failure_count += 1;
        self.success_count = 0;
        self.last_failure = Some(Instant::now());

        match self.state {
            BreakerState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = BreakerState::Open { since: None };
                    self.last_state_change = Some(Instant::now());
                    self.state_changes += 1;
                }
            }
            BreakerState::HalfOpen => {
                self.state = BreakerState::Open { since: None };
                self.last_state_change = Some(Instant::now());
                self.half_open_probes_used = 0;
                self.state_changes += 1;
            }
            BreakerState::Open { .. } => {}
        }
    }

    pub fn is_open(&self) -> bool {
        match self.state {
            BreakerState::Open { .. } => {
                if let Some(t) = self.last_state_change {
                    t.elapsed() < self.cooldown
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    pub fn reset(&mut self) {
        self.state = BreakerState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.last_failure = None;
        self.half_open_probes_used = 0;
        self.state_changes += 1;
    }

    pub fn force_open(&mut self) {
        self.state = BreakerState::Open { since: None };
        self.last_state_change = Some(Instant::now());
        self.half_open_probes_used = 0;
    }

    pub fn failure_rate(&self) -> f64 {
        let total = self.successful_calls + self.failed_calls;
        if total == 0 {
            return 0.0;
        }
        self.failed_calls as f64 / total as f64
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, 60)
    }
}
