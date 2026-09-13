use std::collections::VecDeque;
use std::time::{Duration, Instant};

use neotrix_types::shared::CircuitBreaker as CanonicalCircuitBreaker;
pub use neotrix_types::shared::BreakerState;

/// Provider circuit breaker — wraps canonical `CircuitBreaker` with
/// sliding-window failure tracking, force-open, and health-penalty scoring.
#[derive(Debug, Clone)]
pub struct ProviderBreaker {
    inner: CanonicalCircuitBreaker,
    sliding_window: VecDeque<bool>,
    window_size: usize,
}

/// Backward-compatible alias.
pub type CircuitBreaker = ProviderBreaker;

impl ProviderBreaker {
    /// Create a new circuit breaker with sliding-window failure tracking.
    ///
    /// Note: Real implementation needs — `window_size` controls how many recent
    /// outcomes are tracked. Consider: making window_size time-based (e.g., last
    /// 60 seconds) instead of count-based for more predictable behavior under
    /// variable call rates.
    pub fn new(failure_threshold: u64, cooldown_secs: u64, window_size: usize) -> Self {
        Self {
            inner: CanonicalCircuitBreaker::new(failure_threshold, cooldown_secs),
            sliding_window: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    pub fn state(&self) -> BreakerState {
        self.inner.state
    }

    pub fn is_available(&self) -> bool {
        self.inner.is_available()
    }

    pub fn set_half_open_max_probes(&mut self, n: u64) {
        self.inner.half_open_max_probes = n;
    }

    pub fn half_open_max_probes(&self) -> u64 {
        self.inner.half_open_max_probes
    }

    /// Health penalty score for provider selection weighting.
    ///
    /// Note: Real implementation needs — linear mapping (1.0/0.5/0.0) is simplistic.
    /// Consider: time-decay penalty during Open state (penalty decreases as cooldown
    /// elapses), and partial penalty during HalfOpen based on probe success rate.
    pub fn health_penalty(&self) -> f64 {
        match self.inner.state {
            BreakerState::Closed => 1.0,
            BreakerState::HalfOpen => 0.5,
            BreakerState::Open { .. } => 0.0,
        }
    }

    pub fn force_open(&mut self) {
        self.inner.force_open();
    }

    /// Force the circuit breaker open with a custom cooldown duration.
    ///
    /// Note: Real implementation needs — resets half_open_probes_used but does not
    /// log the forced-open event. Consider: emitting an EventBus event for telemetry,
    /// and supporting force-open with a reason string for audit trails.
    pub fn force_open_secs(&mut self, cooldown_secs: u64) {
        self.inner.state = BreakerState::Open;
        self.inner.cooldown = Duration::from_secs(cooldown_secs);
        self.inner.last_state_change = Some(Instant::now());
        self.inner.half_open_probes_used = 0;
    }

    pub fn cooldown_elapsed(&self) -> bool {
        match self.inner.last_state_change {
            Some(t) => t.elapsed() >= self.inner.cooldown,
            None => true,
        }
    }

    /// Record a successful call and update sliding window.
    ///
    /// Note: Real implementation needs — does not transition from HalfOpen to Closed
    /// on success. Consider: after N consecutive successes in HalfOpen, transition
    /// to Closed and reset failure counters.
    pub fn on_success(&mut self) {
        self.sliding_window.push_back(true);
        if self.sliding_window.len() > self.window_size {
            self.sliding_window.pop_front();
        }
        self.inner.on_success();
    }

    /// Record a failed call and update sliding window.
    ///
    /// Note: Real implementation needs — if sliding window failure count exceeds
    /// threshold, forces breaker Open directly (bypassing canonical on_failure).
    /// Consider: logging the transition event, and tracking failure reasons
    /// (rate-limit vs timeout vs error) for diagnostic purposes.
    pub fn on_failure(&mut self) {
        self.sliding_window.push_back(false);
        if self.sliding_window.len() > self.window_size {
            self.sliding_window.pop_front();
        }

        let recent_failures = self.sliding_window.iter().filter(|&&s| !s).count();
        if recent_failures >= self.inner.failure_threshold as usize {
            self.inner.state = BreakerState::Open;
            self.inner.last_state_change = Some(Instant::now());
        } else {
            self.inner.on_failure();
        }
    }

    /// Calculate the current failure rate from the sliding window.
    ///
    /// Note: Real implementation needs — returns 0.0 for empty window.
    /// Consider: adding a minimum sample size requirement before computing
    /// rate (e.g., require at least 5 calls before returning meaningful rate).
    pub fn failure_rate(&self) -> f64 {
        let len = self.sliding_window.len();
        if len == 0 {
            return 0.0;
        }
        let failures = self.sliding_window.iter().filter(|&&s| !s).count();
        failures as f64 / len as f64
    }

    /// Check if cooldown has elapsed and transition Open → HalfOpen.
    ///
    /// STUB: This method is the primary entry point for breaker state progression.
    /// Real implementation needs — should be called periodically (e.g., by a background
    /// health check loop) rather than on each request. Currently only transitions
    /// Open → HalfOpen; does not handle HalfOpen → Closed (success) or
    /// HalfOpen → Open (probe failure).
    pub fn cooldown_reset(&mut self) {
        if self.inner.state == BreakerState::Open {
            if let Some(t) = self.inner.last_state_change {
                let elapsed = t.elapsed();
                if elapsed >= self.inner.cooldown {
                    self.inner.state = BreakerState::HalfOpen;
                    self.inner.last_state_change = Some(Instant::now());
                }
            }
        }
    }
}

impl Default for ProviderBreaker {
    fn default() -> Self {
        Self::new(5, 60, 20)
    }
}
