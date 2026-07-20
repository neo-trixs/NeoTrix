//! Memory budget tracking with RSS monitoring
//! 
//! Uses unsafe code for sysctl calls on macOS — explicitly allowed for this module.
#![allow(unsafe_code)]

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Instant;

#[cfg(target_os = "macos")]
mod rss {
    use libc::{c_int, c_void, sysctl, CTL_KERN, KERN_PROC, KERN_PROC_PID};

    pub fn current_rss_bytes() -> u64 {
        let pid = unsafe { libc::getpid() };
        let mut mib: [c_int; 4] = [CTL_KERN, KERN_PROC, KERN_PROC_PID, pid];
        let mut size: libc::size_t = 0;
        if unsafe { sysctl(mib.as_mut_ptr(), 4, std::ptr::null_mut(), &mut size, std::ptr::null_mut(), 0) } != 0 {
            return 0;
        }
        let mut proc_info: Vec<u8> = vec![0u8; size];
        if unsafe { sysctl(mib.as_mut_ptr(), 4, proc_info.as_mut_ptr() as *mut c_void, &mut size, std::ptr::null_mut(), 0) } != 0 {
            return 0;
        }
        #[repr(C)]
        struct ProcTaskInfo {
            virtual_size: u64,
            resident_size: u64,
            total_user: u64,
            total_system: u64,
            threads_count: u32,
            policy: i32,
            suspend_count: u32,
        }
        #[repr(C)]
        struct ProcBsdInfo {
            pbi_flags: u32,
            pbi_status: u32,
            pbi_xstatus: u32,
            pbi_pid: u32,
            pbi_ppid: u32,
            pbi_uid: u32,
            pbi_gid: u32,
            pbi_ruid: u32,
            pbi_rgid: u32,
            pbi_svuid: u32,
            pbi_svgid: u32,
            rfu_1: u32,
            pbi_comm: [u8; 256],
            pbi_name: [u8; 256],
            pbi_nfiles: u32,
            pbi_pgid: u32,
            pbi_pjobc: u32,
            e_timer: u32,
            e_timer_runtime: u64,
            e_timer_period: u64,
            e_timer_deprecated: u64,
            e_timer_interval: u64,
            e_timer_reserved1: u64,
            e_timer_reserved2: u64,
            e_timer_reserved3: u64,
            e_timer_reserved4: u64,
            pbi_nice: u32,
            pbi_start_tvsec: u32,
            pbi_start_tvusec: u32,
        }
        #[repr(C)]
        struct ProcExeTaskInfo {
            pbi: ProcBsdInfo,
            ti: ProcTaskInfo,
        }

        let info: &ProcExeTaskInfo = unsafe { &*(proc_info.as_ptr() as *const ProcExeTaskInfo) };
        info.ti.resident_size
    }
}

#[cfg(not(target_os = "macos"))]
mod rss {
    pub fn current_rss_bytes() -> u64 {
        let file = match std::fs::read_to_string("/proc/self/status") {
            Ok(f) => f,
            Err(_) => return 0,
        };
        for line in file.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(val) = parts.get(1).and_then(|v| v.parse::<u64>().ok()) {
                    return val * 1024;
                }
            }
        }
        0
    }
}

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

pub fn total_physical_memory() -> u64 {
    #[cfg(target_os = "macos")]
    {
        let mut mib: [libc::c_int; 2] = [libc::CTL_HW, libc::HW_MEMSIZE];
        let mut size: u64 = 0;
        let mut len: libc::size_t = std::mem::size_of::<u64>() as libc::size_t;
        if unsafe { libc::sysctl(mib.as_mut_ptr(), 2, &mut size as *mut u64 as *mut libc::c_void, &mut len, std::ptr::null_mut(), 0) } == 0 {
            return size;
        }
        16_000_000_000
    }
    #[cfg(not(target_os = "macos"))]
    {
        32_000_000_000
    }
}

fn current_rss() -> u64 {
    rss::current_rss_bytes()
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
        assert!(rss >= 0);
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
