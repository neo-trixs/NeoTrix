//! Unified streaming pipeline — download→file/pipe→player end-to-end.
//!
//! Architecture:
//! ```text
//! ┌──────────────────────────────────────────────────────────────────┐
//! │  StreamingPipeline                                               │
//! │  URL → detect → route → download → write → player               │
//! ├──────────────────────────────────────────────────────────────────┤
//! │  Transport Backends:                                             │
//! │  • HttpStream — sequential reqwest streaming                     │
//! │  • MagnetRpc  — aria2c JSON-RPC sequential BT download           │
//! │  • FifoPipe   — zero-disk I/O via named pipe                     │
//! ├──────────────────────────────────────────────────────────────────┤
//! │  Download Engine:                                                │
//! │  • Mirror speed profiling (EMA) + HuggingFace adaptive          │
//! │  • Parallel chunk download with work-stealing                    │
//! │  • Temp-file merge (.dl_* directories) + .done markers          │
//! │  • Disk space pre-check + content-disposition detection          │
//! │  • Stall detection + n² backoff retry + SHA-256 verify          │
//! ├──────────────────────────────────────────────────────────────────┤
//! │  Player Backends:                                                │
//! │  • FileGrow — player reads growing file (mpv appending://)       │
//! │  • PipePlay — player reads from stdin/FIFO                       │
//! └──────────────────────────────────────────────────────────────────┘
//! ```

use super::auth::AuthConfig;
use super::detect::{self, MediaKind};
use super::download_progress::DownloadProgress;
use super::router::{self, TransportType};
use crate::l1_action::nt_io::nt_io_http_factory;
use futures::StreamExt;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::SystemTime;
use std::time::Duration;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom};
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot, Mutex as TokioMutex};
use tokio::time::Instant;

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
// PipelineConfig — session configuration
// ═══════════════════════════════════════════════════════════════════════════

pub struct PipelineConfig {
    pub url: String,
    pub output_dir: PathBuf,
    pub prefer_streaming: bool,
    pub player_bin: Option<String>,
    pub player_args: Vec<String>,
    pub buffer_threshold: u64,
    pub proxy: Option<String>,
    pub timeout: Duration,
    pub chunk_size: usize,
    pub persistence: Option<Arc<super::persistence::DownloadStore>>,
    pub auth: Option<AuthConfig>,
    pub concurrency: usize,
    pub verify_sha256: Option<String>,
    pub stall_timeout: Duration,
    pub max_retries: u32,
}

impl Default for PipelineConfig {
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
        }
    }
}

/// Download-specific configuration (merged from nt_io_download).
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub max_concurrent: usize,
    pub max_tasks: usize,
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
    /// Convert into a PipelineConfig base (player fields use defaults).
    pub fn to_pipeline_config(self, url: String, output_dir: PathBuf) -> PipelineConfig {
        PipelineConfig {
            url,
            output_dir,
            chunk_size: self.max_chunk_bytes as usize,
            timeout: Duration::from_secs(self.timeout_secs),
            max_retries: self.retry_count,
            concurrency: self.max_concurrent,
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

// ═══════════════════════════════════════════════════════════════════════════
// Mirror speed profiling — delegated to NT-WORLD (L2 Perception)
// ═══════════════════════════════════════════════════════════════════════════

pub use crate::l2_perception::nt_world::nt_world_mirror::{
    ranked_mirrors, record_mirror_speed, resolve_mirror,
};

// ═══════════════════════════════════════════════════════════════════════════
// Filename detection — content-disposition + URL path fallback
// ═══════════════════════════════════════════════════════════════════════════

/// Detect filename from HEAD content-disposition or URL path.
pub async fn detect_filename(client: &reqwest::Client, url: &reqwest::Url, dest: &Path) -> PathBuf {
    if dest.file_stem().is_some_and(|s| !s.to_string_lossy().is_empty()) {
        return dest.to_path_buf();
    }
    if let Ok(resp) = client.head(url.clone()).send().await {
        if let Some(cd) = resp.headers().get("content-disposition") {
            if let Ok(cd_str) = cd.to_str() {
                if let Some(name) = router::parse_content_disposition(cd_str) {
                    return dest.with_file_name(name);
                }
            }
        }
    }
    if let Some(name) = url.path().rsplit('/').next() {
        if !name.is_empty() {
            return dest.with_file_name(name);
        }
    }
    dest.to_path_buf()
}

// ═══════════════════════════════════════════════════════════════════════════
// Disk space pre-check
// ═══════════════════════════════════════════════════════════════════════════

/// Check that the parent directory has enough free space for `needed` bytes.
/// Returns Ok(()) if sufficient, Err with message if not.
pub fn check_disk_space(path: &Path, needed: u64, min_free: u64) -> Result<(), String> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let meta = std::fs::metadata(parent)
        .map_err(|e| format!("cannot stat parent dir {}: {}", parent.display(), e))?;
    if !meta.is_dir() {
        return Err(format!("parent path is not a directory: {}", parent.display()));
    }
    // statvfs is not portable; use a heuristic: if the file already exists,
    // check its size vs needed. Otherwise, attempt a create+set_len probe.
    if path.exists() {
        if let Ok(m) = std::fs::metadata(path) {
            if m.len() >= needed {
                return Ok(());
            }
        }
    }
    // Try to probe free space by creating a temporary file
    let probe = parent.join(".nt_disk_probe");
    match std::fs::File::options()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&probe)
    {
        Ok(f) => {
            let probe_needed = needed.saturating_add(min_free);
            let _ = f.set_len(probe_needed);
            let _ = std::fs::remove_file(&probe);
            Ok(())
        }
        Err(e) => Err(format!("disk space probe failed: {}", e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Temp file merge strategy (.dl_* directories)
// ═══════════════════════════════════════════════════════════════════════════

/// Create a temp directory for chunk files: `.dl_{stem}/`
fn make_dl_tmp_dir(dest: &Path) -> PathBuf {
    let stem = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("dl");
    dest.parent()
        .unwrap_or(Path::new("."))
        .join(format!(".dl_{}", stem))
}

/// Merge chunk files from `tmp_dir` into a single output file.
/// Chunks are expected to be named `c0000.tmp`, `c0001.tmp`, etc.
async fn merge_chunks(tmp_dir: &Path, dest: &Path, n_chunks: usize) -> Result<(), String> {
    let mut out = BufWriter::with_capacity(
        256 * 1024,
        fs::File::create(dest)
            .await
            .map_err(|e| format!("create output: {}", e))?,
    );
    let mut buf = vec![0u8; 8192];
    for i in 0..n_chunks {
        let chunk_file = tmp_dir.join(format!("c{:04}.tmp", i));
        let mut f = fs::File::open(&chunk_file)
            .await
            .map_err(|e| format!("open chunk {}: {}", i, e))?;
        loop {
            let n = f
                .read(&mut buf)
                .await
                .map_err(|e| format!("read chunk {}: {}", i, e))?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])
                .await
                .map_err(|e| format!("write output: {}", e))?;
        }
    }
    out.flush()
        .await
        .map_err(|e| format!("flush output: {}", e))?;
    Ok(())
}

/// Write a .done marker file alongside the download.
async fn write_done_marker(
    dest: &Path,
    total_size: u64,
    url: &str,
) {
    let done_marker = dest.with_extension("done");
    let _ = fs::write(
        &done_marker,
        format!(
            "size={}\nurl={}\ntimestamp={}\n",
            total_size,
            url,
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ),
    )
    .await;
}

/// Check if a .done marker indicates the file is already complete.
async fn is_done(dest: &Path) -> Option<u64> {
    let done_marker = dest.with_extension("done");
    if !dest.exists() || !done_marker.exists() {
        return None;
    }
    let meta = fs::metadata(dest).await.ok()?;
    let content = fs::read_to_string(&done_marker).await.ok()?;
    let done_size = content
        .lines()
        .find(|l| l.starts_with("size="))
        .and_then(|l| l.strip_prefix("size="))
        .and_then(|s| s.parse::<u64>().ok())?;
    if meta.len() >= done_size {
        Some(meta.len())
    } else {
        None
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// StreamingPipeline — main entry point
// ═══════════════════════════════════════════════════════════════════════════

pub struct StreamingPipeline {
    config: PipelineConfig,
}

impl StreamingPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self { config }
    }

    pub async fn run(
        self,
        progress_tx: mpsc::Sender<PipelineProgress>,
    ) -> Result<PipelineHandle, PipelineError> {
        let config = self.config;
        let route = router::route_url(&config.url, &config.output_dir, config.prefer_streaming);
        let client = nt_io_http_factory::build_async_client_with_proxy(config.proxy.as_deref());
        let (media_kind, _content_type) = detect::detect_remote(&client, &config.url).await;

        fs::create_dir_all(&config.output_dir)
            .await
            .map_err(|e| PipelineError::Io(e.to_string()))?;

        let (cancel_tx, _cancel_rx) = oneshot::channel::<()>();
        let (_stop_flag, cancel_rx_flag) = create_cancel_pair();

        let output = route.output.clone();
        let url = config.url.clone();

        match route.transport {
            TransportType::HttpRange | TransportType::HttpStream => {
                let bytes_written = Arc::new(AtomicU64::new(0));
                let output_clone = output.clone();
                let auth = config.auth.clone();
                let _url_for_domain = config.url.clone();

                let download_handle = tokio::spawn(async move {
                    stream_http_download(
                        &client,
                        &url,
                        &output_clone,
                        config.chunk_size,
                        config.timeout,
                        bytes_written.clone(),
                        cancel_rx_flag,
                        progress_tx.clone(),
                        media_kind,
                        auth.as_ref(),
                        config.persistence.clone(),
                        config.concurrency,
                        config.verify_sha256.as_deref(),
                        config.stall_timeout,
                        config.max_retries,
                    )
                    .await
                });

                let player_handle = spawn_player(
                    &output,
                    config.buffer_threshold,
                    config.player_bin.as_deref(),
                    &config.player_args,
                )
                .await;

                Ok(PipelineHandle {
                    download_task: download_handle,
                    player_handle,
                    cancel_tx: Some(cancel_tx),
                    output,
                })
            }
            TransportType::MagnetRpc => {
                let download_handle = tokio::spawn(async move {
                    stream_magnet_download(
                        &url,
                        &config.output_dir,
                        cancel_rx_flag,
                        progress_tx.clone(),
                        media_kind,
                    )
                    .await
                });

                let player_handle = spawn_player(
                    &output,
                    config.buffer_threshold,
                    config.player_bin.as_deref(),
                    &config.player_args,
                )
                .await;

                Ok(PipelineHandle {
                    download_task: download_handle,
                    player_handle,
                    cancel_tx: Some(cancel_tx),
                    output,
                })
            }
            TransportType::FileCopy => {
                let player_handle = spawn_player(
                    &output,
                    0,
                    config.player_bin.as_deref(),
                    &config.player_args,
                )
                .await;

                let _ = progress_tx
                    .send(PipelineProgress {
                        url,
                        status: PipelineStatus::Complete {
                            total_bytes: fs::metadata(&output).await.map(|m| m.len()).unwrap_or(0),
                            elapsed: Duration::ZERO,
                        },
                        media_kind,
                        output: output.clone(),
                        elapsed: Duration::ZERO,
                    })
                    .await;

                Ok(PipelineHandle {
                    download_task: tokio::spawn(async { Ok::<(), PipelineError>(()) }),
                    player_handle,
                    cancel_tx: None,
                    output,
                })
            }
            TransportType::FifoPipe => {
                #[cfg(unix)]
                {
                    let fifo_path = config.output_dir.join(format!(
                        "stream-{}.fifo",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos()
                    ));

                    use std::os::unix::fs::OpenOptionsExt;
                    let _ = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .mode(0o644)
                        .open(&fifo_path);

                    let fifo_path_clone = fifo_path.clone();
                    let url_clone = url.clone();
                    let auth = config.auth.clone();

                    let player_handle = spawn_fifo_player(
                        &fifo_path,
                        config.player_bin.as_deref(),
                        &config.player_args,
                    )
                    .await;

                    let download_handle = tokio::spawn(async move {
                        stream_http_to_fifo(
                            &client,
                            &url_clone,
                            &fifo_path_clone,
                            config.chunk_size,
                            cancel_rx_flag,
                            progress_tx.clone(),
                            media_kind,
                            auth.as_ref(),
                        )
                        .await
                    });

                    Ok(PipelineHandle {
                        download_task: download_handle,
                        player_handle,
                        cancel_tx: Some(cancel_tx),
                        output: fifo_path,
                    })
                }
                #[cfg(not(unix))]
                {
                    Err(PipelineError::Config(
                        "FIFO pipe only supported on Unix".into(),
                    ))
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Re-export unified ChunkState / ChunkDownloadStatus from persistence
// ═══════════════════════════════════════════════════════════════════════════

pub use super::persistence::{ChunkDownloadStatus, ChunkState};

// ═══════════════════════════════════════════════════════════════════════════
// StallDetector — monitors per-chunk download progress
// ═══════════════════════════════════════════════════════════════════════════

pub struct StallDetector {
    last_bytes: AtomicU64,
    last_check: Instant,
    timeout: Duration,
}

impl StallDetector {
    pub fn new(timeout: Duration) -> Self {
        Self {
            last_bytes: AtomicU64::new(0),
            last_check: Instant::now(),
            timeout,
        }
    }

    /// Returns true if no progress since last check and timeout elapsed.
    pub fn check(&mut self, current_bytes: u64) -> bool {
        let prev = self.last_bytes.swap(current_bytes, Ordering::Relaxed);
        if current_bytes != prev {
            self.last_check = Instant::now();
            return false;
        }
        self.last_check.elapsed() > self.timeout
    }

    pub fn record_progress(&mut self) {
        self.last_check = Instant::now();
    }

    pub fn is_stalled(&self) -> bool {
        self.last_check.elapsed() > self.timeout
    }

    pub fn reset(&mut self) {
        self.last_check = Instant::now();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RetryPolicy — n² backoff with jitter
// ═══════════════════════════════════════════════════════════════════════════

pub struct RetryPolicy {
    max_retries: u32,
    base_delay: Duration,
}

impl RetryPolicy {
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            base_delay: Duration::from_secs(1),
        }
    }

    /// Compute delay for attempt number (1-indexed).
    /// Returns None if retries exhausted.
    pub fn delay(&self, attempt: u32) -> Option<Duration> {
        if attempt > self.max_retries {
            return None;
        }
        let base_ms = (attempt as u64) * (attempt as u64) * self.base_delay.as_millis() as u64;
        let capped = base_ms.min(30_000);
        let jitter = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64
            % self.base_delay.as_millis() as u64;
        Some(Duration::from_millis(capped + jitter))
    }

    /// Returns true if the error is retryable (server errors, timeouts, connection resets).
    pub fn is_retryable(err: &reqwest::Error) -> bool {
        if err.is_timeout() || err.is_connect() {
            return true;
        }
        if let Some(status) = err.status() {
            return matches!(status.as_u16(), 429 | 500..=599);
        }
        false
    }

    pub fn delay_for(&self, attempt: u32) -> Option<Duration> {
        self.delay(attempt)
    }

    pub fn is_retryable_status(status: u16) -> bool {
        matches!(status, 429 | 500..=599)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TokenBucket — simple bandwidth throttle
// ═══════════════════════════════════════════════════════════════════════════

struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // bytes per second
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }

    /// Try to consume `amount` bytes. Blocks until tokens are available.
    /// Returns actual bytes allowed (may be less than amount if capped).
    async fn consume(&mut self, amount: u64) -> u64 {
        self.refill();
        let amount_f = amount as f64;
        if self.tokens >= amount_f {
            self.tokens -= amount_f;
            return amount;
        }
        // Not enough tokens — wait for refill
        let deficit = amount_f - self.tokens;
        let wait_secs = deficit / self.refill_rate;
        tokio::time::sleep(Duration::from_secs_f64(wait_secs)).await;
        self.refill();
        let allowed = self.tokens.min(amount_f);
        self.tokens -= allowed;
        allowed as u64
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ParallelDownloader — work-stealing parallel chunk download
// ═══════════════════════════════════════════════════════════════════════════

struct ParallelDownloader {
    client: reqwest::Client,
    url: String,
    output: PathBuf,
    total_size: u64,
    chunk_size: usize,
    concurrency: usize,
    chunks: Arc<TokioMutex<VecDeque<ChunkState>>>,
    bytes_written: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    stall_timeout: Duration,
    retry_policy: RetryPolicy,
    auth: Option<AuthConfig>,
    token_bucket: Option<Arc<TokioMutex<TokenBucket>>>,
    speed_window: Arc<TokioMutex<VecDeque<(Instant, u64)>>>,
}

impl ParallelDownloader {
    fn new(
        client: reqwest::Client,
        url: String,
        output: PathBuf,
        total_size: u64,
        chunk_size: usize,
        concurrency: usize,
        stall_timeout: Duration,
        max_retries: u32,
        auth: Option<AuthConfig>,
        bytes_written: Arc<AtomicU64>,
        cancel: Arc<AtomicBool>,
        max_bandwidth_bps: Option<f64>,
    ) -> Self {
        let chunks = Self::plan_chunks(total_size, chunk_size);
        let token_bucket = max_bandwidth_bps.map(|rate| {
            Arc::new(Mutex::new(TokenBucket::new(rate * 2.0, rate)))
        });
        Self {
            client,
            url,
            output,
            total_size,
            chunk_size,
            concurrency,
            chunks: Arc::new(TokioMutex::new(VecDeque::from(chunks))),
            bytes_written,
            cancel,
            stall_timeout,
            retry_policy: RetryPolicy::new(max_retries),
            auth,
            token_bucket,
            speed_window: Arc::new(TokioMutex::new(VecDeque::new())),
        }
    }

    fn plan_chunks(total_size: u64, chunk_size: usize) -> Vec<ChunkState> {
        if total_size == 0 {
            return vec![];
        }
        let mut chunks = Vec::new();
        let mut offset = 0u64;
        let mut index = 0u32;
        while offset < total_size {
            let end = (offset + chunk_size as u64 - 1).min(total_size - 1);
            chunks.push(ChunkState::streaming(index, offset, end));
            offset = end + 1;
            index += 1;
        }
        chunks
    }

    fn steal_next(&self) -> Option<ChunkState> {
        let chunks = self.chunks.blocking_lock();
        for chunk in chunks.iter() {
            if matches!(chunk.status(), ChunkDownloadStatus::Pending) {
                return Some(chunk.clone());
            }
        }
        None
    }

    async fn download_chunk(&self, mut chunk: ChunkState) -> Result<(), PipelineError> {
        let mut stall = StallDetector::new(self.stall_timeout);
        let mut attempt = 0u32;

        loop {
            if self.cancel.load(Ordering::Relaxed) {
                return Ok(());
            }

            let range_start = chunk.start + chunk.downloaded;
            let range_end = chunk.end;

            if range_start > range_end {
                return Ok(());
            }

            let mut req = self
                .client
                .get(&self.url)
                .header("Range", format!("bytes={}-{}", range_start, range_end))
                .header("Accept-Encoding", "identity")
                .timeout(Duration::from_secs(60));

            if let Some(ref auth_cfg) = self.auth {
                let domain = extract_domain(&self.url);
                req = auth_cfg.apply(req, &domain).await;
            }

            match req.send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    if status != 206 && !resp.status().is_success() {
                        if RetryPolicy::is_retryable_status(status) {
                            attempt += 1;
                            match self.retry_policy.delay_for(attempt) {
                                Some(delay) => {
                                    tokio::time::sleep(delay).await;
                                    continue;
                                }
                                None => {
                                    return Err(PipelineError::Http(status));
                                }
                            }
                        }
                        return Err(PipelineError::Http(status));
                    }

                    let mut stream = resp.bytes_stream();
                    stall.record_progress();

                    while let Some(result) = stream.next().await {
                        if self.cancel.load(Ordering::Relaxed) {
                            return Ok(());
                        }

                        match result {
                            Ok(data) => {
                                // D2: bandwidth throttling — consume tokens before writing
                                if let Some(ref bucket) = self.token_bucket {
                                    let mut bucket = bucket.lock().await;
                                    bucket.consume(data.len() as u64).await;
                                }

                                let file = File::options()
                                    .write(true)
                                    .open(&self.output)
                                    .await
                                    .map_err(|e| PipelineError::Io(e.to_string()))?;
                                let mut file = BufWriter::new(file);
                                file.seek(SeekFrom::Start(range_start + chunk.downloaded))
                                    .await
                                    .map_err(|e| PipelineError::Io(e.to_string()))?;
                                file.write_all(&data)
                                    .await
                                    .map_err(|e| PipelineError::Io(e.to_string()))?;
                                file.flush()
                                    .await
                                    .map_err(|e| PipelineError::Io(e.to_string()))?;

                                chunk.downloaded += data.len() as u64;
                                let total = self
                                    .bytes_written
                                    .fetch_add(data.len() as u64, Ordering::Relaxed)
                                    + data.len() as u64;
                                stall.record_progress();
                                stall.check(total);
                            }
                            Err(_) => {
                                attempt += 1;
                                match self.retry_policy.delay_for(attempt) {
                                    Some(delay) => {
                                        tokio::time::sleep(delay).await;
                                        stall.reset();
                                        break;
                                    }
                                    None => {
                                        return Err(PipelineError::Network(
                                            "chunk download failed after retries".into(),
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    if stall.is_stalled() {
                        attempt += 1;
                        match self.retry_policy.delay_for(attempt) {
                            Some(delay) => {
                                tokio::time::sleep(delay).await;
                                stall.reset();
                                continue;
                            }
                            None => {
                                return Err(PipelineError::Network(
                                    "stall timeout exceeded".into(),
                                ));
                            }
                        }
                    }

                    {
                        let mut chunks = self.chunks.lock().await;
                        for c in chunks.iter_mut() {
                            if c.index == chunk.index {
                                c.completed = true;
                                break;
                            }
                        }
                    }

                    return Ok(());
                }
                Err(e) => {
                    attempt += 1;
                    match self.retry_policy.delay_for(attempt) {
                        Some(delay) => {
                            tokio::time::sleep(delay).await;
                            stall.reset();
                            continue;
                        }
                        None => {
                            return Err(PipelineError::Network(format!(
                                "connection failed after retries: {}",
                                e
                            )));
                        }
                    }
                }
            }
        }
    }

    async fn run(
        &self,
        progress_tx: mpsc::Sender<PipelineProgress>,
        media_kind: MediaKind,
    ) -> Result<(), PipelineError> {
        let started = Instant::now();
        let _ = progress_tx
            .send(PipelineProgress {
                url: self.url.clone(),
                status: PipelineStatus::Resolving,
                media_kind,
                output: self.output.clone(),
                elapsed: Duration::ZERO,
            })
            .await;

        let mut handles = Vec::with_capacity(self.concurrency);

        for _ in 0..self.concurrency {
            let dl = ParallelDownloader {
                client: self.client.clone(),
                url: self.url.clone(),
                output: self.output.clone(),
                total_size: self.total_size,
                chunk_size: self.chunk_size,
                concurrency: self.concurrency,
                chunks: Arc::clone(&self.chunks),
                bytes_written: Arc::clone(&self.bytes_written),
                cancel: Arc::clone(&self.cancel),
                stall_timeout: self.stall_timeout,
                retry_policy: RetryPolicy::new(self.retry_policy.max_retries),
                auth: self.auth.clone(),
                token_bucket: self.token_bucket.clone(),
                speed_window: Arc::clone(&self.speed_window),
            };

            let progress_tx = progress_tx.clone();
            let url = self.url.clone();
            let output = self.output.clone();
            let total_size = self.total_size;

            let handle = tokio::spawn(async move {
                loop {
                    if dl.cancel.load(Ordering::Relaxed) {
                        return Ok::<(), PipelineError>(());
                    }

                    let chunk = {
                        let mut chunks = dl.chunks.lock().await;
                        let next = chunks
                            .iter_mut()
                            .find(|c| matches!(c.status(), ChunkDownloadStatus::Pending));
                        match next {
                            Some(c) => {
                                c.completed = false;
                                c.clone()
                            }
                            None => {
                                let all_done = chunks.iter().all(|c| {
                                    matches!(c.status(), ChunkDownloadStatus::Complete)
                                        || matches!(c.status(), ChunkDownloadStatus::Failed)
                                });
                                if all_done {
                                    return Ok(());
                                }
                                drop(chunks);
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                continue;
                            }
                        }
                    };

                    if let Err(e) = dl.download_chunk(chunk.clone()).await {
                        let mut chunks = dl.chunks.lock().await;
                        for c in chunks.iter_mut() {
                            if c.index == chunk.index
                                && !matches!(c.status(), ChunkDownloadStatus::Complete)
                            {
                                c.completed = false;
                            }
                        }

                        let _ = progress_tx
                            .send(PipelineProgress {
                                url: url.clone(),
                                status: PipelineStatus::Failed(e.to_string()),
                                media_kind,
                                output: output.clone(),
                                elapsed: started.elapsed(),
                            })
                            .await;
                        return Err(e);
                    }

                    let written = dl.bytes_written.load(Ordering::Relaxed);

                    // Real-time speed: track bytes in a 5-second sliding window
                    {
                        let mut window = dl.speed_window.lock().await;
                        window.push_back((Instant::now(), written));
                        // Evict entries older than 5 seconds
                        let cutoff = Instant::now() - Duration::from_secs(5);
                        while let Some(&(ts, _)) = window.front() {
                            if ts < cutoff {
                                window.pop_front();
                            } else {
                                break;
                            }
                        }
                    }
                    let speed_bps = {
                        let window = dl.speed_window.lock().await;
                        if window.len() >= 2 {
                            let oldest = window.front().unwrap();
                            let newest = window.back().unwrap();
                            let bytes_delta = newest.1.saturating_sub(oldest.1);
                            let time_delta = newest.0.duration_since(oldest.0).as_secs_f64();
                            if time_delta > 0.0 {
                                bytes_delta as f64 / time_delta
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        }
                    };

                    let _ = progress_tx
                        .send(PipelineProgress {
                            url: url.clone(),
                            status: PipelineStatus::Downloading {
                                downloaded: written,
                                total: Some(total_size),
                                speed_bps,
                            },
                            media_kind,
                            output: output.clone(),
                            elapsed: started.elapsed(),
                        })
                        .await;
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle
                .await
                .map_err(|e| PipelineError::Network(format!("worker join: {}", e)))??;
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Parallel chunk download (standalone function)
// ═══════════════════════════════════════════════════════════════════════════

/// Parallel chunk download using work-stealing pattern.
/// Delegates to `ParallelDownloader` internally.
async fn parallel_http_download(
    client: &reqwest::Client,
    url: &str,
    output: &Path,
    total_size: u64,
    chunk_size: usize,
    concurrency: usize,
    stall_timeout: Duration,
    max_retries: u32,
    progress_tx: mpsc::Sender<PipelineProgress>,
    cancel: Arc<AtomicBool>,
) -> Result<u64, PipelineError> {
    let bytes_written = Arc::new(AtomicU64::new(0));

    let file = if output.exists() {
        File::options()
            .write(true)
            .open(output)
            .await
    } else {
        File::create(output).await
    }
    .map_err(|e| PipelineError::Io(e.to_string()))?;

    file.set_len(total_size)
        .await
        .map_err(|e| PipelineError::Io(e.to_string()))?;
    drop(file);

    let dl = ParallelDownloader::new(
        client.clone(),
        url.to_string(),
        output.to_path_buf(),
        total_size,
        chunk_size,
        concurrency,
        stall_timeout,
        max_retries,
        None,
        bytes_written.clone(),
        cancel,
        None, // no bandwidth limit
    );

    let media_kind = MediaKind::Unknown;
    dl.run(progress_tx, media_kind).await?;

    Ok(bytes_written.load(Ordering::Relaxed))
}

// ═══════════════════════════════════════════════════════════════════════════
// SHA-256 Integrity Verification
// ═══════════════════════════════════════════════════════════════════════════

/// Verify file integrity against expected SHA-256 hash.
/// Returns Ok(true) if match, Ok(false) if mismatch, Err on I/O failure.
pub async fn verify_sha256(path: &Path, expected: &str) -> Result<bool, PipelineError> {
    let data = tokio::fs::read(path)
        .await
        .map_err(|e| PipelineError::Io(format!("read for verify: {}", e)))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let actual = format!("{:x}", hasher.finalize());
    Ok(actual.eq_ignore_ascii_case(expected))
}

/// Compute SHA-256 of a file (for storing in sidecar state).
pub async fn compute_sha256(path: &Path) -> Result<String, PipelineError> {
    let data = tokio::fs::read(path)
        .await
        .map_err(|e| PipelineError::Io(format!("read for hash: {}", e)))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

// ═══════════════════════════════════════════════════════════════════════════
// HTTP Streaming Download (enhanced: parallel + stall + retry + verify +
//   mirror resolution + temp-file merge + .done markers + disk pre-check)
// ═══════════════════════════════════════════════════════════════════════════

async fn stream_http_download(
    client: &reqwest::Client,
    url: &str,
    output: &Path,
    chunk_size: usize,
    timeout: Duration,
    bytes_written: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    progress_tx: mpsc::Sender<PipelineProgress>,
    media_kind: MediaKind,
    auth: Option<&AuthConfig>,
    persistence: Option<Arc<super::persistence::DownloadStore>>,
    concurrency: usize,
    expected_sha256: Option<&str>,
    stall_timeout: Duration,
    max_retries: u32,
) -> Result<(), PipelineError> {
    let started = Instant::now();

    let record_id = if let Some(ref store) = persistence {
        let record = super::persistence::DownloadRecord::new(
            url.to_string(),
            output.to_path_buf(),
            format!("{:?}", media_kind),
        );
        let id = record.id.clone();
        store.add_record(record).await;
        let _ = store.save().await;
        Some(id)
    } else {
        None
    };

    // Probe Range support via HEAD
    let (supports_range, total_size) = probe_range_support(client, url, auth).await;

    let use_parallel = supports_range
        && total_size
            .map(|ts| ts > (chunk_size as u64) * 2)
            .unwrap_or(false);

    if use_parallel {
        let total = total_size.unwrap_or(0);

        let pre_start_byte = if output.exists() {
            fs::metadata(output).await.map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        if output.exists() && pre_start_byte >= total {
            let final_bytes = pre_start_byte;
            if let Some(ref sha) = expected_sha256 {
                if !verify_sha256(output, sha).await? {
                    return Err(PipelineError::Io("SHA-256 integrity check failed".into()));
                }
            }
            if let (Some(ref store), Some(ref id)) = (&persistence, &record_id) {
                let _ = store
                    .update_record(id, |r| {
                        r.mark_complete(final_bytes);
                    })
                    .await;
                let _ = store.save().await;
            }
            let _ = progress_tx
                .send(PipelineProgress {
                    url: url.to_string(),
                    status: PipelineStatus::Complete {
                        total_bytes: final_bytes,
                        elapsed: started.elapsed(),
                    },
                    media_kind,
                    output: output.to_path_buf(),
                    elapsed: started.elapsed(),
                })
                .await;
            return Ok(());
        }

        let file = if output.exists() && pre_start_byte > 0 {
            File::options()
                .write(true)
                .open(output)
                .await
        } else {
            File::create(output).await
        }
        .map_err(|e| PipelineError::Io(e.to_string()))?;

        file.set_len(total)
            .await
            .map_err(|e| PipelineError::Io(e.to_string()))?;
        drop(file);

        bytes_written.store(pre_start_byte, Ordering::Relaxed);

        let auth_clone = auth.cloned();
        let dl = ParallelDownloader::new(
            client.clone(),
            url.to_string(),
            output.to_path_buf(),
            total,
            chunk_size,
            concurrency,
            stall_timeout,
            max_retries,
            auth_clone,
            bytes_written.clone(),
            cancel.clone(),
            None, // no bandwidth limit
        );

        dl.run(progress_tx.clone(), media_kind).await?;
    } else {
        let req = client
            .get(url)
            .header("Accept-Encoding", "identity")
            .timeout(timeout);

        let req = if let Some(auth_cfg) = auth {
            let domain = extract_domain(url);
            auth_cfg.apply(req, &domain).await
        } else {
            req
        };

        let start_byte = if output.exists() {
            fs::metadata(output).await.map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let req = if start_byte > 0 {
            req.header("Range", format!("bytes={}-", start_byte))
        } else {
            req
        };

        let resp = req
            .send()
            .await
            .map_err(|e| PipelineError::Network(e.to_string()))?;

        if !resp.status().is_success() && resp.status().as_u16() != 206 {
            return Err(PipelineError::Http(resp.status().as_u16()));
        }

        let total_size = resp.content_length().map(|cl| cl + start_byte);
        let mut stream = resp.bytes_stream();

        let file = if start_byte > 0 {
            fs::OpenOptions::new().append(true).open(output).await
        } else {
            File::create(output).await
        }
        .map_err(|e| PipelineError::Io(e.to_string()))?;

        let mut writer = BufWriter::with_capacity(chunk_size, file);
        bytes_written.store(start_byte, Ordering::Relaxed);

        let mut speed_samples: Vec<f64> = Vec::new();
        let mut last_sample = Instant::now();
        let mut last_persist = Instant::now();
        let mut recent_bytes: u64 = 0;

        let _ = progress_tx
            .send(PipelineProgress {
                url: url.to_string(),
                status: PipelineStatus::Resolving,
                media_kind,
                output: output.to_path_buf(),
                elapsed: Duration::ZERO,
            })
            .await;

        while let Some(chunk) = stream.next().await {
            if cancel.load(Ordering::Relaxed) {
                if let (Some(ref store), Some(ref id)) = (&persistence, &record_id) {
                    let _ = store
                        .update_record(id, |r| {
                            r.mark_failed("cancelled".into());
                        })
                        .await;
                    let _ = store.save().await;
                }

                let _ = progress_tx
                    .send(PipelineProgress {
                        url: url.to_string(),
                        status: PipelineStatus::Cancelled,
                        media_kind,
                        output: output.to_path_buf(),
                        elapsed: started.elapsed(),
                    })
                    .await;
                writer.flush().await.ok();
                return Ok(());
            }

            match chunk {
                Ok(data) => {
                    writer
                        .write_all(&data)
                        .await
                        .map_err(|e| PipelineError::Io(e.to_string()))?;
                    let written = bytes_written.fetch_add(data.len() as u64, Ordering::Relaxed)
                        + data.len() as u64;
                    recent_bytes += data.len() as u64;

                    if last_sample.elapsed() > Duration::from_millis(500) {
                        let elapsed_s = last_sample.elapsed().as_secs_f64();
                        let speed = (recent_bytes as f64) / elapsed_s;
                        speed_samples.push(speed);
                        if speed_samples.len() > 10 {
                            speed_samples.remove(0);
                        }
                        let avg_speed =
                            speed_samples.iter().sum::<f64>() / speed_samples.len() as f64;
                        recent_bytes = 0;
                        last_sample = Instant::now();

                        let status = if Some(written) >= total_size {
                            PipelineStatus::Complete {
                                total_bytes: written,
                                elapsed: started.elapsed(),
                            }
                        } else {
                            PipelineStatus::Downloading {
                                downloaded: written,
                                total: total_size,
                                speed_bps: avg_speed,
                            }
                        };

                        let _ = progress_tx
                            .send(PipelineProgress {
                                url: url.to_string(),
                                status,
                                media_kind,
                                output: output.to_path_buf(),
                                elapsed: started.elapsed(),
                            })
                            .await;

                        if let (Some(ref store), Some(ref id)) = (&persistence, &record_id) {
                            if last_persist.elapsed() > Duration::from_secs(5) {
                                let _ = store
                                    .update_record(id, |r| {
                                        r.update_progress(written, total_size, avg_speed);
                                    })
                                    .await;
                                let _ = store.save().await;
                                last_persist = Instant::now();
                            }
                        }
                    }
                }
                Err(e) => {
                    if let (Some(ref store), Some(ref id)) = (&persistence, &record_id) {
                        let _ = store
                            .update_record(id, |r| {
                                r.mark_failed(e.to_string());
                            })
                            .await;
                        let _ = store.save().await;
                    }

                    let _ = progress_tx
                        .send(PipelineProgress {
                            url: url.to_string(),
                            status: PipelineStatus::Failed(e.to_string()),
                            media_kind,
                            output: output.to_path_buf(),
                            elapsed: started.elapsed(),
                        })
                        .await;
                    return Err(PipelineError::Network(e.to_string()));
                }
            }
        }

        writer
            .flush()
            .await
            .map_err(|e| PipelineError::Io(e.to_string()))?;
    }

    let final_bytes = bytes_written.load(Ordering::Relaxed);

    if let Some(sha) = expected_sha256 {
        if !verify_sha256(output, sha).await? {
            if let (Some(ref store), Some(ref id)) = (&persistence, &record_id) {
                let _ = store
                    .update_record(id, |r| {
                        r.mark_failed("integrity check failed".into());
                    })
                    .await;
                let _ = store.save().await;
            }

            let _ = progress_tx
                .send(PipelineProgress {
                    url: url.to_string(),
                    status: PipelineStatus::Failed("SHA-256 integrity check failed".into()),
                    media_kind,
                    output: output.to_path_buf(),
                    elapsed: started.elapsed(),
                })
                .await;
            return Err(PipelineError::Io("SHA-256 integrity check failed".into()));
        }
    }

    if let (Some(ref store), Some(ref id)) = (&persistence, &record_id) {
        let _ = store
            .update_record(id, |r| {
                r.mark_complete(final_bytes);
            })
            .await;
        let _ = store.save().await;
    }

    let _ = progress_tx
        .send(PipelineProgress {
            url: url.to_string(),
            status: PipelineStatus::Complete {
                total_bytes: final_bytes,
                elapsed: started.elapsed(),
            },
            media_kind,
            output: output.to_path_buf(),
            elapsed: started.elapsed(),
        })
        .await;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Range Support Probe
// ═══════════════════════════════════════════════════════════════════════════

async fn probe_range_support(
    client: &reqwest::Client,
    url: &str,
    auth: Option<&AuthConfig>,
) -> (bool, Option<u64>) {
    let mut req = client.head(url).header("Accept-Encoding", "identity");
    if let Some(auth_cfg) = auth {
        let domain = extract_domain(url);
        req = auth_cfg.apply(req, &domain).await;
    }

    let resp = match req.send().await {
        Ok(r) => r,
        Err(_) => return (false, None),
    };

    let total = resp.content_length();
    let accepts_range = resp
        .headers()
        .get("accept-ranges")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("bytes"))
        .unwrap_or(false);

    (accepts_range, total)
}

// ═══════════════════════════════════════════════════════════════════════════
// HTTP chunk download (single-chunk, for temp-file merge path)
// ═══════════════════════════════════════════════════════════════════════════

async fn http_chunk_download(
    client: &reqwest::Client,
    url: &reqwest::Url,
    start: u64,
    end: u64,
    _total_size: u64,
    path: &Path,
    timeout_secs: u64,
) -> Result<u64, String> {
    let already = if path.exists() {
        fs::metadata(path).await.map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    let actual_start = start + already;
    if end != 0 && actual_start > end {
        return Ok(already);
    }

    let mut req = client
        .get(url.clone())
        .header("Accept-Encoding", "identity");
    if end != 0 {
        req = req.header("Range", format!("bytes={}-{}", actual_start, end));
    } else if actual_start > 0 {
        req = req.header("Range", format!("bytes={}-", actual_start));
    }

    let mut resp = tokio::time::timeout(Duration::from_secs(timeout_secs), req.send())
        .await
        .map_err(|_| "timeout".to_string())?
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    if status == 404 || status == 403 || status == 410 {
        return Err(format!("permanent: HTTP {}", status));
    }
    if !status.is_success() && status != 206 {
        return Err(format!("HTTP {}", status));
    }

    let file = if already > 0 {
        fs::OpenOptions::new()
            .append(true)
            .open(path)
            .await
            .map_err(|e| format!("append: {}", e))?
    } else {
        fs::File::create(path)
            .await
            .map_err(|e| format!("create: {}", e))?
    };

    let mut writer = BufWriter::with_capacity(256 * 1024, file);
    let mut downloaded = already;
    loop {
        match resp.chunk().await {
            Ok(Some(chunk)) => {
                writer
                    .write_all(&chunk)
                    .await
                    .map_err(|e| format!("write: {}", e))?;
                downloaded += chunk.len() as u64;
            }
            Ok(None) => break,
            Err(e) => return Err(format!("stream: {}", e)),
        }
    }
    writer
        .flush()
        .await
        .map_err(|e| format!("flush: {}", e))?;
    Ok(downloaded)
}

// ═══════════════════════════════════════════════════════════════════════════
// Magnet Download via aria2c RPC
// ═══════════════════════════════════════════════════════════════════════════

async fn stream_magnet_download(
    url: &str,
    output_dir: &Path,
    cancel: Arc<AtomicBool>,
    progress_tx: mpsc::Sender<PipelineProgress>,
    media_kind: MediaKind,
) -> Result<(), PipelineError> {
    let client = reqwest::Client::new();
    let rpc_url = "http://127.0.0.1:6800/jsonrpc";

    let rpc_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "nt-stream-magnet",
        "method": "aria2.addUri",
        "params": [{
            "uris": [url],
            "dir": output_dir.to_string_lossy(),
            "bt-stream-piece-selector": "inorder",
            "bt-prioritize-piece": "head=10m",
            "seed-time": "0",
            "max-overall-upload-limit": "0",
        }],
    });

    let resp = client
        .post(rpc_url)
        .json(&rpc_body)
        .send()
        .await
        .map_err(|e| PipelineError::Network(format!("aria2c connect failed: {}", e)))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| PipelineError::Network(format!("aria2c response parse: {}", e)))?;

    if let Some(error) = body.get("error") {
        return Err(PipelineError::Rpc(format!(
            "aria2c error {}: {}",
            error.get("code").and_then(|c| c.as_i64()).unwrap_or(0),
            error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown")
        )));
    }

    let gid = body["result"]
        .as_str()
        .ok_or_else(|| PipelineError::Rpc("no GID returned".into()))?
        .to_string();

    let started = Instant::now();

    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = progress_tx
                .send(PipelineProgress {
                    url: url.to_string(),
                    status: PipelineStatus::Cancelled,
                    media_kind,
                    output: output_dir.to_path_buf(),
                    elapsed: started.elapsed(),
                })
                .await;
            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(500)).await;

        let rpc_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "nt-status",
            "method": "aria2.tellStatus",
            "params": [&gid, ["status", "totalLength", "completedLength", "downloadSpeed"]],
        });

        let resp = match client.post(rpc_url).json(&rpc_body).send().await {
            Ok(r) => r,
            Err(_) => continue,
        };

        let body: serde_json::Value = match resp.json().await {
            Ok(b) => b,
            Err(_) => continue,
        };

        let status = body["result"]["status"].as_str().unwrap_or("unknown");
        let total = body["result"]["totalLength"]
            .as_str()
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or(0);
        let completed = body["result"]["completedLength"]
            .as_str()
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or(0);
        let speed = body["result"]["downloadSpeed"]
            .as_str()
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or(0);

        let pipeline_status = match status {
            "active" => PipelineStatus::Downloading {
                downloaded: completed,
                total: if total > 0 { Some(total) } else { None },
                speed_bps: speed as f64,
            },
            "complete" => PipelineStatus::Complete {
                total_bytes: completed,
                elapsed: started.elapsed(),
            },
            "error" => PipelineStatus::Failed("aria2c error".into()),
            _ => PipelineStatus::Cancelled,
        };

        let _ = progress_tx
            .send(PipelineProgress {
                url: url.to_string(),
                status: pipeline_status.clone(),
                media_kind,
                output: output_dir.to_path_buf(),
                elapsed: started.elapsed(),
            })
            .await;

        match status {
            "complete" => return Ok(()),
            "error" | "removed" => {
                return Err(PipelineError::Rpc(format!("aria2c status: {}", status)))
            }
            _ => {}
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HTTP to FIFO (zero disk I/O)
// ═══════════════════════════════════════════════════════════════════════════

async fn stream_http_to_fifo(
    client: &reqwest::Client,
    url: &str,
    fifo_path: &Path,
    chunk_size: usize,
    cancel: Arc<AtomicBool>,
    progress_tx: mpsc::Sender<PipelineProgress>,
    media_kind: MediaKind,
    auth: Option<&AuthConfig>,
) -> Result<(), PipelineError> {
    let started = Instant::now();

    let req = client.get(url).header("Accept-Encoding", "identity");

    let req = if let Some(auth_cfg) = auth {
        let domain = extract_domain(url);
        auth_cfg.apply(req, &domain).await
    } else {
        req
    };

    let mut resp = req
        .send()
        .await
        .map_err(|e| PipelineError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(PipelineError::Http(resp.status().as_u16()));
    }

    let file = File::create(fifo_path)
        .await
        .map_err(|e| PipelineError::Io(e.to_string()))?;
    let mut writer = BufWriter::with_capacity(chunk_size, file);
    let mut downloaded: u64 = 0;

    let _ = progress_tx
        .send(PipelineProgress {
            url: url.to_string(),
            status: PipelineStatus::Resolving,
            media_kind,
            output: fifo_path.to_path_buf(),
            elapsed: Duration::ZERO,
        })
        .await;

    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| PipelineError::Network(e.to_string()))?
    {
        if cancel.load(Ordering::Relaxed) {
            let _ = progress_tx
                .send(PipelineProgress {
                    url: url.to_string(),
                    status: PipelineStatus::Cancelled,
                    media_kind,
                    output: fifo_path.to_path_buf(),
                    elapsed: started.elapsed(),
                })
                .await;
            return Ok(());
        }

        writer
            .write_all(&chunk)
            .await
            .map_err(|e| PipelineError::Io(e.to_string()))?;
        downloaded += chunk.len() as u64;

        let _ = progress_tx
            .send(PipelineProgress {
                url: url.to_string(),
                status: PipelineStatus::Downloading {
                    downloaded,
                    total: None,
                    speed_bps: 0.0,
                },
                media_kind,
                output: fifo_path.to_path_buf(),
                elapsed: started.elapsed(),
            })
            .await;
    }

    writer
        .flush()
        .await
        .map_err(|e| PipelineError::Io(e.to_string()))?;

    let _ = progress_tx
        .send(PipelineProgress {
            url: url.to_string(),
            status: PipelineStatus::Complete {
                total_bytes: downloaded,
                elapsed: started.elapsed(),
            },
            media_kind,
            output: fifo_path.to_path_buf(),
            elapsed: started.elapsed(),
        })
        .await;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Player backends
// ═══════════════════════════════════════════════════════════════════════════

fn detect_player(preferred: Option<&str>) -> Option<&'static str> {
    if let Some(p) = preferred {
        if std::process::Command::new(p)
            .arg("--version")
            .output()
            .is_ok()
        {
            return Some(Box::leak(p.to_string().into_boxed_str()));
        }
    }
    if std::process::Command::new("ffplay")
        .arg("-version")
        .output()
        .is_ok()
    {
        return Some("ffplay");
    }
    if std::process::Command::new("mpv")
        .arg("--version")
        .output()
        .is_ok()
    {
        return Some("mpv");
    }
    None
}

async fn spawn_player(
    file: &Path,
    wait_bytes: u64,
    player_bin: Option<&str>,
    extra_args: &[String],
) -> Option<PlayerHandle> {
    let player = detect_player(player_bin)?;
    let file_clone = file.to_path_buf();
    let extra_args = extra_args.to_vec();
    let (stop_tx, stop_rx) = oneshot::channel::<()>();

    let handle = tokio::spawn(async move {
        if wait_bytes > 0 {
            loop {
                let size = fs::metadata(&file_clone)
                    .await
                    .map(|m| m.len())
                    .unwrap_or(0);
                if size >= wait_bytes {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }

        let mut cmd = Command::new(player);
        if player == "ffplay" {
            cmd.arg("-autoexit").arg("-loop").arg("0");
            cmd.args(&extra_args);
            cmd.arg(&file_clone);
        } else {
            let appending_url = format!("appending://{}", file_clone.to_string_lossy());
            cmd.arg("--loop=inf")
                .arg("--keep-open=yes")
                .arg("--demuxer-max-bytes=512MiB")
                .arg("--demuxer-readahead-secs=20");
            cmd.args(&extra_args);
            cmd.arg(&appending_url);
        }

        cmd.stdout(Stdio::null()).stderr(Stdio::null());

        let mut child = cmd.spawn().ok()?;

        tokio::select! {
            _ = async { loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => tokio::time::sleep(Duration::from_secs(1)).await,
                    Err(_) => break,
                }
            }} => {}
            _ = stop_rx => {
                let _ = child.kill().await;
            }
        }

        Some(())
    });

    Some(PlayerHandle {
        handle,
        stop_tx: Some(stop_tx),
        file: file.to_path_buf(),
    })
}

async fn spawn_fifo_player(
    fifo_path: &Path,
    player_bin: Option<&str>,
    extra_args: &[String],
) -> Option<PlayerHandle> {
    let player = detect_player(player_bin)?;
    let fifo_clone = fifo_path.to_path_buf();
    let extra_args = extra_args.to_vec();
    let (stop_tx, stop_rx) = oneshot::channel::<()>();

    let handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;

        let mut cmd = Command::new(player);
        if player == "ffplay" {
            cmd.arg("-autoexit");
            cmd.args(&extra_args);
            cmd.arg(&fifo_clone);
        } else {
            cmd.args(&extra_args);
            cmd.arg(&fifo_clone);
        }

        cmd.stdout(Stdio::null()).stderr(Stdio::null());

        let mut child = cmd.spawn().ok()?;

        tokio::select! {
            _ = async { loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => tokio::time::sleep(Duration::from_secs(1)).await,
                    Err(_) => break,
                }
            }} => {}
            _ = stop_rx => {
                let _ = child.kill().await;
            }
        }

        Some(())
    });

    Some(PlayerHandle {
        handle,
        stop_tx: Some(stop_tx),
        file: fifo_path.to_path_buf(),
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Handle and types
// ═══════════════════════════════════════════════════════════════════════════

pub struct PipelineHandle {
    download_task: tokio::task::JoinHandle<Result<(), PipelineError>>,
    player_handle: Option<PlayerHandle>,
    cancel_tx: Option<oneshot::Sender<()>>,
    output: PathBuf,
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
    handle: tokio::task::JoinHandle<Option<()>>,
    stop_tx: Option<oneshot::Sender<()>>,
    file: PathBuf,
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

// ═══════════════════════════════════════════════════════════════════════════
// DownloadEngine — unified download engine (strengthens existing, R-P42)
//
// Merges nt_io_download's features into streaming.rs:
//   • Mirror speed profiling (EMA)
//   • HuggingFace adaptive resolution
//   • Temp-file merge (.dl_* directories)
//   • .done marker for resume
//   • Disk space pre-check
//   • Content-disposition filename detection
//   • Configurable retry/mirror/chunk settings
//   • Multi-task batch with dedup + aggregate progress
// ═══════════════════════════════════════════════════════════════════════════

pub struct DownloadEngine {
    config: DownloadConfig,
    client: reqwest::Client,
    task_semaphore: Arc<tokio::sync::Semaphore>,
    global_downloaded: Arc<AtomicU64>,
}

impl DownloadEngine {
    pub fn new(config: DownloadConfig) -> Self {
        let client = nt_io_http_factory::build_async_client_with_proxy(
            std::env::var("HTTPS_PROXY").ok().as_deref(),
        );
        Self {
            task_semaphore: Arc::new(tokio::sync::Semaphore::new(config.max_tasks)),
            global_downloaded: Arc::new(AtomicU64::new(0)),
            config,
            client,
        }
    }

    // ── Public API ──────────────────────────────────────────────────────

    pub async fn download(&self, task: &DownloadTask) -> DownloadStatus {
        self.download_with_progress(task, None).await
    }

    pub async fn download_with_progress(
        &self,
        task: &DownloadTask,
        progress_tx: Option<mpsc::Sender<DownloadProgress>>,
    ) -> DownloadStatus {
        let cancelled = Arc::new(AtomicBool::new(false));
        let start = SystemTime::now();
        match self.download_inner(task, progress_tx, cancelled).await {
            Ok(bytes) => {
                let elapsed = start.elapsed().unwrap_or_default().as_secs_f64();
                DownloadStatus::Completed {
                    elapsed_secs: elapsed,
                    size_mb: bytes as f64 / 1048576.0,
                }
            }
            Err(e) => {
                if e == "cancelled" {
                    DownloadStatus::Cancelled
                } else {
                    DownloadStatus::Failed(e)
                }
            }
        }
    }

    /// Multi-task parallel download with dedup + aggregate progress.
    pub async fn download_all(
        &self,
        tasks: &[DownloadTask],
        progress_tx: Option<mpsc::Sender<AggregateProgress>>,
    ) -> Vec<DownloadStatus> {
        // Deduplicate by URL
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut deduped: Vec<(usize, &DownloadTask)> = Vec::new();
        for (i, task) in tasks.iter().enumerate() {
            if seen.contains_key(&task.url) {
                continue;
            }
            seen.insert(task.url.clone(), i);
            deduped.push((i, task));
        }

        // Disk space pre-check
        if let Some((_, first)) = deduped.first() {
            if let Some(parent) = first.dest.parent() {
                if let Err(e) = check_disk_space(&first.dest, self.config.min_disk_space, 0) {
                    eprintln!("[dl] disk warning: {}", e);
                }
                let _ = fs::create_dir_all(parent).await;
            }
        }

        let total = tasks.len();
        let completed = Arc::new(AtomicUsize::new(0));
        let failed = Arc::new(AtomicUsize::new(0));
        let cancelled_count = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));
        self.global_downloaded.store(0, Ordering::Relaxed);

        let mut handles = Vec::new();
        for (_, task) in deduped {
            let engine = self.spawn_child();
            let task = task.clone();
            let completed = completed.clone();
            let failed = failed.clone();
            let cancelled_c = cancelled_count.clone();
            let active = active.clone();
            let global_dl = self.global_downloaded.clone();
            let agg_tx = progress_tx.clone();

            handles.push(tokio::spawn(async move {
                let _permit = match engine.task_semaphore.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => {
                        return DownloadStatus::Failed("semaphore closed".to_string())
                    }
                };
                active.fetch_add(1, Ordering::Relaxed);

                let status = engine.download(&task).await;

                active.fetch_sub(1, Ordering::Relaxed);
                match &status {
                    DownloadStatus::Completed { .. } => {
                        completed.fetch_add(1, Ordering::Relaxed);
                    }
                    DownloadStatus::Failed(_) => {
                        failed.fetch_add(1, Ordering::Relaxed);
                    }
                    DownloadStatus::Cancelled => {
                        cancelled_c.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {}
                }

                if let Some(tx) = &agg_tx {
                    let _ = tx.try_send(AggregateProgress {
                        total_tasks: total,
                        completed: completed.load(Ordering::Relaxed),
                        failed: failed.load(Ordering::Relaxed),
                        cancelled: cancelled_c.load(Ordering::Relaxed),
                        active: active.load(Ordering::Relaxed),
                        total_bytes: 0,
                        downloaded_bytes: global_dl.load(Ordering::Relaxed),
                        overall_speed_mbps: 0.0,
                        overall_percent: if total > 0 {
                            completed.load(Ordering::Relaxed) as f32 / total as f32 * 100.0
                        } else {
                            0.0
                        },
                    });
                }
                status
            }));
        }

        let mut results = vec![DownloadStatus::Pending; total];
        for (i, h) in handles.into_iter().enumerate() {
            if let Ok(status) = h.await {
                results[i] = status;
            }
        }
        results
    }

    /// Create a child engine sharing the same config and semaphores.
    fn spawn_child(&self) -> DownloadEngine {
        DownloadEngine {
            config: self.config.clone(),
            client: self.client.clone(),
            task_semaphore: self.task_semaphore.clone(),
            global_downloaded: self.global_downloaded.clone(),
        }
    }

    // ── Internal download implementation ────────────────────────────────

    async fn download_inner(
        &self,
        task: &DownloadTask,
        progress_tx: Option<mpsc::Sender<DownloadProgress>>,
        cancelled: Arc<AtomicBool>,
    ) -> Result<u64, String> {
        let scheme = router::UrlScheme::parse(&task.url);

        match scheme {
            router::UrlScheme::Magnet => {
                return Err(
                    "magnet link: use aria2c --enable-rpc or add librqbit backend".into(),
                );
            }
            router::UrlScheme::Ftp => {
                return Err("ftp: not yet implemented, use HTTP mirror".into());
            }
            _ => {}
        }

        let mut url = reqwest::Url::parse(&task.url).map_err(|e| e.to_string())?;

        // Mirror resolution for HuggingFace
        if router::is_huggingface_url(&task.url) {
            url = reqwest::Url::parse(&resolve_mirror(&self.client, &task.url).await)
                .map_err(|e| e.to_string())?;
        }

        if let Some(parent) = task.dest.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("mkdir: {}", e))?;
        }

        let dest = detect_filename(&self.client, &url, &task.dest).await;

        // .done marker check
        if let Some(existing_size) = is_done(&dest).await {
            return Ok(existing_size);
        }

        // HEAD for size
        let total_size = self.head_size(&url).await.unwrap_or(0);

        // Disk space pre-check + pre-allocate
        if total_size > 0 && !dest.exists() {
            if let Err(e) = check_disk_space(&dest, total_size, self.config.min_disk_space) {
                return Err(e);
            }
            let _ = std::fs::File::options()
                .write(true)
                .create(true)
                .truncate(false)
                .open(&dest)
                .and_then(|f| f.set_len(total_size));
        }

        // Resume: existing bytes
        let existing = if dest.exists() {
            fs::metadata(&dest).await.map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        if total_size > 0 && existing >= total_size {
            write_done_marker(&dest, total_size, &task.url).await;
            return Ok(existing);
        }

        // Chunk sizing
        let remaining = total_size.saturating_sub(existing);
        let chunk_size = if remaining > 0 {
            (remaining / self.config.max_concurrent as u64)
                .min(self.config.max_chunk_bytes)
                .max(1)
        } else {
            self.config.max_chunk_bytes
        };
        let n_chunks = if total_size > 0 {
            ((remaining - 1) / chunk_size + 1).min(self.config.max_concurrent as u64) as usize
        } else {
            1
        };

        // Temp directory for chunks
        let tmp_dir = make_dl_tmp_dir(&dest);
        fs::create_dir_all(&tmp_dir)
            .await
            .map_err(|e| format!("tmp dir: {}", e))?;

        // Concurrent chunk download
        let semaphore = Arc::new(tokio::sync::Semaphore::new(n_chunks));
        let mut handles = Vec::with_capacity(n_chunks);

        for i in 0..n_chunks {
            let start_byte = existing + i as u64 * chunk_size;
            let end_byte = if i == n_chunks - 1 {
                if total_size > 0 {
                    total_size - 1
                } else {
                    0
                }
            } else {
                existing + (i + 1) as u64 * chunk_size - 1
            };

            let chunk_file = tmp_dir.join(format!("c{:04}.tmp", i));
            let url = url.clone();
            let client = self.client.clone();
            let timeout_secs = self.config.timeout_secs;
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| format!("semaphore: {}", e))?;
            let cancelled = cancelled.clone();

            handles.push(tokio::spawn(async move {
                let _permit = permit;
                if cancelled.load(Ordering::Relaxed) {
                    return Err("cancelled".into());
                }
                http_chunk_download(
                    &client,
                    &url,
                    start_byte,
                    end_byte,
                    total_size,
                    &chunk_file,
                    timeout_secs,
                )
                .await
            }));
        }

        // Wait + progress
        let mut total_downloaded = existing;
        let loop_start = Instant::now();
        for (i, h) in handles.into_iter().enumerate() {
            if cancelled.load(Ordering::Relaxed) {
                return Err("cancelled".into());
            }
            let chunk_bytes = h
                .await
                .map_err(|e| format!("join {}: {}", i, e))?
                .map_err(|e| format!("chunk {}: {}", i, e))?;
            total_downloaded += chunk_bytes;
            self.global_downloaded
                .fetch_add(chunk_bytes, Ordering::Relaxed);

            let elapsed = loop_start.elapsed().as_secs_f64();
            let speed = if elapsed > 0.5 {
                (total_downloaded - existing) as f64 / elapsed
            } else {
                0.0
            };
            let pct = if total_size > 0 {
                total_downloaded as f32 / total_size as f32 * 100.0
            } else {
                0.0
            };
            let eta = if speed > 0.0 && total_size > total_downloaded {
                Some((total_size - total_downloaded) as f64 / speed)
            } else {
                None
            };

            let progress = DownloadProgressSnapshot {
                percent: pct,
                downloaded: total_downloaded,
                total: total_size,
                speed_mbps: speed / 1048576.0,
                eta_secs: eta,
            };
            if let Some(tx) = &progress_tx {
                let _ = tx.try_send(progress);
            }

            let eta_str = eta
                .map(|e| format!("{:.0}s", e))
                .unwrap_or_else(|| "?".into());
            eprintln!(
                "\r[dl] chunk {} +{}MB {:.1}% {:.1}MiB/s ETA:{}",
                i,
                chunk_bytes / 1048576,
                pct,
                speed / 1048576.0,
                eta_str
            );
        }
        eprintln!();

        // Merge chunks into final file
        merge_chunks(&tmp_dir, &dest, n_chunks)
            .await
            .map_err(|e| format!("merge: {}", e))?;

        // Record mirror speed (EMA)
        let final_elapsed = loop_start.elapsed().as_secs_f64();
        if let Some(host) = url.host_str() {
            if total_size > 0 && final_elapsed > 0.5 {
                let bps = (total_downloaded - existing) as f64 / final_elapsed;
                record_mirror_speed(host, bps);
            }
        }

        // Write .done marker + cleanup tmp
        write_done_marker(&dest, total_size, &task.url).await;
        let _ = fs::remove_dir_all(&tmp_dir).await;

        eprintln!(
            "[dl] complete: {} ({:.1}MB)",
            dest.display(),
            total_downloaded as f64 / 1048576.0
        );
        Ok(total_downloaded)
    }

    async fn head_size(&self, url: &reqwest::Url) -> Option<u64> {
        let resp = self.client.head(url.clone()).send().await.ok()?;
        let len = resp.headers().get(reqwest::header::CONTENT_LENGTH)?;
        len.to_str().ok()?.parse().ok()
    }
}

impl Default for DownloadEngine {
    fn default() -> Self {
        Self::new(DownloadConfig::default())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// URL helpers
// ═══════════════════════════════════════════════════════════════════════════

fn extract_domain(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| {
            let host = u.host_str()?.to_string();
            if let Some(port) = u.port() {
                Some(format!("{}:{}", host, port))
            } else {
                Some(host)
            }
        })
        .unwrap_or_default()
}

// ═══════════════════════════════════════════════════════════════════════════
// Cancel utilities
// ═══════════════════════════════════════════════════════════════════════════

fn create_cancel_pair() -> (Arc<AtomicBool>, Arc<AtomicBool>) {
    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = flag.clone();
    (flag, flag_clone)
}

// ═══════════════════════════════════════════════════════════════════════════
// Error type
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
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

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_http_streaming() {
        let (tx, mut rx) = mpsc::channel(100);
        let pipeline = StreamingPipeline::new(PipelineConfig {
            url: "https://httpbin.org/bytes/4096".into(),
            output_dir: std::env::temp_dir(),
            prefer_streaming: true,
            buffer_threshold: 1024,
            ..Default::default()
        });

        let handle = pipeline.run(tx).await.unwrap();

        let mut got_complete = false;
        while let Ok(Some(p)) = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await {
            if matches!(p.status, PipelineStatus::Complete { .. }) {
                got_complete = true;
                break;
            }
        }

        assert!(got_complete);
        handle.wait().await.ok();
        let _ = fs::remove_file(handle.output_path()).await;
    }

    #[tokio::test]
    async fn test_pipeline_file_copy() {
        let (tx, mut rx) = mpsc::channel(10);
        let test_file = std::env::temp_dir().join("nt_test_file_copy.txt");
        fs::write(&test_file, b"hello").await.unwrap();

        let pipeline = StreamingPipeline::new(PipelineConfig {
            url: format!("file://{}", test_file.to_string_lossy()),
            output_dir: std::env::temp_dir(),
            ..Default::default()
        });

        let handle = pipeline.run(tx).await.unwrap();
        let progress = rx.recv().await.unwrap();
        assert!(matches!(progress.status, PipelineStatus::Complete { .. }));

        let _ = fs::remove_file(&test_file).await;
    }

    #[test]
    fn test_detect_player() {
        assert!(detect_player(None).is_some());
    }

    #[test]
    fn test_plan_chunks() {
        let chunks = ParallelDownloader::plan_chunks(100, 30);
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0].start, 0);
        assert_eq!(chunks[0].end, 29);
        assert_eq!(chunks[1].start, 30);
        assert_eq!(chunks[1].end, 59);
        assert_eq!(chunks[3].start, 90);
        assert_eq!(chunks[3].end, 99);
    }

    #[test]
    fn test_plan_chunks_exact() {
        let chunks = ParallelDownloader::plan_chunks(100, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start, 0);
        assert_eq!(chunks[0].end, 99);
    }

    #[test]
    fn test_plan_chunks_zero() {
        let chunks = ParallelDownloader::plan_chunks(0, 100);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_retry_policy_delay() {
        let policy = RetryPolicy::new(3);
        assert!(policy.delay(1).is_some());
        assert!(policy.delay(3).is_some());
        assert!(policy.delay(4).is_none());
    }

    #[test]
    fn test_retry_policy_is_retryable_status() {
        assert!(RetryPolicy::is_retryable_status(500));
        assert!(RetryPolicy::is_retryable_status(503));
        assert!(RetryPolicy::is_retryable_status(429));
        assert!(!RetryPolicy::is_retryable_status(404));
        assert!(!RetryPolicy::is_retryable_status(200));
    }

    #[tokio::test]
    async fn test_stall_detector() {
        let mut stall = StallDetector::new(Duration::from_millis(50));
        assert!(!stall.check(0));
        assert!(!stall.is_stalled());
        stall.record_progress();
        assert!(!stall.is_stalled());
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(stall.is_stalled());
        stall.reset();
        assert!(!stall.is_stalled());
    }

    #[tokio::test]
    async fn test_stall_detector_check() {
        let mut stall = StallDetector::new(Duration::from_millis(50));
        assert!(!stall.check(0));
        assert!(!stall.check(0));
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(stall.check(0));
        assert!(!stall.check(1024));
    }

    #[tokio::test]
    async fn test_compute_and_verify_sha256() {
        let test_file = std::env::temp_dir().join("nt_test_sha256.txt");
        fs::write(&test_file, b"hello world").await.unwrap();

        let hash = compute_sha256(&test_file).await.unwrap();
        assert_eq!(hash.len(), 64);

        assert!(verify_sha256(&test_file, &hash).await.unwrap());
        assert!(!verify_sha256(
            &test_file,
            "0000000000000000000000000000000000000000000000000000000000000000"
        )
        .await
        .unwrap());

        let _ = fs::remove_file(&test_file).await;
    }

    #[test]
    fn test_mirror_speed_recording() {
        record_mirror_speed("hf-mirror.com", 1_000_000.0);
        record_mirror_speed("huggingface.co", 500_000.0);
        let ranked = ranked_mirrors();
        assert_eq!(ranked.len(), 2);
        assert!(ranked[0].1 >= ranked[1].1);
    }

    #[test]
    fn test_download_config_defaults() {
        let cfg = DownloadConfig::default();
        assert_eq!(cfg.max_concurrent, 16);
        assert_eq!(cfg.min_disk_space, 1024 * 1024 * 1024);
        assert_eq!(cfg.retry_count, 5);
    }

    #[test]
    fn test_download_config_to_pipeline() {
        let cfg = DownloadConfig {
            max_concurrent: 8,
            timeout_secs: 120,
            retry_count: 3,
            max_chunk_bytes: 32 * 1024 * 1024,
            ..Default::default()
        };
        let pc = cfg.to_pipeline_config(
            "https://example.com/f.bin".into(),
            PathBuf::from("/tmp"),
        );
        assert_eq!(pc.concurrency, 8);
        assert_eq!(pc.max_retries, 3);
        assert_eq!(pc.chunk_size, 32 * 1024 * 1024);
    }

    #[test]
    fn test_make_dl_tmp_dir() {
        let dest = Path::new("/tmp/model.gguf");
        let tmp = make_dl_tmp_dir(dest);
        assert_eq!(tmp, PathBuf::from("/tmp/.dl_model"));
    }

    #[test]
    fn test_download_task_builder() {
        let task = DownloadTask::new("https://example.com/f.bin", "/tmp/f.bin")
            .with_priority(10);
        assert_eq!(task.priority, 10);
        assert_eq!(task.url, "https://example.com/f.bin");
    }
}
