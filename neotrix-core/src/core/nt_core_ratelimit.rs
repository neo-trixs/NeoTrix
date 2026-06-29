// FUTURE - not yet wired
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// 令牌桶算法 — 平滑速率限制
///
/// 受 OmniGet host_limiter + YtRateLimiter 启发
#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: u32,
    tokens: f64,
    refill_per_sec: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_per_sec: f64) -> Self {
        Self {
            capacity,
            tokens: capacity as f64,
            refill_per_sec,
            last_refill: Instant::now(),
        }
    }

    /// 每 60 秒 N 次请求
    pub fn per_minute(max_per_minute: u32) -> Self {
        Self::new(max_per_minute, max_per_minute as f64 / 60.0)
    }

    /// 每 3600 秒 N 次请求
    pub fn per_hour(max_per_hour: u32) -> Self {
        Self::new(max_per_hour, max_per_hour as f64 / 3600.0)
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity as f64);
        self.last_refill = now;
    }

    /// 尝试消耗一个令牌。返回等待时间（如果有）。
    pub fn try_consume(&mut self) -> Result<(), Duration> {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Ok(())
        } else {
            let wait = Duration::from_secs_f64((1.0 - self.tokens) / self.refill_per_sec);
            Err(wait)
        }
    }

    pub fn available(&mut self) -> u32 {
        self.refill();
        self.tokens as u32
    }

    pub fn reset(&mut self) {
        self.tokens = self.capacity as f64;
        self.last_refill = Instant::now();
    }
}

/// 滑动窗口速率限制器 — 防止突发
#[derive(Debug, Clone)]
pub struct SlidingWindowRateLimiter {
    max_calls: u32,
    window_secs: f64,
    timestamps: Vec<Instant>,
}

impl SlidingWindowRateLimiter {
    pub fn new(max_calls: u32, window_secs: f64) -> Self {
        Self {
            max_calls,
            window_secs,
            timestamps: Vec::new(),
        }
    }

    pub fn per_minute(max_per_minute: u32) -> Self {
        Self::new(max_per_minute, 60.0)
    }

    pub fn can_proceed(&mut self) -> bool {
        let now = Instant::now();
        let cutoff = now - Duration::from_secs_f64(self.window_secs);
        self.timestamps.retain(|t| *t > cutoff);
        if self.timestamps.len() < self.max_calls as usize {
            self.timestamps.push(now);
            true
        } else {
            false
        }
    }

    pub fn remaining(&self) -> u32 {
        let now = Instant::now();
        let cutoff = now - Duration::from_secs_f64(self.window_secs);
        let active = self.timestamps.iter().filter(|t| **t > cutoff).count();
        self.max_calls.saturating_sub(active as u32)
    }

    pub fn reset(&mut self) {
        self.timestamps.clear();
    }
}

/// 域级别速率限制器 — 每域令牌桶
pub struct DomainRateLimiter {
    buckets: Mutex<HashMap<String, TokenBucket>>,
    default_capacity: u32,
    default_refill: f64,
}

impl DomainRateLimiter {
    pub fn new(default_capacity: u32, default_refill_per_sec: f64) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            default_capacity,
            default_refill: default_refill_per_sec,
        }
    }

    pub fn per_minute(max_per_minute: u32) -> Self {
        Self::new(max_per_minute, max_per_minute as f64 / 60.0)
    }

    /// 为特定域注册自定义限制
    pub fn register_domain(&self, domain: &str, capacity: u32, refill_per_sec: f64) {
        if let Ok(mut buckets) = self.buckets.lock() {
            buckets.insert(
                domain.to_string(),
                TokenBucket::new(capacity, refill_per_sec),
            );
        }
    }

    /// 尝试为域消耗一个令牌
    pub fn try_consume(&self, domain: &str) -> Result<(), Duration> {
        let mut buckets = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        let bucket = buckets
            .entry(domain.to_string())
            .or_insert_with(|| TokenBucket::new(self.default_capacity, self.default_refill));
        bucket.try_consume()
    }

    /// 检查域是否可以处理请求（不消耗）
    pub fn can_proceed(&self, domain: &str) -> bool {
        if let Ok(mut buckets) = self.buckets.lock() {
            let bucket = buckets
                .entry(domain.to_string())
                .or_insert_with(|| TokenBucket::new(self.default_capacity, self.default_refill));
            bucket.available() > 0
        } else {
            true
        }
    }

    /// 获取特定域的可用令牌数
    pub fn available(&self, domain: &str) -> u32 {
        if let Ok(mut buckets) = self.buckets.lock() {
            buckets.get_mut(domain).map(|b| b.available()).unwrap_or(0)
        } else {
            0
        }
    }

    /// 重置特定域
    pub fn reset_domain(&self, domain: &str) {
        if let Ok(mut buckets) = self.buckets.lock() {
            if let Some(bucket) = buckets.get_mut(domain) {
                bucket.reset();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_init() {
        let mut tb = TokenBucket::new(10, 1.0);
        assert_eq!(tb.available(), 10);
        assert!(tb.try_consume().is_ok());
        assert_eq!(tb.available(), 9);
    }

    #[test]
    fn test_token_bucket_exhaustion() {
        let mut tb = TokenBucket::new(2, 0.0);
        assert!(tb.try_consume().is_ok());
        assert!(tb.try_consume().is_ok());
        assert!(tb.try_consume().is_err());
    }

    #[tokio::test]
    async fn test_token_bucket_refill() {
        let mut tb = TokenBucket::new(5, 100.0);
        for _ in 0..5 {
            assert!(tb.try_consume().is_ok());
        }
        // 快速补满
        tokio::time::sleep(Duration::from_millis(50)).await;
        let result = tb.try_consume();
        assert!(result.is_ok() || tb.available() > 0);
    }

    #[test]
    fn test_sliding_window_rate_limiter() {
        let mut sw = SlidingWindowRateLimiter::new(3, 60.0);
        assert!(sw.can_proceed());
        assert!(sw.can_proceed());
        assert!(sw.can_proceed());
        assert!(!sw.can_proceed());
        assert_eq!(sw.remaining(), 0);
    }

    #[test]
    fn test_domain_rate_limiter_independent() {
        let rl = DomainRateLimiter::new(5, 1.0);
        assert!(rl.try_consume("api.openai.com").is_ok());
        assert!(rl.try_consume("api.anthropic.com").is_ok());
        // 消耗 openai 到满
        for _ in 0..4 {
            let _ = rl.try_consume("api.openai.com");
        }
        assert_eq!(rl.available("api.openai.com"), 0);
        assert_eq!(rl.available("api.anthropic.com"), 4);
    }
}
