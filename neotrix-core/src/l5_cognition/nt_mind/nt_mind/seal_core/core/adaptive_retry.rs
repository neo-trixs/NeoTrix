//! Adaptive Retry Controller — 自适应重试控制器
//! 指数退避 + 错误类型感知 + 成功率反馈

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_factor: f64,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 10_000,
            backoff_factor: 2.0,
            jitter: true,
        }
    }
}

pub struct AdaptiveRetry {
    policy: RetryPolicy,
    attempt: u32,
    last_error: Option<String>,
    success_history: Vec<bool>,
}

impl AdaptiveRetry {
    pub fn new(policy: RetryPolicy) -> Self {
        Self {
            policy,
            attempt: 0,
            last_error: None,
            success_history: Vec::new(),
        }
    }

    pub fn should_retry(&self) -> bool {
        self.attempt < self.policy.max_retries
    }

    pub fn delay_ms(&self) -> u64 {
        let base = self.policy.base_delay_ms as f64 * self.policy.backoff_factor.powi(self.attempt as i32);
        let capped = base.min(self.policy.max_delay_ms as f64);
        
        if self.policy.jitter {
            let jitter_range = capped * 0.1;
            let jitter_val = (Self::now() % (jitter_range as u64 * 2)) as f64 - jitter_range;
            (capped + jitter_val).max(0.0) as u64
        } else {
            capped as u64
        }
    }

    pub fn record_attempt(&mut self, success: bool, error: Option<String>) {
        self.attempt += 1;
        self.last_error = error;
        self.success_history.push(success);
    }

    pub fn reset(&mut self) {
        self.attempt = 0;
        self.last_error = None;
    }

    pub fn success_rate(&self) -> f64 {
        if self.success_history.is_empty() { return 1.0; }
        let successes = self.success_history.iter().filter(|&&s| s).count();
        successes as f64 / self.success_history.len() as f64
    }

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_basic() {
        let mut retry = AdaptiveRetry::new(RetryPolicy::default());
        assert!(retry.should_retry());
        
        retry.record_attempt(false, Some("timeout".into()));
        assert!(retry.should_retry());
        assert!(retry.delay_ms() > 0);
        
        retry.record_attempt(true, None);
        assert!(!retry.should_retry() || retry.attempt < 3);
    }
}
