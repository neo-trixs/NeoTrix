use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    failure_rate_threshold: f64,
    cooldown_duration: Duration,
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    half_open_max_calls: u32,
    half_open_calls: u32,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown_duration: Duration) -> Self {
        Self {
            failure_threshold,
            failure_rate_threshold: 0.5,
            cooldown_duration,
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            half_open_max_calls: 1,
            half_open_calls: 0,
        }
    }

    pub fn should_allow(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if Instant::now() > last_failure + self.cooldown_duration {
                        self.state = CircuitState::HalfOpen;
                        self.half_open_calls = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                if self.half_open_calls < self.half_open_max_calls {
                    self.half_open_calls += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn record_result(&mut self, success: bool) {
        match self.state {
            CircuitState::Closed => {
                if success {
                    self.failure_count = 0;
                } else {
                    self.failure_count += 1;
                    if self.failure_count >= self.failure_threshold {
                        self.state = CircuitState::Open;
                        self.last_failure_time = Some(Instant::now());
                    }
                }
            }
            CircuitState::HalfOpen => {
                if success {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                } else {
                    self.state = CircuitState::Open;
                    self.last_failure_time = Some(Instant::now());
                }
            }
            CircuitState::Open => {}
        }
    }

    pub fn state(&self) -> &CircuitState {
        &self.state
    }
}