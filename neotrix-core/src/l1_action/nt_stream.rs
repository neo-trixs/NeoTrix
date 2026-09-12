//! Streaming download + playback (边下载边播放)
//!
//! Architecture:
//! ```text
//! StreamPlayer (ffplay/mpv)  ←  reads from growing file
//! StreamDownload              ←  sequential chunk write + progress
//! MagnetTransport            ←  aria2c RPC for magnet links
//! ```

use bytes::Bytes;
use futures::StreamExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::{self, File};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::unix::pipe;
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::Instant;

// ── Progress types ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum StreamStatus {
    Connecting,
    Buffering { buffered_bytes: u64, total: Option<u64> },
    Playing { played: u64, total: Option<u64> },
    Complete,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct StreamProgress {
    pub url: String,
    pub status: StreamStatus,
    pub download_speed_bps: f64,
    pub download_total: u64,
    pub download_bytes: u64,
    pub elapsed: Duration,
    pub buffered_percent: Option<f32>,
}

// ── StreamDownload ──────────────────────────────────────────────

/// Sequential chunk download with progress, writing to file for player to read.
///
/// Unlike `DownloadEngine` which downloads full file before playing,
/// this writes chunks sequentially and signals the player when bytes are available.
pub struct StreamDownload {
    /// URL to stream
    url: String,
    /// Output file path
    output: PathBuf,
    /// Chunk size for sequential writes (default 256KB)
    chunk_size: usize,
    /// HTTP timeout
    timeout: Duration,
    /// Proxy URL
    proxy: Option<String>,
    /// Progress sender
    progress_tx: mpsc::Sender<StreamProgress>,
    /// Cancel signal
    cancel_rx: Option<oneshot::Receiver<()>>,
    /// Bytes written so far
    bytes_written: Arc<AtomicU64>,
    /// Stop flag
    stopped: Arc<AtomicBool>,
}

impl StreamDownload {
    pub fn builder() -> StreamDownloadBuilder {
        StreamDownloadBuilder::default()
    }

    pub async fn start(self) -> Result<StreamHandle, StreamError> {
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .danger_accept_invalid_certs(true)
            .no_proxy()
            .build()
            .map_err(|e| StreamError::Client(e.to_string()))?;

        // Disable compression to avoid chunked encoding bugs
        let req = client
            .get(&self.url)
            .header("Accept-Encoding", "identity")
            .no_proxy();

        // If resuming, set Range header
        let start_byte = if self.output.exists() {
            fs::metadata(&self.output).await.map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let req = if start_byte > 0 {
            req.header("Range", format!("bytes={}-", start_byte))
        } else {
            req
        };

        let resp = req.send().await.map_err(|e| StreamError::Network(e.to_string()))?;

        if !resp.status().is_success() && resp.status().as_u16() != 206 {
            return Err(StreamError::Http(resp.status().as_u16()));
        }

        let total_size = resp.content_length().map(|cl| cl + start_byte);
        let mut stream = resp.bytes_stream();
        let mut file = BufWriter::with_capacity(256 * 1024, File::create(&self.output).await?);

        let bytes_written = self.bytes_written.clone();
        bytes_written.store(start_byte, Ordering::Relaxed);

        let started = Instant::now();
        let mut speed_samples: Vec<f64> = Vec::new();
        let mut last_sample = Instant::now();
        let mut recent_bytes: u64 = 0;

        // Spawn background task that writes chunks
        let output_path = self.output.clone();
        let progress_tx = self.progress_tx.clone();
        let stopped = self.stopped.clone();
        let cancel_rx = self.cancel_rx;

        let handle = tokio::spawn(async move {
            let _ = progress_tx.send(StreamProgress {
                url: self.url.clone(),
                status: StreamStatus::Connecting,
                download_speed_bps: 0.0,
                download_total: start_byte,
                download_bytes: start_byte,
                elapsed: Duration::ZERO,
                buffered_percent: None,
            })
            .await;

            while let Some(chunk) = stream.next().await {
                // Check cancel
                if stopped.load(Ordering::Relaxed) || (cancel_rx.as_ref().map(|rx| rx.is_closed()).unwrap_or(false)) {
                    let _ = progress_tx.send(StreamProgress {
                        url: self.url.clone(),
                        status: StreamStatus::Cancelled,
                        download_speed_bps: 0.0,
                        download_total: bytes_written.load(Ordering::Relaxed),
                        download_bytes: bytes_written.load(Ordering::Relaxed),
                        elapsed: started.elapsed(),
                        buffered_percent: None,
                    })
                    .await;
                    return Ok(());
                }

                match chunk {
                    Ok(data) => {
                        file.write_all(&data).await?;
                        let written = bytes_written.fetch_add(data.len() as u64, Ordering::Relaxed) + data.len() as u64;
                        recent_bytes += data.len() as u64;

                        // Calculate speed every 500ms
                        if last_sample.elapsed() > Duration::from_millis(500) {
                            let elapsed_s = last_sample.elapsed().as_secs_f64();
                            let speed = (recent_bytes as f64) / elapsed_s;
                            speed_samples.push(speed);
                            if speed_samples.len() > 10 {
                                speed_samples.remove(0);
                            }
                            let avg_speed = speed_samples.iter().sum::<f64>() / speed_samples.len() as f64;
                            recent_bytes = 0;
                            last_sample = Instant::now();

                            let buffered_pct = total_size.map(|t| (written as f32 / t as f32) * 100.0);
                            let _ = progress_tx.send(StreamProgress {
                                url: self.url.clone(),
                                status: if written == total_size.unwrap_or(0) {
                                    StreamStatus::Complete
                                } else {
                                    StreamStatus::Buffering { buffered_bytes: written, total: total_size }
                                },
                                download_speed_bps: avg_speed,
                                download_total: total_size.unwrap_or(0),
                                download_bytes: written,
                                elapsed: started.elapsed(),
                                buffered_percent: buffered_pct,
                            })
                            .await;
                        }
                    }
                    Err(e) => {
                        let _ = progress_tx.send(StreamProgress {
                            url: self.url.clone(),
                            status: StreamStatus::Failed(e.to_string()),
                            download_speed_bps: 0.0,
                            download_total: bytes_written.load(Ordering::Relaxed),
                            download_bytes: bytes_written.load(Ordering::Relaxed),
                            elapsed: started.elapsed(),
                            buffered_percent: None,
                        })
                        .await;
                        return Err(StreamError::Chunk(e.to_string()));
                    }
                }
            }

            file.flush().await?;
            let _ = progress_tx.send(StreamProgress {
                url: self.url,
                status: StreamStatus::Complete,
                download_speed_bps: 0.0,
                download_total: bytes_written.load(Ordering::Relaxed),
                download_bytes: bytes_written.load(Ordering::Relaxed),
                elapsed: started.elapsed(),
                buffered_percent: Some(100.0),
            })
            .await;

            Ok::<(), StreamError>(())
        });

        Ok(StreamHandle {
            task: handle,
            output: output_path,
            bytes_written,
            stopped,
        })
    }
}

pub struct StreamDownloadBuilder {
    url: String,
    output: PathBuf,
    chunk_size: usize,
    timeout: Duration,
    proxy: Option<String>,
    progress_tx: Option<mpsc::Sender<StreamProgress>>,
    cancel_rx: Option<oneshot::Receiver<()>>,
}

impl Default for StreamDownloadBuilder {
    fn default() -> Self {
        Self {
            url: String::new(),
            output: PathBuf::new(),
            chunk_size: 256 * 1024,
            timeout: Duration::from_secs(30),
            proxy: None,
            progress_tx: None,
            cancel_rx: None,
        }
    }
}

impl StreamDownloadBuilder {
    pub fn url(mut self, url: impl Into<String>) -> Self { self.url = url.into(); self }
    pub fn output(mut self, path: impl Into<PathBuf>) -> Self { self.output = path.into(); self }
    pub fn chunk_size(mut self, size: usize) -> Self { self.chunk_size = size; self }
    pub fn timeout(mut self, timeout: Duration) -> Self { self.timeout = timeout; self }
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self { self.proxy = Some(proxy.into()); self }

    pub fn progress(mut self, tx: mpsc::Sender<StreamProgress>) -> Self { self.progress_tx = Some(tx); self }
    pub fn cancel(mut self, rx: oneshot::Receiver<()>) -> Self { self.cancel_rx = Some(rx); self }

    pub fn build(self) -> Result<StreamDownload, StreamError> {
        if self.url.is_empty() { return Err(StreamError::Config("url is required".into())); }
        if self.output.as_os_str().is_empty() { return Err(StreamError::Config("output path is required".into())); }

        Ok(StreamDownload {
            url: self.url,
            output: self.output,
            chunk_size: self.chunk_size,
            timeout: self.timeout,
            proxy: self.proxy,
            progress_tx: self.progress_tx.ok_or_else(|| StreamError::Config("progress channel required".into()))?,
            cancel_rx: self.cancel_rx,
            bytes_written: Arc::new(AtomicU64::new(0)),
            stopped: Arc::new(AtomicBool::new(false)),
        })
    }
}

// ── StreamHandle ────────────────────────────────────────────────

pub struct StreamHandle {
    task: tokio::task::JoinHandle<Result<(), StreamError>>,
    output: PathBuf,
    bytes_written: Arc<AtomicU64>,
    stopped: Arc<AtomicBool>,
}

impl StreamHandle {
    pub fn output_path(&self) -> &Path { &self.output }
    pub fn bytes_written(&self) -> u64 { self.bytes_written.load(Ordering::Relaxed) }

    pub fn available_bytes(&self) -> u64 {
        std::fs::metadata(&self.output).map(|m| m.len()).unwrap_or(0)
    }

    pub fn cancel(&self) {
        self.stopped.store(true, Ordering::Relaxed);
    }

    pub async fn wait(self) -> Result<(), StreamError> {
        self.task.await.map_err(|e| StreamError::Task(e.to_string()))?
    }
}

// ── StreamPlayer ────────────────────────────────────────────────

/// Player that reads a growing file via ffplay/mpv.
///
/// Usage:
/// ```text
/// let stream = StreamDownload::builder()
///     .url("https://example.com/file.gguf")
///     .output("/tmp/model.gguf")
///     .progress(tx)
///     .build()?;
///
/// let handle = stream.start().await?;
/// let player = StreamPlayer::new("/tmp/model.gguf")
///     .wait_for_bytes(1024 * 1024) // wait 1MB before playing
///     .spawn();
/// ```
pub struct StreamPlayer {
    file: PathBuf,
    wait_bytes: u64,
    poll_interval: Duration,
    player_bin: Option<String>,
    player_args: Vec<String>,
}

impl StreamPlayer {
    pub fn new(file: impl Into<PathBuf>) -> Self {
        Self {
            file: file.into(),
            wait_bytes: 512 * 1024, // default: 512KB
            poll_interval: Duration::from_millis(200),
            player_bin: None,
            player_args: Vec::new(),
        }
    }

    /// Bytes to wait for before starting playback
    pub fn wait_for_bytes(mut self, bytes: u64) -> Self { self.wait_bytes = bytes; self }

    /// Poll interval for checking file growth
    pub fn poll_interval(mut self, interval: Duration) -> Self { self.poll_interval = interval; self }

    /// Custom player binary
    pub fn player(mut self, bin: impl Into<String>) -> Self { self.player_bin = Some(bin.into()); self }

    /// Additional player args
    pub fn args(mut self, args: Vec<String>) -> Self { self.player_args = args; self }

    fn detect_player(&self) -> Option<&str> {
        if self.player_bin.is_some() { return self.player_bin.as_deref(); }
        // Try ffplay first, then mpv
        if std::process::Command::new("ffplay").arg("-version").output().is_ok() {
            return Some("ffplay");
        }
        if std::process::Command::new("mpv").arg("--version").output().is_ok() {
            return Some("mpv");
        }
        None
    }

    /// Spawn the player and return a handle.
    /// Player will read from the file as it grows, and exit when the file stops growing or closes.
    pub fn spawn(self) -> Result<PlayerHandle, StreamError> {
        let player = self.detect_player()
            .ok_or_else(|| StreamError::Config("no player found (ffplay/mpv)".into()))?;

        let file_clone = self.file.clone();

        let (stop_tx, stop_rx) = oneshot::channel::<()>();

        let wait_bytes = self.wait_bytes;
        let poll_interval = self.poll_interval;
        let extra_args = self.player_args.clone();

        let handle = tokio::spawn(async move {
            // Wait for file to have enough bytes
            loop {
                let size = fs::metadata(&file_clone).await.map(|m| m.len()).unwrap_or(0);
                if size >= wait_bytes {
                    break;
                }
                tokio::time::sleep(poll_interval).await;
            }

            // Spawn player
            let mut cmd = Command::new(player);
            if player == "ffplay" {
                // ffplay: loop, no video, just audio, use file as input
                cmd.arg("-autoexit")
                    .arg("-loop").arg("0")
                    .args(&extra_args)
                    .arg(&file_clone);
            } else {
                // mpv: loop, keep-open, demuxer reads growing file
                cmd.arg("--loop=inf")
                    .arg("--keep-open=yes")
                    .arg("--demuxer-max-bytes=1M")
                    .args(&extra_args)
                    .arg(&file_clone);
            }

            cmd.stdout(Stdio::null())
                .stderr(Stdio::null());

            let mut child = cmd.spawn()
                .map_err(|e| StreamError::Player(format!("failed to spawn {}: {}", player, e)))?;

            // Wait for stop signal or player exit
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

            Ok::<(), StreamError>(())
        });

        Ok(PlayerHandle { handle, stop_tx, file: file_clone })
    }
}

pub struct PlayerHandle {
    handle: tokio::task::JoinHandle<Result<(), StreamError>>,
    stop_tx: oneshot::Sender<()>,
    file: PathBuf,
}

impl PlayerHandle {
    pub fn stop(self) -> Result<(), StreamError> {
        let _ = self.stop_tx.send(());
        self.handle.try_join().ok();
        Ok(())
    }

    pub fn file(&self) -> &Path { &self.file }
}

// ── MagnetTransport ─────────────────────────────────────────────

/// aria2c JSON-RPC client for magnet link downloads.
pub struct MagnetTransport {
    rpc_url: String,
    rpc_secret: Option<String>,
}

impl MagnetTransport {
    /// Connect to running aria2c RPC daemon.
    /// Default: http://127.0.0.1:6800/jsonrpc
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self { rpc_url: rpc_url.into(), rpc_secret: None }
    }

    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.rpc_secret = Some(secret.into());
        self
    }

    /// Add a magnet link and return the GID.
    pub async fn add_magnet(
        &self,
        magnet: &str,
        output_dir: &Path,
        progress_tx: mpsc::Sender<StreamProgress>,
    ) -> Result<MagnetHandle, StreamError> {
        let client = reqwest::Client::new();

        let mut params = serde_json::json!({
            "uris": [magnet], // must be array, not string
            "dir": output_dir.to_string_lossy(),
            "bt-stream-piece-selector": "inorder", // sequential for streaming
            "follow-torrent": "mem", // download into memory first
            "max-overall-upload-limit": "0",
            "seed-time": "0",
            "dir": output_dir.to_string_lossy(),
        });

        if let Some(ref secret) = self.rpc_secret {
            params["secret"] = serde_json::json!(format!("token:{}", secret));
        }

        let rpc_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "nt-magnet",
            "method": "aria2.addUri",
            "params": [params],
        });

        let resp = client
            .post(&self.rpc_url)
            .json(&rpc_body)
            .send()
            .await
            .map_err(|e| StreamError::Rpc(format!("aria2c connect failed: {}", e)))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| StreamError::Rpc(format!("aria2c response parse failed: {}", e)))?;

        if let Some(error) = body.get("error") {
            return Err(StreamError::Rpc(format!(
                "aria2c error {}: {}",
                error.get("code").and_then(|c| c.as_i64()).unwrap_or(0),
                error.get("message").and_then(|m| m.as_str()).unwrap_or("unknown")
            )));
        }

        let gid = body["result"]
            .as_str()
            .ok_or_else(|| StreamError::Rpc("aria2c returned no GID".into()))?
            .to_string();

        // Spawn polling task
        let rpc_url = self.rpc_url.clone();
        let rpc_secret = self.rpc_secret.clone();
        let (stop_tx, stop_rx) = oneshot::channel::<()>();

        let poll_handle = tokio::spawn(async move {
            let client = reqwest::Client::new();
            let mut interval = tokio::time::interval(Duration::from_millis(500));

            loop {
                tokio::select! {
                    _ = interval.tick() => {}
                    _ = &mut stop_rx.into() => return Ok::<(), StreamError>(()),
                }

                let rpc_body = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "nt-status",
                    "method": "aria2.tellStatus",
                    "params": [&gid, ["status", "totalLength", "completedLength", "downloadSpeed", "uploadSpeed", "files"]],
                });

                let resp = match client.post(&rpc_url).json(&rpc_body).send().await {
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

                let stream_status = match status {
                    "active" => StreamStatus::Buffering { buffered_bytes: completed, total: if total > 0 { Some(total) } else { None } },
                    "complete" => StreamStatus::Complete,
                    "error" => StreamStatus::Failed("aria2c error".into()),
                    "paused" => StreamStatus::Cancelled,
                    "removed" => StreamStatus::Cancelled,
                    _ => StreamStatus::Buffering { buffered_bytes: completed, total: if total > 0 { Some(total) } else { None } },
                };

                let _ = progress_tx.send(StreamProgress {
                    url: format!("magnet:...&dn={}", gid),
                    status: stream_status.clone(),
                    download_speed_bps: speed as f64,
                    download_total: total,
                    download_bytes: completed,
                    elapsed: Duration::ZERO,
                    buffered_percent: if total > 0 { Some((completed as f32 / total as f32) * 100.0) } else { None },
                })
                .await;

                if status == "complete" || status == "error" || status == "removed" {
                    break;
                }
            }

            Ok(())
        });

        Ok(MagnetHandle {
            gid,
            rpc_url: self.rpc_url.clone(),
            rpc_secret: self.rpc_secret.clone(),
            stop_tx: Some(stop_tx),
            poll_handle,
        })
    }

    /// Remove a torrent by GID.
    pub async fn remove(&self, gid: &str) -> Result<(), StreamError> {
        let client = reqwest::Client::new();
        let rpc_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "nt-remove",
            "method": "aria2.remove",
            "params": [gid],
        });
        let _ = client.post(&self.rpc_url).json(&rpc_body).send().await;
        Ok(())
    }
}

pub struct MagnetHandle {
    gid: String,
    rpc_url: String,
    rpc_secret: Option<String>,
    stop_tx: Option<oneshot::Sender<()>>,
    poll_handle: tokio::task::JoinHandle<Result<(), StreamError>>,
}

impl MagnetHandle {
    pub fn gid(&self) -> &str { &self.gid }

    pub fn cancel(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        let rpc_url = self.rpc_url.clone();
        let gid = self.gid.clone();
        let rpc_secret = self.rpc_secret.clone();

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let mut params = vec![serde_json::json!(gid)];
            if let Some(secret) = rpc_secret {
                params.insert(0, serde_json::json!(format!("token:{}", secret)));
            }
            let _ = client.post(&rpc_url)
                .json(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "nt-cancel",
                    "method": "aria2.remove",
                    "params": params,
                }))
                .send()
                .await;
        });
    }

    pub async fn wait(self) -> Result<(), StreamError> {
        self.poll_handle.await.map_err(|e| StreamError::Task(e.to_string()))?
    }
}

// ── Error type ──────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("network error: {0}")]
    Network(String),
    #[error("HTTP status {0}")]
    Http(u16),
    #[error("chunk error: {0}")]
    Chunk(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("player error: {0}")]
    Player(String),
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("client error: {0}")]
    Client(String),
    #[error("task error: {0}")]
    Task(String),
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_download_progress() {
        let (tx, mut rx) = mpsc::channel(100);
        let output = std::env::temp_dir().join("nt_stream_test.bin");

        // Small HTTP endpoint
        let stream = StreamDownload::builder()
            .url("https://httpbin.org/bytes/4096")
            .output(&output)
            .progress(tx)
            .build()
            .unwrap();

        let handle = stream.start().await.unwrap();

        // Should get progress updates
        let mut got_progress = false;
        while let Ok(Some(p)) = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await {
            got_progress = true;
            match &p.status {
                StreamStatus::Buffering { .. } => {}
                StreamStatus::Complete => break,
                _ => {}
            }
        }

        assert!(got_progress);
        handle.wait().await.ok();
        let _ = fs::remove_file(&output).await;
    }

    #[tokio::test]
    async fn test_magnet_transport_rpc() {
        let transport = MagnetTransport::new("http://127.0.0.1:6800/jsonrpc");

        // Test RPC connectivity
        let client = reqwest::Client::new();
        let resp = client.post("http://127.0.0.1:6800/jsonrpc")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": "nt-test",
                "method": "aria2.getVersion",
                "params": [],
            }))
            .send()
            .await;

        assert!(resp.is_ok());
        let body: serde_json::Value = resp.unwrap().json().await.unwrap();
        assert!(body.get("result").is_some());
    }

    #[test]
    fn test_player_detection() {
        let player = StreamPlayer::new("/tmp/test.bin");
        // Should find ffplay or mpv
        assert!(player.detect_player().is_some());
    }
}
