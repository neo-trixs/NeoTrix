use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_per_sec: f64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate: refill_per_sec,
            last_refill: Instant::now(),
        }
    }

    pub fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    pub fn consume(&mut self, tokens: f64) {
        self.refill();
        self.tokens = (self.tokens - tokens).max(0.0);
    }

    pub fn available(&self) -> f64 {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        (self.tokens + elapsed * self.refill_rate).min(self.capacity)
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
            self.last_refill = Instant::now();
        }
    }

    pub fn reset(&mut self) {
        self.tokens = self.capacity;
        self.last_refill = Instant::now();
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    rpm_bucket: TokenBucket,
    tpm_bucket: TokenBucket,
    max_retries: u32,
}

impl RateLimiter {
    pub fn new(rpm: f64, tpm: f64, max_retries: u32) -> Self {
        Self {
            rpm_bucket: TokenBucket::new(rpm, rpm / 60.0),
            tpm_bucket: TokenBucket::new(tpm, tpm / 60.0),
            max_retries,
        }
    }

    pub fn allow_request(&mut self, estimated_tokens: f64) -> bool {
        self.rpm_bucket.try_consume(1.0) && self.tpm_bucket.try_consume(estimated_tokens)
    }

    pub fn record_usage(&mut self, tokens: f64) {
        self.tpm_bucket.consume(tokens);
    }

    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }

    pub fn reset_all(&mut self) {
        self.rpm_bucket.reset();
        self.tpm_bucket.reset();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(60.0, 100000.0, 3)
    }
}
