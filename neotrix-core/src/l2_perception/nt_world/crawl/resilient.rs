//! P11 Async-Resilient crawl scheduling (scrapy + crawlee 吸收) —
//! 无真实网络 / 无 sleep / 无 tokio 的纯状态机:
//! 内存持久队列 + 指数退避重试 + 延迟自适应节流 (模拟 tick)。
//!
//! 核心特征:
//! - PersistentQueue: VecDeque 请求队列, 逐 URL 追踪重试计数
//! - AutoThrottle: 观测延迟上升 → 加大延迟 (封顶), 延迟低 → 回落至下限
//! - ResilientCrawler: 确定性模拟爬取, 失败 → 退避重排, 超 max_retries → 丢弃

use std::collections::{HashMap, VecDeque};

/// 退避时长封顶 (ms), 防止指数退避溢出。
pub const MAX_BACKOFF_MS: u64 = 60_000;

/// 节流策略。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThrottlePolicy {
    pub min_delay_ms: u64,
    pub max_concurrency: usize,
    pub adaptive: bool,
}

impl Default for ThrottlePolicy {
    fn default() -> Self {
        Self {
            min_delay_ms: 100,
            max_concurrency: 10,
            adaptive: true,
        }
    }
}

/// 内存持久请求队列 (VecDeque), 追踪每 URL 重试次数。
#[derive(Debug, Clone)]
pub struct PersistentQueue {
    requests: VecDeque<String>,
    retry_count: HashMap<String, u8>,
    backoff_base_ms: u64,
}

impl Default for PersistentQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl PersistentQueue {
    pub fn new() -> Self {
        Self::with_backoff(1_000)
    }

    pub fn with_backoff(backoff_base_ms: u64) -> Self {
        Self {
            requests: VecDeque::new(),
            retry_count: HashMap::new(),
            backoff_base_ms,
        }
    }

    /// 入队 (队尾), 首次出现的 URL 重试计数置 0。
    pub fn push(&mut self, url: &str) {
        if !self.retry_count.contains_key(url) {
            self.retry_count.insert(url.to_string(), 0);
        }
        self.requests.push_back(url.to_string());
    }

    /// 出队 (队头)。
    pub fn pop_front(&mut self) -> Option<String> {
        self.requests.pop_front()
    }

    /// 失败重排 (队尾) + 指数退避; 返回模拟等待时长 (ms)。
    pub fn requeue(&mut self, url: &str) -> u64 {
        let retries = self.retry_count.get(url).copied().unwrap_or(0);
        self.retry_count.insert(url.to_string(), retries + 1);
        self.requests.push_back(url.to_string());
        Self::backoff_ms(self.backoff_base_ms, retries)
    }

    /// 退避时长: base × 2^retries, 封顶 MAX_BACKOFF_MS。
    pub fn backoff_ms(base: u64, retries: u8) -> u64 {
        let shift = (retries as u32).min(16);
        base.saturating_mul(1_u64 << shift).min(MAX_BACKOFF_MS)
    }

    pub fn len(&self) -> usize {
        self.requests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    /// 当前 URL 的重试计数。
    pub fn retry_count(&self, url: &str) -> u8 {
        self.retry_count.get(url).copied().unwrap_or(0)
    }
}

/// 基于延迟的自适应节流 (Scrapy AutoThrottle 思路)。
#[derive(Debug, Clone)]
pub struct AutoThrottle {
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub current_delay_ms: u64,
    pub last_latency_ms: u64,
    pub increase_factor: f64,
    pub decrease_factor: f64,
}

impl AutoThrottle {
    pub fn new(min_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            min_delay_ms,
            max_delay_ms,
            current_delay_ms: min_delay_ms,
            last_latency_ms: 0,
            increase_factor: 1.5,
            decrease_factor: 1.5,
        }
    }

    /// 依据观测延迟调整当前延迟: 延迟高于当前 → 加大 (上限 max);
    /// 延迟低于当前 → 减小 (下限 min)。返回当前延迟 (ms)。
    pub fn adjust(&mut self, latency_ms: u64) -> u64 {
        self.last_latency_ms = latency_ms;
        if latency_ms > self.current_delay_ms {
            let next = (self.current_delay_ms as f64 * self.increase_factor).round();
            self.current_delay_ms = (next.min(self.max_delay_ms as f64)) as u64;
        } else if latency_ms < self.current_delay_ms {
            let next = (self.current_delay_ms as f64 / self.decrease_factor).round();
            self.current_delay_ms = (next.max(self.min_delay_ms as f64)) as u64;
        }
        self.current_delay_ms
    }
}

/// 会话指标。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CrawlSession {
    pub queued: usize,
    pub throttled: usize,
    pub completed: usize,
    pub failed: usize,
}

/// 爬取报告。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CrawlReport {
    pub completed: Vec<String>,
    pub failed: Vec<String>,
    /// 模拟等待总时长 (ms, 含节流 + 退避), 不做真实 sleep。
    pub total_delay_ms: u64,
    pub retries_used: usize,
    pub throttled_count: usize,
}

/// 韧性爬虫 — 确定性纯状态机 (无网络 / 无 sleep / 无 tokio)。
pub struct ResilientCrawler {
    pub policy: ThrottlePolicy,
    pub queue: PersistentQueue,
    pub throttle: AutoThrottle,
    pub session: CrawlSession,
    simulate_failure: Box<dyn Fn(&str) -> bool + Send + Sync>,
}

impl ResilientCrawler {
    pub fn new(policy: ThrottlePolicy) -> Self {
        Self::with_failure(policy, Box::new(|url: &str| url.contains("fail")))
    }

    pub fn with_failure(
        policy: ThrottlePolicy,
        simulate_failure: Box<dyn Fn(&str) -> bool + Send + Sync>,
    ) -> Self {
        let min_delay_ms = policy.min_delay_ms;
        let max_delay = (min_delay_ms * 20).max(min_delay_ms + 1);
        Self {
            policy,
            queue: PersistentQueue::new(),
            throttle: AutoThrottle::new(min_delay_ms, max_delay),
            session: CrawlSession::default(),
            simulate_failure,
        }
    }

    /// 确定性模拟延迟 (URL + 尝试次数 → 稳定伪随机), 无时间依赖。
    fn simulate_latency(url: &str, attempt: u32) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut h);
        attempt.hash(&mut h);
        5 + (h.finish() % 90)
    }

    /// 执行韧性爬取模拟: 每 URL 出队 → 节流(模拟 tick, 记入 total_delay_ms) →
    /// 失败则退避重排, 重试超 max_retries 则丢弃入 failed。
    pub fn run_resilient(&mut self, urls: &[&str], max_retries: u8) -> CrawlReport {
        self.session = CrawlSession {
            queued: urls.len(),
            ..CrawlSession::default()
        };
        self.queue = PersistentQueue::new();
        self.throttle = AutoThrottle::new(self.policy.min_delay_ms, self.throttle.max_delay_ms);
        for u in urls {
            self.queue.push(u);
        }

        let mut report = CrawlReport::default();
        let mut attempt = 0_u32;
        while let Some(url) = self.queue.pop_front() {
            let retries = self.queue.retry_count(&url);
            if self.policy.adaptive {
                let latency = Self::simulate_latency(&url, attempt);
                let delay = self.throttle.adjust(latency);
                report.total_delay_ms = report.total_delay_ms.saturating_add(delay);
                report.throttled_count += 1;
                self.session.throttled += 1;
            }
            attempt += 1;

            if (self.simulate_failure)(&url) {
                if retries < max_retries {
                    let backoff = self.queue.requeue(&url);
                    report.total_delay_ms = report.total_delay_ms.saturating_add(backoff);
                    report.retries_used += 1;
                } else {
                    self.session.failed += 1;
                    report.failed.push(url);
                }
            } else {
                self.session.completed += 1;
                report.completed.push(url);
            }
        }
        report
    }
}

impl crate::core::nt_core_self_test::SelfTest for ResilientCrawler {
    fn name(&self) -> &str {
        "nt_world_crawl_async_resilient"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut crawler = ResilientCrawler::new(ThrottlePolicy::default());
        let report = crawler.run_resilient(&["ok.com", "fail.com"], 1);
        if report.completed != ["ok.com".to_string()] {
            failures.push(format!("expected ok.com completed, got {:?}", report.completed));
        }
        if report.failed != ["fail.com".to_string()] {
            failures.push(format!("expected fail.com failed, got {:?}", report.failed));
        }
        if crawler.session.queued != 2 {
            failures.push("queued metric mismatch".into());
        }
        if crawler.session.completed + crawler.session.failed != crawler.session.queued {
            failures.push("completed+failed must equal queued".into());
        }
        if !crawler.queue.is_empty() {
            failures.push("queue must fully drain".into());
        }
        if report.retries_used < 1 {
            failures.push("failed URL must be retried at least once".into());
        }
        let mut prev = 0_u64;
        for r in 0..5_u8 {
            let b = PersistentQueue::backoff_ms(1_000, r);
            if b < prev {
                failures.push("backoff must be monotonic non-decreasing".into());
            }
            prev = b;
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_push_pop_len() {
        let mut q = PersistentQueue::new();
        assert!(q.is_empty());
        q.push("a.com");
        q.push("b.com");
        q.push("c.com");
        assert_eq!(q.len(), 3);
        assert_eq!(q.pop_front().as_deref(), Some("a.com"));
        assert_eq!(q.pop_front().as_deref(), Some("b.com"));
        assert_eq!(q.pop_front().as_deref(), Some("c.com"));
        assert_eq!(q.pop_front(), None);
        assert!(q.is_empty());
    }

    #[test]
    fn test_queue_requeue_retry_count_and_backoff() {
        let mut q = PersistentQueue::with_backoff(1_000);
        q.push("x.com");
        assert_eq!(q.retry_count("x.com"), 0);
        assert_eq!(q.pop_front().as_deref(), Some("x.com"));
        assert!(q.is_empty());
        let b1 = q.requeue("x.com");
        assert_eq!(q.len(), 1);
        assert_eq!(q.retry_count("x.com"), 1);
        assert_eq!(b1, 1_000, "1st retry backoff -> base*2^0");
        q.pop_front();
        let b2 = q.requeue("x.com");
        assert_eq!(q.retry_count("x.com"), 2);
        assert_eq!(b2, 2_000, "2nd retry backoff -> base*2^1");
    }

    #[test]
    fn test_backoff_cap_and_monotonic() {
        assert_eq!(PersistentQueue::backoff_ms(1_000, 0), 1_000);
        assert_eq!(PersistentQueue::backoff_ms(1_000, 1), 2_000);
        assert_eq!(PersistentQueue::backoff_ms(1_000, 16), MAX_BACKOFF_MS);
        assert_eq!(PersistentQueue::backoff_ms(1_000, 20), MAX_BACKOFF_MS, "cap at max");
        assert_eq!(PersistentQueue::backoff_ms(1_000, 255), MAX_BACKOFF_MS, "overflow-safe");
        let mut prev = 0_u64;
        for r in 0..20_u8 {
            let b = PersistentQueue::backoff_ms(500, r);
            assert!(b >= prev, "backoff must never decrease");
            assert!(b <= MAX_BACKOFF_MS, "backoff must respect cap");
            prev = b;
        }
    }

    #[test]
    fn test_auto_throttle_increase_caps_at_max() {
        let mut t = AutoThrottle::new(100, 2_000);
        assert_eq!(t.current_delay_ms, 100);
        // 高延迟持续涌入 → 延迟逐步加大并封顶
        let mut prev = 0_u64;
        let mut capped = false;
        for _ in 0..30 {
            let d = t.adjust(9_999);
            assert!(d >= prev, "delay must not decrease while latency high");
            assert!(d <= 2_000, "delay must respect max");
            prev = d;
            if d == 2_000 {
                capped = true;
            }
        }
        assert!(capped, "delay must reach the max cap");
        assert_eq!(t.adjust(9_999), 2_000, "stays capped");
        assert_eq!(t.last_latency_ms, 9_999);
    }

    #[test]
    fn test_auto_throttle_decrease_toward_min() {
        let mut t = AutoThrottle::new(100, 2_000);
        t.adjust(9_999); // 抬升
        assert!(t.current_delay_ms > 100);
        // 低延迟持续 → 回落至下限
        for _ in 0..30 {
            let d = t.adjust(10);
            assert!(d >= 100, "delay must respect min");
            if d == 100 {
                break;
            }
        }
        assert_eq!(t.adjust(10), 100, "returns to min_delay");
    }

    #[test]
    fn test_run_resilient_success_path() {
        let mut c = ResilientCrawler::new(ThrottlePolicy::default());
        let report = c.run_resilient(&["ok.com", "great.io", "fine.net"], 3);
        assert_eq!(report.completed.len(), 3);
        assert!(report.failed.is_empty());
        assert_eq!(report.retries_used, 0);
        assert_eq!(c.session.completed, 3);
        assert_eq!(c.session.failed, 0);
        assert_eq!(c.session.queued, 3);
        assert!(c.queue.is_empty());
        assert!(report.total_delay_ms > 0, "throttle delay simulated as ticks");
        assert_eq!(report.throttled_count, 3);
    }

    #[test]
    fn test_run_resilient_retry_then_success() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        let attempts = Arc::new(AtomicU32::new(0));
        let counter = Arc::clone(&attempts);
        let mut c = ResilientCrawler::with_failure(
            ThrottlePolicy::default(),
            Box::new(move |_url: &str| {
                let n = counter.fetch_add(1, Ordering::SeqCst) + 1;
                n <= 2 // 前两次尝试失败, 之后成功
            }),
        );
        let report = c.run_resilient(&["flaky.io"], 3);
        assert_eq!(report.completed, vec!["flaky.io".to_string()]);
        assert!(report.failed.is_empty());
        assert_eq!(report.retries_used, 2, "two retries before success");
        assert_eq!(c.session.completed, 1);
        assert_eq!(c.session.failed, 0);
    }

    #[test]
    fn test_run_resilient_drop_after_max_retries() {
        let mut c = ResilientCrawler::new(ThrottlePolicy::default());
        let report = c.run_resilient(&["ok.com", "fail.com"], 2);
        assert_eq!(report.completed, vec!["ok.com".to_string()]);
        assert_eq!(report.failed, vec!["fail.com".to_string()]);
        assert_eq!(report.retries_used, 2, "fail.com retried max_retries times then dropped");
        assert_eq!(c.session.completed, 1);
        assert_eq!(c.session.failed, 1);
        assert_eq!(c.session.queued, 2);
    }

    #[test]
    fn test_run_resilient_zero_retries_drops_immediately() {
        let mut c = ResilientCrawler::new(ThrottlePolicy::default());
        let report = c.run_resilient(&["fail.com"], 0);
        assert!(report.completed.is_empty());
        assert_eq!(report.failed, vec!["fail.com".to_string()]);
        assert_eq!(report.retries_used, 0);
        assert_eq!(c.session.failed, 1);
    }

    #[test]
    fn test_run_resilient_non_adaptive_no_throttle() {
        let policy = ThrottlePolicy {
            adaptive: false,
            ..ThrottlePolicy::default()
        };
        let mut c = ResilientCrawler::new(policy);
        let report = c.run_resilient(&["ok.com", "fail.com"], 1);
        assert_eq!(report.throttled_count, 0, "adaptive off -> no throttle delay");
        assert_eq!(c.session.throttled, 0);
        // 退避重试仍计入模拟等待 (与节流无关)
        assert_eq!(report.total_delay_ms, 1_000, "single retry backoff still tracked");
        assert_eq!(c.session.completed, 1);
        assert_eq!(c.session.failed, 1);
    }

    #[test]
    fn test_run_resilient_deterministic() {
        let mut a = ResilientCrawler::new(ThrottlePolicy::default());
        let mut b = ResilientCrawler::new(ThrottlePolicy::default());
        let ra = a.run_resilient(&["ok.com", "fail.com", "slow.io"], 2);
        let rb = b.run_resilient(&["ok.com", "fail.com", "slow.io"], 2);
        assert_eq!(ra, rb, "two identical runs must produce identical reports");
        assert_eq!(a.session, b.session);
    }

    #[test]
    fn test_async_resilient_self_test_name() {
        let c = ResilientCrawler::new(ThrottlePolicy::default());
        let name = crate::core::nt_core_self_test::SelfTest::name(&c);
        assert_eq!(name, "nt_world_crawl_async_resilient");
        assert!(crate::core::nt_core_self_test::SelfTest::self_test(&c).is_ok());
    }
}
