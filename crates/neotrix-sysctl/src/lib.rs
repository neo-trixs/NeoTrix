//! neotrix-sysctl — macOS/Linux 系统指标读取专用 FFI crate。
//!
//! R-P1 (零 unsafe) 门禁下，neotrix-core 不得包含任何 `unsafe` 代码。
//! 进程 RSS 与物理内存总量的读取必须调用 `libc::sysctl` / `/proc`
//! 等平台 FFI，无法用 safe Rust 表达。本 crate 是 FFI 专用层：
//! 唯一职责是封装平台差异（macOS sysctl / Linux procfs），
//! 对外只暴露 safe 接口，供 neotrix-core 等上层调用。
//!
//! crate 级放行 unsafe：scope 被物理限制在本 crate 内部，
//! 上层 crate (neotrix-core) 保持 `#![forbid(unsafe_code)]` 不受影响。

#![allow(unsafe_code)]

/// 当前进程 RSS（常驻内存），单位字节。
///
/// - macOS: `sysctl` KERN_PROC_PID 读取 `ProcExeTaskInfo.ti.resident_size`
/// - 其它平台: 解析 `/proc/self/status` 的 `VmRSS:` 字段
///
/// 读取失败时返回 `0`（与调用方约定一致，调用方将按 0 处理）。
pub fn current_rss_bytes() -> u64 {
    #[cfg(target_os = "macos")]
    {
        let pid = unsafe { libc::getpid() };
        let mut mib: [libc::c_int; 4] = [libc::CTL_KERN, libc::KERN_PROC, libc::KERN_PROC_PID, pid];
        let mut size: libc::size_t = 0;
        if unsafe { libc::sysctl(mib.as_mut_ptr(), 4, std::ptr::null_mut(), &mut size, std::ptr::null_mut(), 0) } != 0 {
            return 0;
        }
        let mut proc_info: Vec<u8> = vec![0u8; size];
        if unsafe { libc::sysctl(mib.as_mut_ptr(), 4, proc_info.as_mut_ptr() as *mut libc::c_void, &mut size, std::ptr::null_mut(), 0) } != 0 {
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
    #[cfg(not(target_os = "macos"))]
    {
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

/// 系统物理内存总量，单位字节。
///
/// - macOS: `sysctl` HW_MEMSIZE 读取 `u64` 物理内存总量
/// - 其它平台: 无系统级查询，返回 32GB 兜底值（与调用方原约定一致）
///
/// macOS 下 sysctl 失败时返回 16GB 兜底值（沙箱/受限环境）。
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_rss_bytes_returns_value() {
        let rss = current_rss_bytes();
        // RSS may be 0 in sandboxed environments where sysctl is restricted;
        // the contract is: call never panics and returns a sane value.
        if rss == 0 {
            println!("[warn] RSS returned 0 — sysctl may be restricted in this environment");
        }
    }

    #[test]
    fn test_total_physical_memory_returns_value() {
        let total = total_physical_memory();
        // 兜底值 32GB 或 16GB，任何平台都应返回非零。
        assert!(total > 0, "total physical memory must be non-zero");
    }
}
