use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

/// GracefulShutdown — 意识体优雅关闭
///
/// 三个层级:
/// 1. ShutdownSignal (CancellationToken 等价) — 通知各子系统停止
/// 2. DropGuard (RAII) — 作用域结束时自动信号
/// 3. Drain — 等待正在运行的任务完成
///
/// 受 OmniGet ActiveJobSlot + Zeltrex 6-Layer 启发
#[derive(Clone, Debug)]
pub struct ShutdownSignal {
    triggered: Arc<AtomicBool>,
    reason: Arc<std::sync::Mutex<Option<String>>>,
    notify: Arc<Notify>,
}

impl ShutdownSignal {
    pub fn new() -> Self {
        Self {
            triggered: Arc::new(AtomicBool::new(false)),
            reason: Arc::new(std::sync::Mutex::new(None)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn trigger(&self, reason: &str) {
        self.triggered.store(true, Ordering::SeqCst);
        if let Ok(mut r) = self.reason.lock() {
            *r = Some(reason.to_string());
        }
        self.notify.notify_one();
    }

    /// Async wait for shutdown signal.
    /// Checks AtomicBool first (handles trigger() between select! iterations),
    /// then awaits the async notification for zero-polling shutdown.
    pub async fn wait_shutdown(&self) {
        if self.is_shutdown() {
            return;
        }
        self.notify.notified().await;
    }

    pub fn is_shutdown(&self) -> bool {
        self.triggered.load(Ordering::SeqCst)
    }

    pub fn reason(&self) -> Option<String> {
        self.reason.lock().ok().and_then(|r| r.clone())
    }

    pub fn reset(&self) {
        self.triggered.store(false, Ordering::SeqCst);
        if let Ok(mut r) = self.reason.lock() {
            *r = None;
        }
    }
}

impl Default for ShutdownSignal {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII DropGuard — 作用域结束时自动触发 shutdown
pub struct DropGuard {
    signal: ShutdownSignal,
    reason: String,
}

impl DropGuard {
    pub fn new(signal: ShutdownSignal, reason: &str) -> Self {
        Self {
            signal,
            reason: reason.to_string(),
        }
    }
}

impl Drop for DropGuard {
    fn drop(&mut self) {
        self.signal.trigger(&self.reason);
    }
}

/// Phase 状态机 — 关闭阶段追踪
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ShutdownPhase {
    Running,
    Draining,
    Saving,
    FlushingEvents,
    Stopped,
}

impl ShutdownPhase {
    pub fn is_stopped(&self) -> bool {
        matches!(self, Self::Stopped)
    }

    pub fn can_proceed(&self) -> bool {
        !matches!(self, Self::Stopped)
    }
}

/// 完整的关闭管理器
pub struct GracefulShutdown {
    signal: ShutdownSignal,
    phase: std::sync::Mutex<ShutdownPhase>,
    drain_timeout: Duration,
}

impl GracefulShutdown {
    pub fn new(drain_timeout_secs: u64) -> Self {
        Self {
            signal: ShutdownSignal::new(),
            phase: std::sync::Mutex::new(ShutdownPhase::Running),
            drain_timeout: Duration::from_secs(drain_timeout_secs),
        }
    }

    pub fn signal(&self) -> &ShutdownSignal {
        &self.signal
    }

    pub fn phase(&self) -> ShutdownPhase {
        self.phase.lock().map(|p| *p).unwrap_or(ShutdownPhase::Stopped)
    }

    /// 开始关闭 — 设置信号 + 转换 phase
    pub fn initiate(&self, reason: &str) {
        self.signal.trigger(reason);
        if let Ok(mut p) = self.phase.lock() {
            if *p == ShutdownPhase::Running {
                *p = ShutdownPhase::Draining;
            }
        }
    }

    /// 标记进入 save phase
    pub fn enter_save(&self) {
        if let Ok(mut p) = self.phase.lock() {
            if *p == ShutdownPhase::Draining {
                *p = ShutdownPhase::Saving;
            }
        }
    }

    /// 标记关闭完成
    pub fn complete(&self) {
        if let Ok(mut p) = self.phase.lock() {
            *p = ShutdownPhase::Stopped;
        }
    }

    /// 等待 drain 完成或超时
    pub async fn drain(&self) {
        let mut elapsed: Duration = Duration::ZERO;
        let check_interval = Duration::from_millis(100);

        while elapsed < self.drain_timeout {
            if !self.signal.is_shutdown() {
                tokio::time::sleep(check_interval).await;
                elapsed += check_interval;
                continue;
            }
            // 检查是否有活跃工作
            let phase = self.phase();
            if phase == ShutdownPhase::Stopped || phase == ShutdownPhase::Saving {
                return;
            }
            tokio::time::sleep(check_interval).await;
            elapsed += check_interval;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_signal_default_not_triggered() {
        let s = ShutdownSignal::new();
        assert!(!s.is_shutdown());
        assert!(s.reason().is_none());
    }

    #[test]
    fn test_shutdown_signal_trigger() {
        let s = ShutdownSignal::new();
        s.trigger("test");
        assert!(s.is_shutdown());
        assert_eq!(s.reason().unwrap(), "test");
    }

    #[test]
    fn test_shutdown_signal_reset() {
        let s = ShutdownSignal::new();
        s.trigger("test");
        s.reset();
        assert!(!s.is_shutdown());
    }

    #[test]
    fn test_drop_guard_triggers_on_drop() {
        let signal = ShutdownSignal::new();
        {
            let _guard = DropGuard::new(signal.clone(), "scope exit");
        }
        assert!(signal.is_shutdown());
        assert_eq!(signal.reason().unwrap(), "scope exit");
    }

    #[test]
    fn test_graceful_shutdown_phase_transitions() {
        let gs = GracefulShutdown::new(5);
        assert_eq!(gs.phase(), ShutdownPhase::Running);

        gs.initiate("test");
        assert!(gs.signal().is_shutdown());
        assert_eq!(gs.phase(), ShutdownPhase::Draining);

        gs.enter_save();
        assert_eq!(gs.phase(), ShutdownPhase::Saving);

        gs.complete();
        assert_eq!(gs.phase(), ShutdownPhase::Stopped);
    }

    #[test]
    fn test_shutdown_phase_is_stopped() {
        assert!(!ShutdownPhase::Running.is_stopped());
        assert!(ShutdownPhase::Stopped.is_stopped());
    }
}
