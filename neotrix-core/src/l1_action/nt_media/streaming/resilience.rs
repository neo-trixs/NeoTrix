//! resilience — 从 `streaming.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::time::{Duration, Instant};

use std::sync::atomic::{Ordering, AtomicU64};
use std::time::SystemTime;

// ═══════════════════════════════════════════════════════════════════════════
// StallDetector — monitors per-chunk download progress
// ═══════════════════════════════════════════════════════════════════════════

pub struct StallDetector {
    last_bytes: AtomicU64,
    last_check: Instant,
    timeout: Duration,
}

impl StallDetector {
    pub fn new(timeout: Duration) -> Self {
        Self {
            last_bytes: AtomicU64::new(0),
            last_check: Instant::now(),
            timeout,
        }
    }

    /// Returns true if no progress since last check and timeout elapsed.
    pub fn check(&mut self, current_bytes: u64) -> bool {
        let prev = self.last_bytes.swap(current_bytes, Ordering::Relaxed);
        if current_bytes != prev {
            self.last_check = Instant::now();
            return false;
        }
        self.last_check.elapsed() > self.timeout
    }

    pub fn record_progress(&mut self) {
        self.last_check = Instant::now();
    }

    pub fn is_stalled(&self) -> bool {
        self.last_check.elapsed() > self.timeout
    }

    pub fn reset(&mut self) {
        self.last_check = Instant::now();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RetryPolicy — n² backoff with jitter
// ═══════════════════════════════════════════════════════════════════════════

pub struct RetryPolicy {
    pub(crate) max_retries: u32,
    base_delay: Duration,
    delay_table: Vec<Duration>,
}

impl RetryPolicy {
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            base_delay: Duration::from_secs(1),
            delay_table: Vec::new(),
        }
    }

    pub fn with_delay_table(max_retries: u32, delay_table: Vec<Duration>) -> Self {
        Self {
            max_retries,
            base_delay: Duration::from_secs(1),
            delay_table,
        }
    }

    /// Compute delay for attempt number (1-indexed).
    /// Returns None if retries exhausted (or attempt == 0).
    /// 表模式：按表取（超表长用末项封顶），确定性；空表回退 n²+jitter。
    pub fn delay(&self, attempt: u32) -> Option<Duration> {
        if attempt == 0 || attempt > self.max_retries {
            return None;
        }
        if !self.delay_table.is_empty() {
            let idx = (attempt as usize - 1).min(self.delay_table.len() - 1);
            return Some(self.delay_table[idx]);
        }
        let base_ms = (attempt as u64) * (attempt as u64) * self.base_delay.as_millis() as u64;
        let capped = base_ms.min(30_000);
        let jitter = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64
            % self.base_delay.as_millis() as u64;
        Some(Duration::from_millis(capped + jitter))
    }

    /// Returns true if the error is retryable (server errors, timeouts, connection resets).
    pub fn is_retryable(err: &reqwest::Error) -> bool {
        if err.is_timeout() || err.is_connect() {
            return true;
        }
        if let Some(status) = err.status() {
            return matches!(status.as_u16(), 429 | 500..=599);
        }
        false
    }

    pub fn delay_for(&self, attempt: u32) -> Option<Duration> {
        self.delay(attempt)
    }

    pub fn is_retryable_status(status: u16) -> bool {
        matches!(status, 429 | 500..=599)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TokenBucket — simple bandwidth throttle
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // bytes per second
    last_refill: Instant,
}

impl TokenBucket {
    pub(crate) fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }

    /// Try to consume `amount` bytes. Blocks until tokens are available.
    /// Returns actual bytes allowed (may be less than amount if capped).
    pub(crate) async fn consume(&mut self, amount: u64) -> u64 {
        self.refill();
        let amount_f = amount as f64;
        if self.tokens >= amount_f {
            self.tokens -= amount_f;
            return amount;
        }
        // Not enough tokens — wait for refill
        let deficit = amount_f - self.tokens;
        let wait_secs = deficit / self.refill_rate;
        tokio::time::sleep(Duration::from_secs_f64(wait_secs)).await;
        self.refill();
        let allowed = self.tokens.min(amount_f);
        self.tokens -= allowed;
        allowed as u64
    }
}
