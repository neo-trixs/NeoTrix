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

use super::detect::{self, MediaKind};
use super::router::{self, MediaRoute, TransportType, UrlScheme};
use crate::l1_action::nt_io::nt_io_http_factory;
use bytes::Bytes;
use futures::StreamExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::{self, File};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot};
use tokio::time::Instant;

// ═══════════════════════════════════════════════════════════════════════════
// Shared progress types — single source of truth
// ═══════════════════════════════════════════════════════════════════════════

/// Pipeline status
#[derive(Debug, Clone)]
pub enum PipelineStatus {
    /// Resolving URL (HEAD request, mirror selection)
    Resolving,
    /// Downloading with progress
    Downloading {
        downloaded: u64,
        total: Option<u64>,
        speed_bps: f64,
    },
    /// Playing media
    Playing {
        downloaded: u64,
        total: Option<u64>,
        speed_bps: f64,
    },
    /// Download complete
    Complete { total_bytes: u64, elapsed: Duration },
    /// Error
    Failed(String),
    /// Cancelled by user
    Cancelled,
}

/// Unified progress event
#[derive(Debug, Clone)]
pub struct PipelineProgress {
    pub url: String,
    pub status: PipelineStatus,
    pub media_kind: MediaKind,
    pub output: PathBuf,
    pub elapsed: Duration,
}

// ═══════════════════════════════════════════════════════════════════════════
// StreamingPipeline — the main entry point
// ═══════════════════════════════════════════════════════════════════════════

/// Configuration for a streaming pipeline session.
pub struct PipelineConfig {
    /// URL to stream
    pub url: String,
    /// Output directory
    pub output_dir: PathBuf,
    /// Prefer streaming mode (sequential) over parallel download
    pub prefer_streaming: bool,
    /// Player binary override (auto-detect if None)
    pub player_bin: Option<String>,
    /// Extra player arguments
    pub player_args: Vec<String>,
    /// Minimum bytes before starting playback (buffer threshold)
    pub buffer_threshold: u64,
    /// Proxy URL
    pub proxy: Option<String>,
    /// HTTP timeout
    pub timeout: Duration,
    /// Chunk size for sequential writes
    pub chunk_size: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            output_dir: PathBuf::from("/tmp/neotrix-stream"),
            prefer_streaming: false,
            player_bin: None,
            player_args: Vec::new(),
            buffer_threshold: 512 * 1024, // 512KB
            proxy: None,
            timeout: Duration::from_secs(30),
            chunk_size: 256 * 1024,
        }
    }
}

/// The unified streaming pipeline.
///
/// Usage:
/// ```ignore
/// let (tx, mut rx) = mpsc::channel(100);
/// let pipeline = StreamingPipeline::new(PipelineConfig {
///     url: "https://example.com/video.mp4".into(),
///     output_dir: "/tmp".into(),
///     prefer_streaming: true,
///     buffer_threshold: 1024 * 1024,
///     ..Default::default()
/// });
///
/// let handle = pipeline.run(tx).await?;
///
/// // Monitor progress
/// while let Some(progress) = rx.recv().await {
///     println!("{:?} - {:?}", progress.status, progress.media_kind);
/// }
///
/// handle.wait().await?;
/// ```
pub struct StreamingPipeline {
    config: PipelineConfig,
}

impl StreamingPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self { config }
    }

    /// Run the pipeline: detect → route → download → player.
    /// Returns a handle for cancellation and a progress receiver.
    pub async fn run(
        self,
        progress_tx: mpsc::Sender<PipelineProgress>,
    ) -> Result<PipelineHandle, PipelineError> {
        let config = self.config;

        // 1. Route URL
        let route = router::route_url(&config.url, &config.output_dir, config.prefer_streaming);

        // 2. Get HTTP client from shared factory
        let client = nt_io_http_factory::build_async_client_with_proxy(config.proxy.as_deref());

        // 3. Detect media type
        let (media_kind, _content_type) = detect::detect_remote(&client, &config.url).await;

        // 4. Ensure output directory exists
        fs::create_dir_all(&config.output_dir).await
            .map_err(|e| PipelineError::Io(e.to_string()))?;

        // 5. Dispatch to transport
        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
        let (stop_flag, cancel_rx_flag) = create_cancel_pair();

        let output = route.output.clone();
        let url = config.url.clone();

        match route.transport {
            TransportType::HttpRange | TransportType::HttpStream => {
                // Sequential HTTP streaming
                let bytes_written = Arc::new(AtomicU64::new(0));
                let output_clone = output.clone();

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
                    ).await
                });

                // Spawn player
                let player_handle = spawn_player(
                    &output,
                    config.buffer_threshold,
                    config.player_bin.as_deref(),
                    &config.player_args,
                ).await;

                Ok(PipelineHandle {
                    download_task: download_handle,
                    player_handle,
                    cancel_tx: Some(cancel_tx),
                    output,
                })
            }

            TransportType::MagnetRpc => {
                // Magnet: aria2c RPC
                let download_handle = tokio::spawn(async move {
                    stream_magnet_download(
                        &url,
                        &config.output_dir,
                        cancel_rx_flag,
                        progress_tx.clone(),
                        media_kind,
                    ).await
                });

                let player_handle = spawn_player(
                    &output,
                    config.buffer_threshold,
                    config.player_bin.as_deref(),
                    &config.player_args,
                ).await;

                Ok(PipelineHandle {
                    download_task: download_handle,
                    player_handle,
                    cancel_tx: Some(cancel_tx),
                    output,
                })
            }

            TransportType::FileCopy => {
                // Local file: just spawn player directly
                let player_handle = spawn_player(
                    &output,
                    0, // no buffer needed
                    config.player_bin.as_deref(),
                    &config.player_args,
                ).await;

                // Signal complete immediately
                let _ = progress_tx.send(PipelineProgress {
                    url,
                    status: PipelineStatus::Complete {
                        total_bytes: fs::metadata(&output).await.map(|m| m.len()).unwrap_or(0),
                        elapsed: Duration::ZERO,
                    },
                    media_kind,
                    output: output.clone(),
                    elapsed: Duration::ZERO,
                }).await;

                Ok(PipelineHandle {
                    download_task: tokio::spawn(async { Ok::<(), PipelineError>(()) }),
                    player_handle,
                    cancel_tx: None,
                    output,
                })
            }

            TransportType::FifoPipe => {
                // FIFO pipe: zero-disk streaming
                let fifo_path = config.output_dir.join(format!("stream-{}.fifo",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                ));

                // Create FIFO
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    use std::os::unix::fs::FileTypeExt;

                    // mkfifo via nix or manual
                    unsafe {
                        libc::mkfifo(
                            fifo_path.as_os_str().as_encoded_bytes().as_ptr() as *const i8,
                            0o644,
                        );
                    }
                }

                let fifo_path_clone = fifo_path.clone();
                let url_clone = url.clone();

                // Spawn player reading from FIFO
                let player_handle = spawn_fifo_player(
                    &fifo_path,
                    config.player_bin.as_deref(),
                    &config.player_args,
                ).await;

                // Spawn download writing to FIFO
                let download_handle = tokio::spawn(async move {
                    stream_http_to_fifo(
                        &client,
                        &url_clone,
                        &fifo_path_clone,
                        config.chunk_size,
                        cancel_rx_flag,
                        progress_tx.clone(),
                        media_kind,
                    ).await
                });

                Ok(PipelineHandle {
                    download_task: download_handle,
                    player_handle,
                    cancel_tx: Some(cancel_tx),
                    output: fifo_path,
                })
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Transport implementations
// ═══════════════════════════════════════════════════════════════════════════

/// Sequential HTTP streaming download — writes chunks to growing file.
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
) -> Result<(), PipelineError> {
    let started = Instant::now();

    let req = client
        .get(url)
        .header("Accept-Encoding", "identity")
        .timeout(timeout);

    // Resume support
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

    let resp = req.send().await.map_err(|e| PipelineError::Network(e.to_string()))?;

    if !resp.status().is_success() && resp.status().as_u16() != 206 {
        return Err(PipelineError::Http(resp.status().as_u16()));
    }

    let total_size = resp.content_length().map(|cl| cl + start_byte);
    let mut stream = resp.bytes_stream();

    let file = if start_byte > 0 {
        fs::OpenOptions::new().append(true).open(output).await
    } else {
        File::create(output).await
    }.map_err(|e| PipelineError::Io(e.to_string()))?;

    let mut writer = BufWriter::with_capacity(chunk_size, file);
    bytes_written.store(start_byte, Ordering::Relaxed);

    let mut speed_samples: Vec<f64> = Vec::new();
    let mut last_sample = Instant::now();
    let mut recent_bytes: u64 = 0;

    // Send initial progress
    let _ = progress_tx.send(PipelineProgress {
        url: url.to_string(),
        status: PipelineStatus::Resolving,
        media_kind,
        output: output.to_path_buf(),
        elapsed: Duration::ZERO,
    }).await;

    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            let _ = progress_tx.send(PipelineProgress {
                url: url.to_string(),
                status: PipelineStatus::Cancelled,
                media_kind,
                output: output.to_path_buf(),
                elapsed: started.elapsed(),
            }).await;
            writer.flush().await.ok();
            return Ok(());
        }

        match chunk {
            Ok(data) => {
                writer.write_all(&data).await.map_err(|e| PipelineError::Io(e.to_string()))?;
                let written = bytes_written.fetch_add(data.len() as u64, Ordering::Relaxed) + data.len() as u64;
                recent_bytes += data.len() as u64;

                // Speed calculation every 500ms
                if last_sample.elapsed() > Duration::from_millis(500) {
                    let elapsed_s = last_sample.elapsed().as_secs_f64();
                    let speed = (recent_bytes as f64) / elapsed_s;
                    speed_samples.push(speed);
                    if speed_samples.len() > 10 { speed_samples.remove(0); }
                    let avg_speed = speed_samples.iter().sum::<f64>() / speed_samples.len() as f64;
                    recent_bytes = 0;
                    last_sample = Instant::now();

                    let status = if Some(written) >= total_size {
                        PipelineStatus::Complete { total_bytes: written, elapsed: started.elapsed() }
                    } else {
                        PipelineStatus::Downloading { downloaded: written, total: total_size, speed_bps: avg_speed }
                    };

                    let _ = progress_tx.send(PipelineProgress {
                        url: url.to_string(),
                        status,
                        media_kind,
                        output: output.to_path_buf(),
                        elapsed: started.elapsed(),
                    }).await;
                }
            }
            Err(e) => {
                let _ = progress_tx.send(PipelineProgress {
                    url: url.to_string(),
                    status: PipelineStatus::Failed(e.to_string()),
                    media_kind,
                    output: output.to_path_buf(),
                    elapsed: started.elapsed(),
                }).await;
                return Err(PipelineError::Network(e.to_string()));
            }
        }
    }

    writer.flush().await.map_err(|e| PipelineError::Io(e.to_string()))?;
    let final_bytes = bytes_written.load(Ordering::Relaxed);

    let _ = progress_tx.send(PipelineProgress {
        url: url.to_string(),
        status: PipelineStatus::Complete { total_bytes: final_bytes, elapsed: started.elapsed() },
        media_kind,
        output: output.to_path_buf(),
        elapsed: started.elapsed(),
    }).await;

    Ok(())
}

/// Magnet link download via aria2c JSON-RPC.
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

    let resp = client.post(rpc_url)
        .json(&rpc_body)
        .send()
        .await
        .map_err(|e| PipelineError::Network(format!("aria2c connect failed: {}", e)))?;

    let body: serde_json::Value = resp.json().await
        .map_err(|e| PipelineError::Network(format!("aria2c response parse: {}", e)))?;

    if let Some(error) = body.get("error") {
        return Err(PipelineError::Rpc(format!(
            "aria2c error {}: {}",
            error.get("code").and_then(|c| c.as_i64()).unwrap_or(0),
            error.get("message").and_then(|m| m.as_str()).unwrap_or("unknown")
        )));
    }

    let gid = body["result"].as_str()
        .ok_or_else(|| PipelineError::Rpc("no GID returned".into()))?
        .to_string();

    let started = Instant::now();

    // Poll status
    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = progress_tx.send(PipelineProgress {
                url: url.to_string(),
                status: PipelineStatus::Cancelled,
                media_kind,
                output: output_dir.to_path_buf(),
                elapsed: started.elapsed(),
            }).await;
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
        let total = body["result"]["totalLength"].as_str().unwrap_or("0").parse::<u64>().unwrap_or(0);
        let completed = body["result"]["completedLength"].as_str().unwrap_or("0").parse::<u64>().unwrap_or(0);
        let speed = body["result"]["downloadSpeed"].as_str().unwrap_or("0").parse::<u64>().unwrap_or(0);

        let pipeline_status = match status {
            "active" => PipelineStatus::Downloading {
                downloaded: completed,
                total: if total > 0 { Some(total) } else { None },
                speed_bps: speed as f64,
            },
            "complete" => PipelineStatus::Complete { total_bytes: completed, elapsed: started.elapsed() },
            "error" => PipelineStatus::Failed("aria2c error".into()),
            _ => PipelineStatus::Cancelled,
        };

        let _ = progress_tx.send(PipelineProgress {
            url: url.to_string(),
            status: pipeline_status.clone(),
            media_kind,
            output: output_dir.to_path_buf(),
            elapsed: started.elapsed(),
        }).await;

        match status {
            "complete" => return Ok(()),
            "error" | "removed" => return Err(PipelineError::Rpc(format!("aria2c status: {}", status))),
            _ => {}
        }
    }
}

/// HTTP download to FIFO pipe (zero disk I/O).
async fn stream_http_to_fifo(
    client: &reqwest::Client,
    url: &str,
    fifo_path: &Path,
    chunk_size: usize,
    cancel: Arc<AtomicBool>,
    progress_tx: mpsc::Sender<PipelineProgress>,
    media_kind: MediaKind,
) -> Result<(), PipelineError> {
    let started = Instant::now();

    let req = client
        .get(url)
        .header("Accept-Encoding", "identity");

    let mut resp = req.send().await.map_err(|e| PipelineError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(PipelineError::Http(resp.status().as_u16()));
    }

    // Open FIFO for writing (blocks until reader opens)
    let file = File::create(fifo_path).await.map_err(|e| PipelineError::Io(e.to_string()))?;
    let mut writer = BufWriter::with_capacity(chunk_size, file);

    let mut downloaded: u64 = 0;

    let _ = progress_tx.send(PipelineProgress {
        url: url.to_string(),
        status: PipelineStatus::Resolving,
        media_kind,
        output: fifo_path.to_path_buf(),
        elapsed: Duration::ZERO,
    }).await;

    while let Some(chunk) = resp.chunk().await
        .map_err(|e| PipelineError::Network(e.to_string()))?
    {
        if cancel.load(Ordering::Relaxed) {
            let _ = progress_tx.send(PipelineProgress {
                url: url.to_string(),
                status: PipelineStatus::Cancelled,
                media_kind,
                output: fifo_path.to_path_buf(),
                elapsed: started.elapsed(),
            }).await;
            return Ok(());
        }

        writer.write_all(&chunk).await.map_err(|e| PipelineError::Io(e.to_string()))?;
        downloaded += chunk.len() as u64;

        let _ = progress_tx.send(PipelineProgress {
            url: url.to_string(),
            status: PipelineStatus::Downloading {
                downloaded,
                total: None,
                speed_bps: 0.0,
            },
            media_kind,
            output: fifo_path.to_path_buf(),
            elapsed: started.elapsed(),
        }).await;
    }

    writer.flush().await.map_err(|e| PipelineError::Io(e.to_string()))?;

    let _ = progress_tx.send(PipelineProgress {
        url: url.to_string(),
        status: PipelineStatus::Complete { total_bytes: downloaded, elapsed: started.elapsed() },
        media_kind,
        output: fifo_path.to_path_buf(),
        elapsed: started.elapsed(),
    }).await;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Player backends
// ═══════════════════════════════════════════════════════════════════════════

/// Detect available player binary
fn detect_player(preferred: Option<&str>) -> Option<&'static str> {
    if let Some(p) = preferred {
        // Check if the preferred player is available
        if std::process::Command::new(p).arg("--version").output().is_ok() {
            return Some(Box::leak(p.to_string().into_boxed_str()));
        }
    }
    // Auto-detect: ffplay first (lighter), then mpv
    if std::process::Command::new("ffplay").arg("-version").output().is_ok() {
        return Some("ffplay");
    }
    if std::process::Command::new("mpv").arg("--version").output().is_ok() {
        return Some("mpv");
    }
    None
}

/// Spawn a player that reads a growing file.
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
        // Wait for buffer
        if wait_bytes > 0 {
            loop {
                let size = fs::metadata(&file_clone).await.map(|m| m.len()).unwrap_or(0);
                if size >= wait_bytes { break; }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }

        // Spawn player
        let mut cmd = Command::new(player);
        if player == "ffplay" {
            cmd.arg("-autoexit").arg("-loop").arg("0");
            cmd.args(&extra_args);
            cmd.arg(&file_clone);
        } else {
            // mpv: use appending:// for growing files
            let appending_url = format!("appending://{}", file_clone.to_string_lossy());
            cmd.arg("--loop=inf").arg("--keep-open=yes");
            cmd.arg("--demuxer-max-bytes=512MiB");
            cmd.arg("--demuxer-readahead-secs=20");
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

    Some(PlayerHandle { handle, stop_tx, file: file.to_path_buf() })
}

/// Spawn a player that reads from a FIFO pipe.
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
        // Wait for FIFO to be readable
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

    Some(PlayerHandle { handle, stop_tx, file: fifo_path.to_path_buf() })
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
    pub fn output_path(&self) -> &Path { &self.output }

    pub fn cancel(&self) {
        if let Some(tx) = self.cancel_tx.as_ref() {
            let _ = tx.send(());
        }
    }

    pub async fn wait(self) -> Result<(), PipelineError> {
        // Wait for download
        let _ = self.download_task.await;

        // Player will exit on its own when file stops growing
        if let Some(player) = self.player_handle {
            let _ = player.handle.await;
        }

        Ok(())
    }
}

pub struct PlayerHandle {
    handle: tokio::task::JoinHandle<Option<()>>,
    stop_tx: oneshot::Sender<()>,
    file: PathBuf,
}

impl PlayerHandle {
    pub fn stop(&self) {
        let _ = self.stop_tx.send(());
    }
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

        // Collect progress
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
        // Should find at least one player
        assert!(detect_player(None).is_some());
    }
}
