//! pipeline — 从 `streaming.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, oneshot};
use super::types::{PipelineError, PipelineHandle, PipelineProgress, PipelineStatus, PlayerHandle, StreamingPipelineConfig};
use super::alt::{stream_hls_download, stream_http_to_fifo, stream_magnet_download};
use super::http::stream_http_download;
use super::dl_fs::create_cancel_pair;
use super::super::router::{self, TransportType};
use super::super::detect;
use std::sync::atomic::AtomicU64;
use tokio::process::Command;
use std::sync::OnceLock;
use std::process::Stdio;
use std::path::Path;
use crate::l1_action::nt_io::nt_io_http_factory;
use tokio::fs;

// ═══════════════════════════════════════════════════════════════════════════
// EventBus integration — R-P79: download progress → system-wide visibility
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) static GLOBAL_EVENT_BUS: OnceLock<Arc<crate::l0_substrate::nt_core_event_bus::EventBus>> = OnceLock::new();

/// Register the global EventBus for download progress publishing.
/// Call once at startup; silently no-ops if already set.
pub fn set_download_event_bus(bus: Arc<crate::l0_substrate::nt_core_event_bus::EventBus>) {
    let _ = GLOBAL_EVENT_BUS.set(bus);
}

/// Publish a PipelineProgress event to the global EventBus for system-wide visibility.
/// Silently drops if EventBus is not available or channel is full (non-blocking).
pub fn publish_download_event(progress: &PipelineProgress) {
    let Some(bus) = GLOBAL_EVENT_BUS.get() else {
        return;
    };
    let status_str = match &progress.status {
        PipelineStatus::Resolving => "resolving".into(),
        PipelineStatus::Downloading { downloaded, total, speed_bps: _ } => {
            format!("downloading:{}:{:?}", downloaded, total)
        }
        PipelineStatus::Playing { downloaded, total, speed_bps: _ } => {
            format!("playing:{}:{:?}", downloaded, total)
        }
        PipelineStatus::Complete { total_bytes, elapsed: _ } => {
            format!("complete:{}", total_bytes)
        }
        PipelineStatus::Failed(e) => format!("failed:{}", e),
        PipelineStatus::Cancelled => "cancelled".into(),
    };
    let (downloaded, total, speed_bps) = match &progress.status {
        PipelineStatus::Downloading { downloaded, total, speed_bps }
        | PipelineStatus::Playing { downloaded, total, speed_bps } => {
            (*downloaded, *total, *speed_bps)
        }
        PipelineStatus::Complete { total_bytes, .. } => (*total_bytes, Some(*total_bytes), 0.0),
        _ => (0, None, 0.0),
    };
    (**bus).emit(crate::l0_substrate::nt_core_event::CoreEvent::DownloadProgress {
        url: progress.url.clone(),
        status: status_str,
        downloaded,
        total,
        speed_bps,
        output: progress.output.to_string_lossy().into_owned(),
    });
}

// ═══════════════════════════════════════════════════════════════════════════
// StreamingPipeline — main entry point
// ═══════════════════════════════════════════════════════════════════════════

pub struct StreamingPipeline {
    config: StreamingPipelineConfig,
}

impl StreamingPipeline {
    pub fn new(config: StreamingPipelineConfig) -> Self {
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
            TransportType::Hls => {
                let output_hls = output.clone();
                let url_hls = url.clone();
                let client_hls = client.clone();
                let auth_hls = config.auth.clone();

                let download_handle = tokio::spawn(async move {
                    stream_hls_download(
                        &client_hls,
                        &url_hls,
                        &output_hls,
                        config.timeout,
                        cancel_rx_flag,
                        progress_tx.clone(),
                        auth_hls.as_ref(),
                        config.persistence.clone(),
                        config.concurrency,
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
                    // 2026-10-06（审计 D1）：FIFO 创建失败原本静默，
                    // 而下方代码会继续用这个路径 ⇒ 错误延后到别处以**更难懂**的形式爆。
                    if let Err(e) = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .mode(0o644)
                        .open(&fifo_path)
                    {
                        log::warn!("[streaming] 创建 FIFO 失败（后续写入将失败）: {e}");
                    }

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
// Player backends
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn detect_player(preferred: Option<&str>) -> Option<&'static str> {
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

pub(crate) async fn spawn_player(
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

pub(crate) async fn spawn_fifo_player(
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
