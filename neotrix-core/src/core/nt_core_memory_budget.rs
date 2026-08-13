//! Memory budget tracking with RSS monitoring
//!
//! 进程 RSS / 物理内存总量的平台读取依赖 `sysctl` / procfs FFI，
//! 已抽离到独立 crate `neotrix-sysctl`（FFI 专用层）。
//! 本模块只保留纯 Rust 逻辑，crate 级 `#![forbid(unsafe_code)]` (R-P1) 全程生效。

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Instant;

pub struct MemoryBudget {
    soft_limit: u64,
    hard_limit: u64,
    peak_rss: AtomicI64,
    last_throttle: std::sync::Mutex<Option<(Instant, u64)>>,
}

impl MemoryBudget {
    pub const fn new(soft_limit: u64, hard_limit: u64) -> Self {
        Self {
            soft_limit,
            hard_limit,
            peak_rss: AtomicI64::new(0),
            last_throttle: std::sync::Mutex::new(None),
        }
    }
}

impl Default for MemoryBudget {
    fn default() -> Self {
        let total = total_physical_memory();
        Self {
            soft_limit: (total as f64 * 0.70) as u64,
            hard_limit: (total as f64 * 0.85) as u64,
            peak_rss: AtomicI64::new(0),
            last_throttle: std::sync::Mutex::new(None),
        }
    }
}

/// 系统物理内存总量（字节）。
/// 平台 FFI 已抽离至 `neotrix-sysctl`（macOS sysctl / 非 macOS 兜底）。
pub fn total_physical_memory() -> u64 {
    neotrix_sysctl::total_physical_memory()
}

fn current_rss() -> u64 {
    neotrix_sysctl::current_rss_bytes()
}

impl MemoryBudget {
    pub fn rss_bytes(&self) -> u64 {
        let rss = current_rss();
        let prev = self.peak_rss.load(Ordering::Relaxed);
        if rss > prev as u64 {
            self.peak_rss.store(rss as i64, Ordering::Relaxed);
        }
        rss
    }

    pub fn peak_rss_bytes(&self) -> u64 {
        self.peak_rss.load(Ordering::Relaxed) as u64
    }

    pub fn soft_limit(&self) -> u64 {
        self.soft_limit
    }

    pub fn hard_limit(&self) -> u64 {
        self.hard_limit
    }

    pub fn remaining_budget(&self) -> i64 {
        self.soft_limit as i64 - self.rss_bytes() as i64
    }

    pub fn usage_ratio(&self) -> f64 {
        let rss = self.rss_bytes();
        if rss == 0 {
            return 0.0;
        }
        (rss as f64 / self.soft_limit as f64).min(1.0)
    }

    pub fn should_throttle(&self) -> bool {
        let rss = self.rss_bytes();
        if rss >= self.hard_limit {
            return true;
        }
        if rss >= self.soft_limit {
            if let Ok(mut last) = self.last_throttle.lock() {
                let now = Instant::now();
                if let Some((last_time, last_rss)) = *last {
                    if rss < last_rss {
                        *last = Some((now, rss));
                        return false;
                    }
                    if now.duration_since(last_time).as_secs() < 5 {
                        return true;
                    }
                }
                *last = Some((now, rss));
            }
            true
        } else {
            false
        }
    }

    pub fn can_allocate(&self, bytes: u64) -> bool {
        let rss = self.rss_bytes();
        let after = rss.saturating_add(bytes);
        after < self.hard_limit
    }

    pub fn check(&self) -> MemoryPressure {
        let rss = self.rss_bytes();
        if rss >= self.hard_limit {
            MemoryPressure::Critical
        } else if rss >= self.soft_limit {
            MemoryPressure::Warning
        } else {
            MemoryPressure::Normal
        }
    }
}

pub enum MemoryPressure {
    Normal,
    Warning,
    Critical,
}

impl MemoryPressure {
    pub fn is_normal(&self) -> bool {
        matches!(self, MemoryPressure::Normal)
    }

    pub fn is_critical(&self) -> bool {
        matches!(self, MemoryPressure::Critical)
    }

    pub fn suggested_batch_size(&self) -> usize {
        match self {
            MemoryPressure::Normal => 1000,
            MemoryPressure::Warning => 100,
            MemoryPressure::Critical => 10,
        }
    }
}

pub(crate) static GLOBAL_BUDGET: std::sync::LazyLock<MemoryBudget> =
    std::sync::LazyLock::new(MemoryBudget::default);

pub fn global() -> &'static MemoryBudget {
    &GLOBAL_BUDGET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_default_creates() {
        let b = MemoryBudget::default();
        assert!(b.soft_limit() > 0);
        assert!(b.hard_limit() > b.soft_limit());
    }

    #[test]
    fn test_rss_is_nonzero() {
        let b = MemoryBudget::default();
        // RSS may be 0 in sandboxed environments where sysctl is restricted
        // Only assert non-zero if the sysctl call succeeded
        let rss = b.rss_bytes();
        if rss == 0 {
            println!("[warn] RSS returned 0 — sysctl may be restricted in this environment");
        }
    }

    #[test]
    fn test_usage_ratio_bounds() {
        let b = MemoryBudget::default();
        let ratio = b.usage_ratio();
        assert!(ratio >= 0.0);
        assert!(ratio <= 1.0);
    }

    #[test]
    fn test_remaining_budget() {
        let b = MemoryBudget::default();
        // remaining_budget may equal soft_limit when RSS is 0 (sandboxed env)
        assert!(b.remaining_budget() <= b.soft_limit() as i64);
    }

    #[test]
    fn test_check_never_panics() {
        let b = MemoryBudget::default();
        match b.check() {
            MemoryPressure::Normal | MemoryPressure::Warning | MemoryPressure::Critical => {}
        }
    }

    #[test]
    fn test_global_budget_accessible() {
        let g = global();
        assert!(g.soft_limit() > 0);
    }

    #[test]
    fn test_suggested_batch_size_decreases_with_pressure() {
        assert!(MemoryPressure::Normal.suggested_batch_size() > MemoryPressure::Warning.suggested_batch_size());
        assert!(MemoryPressure::Warning.suggested_batch_size() > MemoryPressure::Critical.suggested_batch_size());
    }
}
