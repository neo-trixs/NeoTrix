use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub platform: String,
    pub arch: String,
    pub neotrix_version: String,
    pub uptime_seconds: u64,
    pub hostname: String,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub cpu_count: usize,
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    let platform = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".into());
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".into());

    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".into());

    let neotrix_version = super::neotrix_cli::get_version().await.unwrap_or_else(|_| "unknown".into());

    let uptime_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let (memory_used_mb, memory_total_mb) = get_memory_info();
    let cpu_count = num_cpus::get();

    Ok(SystemInfo {
        platform,
        arch,
        neotrix_version,
        uptime_seconds,
        hostname,
        memory_used_mb,
        memory_total_mb,
        cpu_count,
    })
}

fn get_memory_info() -> (u64, u64) {
    #[cfg(target_os = "macos")]
    {
        get_macos_memory()
    }
    #[cfg(target_os = "linux")]
    {
        get_linux_memory()
    }
    #[cfg(target_os = "windows")]
    {
        get_windows_memory()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        (0, 0)
    }
}

#[cfg(target_os = "macos")]
fn get_macos_memory() -> (u64, u64) {
    use std::process::Command;
    if let Ok(output) = Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
    {
        if let Ok(total_str) = String::from_utf8(output.stdout) {
            if let Ok(total_bytes) = total_str.trim().parse::<u64>() {
                let total_mb = total_bytes / (1024 * 1024);
                let used_mb = total_mb * 70 / 100;
                return (used_mb, total_mb);
            }
        }
    }
    (0, 0)
}

#[cfg(target_os = "linux")]
fn get_linux_memory() -> (u64, u64) {
    use std::fs;
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        let mut total = 0u64;
        let mut available = 0u64;
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            }
            if line.starts_with("MemAvailable:") {
                available = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            }
        }
        let total_mb = total / 1024;
        let used_mb = (total - available) / 1024;
        return (used_mb, total_mb);
    }
    (0, 0)
}

#[cfg(target_os = "windows")]
fn get_windows_memory() -> (u64, u64) {
    (0, 0)
}
