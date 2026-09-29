//! # SUPERVISOR — 进程级自守护 (替代 entry/mod.rs 内联 supervisor)
//!
//! OTP 风格监督树的简化实现:
//! - PID 文件管理
//! - 心跳活度追踪
//! - 崩溃自动重启 + 指数退避 + 抖动
//! - 恢复日志写入 KB
//! - 不依赖 launchd/systemd
//!
//! 设计原则:
//! - "let it crash" + supervisor restart
//! - 指数退避 + 全抖动 (AWS recommended)
//! - 恢复预算: 超过上限停止重启, 防无限循环
//! - 心跳证明 reactor 进度, 非进程存在

#![forbid(unsafe_code)]

use std::sync::Arc;
use std::time::Duration;

use super::heartbeat::{HeartbeatState, spawn_heartbeat, spawn_health_writer};

/// Supervisor 配置
#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    /// 最大连续重启次数
    pub max_restarts: u32,
    /// 基础退避时间
    pub base_backoff: Duration,
    /// 最大退避时间
    pub max_backoff: Duration,
    /// PID 文件路径
    pub pid_path: std::path::PathBuf,
    /// 健康文件路径
    pub heartbeat_path: std::path::PathBuf,
    /// 恢复日志路径
    pub recovery_log: std::path::PathBuf,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        let home = std::env::var("HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));
        let nt_dir = home.join(".neotrix");
        Self {
            max_restarts: 10,
            base_backoff: Duration::from_secs(2),
            max_backoff: Duration::from_secs(120),
            pid_path: nt_dir.join("daemon.pid"),
            heartbeat_path: nt_dir.join("daemon.heartbeat"),
            recovery_log: nt_dir.join("daemon_recovery.log"),
        }
    }
}

/// Supervisor 事件 (供外部消费)
#[derive(Debug, Clone)]
pub enum SupervisorEvent {
    /// 子进程启动
    ChildStarted { pid: u32 },
    /// 子进程崩溃
    ChildCrashed { error: String, attempt: u32 },
    /// 正在重启 (含退避时间)
    Restarting { attempt: u32, backoff_secs: u64 },
    /// 重启预算耗尽
    BudgetExhausted { total_restarts: u32 },
    /// 正常关闭
    Shutdown { reason: String },
}

/// 计算带抖动的退避时间 (AWS Full Jitter)
pub fn backoff_with_jitter(base: Duration, attempt: u32, max: Duration) -> Duration {
    let exp_ms = base.as_millis() as f64 * 2_f64.powi(attempt as i32 - 1);
    let capped_ms = exp_ms.min(max.as_millis() as f64);
    // Full jitter: random(0, capped)
    let jittered = capped_ms * rand_factor();
    Duration::from_millis(jittered as u64)
}

/// 简易随机因子 (无 rand 依赖, 用时间哈希)
fn rand_factor() -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    // 简单哈希: 取低位做伪随机
    ((nanos & 0xFFFF) as f64) / 65536.0
}

/// 写入 PID 文件
pub fn write_pid_file(path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, std::process::id().to_string())
}

/// 清理 PID 文件
pub fn cleanup_pid_file(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
}

/// 写入恢复日志
pub fn log_recovery(log_path: &std::path::Path, event: &str, detail: &str) {
    use std::io::Write;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("[{}] {} — {}\n", ts, event, detail);
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        let _ = f.write_all(line.as_bytes());
    }
    log::info!("[supervisor] {} — {}", event, detail);
}

/// 创建共享的 HeartbeatState + 启动心跳任务
/// 返回 (heartbeat_state, heartbeat_task, health_writer)
pub fn setup_heartbeat(
    config: &SupervisorConfig,
) -> (Arc<HeartbeatState>, tokio::task::JoinHandle<()>, std::thread::JoinHandle<()>) {
    let state = Arc::new(HeartbeatState::new());
    let hb_task = spawn_heartbeat(state.clone());
    let hw_task = spawn_health_writer(config.heartbeat_path.clone(), state.clone());
    (state, hb_task, hw_task)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Full jitter 下单次抽样的单调性是伪命题（b1∈[0,1000)、b3∈[0,4000)，
    /// P(b1≥b3)≈1/8，2026-09-28 全量跑挂过一次）——测代码真正承诺的不变量：
    /// 每次结果必在 [0, capped] 内，且永不超 max。
    #[test]
    fn test_backoff_bounded_by_cap() {
        let base = Duration::from_secs(1);
        let max = Duration::from_secs(60);
        for attempt in 1..=8u32 {
            let cap_ms = (1000.0 * 2_f64.powi(attempt as i32 - 1)).min(60_000.0) as u128;
            for _ in 0..200 {
                let b = backoff_with_jitter(base, attempt, max);
                assert!(
                    b.as_millis() <= cap_ms,
                    "attempt {attempt}: {}ms > cap {cap_ms}ms",
                    b.as_millis()
                );
            }
        }
    }

    #[test]
    fn test_backoff_never_exceeds_max() {
        let base = Duration::from_secs(1);
        let max = Duration::from_secs(60);
        // attempt 很大（指数早被 cap）时仍 ≤ max。
        for _ in 0..200 {
            let b = backoff_with_jitter(base, 100, max);
            assert!(
                b.as_millis() <= max.as_millis(),
                "overflow max: {}ms",
                b.as_millis()
            );
        }
    }

    #[test]
    fn test_rand_factor_range() {
        for _ in 0..100 {
            let f = rand_factor();
            assert!(f >= 0.0 && f < 1.0, "rand_factor out of range: {}", f);
        }
    }

    #[test]
    fn test_supervisor_config_default() {
        let cfg = SupervisorConfig::default();
        assert_eq!(cfg.max_restarts, 10);
        assert!(cfg.pid_path.to_string_lossy().contains(".neotrix"));
    }
}
