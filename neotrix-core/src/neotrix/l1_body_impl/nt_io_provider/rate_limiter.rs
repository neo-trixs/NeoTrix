use std::time::Instant;

/// 自适应最小调用间隔 pacer (Cumora AdaptivePacer 吸收, COORDINATION.md §3b)。
///
/// 固定 TokenBucket 处理稳态速率; 但 provider 配额是时变的, 静态参数无法
/// 感知瞬时限速 (429) 后需要全局放慢。本 pacer 维护一个「最小 spawn 间隔」:
/// - 每次 rate-limit 事件 (`on_rate_limited`) 将间隔 **翻倍** (封顶 `max_ms`)。
/// - 连续 `ok_threshold` (5) 次成功 (`on_ok`) 后 **减半** 回落向 base。
///
/// 用途: 多 agent / 多调用并发时, 把突发速率做成硬约束而非概率抖动的期望值 —
/// 确定性 pacing 比 random jitter 更能防住 thundering-herd。
#[derive(Debug, Clone)]
pub struct AdaptivePacer {
    base_ms: u64,
    current_ms: u64,
    max_ms: u64,
    consecutive_ok: u32,
    ok_threshold: u32,
    next: Instant,
}

impl AdaptivePacer {
    pub fn new(base_ms: u64) -> Self {
        Self {
            base_ms,
            current_ms: base_ms,
            max_ms: 8000,
            consecutive_ok: 0,
            ok_threshold: 5,
            next: Instant::now(),
        }
    }

    pub fn with_max(base_ms: u64, max_ms: u64) -> Self {
        let mut s = Self::new(base_ms);
        s.max_ms = max_ms;
        s
    }

    /// 当前生效的最小间隔 (诊断/测试用)。
    pub fn interval_ms(&self) -> u64 {
        self.current_ms
    }

    /// 距下次可调用还需等待多少毫秒 (0 = 立即可发)。
    pub fn wait_ms_until_next(&self) -> u64 {
        let now = Instant::now();
        if self.next > now {
            self.next.duration_since(now).as_millis() as u64
        } else {
            0
        }
    }

    /// 排队一次调用: 返回本次应等待的毫秒, 并推进全局计时。
    pub fn gate(&mut self) -> u64 {
        let now = Instant::now();
        let wait = if self.next > now {
            self.next.duration_since(now).as_millis() as u64
        } else {
            0
        };
        let earliest = if self.next > now { self.next } else { now };
        self.next = earliest + std::time::Duration::from_millis(self.current_ms);
        wait
    }

    /// rate-limit 事件: 间隔翻倍 (封顶), 并清零成功连击, 防止过快回落。
    pub fn on_rate_limited(&mut self) {
        let prev = self.current_ms;
        self.current_ms = self.current_ms.saturating_mul(2).min(self.max_ms).max(self.base_ms);
        self.consecutive_ok = 0;
        let _ = prev;
    }

    /// 一次干净调用 (无 rate-limit): 连续 ok_threshold 次后间隔减半回落向 base。
    pub fn on_ok(&mut self) {
        if self.current_ms == self.base_ms {
            return;
        }
        self.consecutive_ok += 1;
        if self.consecutive_ok >= self.ok_threshold {
            self.current_ms = (self.current_ms / 2).max(self.base_ms);
            self.consecutive_ok = 0;
        }
    }

    pub fn reset(&mut self) {
        self.current_ms = self.base_ms;
        self.consecutive_ok = 0;
        self.next = Instant::now();
    }
}

impl Default for AdaptivePacer {
    fn default() -> Self {
        Self::new(500)
    }
}

/// 双脑并发门 (Cumora BigBrainSemaphore + triage semaphore 吸收, COORDINATION.md §2/§3a)。
///
/// 大模型 (big brain, Frontier/Strong tier) 与小模型 (triage/support) 共享同一个
/// provider 账户与调用栈, 但各自应有独立的并发上限:
/// - `big_max`: 大模型并发 (Cumora 默认 6) — 昂贵、慢, 必须压住突发。
/// - `triage_max`: 小模型并发 (Cumora 默认 8) — 便宜, 可稍高, 但仍需上限
///   防 30s 超时链式雪崩 (小脑超时被误判为 rate-limit → 全电脑静默)。
///
/// 关键教训 (anti-pattern §1): **只 cap 一层而不 cap 另一层 = 没 cap** —
/// 两层共享同一 provider 配额, 必须成对配置。
#[derive(Debug, Clone)]
pub struct TieredSemaphore {
    big_max: u32,
    triage_max: u32,
    big_in_flight: u32,
    triage_in_flight: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrainTier {
    Big,
    Triage,
}

impl TieredSemaphore {
    pub fn new(big_max: u32, triage_max: u32) -> Self {
        Self {
            big_max,
            triage_max,
            big_in_flight: 0,
            triage_in_flight: 0,
        }
    }

    /// 尝试获取一个槽位; 该 tier 已满则返回 false (调用方应排队/降级)。
    pub fn try_acquire(&mut self, tier: BrainTier) -> bool {
        match tier {
            BrainTier::Big => {
                if self.big_in_flight < self.big_max {
                    self.big_in_flight += 1;
                    true
                } else {
                    false
                }
            }
            BrainTier::Triage => {
                if self.triage_in_flight < self.triage_max {
                    self.triage_in_flight += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn release(&mut self, tier: BrainTier) {
        match tier {
            BrainTier::Big => self.big_in_flight = self.big_in_flight.saturating_sub(1),
            BrainTier::Triage => self.triage_in_flight = self.triage_in_flight.saturating_sub(1),
        }
    }

    pub fn in_flight(&self, tier: BrainTier) -> u32 {
        match tier {
            BrainTier::Big => self.big_in_flight,
            BrainTier::Triage => self.triage_in_flight,
        }
    }

    pub fn max_for(&self, tier: BrainTier) -> u32 {
        match tier {
            BrainTier::Big => self.big_max,
            BrainTier::Triage => self.triage_max,
        }
    }
}

impl Default for TieredSemaphore {
    fn default() -> Self {
        Self::new(6, 8)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_pacer_doubles_on_rate_limit() {
        let mut pacer = AdaptivePacer::new(500);
        assert_eq!(pacer.interval_ms(), 500);
        pacer.on_rate_limited();
        assert_eq!(pacer.interval_ms(), 1000);
        pacer.on_rate_limited();
        assert_eq!(pacer.interval_ms(), 2000);
    }

    #[test]
    fn adaptive_pacer_caps_at_max() {
        let mut pacer = AdaptivePacer::with_max(500, 2000);
        for _ in 0..10 {
            pacer.on_rate_limited();
        }
        assert_eq!(pacer.interval_ms(), 2000);
    }

    #[test]
    fn adaptive_pacer_halves_after_ok_streak() {
        let mut pacer = AdaptivePacer::new(500);
        pacer.on_rate_limited();
        pacer.on_rate_limited();
        assert_eq!(pacer.interval_ms(), 2000);
        // 连续 5 次 ok → 减半回落
        for _ in 0..5 {
            pacer.on_ok();
        }
        assert_eq!(pacer.interval_ms(), 1000);
        for _ in 0..5 {
            pacer.on_ok();
        }
        assert_eq!(pacer.interval_ms(), 500);
        // base 处不再变化
        pacer.on_ok();
        assert_eq!(pacer.interval_ms(), 500);
    }

    #[test]
    fn adaptive_pacer_gate_enforces_spacing() {
        let mut pacer = AdaptivePacer::new(500);
        // 首次 gate 立即可发 (0 wait)
        let w1 = pacer.gate();
        assert_eq!(w1, 0);
        // 紧接第二次 gate 需等待当前间隔
        let w2 = pacer.gate();
        assert!(w2 > 0 && w2 <= 500);
    }

    #[test]
    fn tiered_semaphore_separate_tier_caps() {
        let mut sem = TieredSemaphore::new(2, 4);
        // big tier: 2 满
        assert!(sem.try_acquire(BrainTier::Big));
        assert!(sem.try_acquire(BrainTier::Big));
        assert!(!sem.try_acquire(BrainTier::Big));
        // triage tier: 不受 big 影响, 4 满
        assert!(sem.try_acquire(BrainTier::Triage));
        assert!(sem.try_acquire(BrainTier::Triage));
        assert!(sem.try_acquire(BrainTier::Triage));
        assert!(sem.try_acquire(BrainTier::Triage));
        assert!(!sem.try_acquire(BrainTier::Triage));
        // 释放 big → 可再获取
        sem.release(BrainTier::Big);
        assert!(sem.try_acquire(BrainTier::Big));
        assert_eq!(sem.in_flight(BrainTier::Big), 2);
        assert_eq!(sem.in_flight(BrainTier::Triage), 4);
    }

    #[test]
    fn tiered_semaphore_release_bounds() {
        let mut sem = TieredSemaphore::default();
        sem.release(BrainTier::Big);
        assert_eq!(sem.in_flight(BrainTier::Big), 0);
        assert_eq!(sem.max_for(BrainTier::Big), 6);
        assert_eq!(sem.max_for(BrainTier::Triage), 8);
    }
}
