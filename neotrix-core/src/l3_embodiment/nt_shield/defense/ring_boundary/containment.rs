//! Containment - 遏制器
//!
//! 异常行为遏制：隔离→降级→恢复

use std::time::{Duration, Instant};

/// 遏制状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainmentState {
    Normal,
    Isolated,
    Degraded,
    Recovering,
}

/// 遏制器
pub struct Containment {
    state: ContainmentState,
    isolation_start: Option<Instant>,
    isolation_timeout: Duration,
}

impl Containment {
    pub fn new() -> Self {
        Self {
            state: ContainmentState::Normal,
            isolation_start: None,
            isolation_timeout: Duration::from_secs(300), // 5 minutes
        }
    }

    /// 遏制异常行为
    pub fn contain(&mut self, severity: f64) -> ContainmentState {
        if severity > 0.8 {
            self.state = ContainmentState::Isolated;
            self.isolation_start = Some(Instant::now());
        } else if severity > 0.5 {
            self.state = ContainmentState::Degraded;
        }

        self.state
    }

    /// 检查是否可以恢复
    pub fn check_recovery(&mut self) -> ContainmentState {
        if let Some(start) = self.isolation_start {
            if start.elapsed() > self.isolation_timeout {
                self.state = ContainmentState::Recovering;
                self.isolation_start = None;
            }
        }

        self.state
    }

    /// 获取当前状态
    pub fn state(&self) -> ContainmentState {
        self.state
    }
}

impl Default for Containment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contain_high_severity() {
        let mut containment = Containment::new();
        let state = containment.contain(0.9);
        assert_eq!(state, ContainmentState::Isolated);
    }

    #[test]
    fn test_contain_medium_severity() {
        let mut containment = Containment::new();
        let state = containment.contain(0.6);
        assert_eq!(state, ContainmentState::Degraded);
    }

    #[test]
    fn test_contain_low_severity() {
        let mut containment = Containment::new();
        let state = containment.contain(0.3);
        assert_eq!(state, ContainmentState::Normal);
    }
}
