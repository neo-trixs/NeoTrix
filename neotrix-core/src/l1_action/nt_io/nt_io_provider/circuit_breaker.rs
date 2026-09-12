use std::collections::VecDeque;
use std::time::{Duration, Instant};

use neotrix_types::shared::CircuitBreaker as CanonicalCircuitBreaker;
pub use neotrix_types::shared::BreakerState;

/// Provider circuit breaker — wraps canonical `CircuitBreaker` with
/// sliding-window failure tracking, force-open, and health-penalty scoring.
#[derive(Debug, Clone)]
pub(crate) struct ProviderBreaker {
    inner: CanonicalCircuitBreaker,
    sliding_window: VecDeque<bool>,
    window_size: usize,
}

/// Backward-compatible alias.
pub type CircuitBreaker = ProviderBreaker;

impl ProviderBreaker {
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

    pub fn on_success(&mut self) {
        self.sliding_window.push_back(true);
        if self.sliding_window.len() > self.window_size {
            self.sliding_window.pop_front();
        }
        self.inner.on_success();
    }

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

    pub fn failure_rate(&self) -> f64 {
        let len = self.sliding_window.len();
        if len == 0 {
            return 0.0;
        }
        let failures = self.sliding_window.iter().filter(|&&s| !s).count();
        failures as f64 / len as f64
    }

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
