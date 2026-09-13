//! Rate Limiter — 请求限流器
//!
//! 吸收 KB 经验:
//! - 令牌桶算法
//! - 滑动窗口
//! - 分布式限流
//! - 自适应限流

use std::collections::{HashMap, VecDeque};

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// 限流器
pub struct RateLimiter {
    limiters: HashMap<String, TokenBucket>,
    config: RateLimiterConfig,
    stats: RateLimiterStats,
}

/// 限流器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterConfig {
    pub default_rate: u32,
    pub default_burst: u32,
    pub window_size: Duration,
    pub adaptive: bool,
    pub 分布式: bool,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            default_rate: 100, // 100 requests per second
            default_burst: 200,
            window_size: Duration::from_secs(1),
            adaptive: true,
            分布式: false,
        }
    }
}

/// 令牌桶
pub struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
    last_access: Instant,
    stats: BucketStats,
}

/// 桶统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketStats {
    pub total_requests: u64,
    pub allowed_requests: u64,
    pub denied_requests: u64,
    pub avg_wait_time: Duration,
}

/// 限流器统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterStats {
    pub total_requests: u64,
    pub allowed_requests: u64,
    pub denied_requests: u64,
    pub active_limiters: usize,
}

/// 限流结果
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed,
    Denied,
    WaitFor(Duration),
}

/// 滑动窗口计数器
pub struct SlidingWindowCounter {
    window: VecDeque<Instant>,
    window_size: Duration,
    max_count: u32,
}

impl TokenBucket {
    /// 创建新的令牌桶
    pub fn new(max_tokens: f64, refill_rate: f64) -> Self {
        let now = Instant::now();
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: now,
            last_access: now,
            stats: BucketStats {
                total_requests: 0,
                allowed_requests: 0,
                denied_requests: 0,
                avg_wait_time: Duration::from_millis(0),
            },
        }
    }

    /// 尝试获取令牌
    pub fn try_acquire(&mut self, tokens: u32) -> RateLimitResult {
        self.refill();
        self.last_access = Instant::now();
        self.stats.total_requests += 1;

        if self.tokens >= tokens as f64 {
            self.tokens -= tokens as f64;
            self.stats.allowed_requests += 1;
            RateLimitResult::Allowed
        } else {
            self.stats.denied_requests += 1;
            let wait_time = Duration::from_secs_f64((tokens as f64 - self.tokens) / self.refill_rate);
            RateLimitResult::WaitFor(wait_time)
        }
    }

    /// 补充令牌
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
        self.last_refill = now;
    }

    /// 获取当前令牌数
    pub(crate) fn _available_tokens(&self) -> f64 {
        self.tokens
    }
}

impl SlidingWindowCounter {
    /// 创建新的滑动窗口计数器
    pub fn new(window_size: Duration, max_count: u32) -> Self {
        Self {
            window: VecDeque::new(),
            window_size,
            max_count,
        }
    }

    /// 尝试请求
    pub fn try_request(&mut self) -> RateLimitResult {
        let now = Instant::now();

        // 清理过期条目
        while let Some(&front) = self.window.front() {
            if now.duration_since(front) > self.window_size {
                self.window.pop_front();
            } else {
                break;
            }
        }

        if self.window.len() < self.max_count as usize {
            self.window.push_back(now);
            RateLimitResult::Allowed
        } else {
            // 计算需要等待的时间
            if let Some(&oldest) = self.window.front() {
                let wait_time = self.window_size - now.duration_since(oldest);
                RateLimitResult::WaitFor(wait_time)
            } else {
                RateLimitResult::Allowed
            }
        }
    }

    /// 获取当前窗口内的请求数
    pub fn current_count(&self) -> usize {
        self.window.len()
    }
}

impl RateLimiter {
    /// 创建新的限流器
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            limiters: HashMap::new(),
            config,
            stats: RateLimiterStats {
                total_requests: 0,
                allowed_requests: 0,
                denied_requests: 0,
                active_limiters: 0,
            },
        }
    }

    /// 获取或创建限流器
    pub fn get_or_create(&mut self, name: &str, rate: Option<u32>, burst: Option<u32>) -> &mut TokenBucket {
        self.limiters
            .entry(name.to_string())
            .or_insert_with(|| {
                let rate = rate.unwrap_or(self.config.default_rate);
                let burst = burst.unwrap_or(self.config.default_burst);
                TokenBucket::new(burst as f64, rate as f64)
            })
    }

    /// 尝试请求
    pub fn try_request(&mut self, name: &str, tokens: u32) -> RateLimitResult {
        self.stats.total_requests += 1;

        if let Some(limiter) = self.limiters.get_mut(name) {
            let result = limiter.try_acquire(tokens);
            match &result {
                RateLimitResult::Allowed => self.stats.allowed_requests += 1,
                RateLimitResult::Denied | RateLimitResult::WaitFor(_) => self.stats.denied_requests += 1,
            }
            result
        } else {
            // 创建默认限流器
            let mut limiter = TokenBucket::new(
                self.config.default_burst as f64,
                self.config.default_rate as f64,
            );
            let result = limiter.try_acquire(tokens);
            self.limiters.insert(name.to_string(), limiter);
            self.stats.active_limiters += 1;
            result
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> &RateLimiterStats {
        &self.stats
    }

    /// 清理不活跃的限流器
    pub fn cleanup(&mut self, max_age: Duration) {
        let now = Instant::now();
        let before_count = self.limiters.len();
        
        self.limiters.retain(|_name, bucket| {
            now.duration_since(bucket.last_access) < max_age
        });
        
        let after_count = self.limiters.len();
        if before_count != after_count {
            tracing::debug!(
                "Rate limiter cleanup: removed {} inactive buckets ({} remaining)",
                before_count - after_count,
                after_count
            );
        }
    }
}
