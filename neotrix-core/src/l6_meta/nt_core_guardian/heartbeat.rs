//! # HEARTBEAT — 进程活度追踪 (AtomicI64, 无锁)
//!
//! 替代 daemon.heartbeat 文件写入 + DaemonMonitor 的 kill -0 检测。
//! 主 runtime 每秒 bump 一次, 读取方通过 tick_age_ms() 判断活度。
//!
//! 设计原则 (来自 zebflow/agentd-core):
//! - "a live PID is not a live agent" — 追踪 reactor 进度, 非进程存在
//! - 独立线程写入健康文件, 不被主 runtime 阻塞
//! - AtomicI64 无锁, 零分配, 每秒一次 store/load

#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicI64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// 活度判定阈值: 超过 5 秒无心跳视为不健康
const STALE_AFTER_MS: u64 = 5_000;

/// 全局活度状态 (Arc 共享, 跨线程零成本)
#[derive(Debug)]
pub struct HeartbeatState {
    /// 最后一次心跳的 Unix 时间戳 (毫秒)
    last_tick_ms: AtomicI64,
    /// 是否已请求关闭
    shutdown: AtomicBool,
    /// 启动时间
    started_at_ms: i64,
}

impl HeartbeatState {
    pub fn new() -> Self {
        let now = now_ms();
        Self {
            last_tick_ms: AtomicI64::new(now),
            shutdown: AtomicBool::new(false),
            started_at_ms: now,
        }
    }

    /// 主 runtime 调用: bump 心跳时间戳
    pub fn tick(&self) {
        self.last_tick_ms.store(now_ms(), Ordering::Relaxed);
    }

    /// 距离最后一次心跳的毫秒数
    pub fn tick_age_ms(&self) -> u64 {
        now_ms().saturating_sub(self.last_tick_ms.load(Ordering::Relaxed)) as u64
    }

    /// 是否存活: 心跳未过期 且 未请求关闭
    pub fn is_alive(&self) -> bool {
        self.tick_age_ms() < STALE_AFTER_MS && !self.shutdown.load(Ordering::Relaxed)
    }

    /// 请求关闭
    pub fn request_shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    /// 是否已请求关闭
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    /// 运行时间 (毫秒)
    pub fn uptime_ms(&self) -> u64 {
        now_ms().saturating_sub(self.started_at_ms) as u64
    }
}

impl Default for HeartbeatState {
    fn default() -> Self {
        Self::new()
    }
}

/// 启动心跳 bump 任务 (每秒)
pub fn spawn_heartbeat(state: Arc<HeartbeatState>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            state.tick();
        }
    })
}

/// 启动独立线程写入健康文件 (不被主 runtime 阻塞)
pub fn spawn_health_writer(
    path: std::path::PathBuf,
    state: Arc<HeartbeatState>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || loop {
        let alive = state.is_alive();
        let age = state.tick_age_ms();
        let uptime = state.uptime_ms();
        let body = serde_json::json!({
            "alive": alive,
            "tick_age_ms": age,
            "uptime_ms": uptime,
            "pid": std::process::id(),
            "shutdown_requested": state.is_shutdown_requested(),
        });
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // 原子写: tmp + rename
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, body.to_string().as_bytes()).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    })
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_fresh() {
        let state = HeartbeatState::new();
        assert!(state.is_alive());
        assert_eq!(state.tick_age_ms(), 0);
    }

    #[test]
    fn test_heartbeat_shutdown() {
        let state = HeartbeatState::new();
        state.request_shutdown();
        assert!(!state.is_alive());
        assert!(state.is_shutdown_requested());
    }

    #[test]
    fn test_heartbeat_uptime() {
        let state = HeartbeatState::new();
        let uptime = state.uptime_ms();
        assert!(uptime < 100); // 应该几乎为 0
    }
}
