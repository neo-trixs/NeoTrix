use std::time::{Duration, Instant};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continue_on_absorb() {
        let mut d = StagnationDetector::new();
        let sig = d.observe(true, true, 0, 0.5, true, false);
        assert_eq!(sig, StagnationSignal::Continue);
    }

    #[test]
    fn test_stop_after_minor_errors() -> Result<(), String> {
        let mut d = StagnationDetector {
            zero_reward_pause: 100,
            error_only_pause: 100,
            pause_duration_secs: 0,
            ..Default::default()
        };
        for _ in 0..25 {
            let sig = d.observe(false, false, 0, 0.5, false, true);
            if let StagnationSignal::Stop(_) = sig {
                return Ok(());
            }
        }
        Err("should have stopped after 20 minor-error cycles".into())
    }

    #[test]
    fn test_pause_after_pure_errors() -> Result<(), String> {
        let mut d = StagnationDetector {
            pause_duration_secs: 0,
            ..Default::default()
        };
        for i in 0..10 {
            let sig = d.observe(false, false, 2, 0.0, false, false);
            if i >= 8 {
                if matches!(sig, StagnationSignal::Pause(_, _)) {
                    return Ok(());
                }
            }
        }
        Err("should have paused after 8+ pure-error cycles".into())
    }

    #[test]
    fn test_stop_after_no_absorb() -> Result<(), String> {
        let mut d = StagnationDetector {
            stop_threshold: 5,
            pause_duration_secs: 0,
            ..Default::default()
        };
        for _ in 0..6 {
            let sig = d.observe(false, false, 0, 0.0, false, false);
            if let StagnationSignal::Stop(_) = sig {
                return Ok(());
            }
        }
        Err("should have stopped after 5 no-absorb cycles".into())
    }

    #[test]
    fn test_reset_clears_counters() {
        let mut d = StagnationDetector::new();
        for _ in 0..6 {
            d.observe(false, false, 0, 0.0, false, true);
        }
        d.reset();
        let sig = d.observe(true, true, 0, 0.5, true, false);
        assert_eq!(sig, StagnationSignal::Continue);
    }

    #[test]
    fn test_pause_check_via_signal() {
        let mut d = StagnationDetector {
            pause_duration_secs: 0,
            ..Default::default()
        };
        assert!(!d.is_paused());
        for _ in 0..8 {
            d.observe(false, false, 2, 0.0, false, false);
        }
        let sig = d.observe(false, false, 2, 0.0, false, false);
        assert!(
            matches!(sig, StagnationSignal::Pause(_, _)),
            "expected Pause, got {:?}",
            sig
        );
    }

    /// 端到端集成测试: StagnationDetector → SelfIteratingBrain 全链路
    /// 离线运行, 不依赖网络
    #[test]
    fn test_stagnation_integration_with_brain() {
        let mut brain = super::super::SelfIteratingBrain::new();
        brain.stagnation = StagnationDetector {
            pause_duration_secs: 0,
            ..Default::default()
        };

        // 1. 首次 SEAL loop 应正常通过
        let r1 = brain.run_seal_loop("test integration", None, None);
        assert!(r1.is_ok(), "first SEAL should succeed");

        // 2. 用短任务跑几次 — 模拟无信息循环
        for i in 0..2 {
            let r = brain.run_seal_loop(&format!("task_{}", i), None, None);
            assert!(
                r.is_ok(),
                "stagnation gate should return Ok, not Err at iter {}",
                i
            );
        }

        // 3. 验证 iteration 正常增长
        assert!(
            brain.iteration >= 2,
            "brain should have iterated >=2 times, got {}",
            brain.iteration
        );
    }

    #[test]
    fn test_stagnation_with_real_absorb_cancels_stall() {
        let mut d = StagnationDetector {
            pause_duration_secs: 0,
            ..Default::default()
        };

        for _ in 0..10 {
            d.observe(false, false, 0, 0.0, false, true);
        }

        // absorb 事件应重置 stagnation
        d.observe(true, true, 0, 0.5, true, false);
        let stats = d.stats();
        assert_eq!(
            stats.consecutive_no_absorb, 0,
            "absorb should reset no-absorb counter"
        );
        assert_eq!(
            stats.consecutive_zero_reward, 0,
            "absorb should reset zero-reward counter"
        );
        assert_eq!(
            stats.consecutive_minor_errors, 0,
            "absorb should reset minor-errors counter"
        );
    }

    /// 验证 evolve 级别的停滞场景: 所有维度=纯错误, 最终触发 Stop
    #[test]
    fn test_evolve_level_stagnation_full_stop() -> Result<(), String> {
        let mut d = StagnationDetector {
            stop_threshold: 5,
            error_only_pause: 100,
            zero_reward_pause: 100,
            pause_duration_secs: 0,
            ..Default::default()
        };

        // 模拟 evolve 中 frontier empty 场景: 无吸收 + 无抓取 + 无新来源
        for i in 0..10 {
            let sig = d.observe(false, false, 0, 0.0, false, false);
            if let StagnationSignal::Stop(_) = sig {
                assert!(
                    i >= 4,
                    "should stop after {}+ cycles, stopped at {}",
                    d.stop_threshold,
                    i
                );
                return Ok(());
            }
        }
        Err(format!(
            "should have stopped after {} no-absorb cycles",
            d.stop_threshold
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StagnationSignal {
    Continue,
    Pause(u64, String),
    Stop(String),
}

pub struct StagnationDetector {
    pub pause_threshold: u64,
    pub stop_threshold: u64,
    pub zero_reward_pause: u64,
    pub error_only_pause: u64,
    pub pause_duration_secs: u64,

    consecutive_no_absorb: u64,
    consecutive_pure_error: u64,
    consecutive_zero_reward: u64,
    consecutive_no_new_sources: u64,
    consecutive_minor_errors: u64,
    total_cycles: u64,
    last_reward: f64,
    pause_until: Option<Instant>,

    /// Negentropy tracking
    pub negentropy_plateau_threshold: u64,
    pub negentropy_decline_threshold: u64,
    consecutive_n_plateau: u64,
    consecutive_n_decline: u64,
    last_n_total: f64,
    last_n_trend: f64,
    n_history: Vec<f64>,
    pub negentropy_mode: bool,
}

impl Default for StagnationDetector {
    fn default() -> Self {
        Self {
            pause_threshold: 5,
            stop_threshold: 20,
            zero_reward_pause: 10,
            error_only_pause: 8,
            pause_duration_secs: 10,
            consecutive_no_absorb: 0,
            consecutive_pure_error: 0,
            consecutive_zero_reward: 0,
            consecutive_no_new_sources: 0,
            consecutive_minor_errors: 0,
            total_cycles: 0,
            last_reward: 0.0,
            pause_until: None,
            negentropy_plateau_threshold: 10,
            negentropy_decline_threshold: 5,
            consecutive_n_plateau: 0,
            consecutive_n_decline: 0,
            last_n_total: 0.0,
            last_n_trend: 0.0,
            n_history: Vec::with_capacity(30),
            negentropy_mode: false,
        }
    }
}

impl StagnationDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_paused(&self) -> bool {
        self.pause_until.is_some_and(|t| Instant::now() < t)
    }

    pub fn observe(
        &mut self,
        absorbed: bool,
        fetched: bool,
        error_count: usize,
        reward: f64,
        new_sources: bool,
        minor_errors: bool,
    ) -> StagnationSignal {
        self.total_cycles += 1;

        if self.is_paused() {
            return StagnationSignal::Continue;
        }

        if absorbed || fetched && !minor_errors {
            self.consecutive_no_absorb = 0;
            self.consecutive_pure_error = 0;
            self.consecutive_minor_errors = 0;
        } else {
            self.consecutive_no_absorb += 1;
        }

        if error_count > 0 && !fetched {
            self.consecutive_pure_error += 1;
        } else {
            self.consecutive_pure_error = 0;
        }

        if reward.abs() < 1e-6 && !absorbed {
            self.consecutive_zero_reward += 1;
        } else {
            self.consecutive_zero_reward = 0;
        }

        if !new_sources {
            self.consecutive_no_new_sources += 1;
        } else {
            self.consecutive_no_new_sources = 0;
        }

        if minor_errors && !absorbed {
            self.consecutive_minor_errors += 1;
        } else {
            self.consecutive_minor_errors = 0;
        }

        self.last_reward = reward;

        if self.consecutive_minor_errors >= self.stop_threshold {
            return StagnationSignal::Stop(format!(
                "连续 {} 次纯 minor errors, 无吸收 → 停止",
                self.consecutive_minor_errors
            ));
        }

        if self.consecutive_pure_error >= self.error_only_pause {
            self.pause_until = Some(Instant::now() + Duration::from_secs(self.pause_duration_secs));
            return StagnationSignal::Pause(
                self.pause_duration_secs,
                format!(
                    "连续 {} 次纯错误循环, 暂停 {}s",
                    self.consecutive_pure_error, self.pause_duration_secs
                ),
            );
        }

        if self.consecutive_zero_reward >= self.zero_reward_pause {
            self.pause_until = Some(Instant::now() + Duration::from_secs(self.pause_duration_secs));
            return StagnationSignal::Pause(
                self.pause_duration_secs,
                format!(
                    "连续 {} 次零奖励循环, 暂停 {}s",
                    self.consecutive_zero_reward, self.pause_duration_secs
                ),
            );
        }

        if self.consecutive_no_absorb >= self.stop_threshold {
            return StagnationSignal::Stop(format!(
                "连续 {} 次无吸收循环, frontier 可能枯竭",
                self.consecutive_no_absorb
            ));
        }

        StagnationSignal::Continue
    }

    pub fn record_negentropy(&mut self, n_total: f64) {
        self.last_n_total = n_total;
        self.n_history.push(n_total);
        if self.n_history.len() > 30 {
            self.n_history.remove(0);
        }

        if self.n_history.len() >= 5 {
            let recent: Vec<f64> = self.n_history.iter().rev().take(5).copied().collect();
            let _mean = recent.iter().sum::<f64>() / recent.len() as f64;
            let first_deriv: Vec<f64> = (1..recent.len())
                .map(|i| recent[i] - recent[i - 1])
                .collect();
            let trend = first_deriv.iter().sum::<f64>() / first_deriv.len() as f64;
            self.last_n_trend = trend;

            if trend.abs() < 0.001 {
                self.consecutive_n_plateau += 1;
                self.consecutive_n_decline = 0;
            } else if trend < -0.01 {
                self.consecutive_n_decline += 1;
                self.consecutive_n_plateau = 0;
            } else {
                self.consecutive_n_plateau = 0;
                self.consecutive_n_decline = 0;
            }
        }
    }

    pub fn negentropy_stagnation(&self) -> Option<StagnationSignal> {
        if !self.negentropy_mode {
            return None;
        }
        if self.consecutive_n_decline >= self.negentropy_decline_threshold {
            Some(StagnationSignal::Stop(format!(
                "N_total declining for {} rounds (trend={:.4}), stopping evolution",
                self.consecutive_n_decline, self.last_n_trend
            )))
        } else if self.consecutive_n_plateau >= self.negentropy_plateau_threshold {
            Some(StagnationSignal::Pause(
                self.pause_duration_secs,
                format!(
                    "N_total plateau for {} rounds (trend={:.4}), triggering exploration",
                    self.consecutive_n_plateau, self.last_n_trend
                ),
            ))
        } else {
            None
        }
    }

    pub fn enable_negentropy_mode(&mut self) {
        self.negentropy_mode = true;
    }

    pub fn reset(&mut self) {
        self.consecutive_no_absorb = 0;
        self.consecutive_pure_error = 0;
        self.consecutive_zero_reward = 0;
        self.consecutive_no_new_sources = 0;
        self.consecutive_minor_errors = 0;
        self.consecutive_n_plateau = 0;
        self.consecutive_n_decline = 0;
        self.pause_until = None;
    }

    pub fn stats(&self) -> StagnationStats {
        StagnationStats {
            total_cycles: self.total_cycles,
            consecutive_no_absorb: self.consecutive_no_absorb,
            consecutive_pure_error: self.consecutive_pure_error,
            consecutive_zero_reward: self.consecutive_zero_reward,
            consecutive_no_new_sources: self.consecutive_no_new_sources,
            consecutive_minor_errors: self.consecutive_minor_errors,
            paused: self.is_paused(),
            negentropy_mode: self.negentropy_mode,
            consecutive_n_plateau: self.consecutive_n_plateau,
            last_n_total: self.last_n_total,
            last_n_trend: self.last_n_trend,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StagnationStats {
    pub total_cycles: u64,
    pub consecutive_no_absorb: u64,
    pub consecutive_pure_error: u64,
    pub consecutive_zero_reward: u64,
    pub consecutive_no_new_sources: u64,
    pub consecutive_minor_errors: u64,
    pub paused: bool,
    pub negentropy_mode: bool,
    pub consecutive_n_plateau: u64,
    pub last_n_total: f64,
    pub last_n_trend: f64,
}
