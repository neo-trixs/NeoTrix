use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct AutoscaledPool {
    pub min_concurrency: usize,
    pub max_concurrency: usize,
    current_concurrency: AtomicUsize,
    cpu_target: f64,
    cpu_margin: f64,
    last_check: Mutex<Instant>,
    check_interval: Duration,
    scale_up_step: usize,
    scale_down_step: usize,
    tasks_succeeded: AtomicUsize,
    tasks_failed: AtomicUsize,
    total_latency_ms: AtomicUsize,
    last_adjust_reason: Mutex<String>,
}

impl AutoscaledPool {
    pub fn new() -> Self {
        Self::with_bounds(1, 10)
    }

    pub fn with_bounds(min: usize, max: usize) -> Self {
        Self {
            min_concurrency: min,
            max_concurrency: max,
            current_concurrency: AtomicUsize::new(min.max(1)),
            cpu_target: 0.7,
            cpu_margin: 0.1,
            last_check: Mutex::new(Instant::now()),
            check_interval: Duration::from_secs(10),
            scale_up_step: 1,
            scale_down_step: 1,
            tasks_succeeded: AtomicUsize::new(0),
            tasks_failed: AtomicUsize::new(0),
            total_latency_ms: AtomicUsize::new(0),
            last_adjust_reason: Mutex::new("initial".into()),
        }
    }

    pub fn adjust(&self) -> usize {
        let mut last = self.last_check.lock().unwrap_or_else(|e| e.into_inner());
        if last.elapsed() < self.check_interval {
            return self.current_concurrency.load(Ordering::Relaxed);
        }
        *last = Instant::now();
        drop(last);

        let succeeded = self.tasks_succeeded.swap(0, Ordering::Relaxed);
        let failed = self.tasks_failed.swap(0, Ordering::Relaxed);
        let lat_total = self.total_latency_ms.swap(0, Ordering::Relaxed);
        let total = succeeded + failed;

        let new_conc = if total == 0 {
            self.current_concurrency.load(Ordering::Relaxed)
        } else {
            let fail_rate = failed as f64 / total as f64;
            let avg_lat = lat_total as f64 / total as f64;
            let current = self.current_concurrency.load(Ordering::Relaxed);

            if fail_rate > 0.10 || avg_lat > 5_000.0 {
                let scaled = current.saturating_sub(self.scale_down_step);
                *self.last_adjust_reason.lock().unwrap_or_else(|e| e.into_inner()) = format!(
                    "scale_down: fail_rate={:.2} avg_lat={:.0}ms",
                    fail_rate, avg_lat
                );
                scaled.max(self.min_concurrency).min(self.max_concurrency)
            } else if fail_rate < 0.05 && avg_lat < 1_000.0 {
                let scaled = current.saturating_add(self.scale_up_step);
                *self.last_adjust_reason.lock().unwrap_or_else(|e| e.into_inner()) = format!(
                    "scale_up: fail_rate={:.2} avg_lat={:.0}ms",
                    fail_rate, avg_lat
                );
                scaled.max(self.min_concurrency).min(self.max_concurrency)
            } else {
                *self.last_adjust_reason.lock().unwrap_or_else(|e| e.into_inner()) = format!(
                    "hold: fail_rate={:.2} avg_lat={:.0}ms",
                    fail_rate, avg_lat
                );
                current
            }
        };

        self.current_concurrency.store(new_conc, Ordering::Relaxed);
        new_conc
    }

    pub fn concurrency(&self) -> usize {
        self.current_concurrency.load(Ordering::Relaxed)
    }

    pub fn record_task(&self, latency_ms: f64, success: bool) {
        if success {
            self.tasks_succeeded.fetch_add(1, Ordering::Relaxed);
        } else {
            self.tasks_failed.fetch_add(1, Ordering::Relaxed);
        }
        self.total_latency_ms
            .fetch_add(latency_ms as usize, Ordering::Relaxed);
    }

    pub fn stats(&self) -> PoolMetrics {
        let reason = self.last_adjust_reason.lock().unwrap_or_else(|e| e.into_inner()).clone();
        PoolMetrics {
            current: self.current_concurrency.load(Ordering::Relaxed),
            min: self.min_concurrency,
            max: self.max_concurrency,
            tasks_completed: self.tasks_succeeded.load(Ordering::Relaxed) as u64,
            tasks_failed: self.tasks_failed.load(Ordering::Relaxed) as u64,
            avg_latency_ms: 0.0,
            last_adjust_reason: reason,
        }
    }
}

impl Default for AutoscaledPool {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PoolMetrics {
    pub current: usize,
    pub min: usize,
    pub max: usize,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub avg_latency_ms: f64,
    pub last_adjust_reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool_has_min_concurrency() {
        let pool = AutoscaledPool::new();
        assert!(pool.concurrency() >= pool.min_concurrency);
        assert!(pool.concurrency() <= pool.max_concurrency);
    }

    #[test]
    fn test_pool_respects_bounds() {
        let pool = AutoscaledPool::with_bounds(3, 7);
        assert_eq!(pool.min_concurrency, 3);
        assert_eq!(pool.max_concurrency, 7);
        let c = pool.concurrency();
        assert!(c >= 3 && c <= 7);
    }

    #[test]
    fn test_adjust_scale_up_on_good_conditions() {
        let pool = AutoscaledPool::with_bounds(1, 10);
        pool.current_concurrency.store(1, Ordering::Relaxed);
        for _ in 0..20 {
            pool.record_task(200.0, true);
        }
        let new = pool.adjust();
        assert!(new > 1, "should scale up when conditions are good");
    }

    #[test]
    fn test_adjust_scale_down_on_high_failure() {
        let pool = AutoscaledPool::with_bounds(1, 10);
        pool.current_concurrency.store(5, Ordering::Relaxed);
        for _ in 0..10 {
            pool.record_task(1000.0, false);
        }
        let new = pool.adjust();
        assert!(new < 5, "should scale down on high failure rate");
    }

    #[test]
    fn test_adjust_scale_down_on_high_latency() {
        let pool = AutoscaledPool::with_bounds(1, 10);
        pool.current_concurrency.store(5, Ordering::Relaxed);
        for _ in 0..10 {
            pool.record_task(10_000.0, true);
        }
        let new = pool.adjust();
        assert!(new < 5, "should scale down on high latency");
    }

    #[test]
    fn test_adjust_unchanged_with_no_data() {
        let pool = AutoscaledPool::with_bounds(1, 10);
        let old = pool.concurrency();
        let new = pool.adjust();
        assert_eq!(new, old, "should hold steady with no data");
    }

    #[test]
    fn test_concurrency_stays_within_bounds() {
        let pool = AutoscaledPool::with_bounds(3, 8);
        assert!(pool.concurrency() >= 3);
        assert!(pool.concurrency() <= 8);
    }

    #[test]
    fn test_adjust_respects_interval() {
        let pool = AutoscaledPool::with_bounds(1, 10);
        pool.current_concurrency.store(1, Ordering::Relaxed);
        for _ in 0..20 {
            pool.record_task(100.0, true);
        }
        let first = pool.adjust();
        assert!(first > 1);

        // Immediate second call should no-op (interval not elapsed)
        let second = pool.adjust();
        assert_eq!(second, first);
    }

    #[test]
    fn test_record_task_tracks_success_and_failure() {
        let pool = AutoscaledPool::new();
        pool.record_task(100.0, true);
        pool.record_task(200.0, false);
        assert_eq!(pool.tasks_succeeded.load(Ordering::Relaxed), 1);
        assert_eq!(pool.tasks_failed.load(Ordering::Relaxed), 1);
        assert!(pool.total_latency_ms.load(Ordering::Relaxed) >= 300);
    }

    #[test]
    fn test_stats_reflect_current_state() {
        let pool = AutoscaledPool::with_bounds(1, 10);
        pool.record_task(100.0, true);
        let s = pool.stats();
        assert_eq!(s.tasks_completed, 1);
        assert_eq!(s.min, 1);
        assert_eq!(s.max, 10);
    }
}
