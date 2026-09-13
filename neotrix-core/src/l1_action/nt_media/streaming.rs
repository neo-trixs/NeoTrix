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
//! │  Player Backends:                                                │
//! │  • FileGrow — player reads growing file (mpv appending://)       │
//! │  • PipePlay — player reads from stdin/FIFO                       │
//! └──────────────────────────────────────────────────────────────────┘
//! ```

use super::auth::AuthConfig;
use super::detect::{self, MediaKind};
use super::router::{self, TransportType};
use crate::l1_action::nt_io::nt_io_http_factory;
use futures::StreamExt;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use std::time::Duration;
use tokio::fs::{self, File};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom};
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot, Mutex};
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
// ChunkState / ChunkStatus — per-chunk state for parallel download
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ChunkState {
    pub index: usize,
    pub start: u64,
    pub end: u64,
    pub downloaded: u64,
    pub status: ChunkStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChunkStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
}

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

    fn record_progress(&mut self) {
        self.last_check = Instant::now();
    }

    fn is_stalled(&self) -> bool {
        self.last_check.elapsed() > self.timeout
    }

    fn reset(&mut self) {
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

    fn delay_for(&self, attempt: u32) -> Option<Duration> {
        self.delay(attempt)
    }

    fn is_retryable_status(status: u16) -> bool {
        matches!(status, 429 | 500..=599)
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
    chunks: Arc<Mutex<VecDeque<ChunkState>>>,
    bytes_written: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    stall_timeout: Duration,
    retry_policy: RetryPolicy,
    auth: Option<AuthConfig>,
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
    ) -> Self {
        let chunks = Self::plan_chunks(total_size, chunk_size);
        Self {
            client,
            url,
            output,
            total_size,
            chunk_size,
            concurrency,
            chunks: Arc::new(Mutex::new(VecDeque::from(chunks))),
            bytes_written,
            cancel,
            stall_timeout,
            retry_policy: RetryPolicy::new(max_retries),
            auth,
        }
    }

    fn plan_chunks(total_size: u64, chunk_size: usize) -> Vec<ChunkState> {
        if total_size == 0 {
            return vec![];
        }
        let mut chunks = Vec::new();
        let mut offset = 0u64;
        let mut index = 0;
        while offset < total_size {
            let end = (offset + chunk_size as u64 - 1).min(total_size - 1);
            chunks.push(ChunkState {
                index,
                start: offset,
                end,
                downloaded: 0,
                status: ChunkStatus::Pending,
            });
            offset = end + 1;
            index += 1;
        }
        chunks
    }

    fn steal_next(&self) -> Option<ChunkState> {
        let chunks = self.chunks.blocking_lock();
        for chunk in chunks.iter() {
            if matches!(chunk.status, ChunkStatus::Pending) {
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
                                let total = self.bytes_written.fetch_add(data.len() as u64, Ordering::Relaxed) + data.len() as u64;
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
                                c.status = ChunkStatus::Complete;
                                c.downloaded = chunk.downloaded;
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
                        let next = chunks.iter_mut().find(|c| matches!(c.status, ChunkStatus::Pending));
                        match next {
                            Some(c) => {
                                c.status = ChunkStatus::InProgress;
                                c.clone()
                            }
                            None => {
                                let all_done = chunks.iter().all(|c| {
                                    matches!(c.status, ChunkStatus::Complete)
                                        || matches!(c.status, ChunkStatus::Failed)
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

                    if let Err(e) = dl.download_chunk(chunk).await {
                        let mut chunks = dl.chunks.lock().await;
                        for c in chunks.iter_mut() {
                            if c.index == chunk.index && !matches!(c.status, ChunkStatus::Complete) {
                                c.status = ChunkStatus::Failed;
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
                    let _ = progress_tx
                        .send(PipelineProgress {
                            url: url.clone(),
                            status: PipelineStatus::Downloading {
                                downloaded: written,
                                total: Some(total_size),
                                speed_bps: 0.0,
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
// HTTP Streaming Download (enhanced: parallel + stall + retry + verify)
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
    let (total_size, supports_range) = probe_range_support(client, url, auth).await;

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
                    let written =
                        bytes_written.fetch_add(data.len() as u64, Ordering::Relaxed)
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
        // No bytes yet — not stalled (just initialized)
        assert!(!stall.check(0));
        // Same bytes, but within timeout
        assert!(!stall.check(0));
        tokio::time::sleep(Duration::from_millis(100)).await;
        // Same bytes, timeout exceeded — stalled
        assert!(stall.check(0));
        // New bytes — not stalled
        assert!(!stall.check(1024));
    }

    #[tokio::test]
    async fn test_compute_and_verify_sha256() {
        let test_file = std::env::temp_dir().join("nt_test_sha256.txt");
        fs::write(&test_file, b"hello world").await.unwrap();

        let hash = compute_sha256(&test_file).await.unwrap();
        assert_eq!(hash.len(), 64);

        assert!(verify_sha256(&test_file, &hash).await.unwrap());
        assert!(!verify_sha256(&test_file, "0000000000000000000000000000000000000000000000000000000000000000").await.unwrap());

        let _ = fs::remove_file(&test_file).await;
    }
}
