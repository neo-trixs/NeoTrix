use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: BreakerState,
    failure_count: u64,
    failure_threshold: u64,
    cooldown: Duration,
    last_state_change: Option<Instant>,
    half_open_probes_used: u64,
    half_open_max_probes: u64,
    sliding_window: VecDeque<bool>,
    window_size: usize,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, cooldown_secs: u64, window_size: usize) -> Self {
        Self {
            state: BreakerState::Closed,
            failure_count: 0,
            failure_threshold,
            cooldown: Duration::from_secs(cooldown_secs),
            last_state_change: None,
            half_open_probes_used: 0,
            half_open_max_probes: 3,
            sliding_window: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    pub fn state(&self) -> BreakerState {
        self.state
    }

    pub fn set_half_open_max_probes(&mut self, n: u64) {
        self.half_open_max_probes = n;
    }

    pub fn half_open_max_probes(&self) -> u64 {
        self.half_open_max_probes
    }

    pub fn health_penalty(&self) -> f64 {
        match self.state {
            BreakerState::Closed => 1.0,
            BreakerState::HalfOpen => 0.5,
            BreakerState::Open => 0.0,
        }
    }

    pub fn is_available(&self) -> bool {
        match self.state {
            BreakerState::Closed => true,
            BreakerState::HalfOpen => self.half_open_probes_used < self.half_open_max_probes,
            BreakerState::Open => {
                if let Some(t) = self.last_state_change {
                    t.elapsed() >= self.cooldown
                } else {
                    false
                }
            }
        }
    }

    /// 强制 Open — 配额耗尽等非瞬时错误直接熔断, 不依赖连续失败计数。
    /// 冷却期沿用默认 cooldown, 期间 select_best 会跳过该 provider。
    pub fn force_open(&mut self) {
        self.state = BreakerState::Open;
        self.last_state_change = Some(Instant::now());
        self.half_open_probes_used = 0;
    }

    /// 熔断冷却是否已过 (配额恢复探测窗口)
    pub fn cooldown_elapsed(&self) -> bool {
        match self.last_state_change {
            Some(t) => t.elapsed() >= self.cooldown,
            None => true,
        }
    }

    pub fn on_success(&mut self) {
        self.sliding_window.push_back(true);
        if self.sliding_window.len() > self.window_size {
            self.sliding_window.pop_front();
        }

        match self.state {
            BreakerState::HalfOpen => {
                self.half_open_probes_used += 1;
                if self.half_open_probes_used >= self.half_open_max_probes {
                    self.state = BreakerState::Closed;
                    self.failure_count = 0;
                    self.half_open_probes_used = 0;
                    self.last_state_change = Some(Instant::now());
                }
            }
            BreakerState::Open => {
                self.state = BreakerState::HalfOpen;
                self.half_open_probes_used = 1;
                self.last_state_change = Some(Instant::now());
            }
            BreakerState::Closed => {
                self.failure_count = self.failure_count.saturating_sub(1);
            }
        }
    }

    pub fn on_failure(&mut self) {
        self.sliding_window.push_back(false);
        if self.sliding_window.len() > self.window_size {
            self.sliding_window.pop_front();
        }

        self.failure_count += 1;

        match self.state {
            BreakerState::Closed => {
                let recent_failures = self.sliding_window.iter().filter(|&&s| !s).count();
                if recent_failures >= self.failure_threshold as usize
                    || self.failure_count >= self.failure_threshold
                {
                    self.state = BreakerState::Open;
                    self.last_state_change = Some(Instant::now());
                }
            }
            BreakerState::HalfOpen => {
                self.state = BreakerState::Open;
                self.last_state_change = Some(Instant::now());
                self.half_open_probes_used = 0;
            }
            BreakerState::Open => {}
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
        if self.state == BreakerState::Open {
            let elapsed = self.last_state_change.map(|t| t.elapsed()).unwrap_or_default();
            if elapsed >= self.cooldown {
                self.state = BreakerState::HalfOpen;
                self.last_state_change = Some(Instant::now());
            }
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, 60, 20)
    }
}
