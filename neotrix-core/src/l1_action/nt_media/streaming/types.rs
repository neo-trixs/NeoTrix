//! types — 从 `streaming.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::oneshot;
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::{Path, PathBuf};
use super::super::auth::AuthConfig;
use super::super::detect::MediaKind;

// ═══════════════════════════════════════════════════════════════════════════
// Shared progress types — single source of truth
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum PipelineStatus {
    Resolving,
    Downloading {
        downloaded: u64,
        total: Option<u64>,
        speed_bps: f64,
    },
    Playing {
        downloaded: u64,
        total: Option<u64>,
        speed_bps: f64,
    },
    Complete {
        total_bytes: u64,
        elapsed: Duration,
    },
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct PipelineProgress {
    pub url: String,
    pub status: PipelineStatus,
    pub media_kind: MediaKind,
    pub output: PathBuf,
    pub elapsed: Duration,
}

/// Single-task progress for wrapper callers (re-exported as `DownloadProgressSnapshot` in nt_io_download).
#[derive(Debug, Clone)]
pub struct DownloadProgressSnapshot {
    pub percent: f32,
    pub downloaded: u64,
    pub total: u64,
    pub speed_mbps: f64,
    pub eta_secs: Option<f64>,
}

/// Multi-task aggregate progress for batch downloads.
#[derive(Debug, Clone)]
pub struct AggregateProgress {
    pub total_tasks: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub active: usize,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub overall_speed_mbps: f64,
    pub overall_percent: f32,
}

/// Wrapper-level download status (returned by DownloadEngine).
#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Pending,
    InProgress(DownloadProgressSnapshot),
    Completed { elapsed_secs: f64, size_mb: f64 },
    Failed(String),
    Cancelled,
}

// ═══════════════════════════════════════════════════════════════════════════
// StreamingPipelineConfig — session configuration
// ═══════════════════════════════════════════════════════════════════════════

pub struct StreamingPipelineConfig {
    pub url: String,
    pub output_dir: PathBuf,
    pub prefer_streaming: bool,
    pub player_bin: Option<String>,
    pub player_args: Vec<String>,
    pub buffer_threshold: u64,
    pub proxy: Option<String>,
    pub timeout: Duration,
    // ⭐ **活的，跨模块消费**：pipeline.rs:116/279 读取（分块写入大小）。
    // ⛔ 死开关门曾把它报成「零读点」—— 那是**门的第二类误报**（详见 nt_dead_flag.py）：
    //    本字段在 types.rs 声明、在 pipeline.rs 被读，而门对「同名多声明」的字段
    //    只统计**声明文件内**的读点 ⇒ 跨模块消费被当成没消费。
    pub chunk_size: usize,
    pub persistence: Option<Arc<super::super::persistence::DownloadStore>>,
    pub auth: Option<AuthConfig>,
    // ⭐ **活的，跨模块消费**：pipeline.rs:124/163 读取（并发度）。
    // ⛔ 死开关门曾把它报成「零读点」—— 那是**门的第二类误报**（详见 nt_dead_flag.py）：
    //    本字段在 types.rs 声明、在 pipeline.rs 被读，而门对「同名多声明」的字段
    //    只统计**声明文件内**的读点 ⇒ 跨模块消费被当成没消费。
    pub concurrency: usize,
    pub verify_sha256: Option<String>,
    pub stall_timeout: Duration,
    pub max_retries: u32,
    pub atomic_write: bool,
    pub min_disk_space: u64,
    pub mirror_fallback_enabled: bool,
}

impl Default for StreamingPipelineConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            output_dir: PathBuf::from("/tmp/neotrix-stream"),
            prefer_streaming: false,
            player_bin: None,
            player_args: Vec::new(),
            buffer_threshold: 512 * 1024,
            proxy: None,
            timeout: Duration::from_secs(30),
            chunk_size: 256 * 1024,
            persistence: None,
            auth: None,
            concurrency: 4,
            verify_sha256: None,
            stall_timeout: Duration::from_secs(30),
            max_retries: 6,
            atomic_write: false,
            min_disk_space: 1024 * 1024 * 1024,
            mirror_fallback_enabled: true,
        }
    }
}

/// Backward-compatible alias
pub type PipelineConfig = StreamingPipelineConfig;

/// Download-specific configuration (merged from nt_io_download).
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub max_concurrent: usize,
    pub max_tasks: usize,
    // nt-unwired-spec: 未接线规格 —— D2 切片实测（2026-10-07）**全仓零读点**。
    // ⛔ B 类（功能没做，不是不要了）⇒ 保留并标注：删掉等于抹掉「带宽上限（MB/s）」这个规格。
    pub max_bandwidth: u64,
    pub min_disk_space: u64,
    pub timeout_secs: u64,
    pub retry_count: u32,
    pub max_chunk_bytes: u64,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 16,
            max_tasks: 8,
            max_bandwidth: 0,
            min_disk_space: 1024 * 1024 * 1024,
            timeout_secs: 600,
            retry_count: 5,
            max_chunk_bytes: 64 * 1024 * 1024,
        }
    }
}

impl DownloadConfig {
    /// Convert into a StreamingPipelineConfig base (player fields use defaults).
    pub fn to_pipeline_config(self, url: String, output_dir: PathBuf) -> StreamingPipelineConfig {
        StreamingPipelineConfig {
            url,
            output_dir,
            chunk_size: self.max_chunk_bytes as usize,
            timeout: Duration::from_secs(self.timeout_secs),
            max_retries: self.retry_count,
            concurrency: self.max_concurrent,
            min_disk_space: self.min_disk_space,
            ..Default::default()
        }
    }
}

/// Simple download task descriptor for wrapper callers.
#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub url: String,
    pub dest: PathBuf,
    pub priority: u8,
}

impl DownloadTask {
    pub fn new(url: impl Into<String>, dest: impl Into<PathBuf>) -> Self {
        Self {
            url: url.into(),
            dest: dest.into(),
            priority: 128,
        }
    }
    pub fn with_priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }
}

pub struct PipelineHandle {
    pub(crate) download_task: tokio::task::JoinHandle<Result<(), PipelineError>>,
    pub(crate) player_handle: Option<PlayerHandle>,
    pub(crate) cancel_tx: Option<oneshot::Sender<()>>,
    pub(crate) output: PathBuf,
}

impl PipelineHandle {
    pub fn output_path(&self) -> &Path {
        &self.output
    }
    pub fn cancel(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(());
        }
    }
    pub async fn wait(self) -> Result<(), PipelineError> {
        let _ = self.download_task.await;
        if let Some(player) = self.player_handle {
            let _ = player.handle.await;
        }
        Ok(())
    }
}

pub struct PlayerHandle {
    pub(crate) handle: tokio::task::JoinHandle<Option<()>>,
    pub(crate) stop_tx: Option<oneshot::Sender<()>>,
    pub(crate) file: PathBuf,
}

impl PlayerHandle {
    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Task cancel handle (exposed to wrapper callers).
pub struct TaskHandle {
    cancelled: Arc<AtomicBool>,
}

impl TaskHandle {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

/// Error type
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("cancelled")]
    Cancelled,
    #[error("network error: {0}")]
    Network(String),
    #[error("HTTP status {0}")]
    Http(u16),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("player error: {0}")]
    Player(String),
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("config error: {0}")]
    Config(String),
}
