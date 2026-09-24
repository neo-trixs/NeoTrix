//! alt — 从 `streaming.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use super::types::{PipelineError, PipelineProgress, PipelineStatus};
use super::dl_fs::extract_domain;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use super::super::auth::AuthConfig;
use super::super::detect::MediaKind;
use futures::StreamExt;

pub(crate) async fn stream_hls_download(
    client: &reqwest::Client,
    url: &str,
    output: &Path,
    timeout: Duration,
    cancel: Arc<AtomicBool>,
    progress_tx: mpsc::Sender<PipelineProgress>,
    auth: Option<&AuthConfig>,
    persistence: Option<Arc<super::super::persistence::DownloadStore>>,
    _concurrency: usize,
    media_kind: MediaKind,
) -> Result<(), PipelineError> {
    use super::super::hls;

    let started = Instant::now();

    // Record in persistence store
    let _record_id = if let Some(ref store) = persistence {
        let record = super::super::persistence::DownloadRecord::new(
            url.to_string(),
            output.to_path_buf(),
            "Hls".to_string(),
        );
        let id = record.id.clone();
        store.add_record(record).await;
        let _ = store.save().await;
        Some(id)
    } else {
        None
    };

    // Fetch and parse manifest
    let mut req = client.get(url).timeout(timeout);
    if let Some(a) = auth {
        req = a.strategy.apply_to_request(req);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| PipelineError::Network(e.to_string()))?;
    let manifest_text = resp
        .text()
        .await
        .map_err(|e| PipelineError::Network(e.to_string()))?;

    let manifest =
        hls::parse_m3u8(&manifest_text).map_err(|e| PipelineError::Network(e.to_string()))?;

    // Resolve to segment URLs
    let segment_urls = match &manifest {
        hls::M3u8Manifest::Master(master) => {
            // Pick best quality variant (highest bandwidth)
            let variant = hls::select_variant(master, None)
                .ok_or_else(|| PipelineError::Network("no variants in master playlist".to_string()))?;
            // Fetch the variant playlist
            let variant_url = hls::to_download_urls(
                &hls::M3u8Manifest::Master(master.clone()),
                url,
            )
            .into_iter()
            .next()
            .unwrap_or_else(|| variant.uri.clone());

            let mut req2 = client.get(&variant_url).timeout(timeout);
            if let Some(a) = auth {
                req2 = a.strategy.apply_to_request(req2);
            }
            let resp2 = req2
                .send()
                .await
                .map_err(|e| PipelineError::Network(e.to_string()))?;
            let variant_text = resp2
                .text()
                .await
                .map_err(|e| PipelineError::Network(e.to_string()))?;
            let variant_manifest =
                hls::parse_m3u8(&variant_text).map_err(|e| PipelineError::Network(e.to_string()))?;
            hls::to_download_urls(&variant_manifest, &variant_url)
        }
        hls::M3u8Manifest::Media(_media) => hls::to_download_urls(&manifest, url),
    };

    let total_segments = segment_urls.len() as u64;
    if total_segments == 0 {
        return Err(PipelineError::Network("no segments in playlist".to_string()));
    }

    let total_size_hint: Option<u64> = None;
    let mut downloaded_bytes: u64 = 0;

    // Write segments sequentially into a temp dir, then merge
    let tmp_dir = output.parent().unwrap_or(Path::new("/tmp")).join(format!(
        ".dl_hls_{}",
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&tmp_dir)
        .await
        .map_err(|e| PipelineError::Io(e.to_string()))?;

    let mut segment_files: Vec<PathBuf> = Vec::with_capacity(segment_urls.len());

    for (i, seg_url) in segment_urls.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            let _ = fs::remove_dir_all(&tmp_dir).await;
            return Err(PipelineError::Cancelled);
        }

        let seg_path = tmp_dir.join(format!("seg{:04}.tmp", i));
        let seg_size = download_single_segment(
            client,
            seg_url,
            &seg_path,
            timeout,
            auth,
        )
        .await?;
        downloaded_bytes += seg_size;
        segment_files.push(seg_path);

        let _ = progress_tx
            .send(PipelineProgress {
                url: url.to_string(),
                status: PipelineStatus::Downloading {
                    downloaded: downloaded_bytes,
                    total: total_size_hint,
                    speed_bps: 0.0,
                },
                media_kind,
                output: output.to_path_buf(),
                elapsed: started.elapsed(),
            })
            .await;
    }

    // Merge all segments into the output file
    let mut out = BufWriter::with_capacity(
        256 * 1024,
        fs::File::create(output)
            .await
            .map_err(|e| PipelineError::Io(e.to_string()))?,
    );
    let mut buf = vec![0u8; 8192];
    for seg_file in &segment_files {
        let mut f = fs::File::open(seg_file)
            .await
            .map_err(|e| PipelineError::Io(e.to_string()))?;
        loop {
            let n = f
                .read(&mut buf)
                .await
                .map_err(|e| PipelineError::Io(e.to_string()))?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])
                .await
                .map_err(|e| PipelineError::Io(e.to_string()))?;
        }
    }
    out.flush()
        .await
        .map_err(|e| PipelineError::Io(e.to_string()))?;

    // Cleanup temp dir
    let _ = fs::remove_dir_all(&tmp_dir).await;

    let _ = progress_tx
        .send(PipelineProgress {
            url: url.to_string(),
            status: PipelineStatus::Complete {
                total_bytes: downloaded_bytes,
                elapsed: started.elapsed(),
            },
            media_kind,
            output: output.to_path_buf(),
            elapsed: started.elapsed(),
        })
        .await;

    Ok(())
}

/// Download a single HLS segment to a file.
pub(crate) async fn download_single_segment(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    timeout: Duration,
    auth: Option<&AuthConfig>,
) -> Result<u64, PipelineError> {
    let mut req = client.get(url).timeout(timeout);
    if let Some(a) = auth {
        req = a.strategy.apply_to_request(req);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| PipelineError::Network(e.to_string()))?;

    let mut file = fs::File::create(dest)
        .await
        .map_err(|e| PipelineError::Io(e.to_string()))?;
    let mut stream = resp.bytes_stream();
    let mut written: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let data = chunk.map_err(|e| PipelineError::Network(e.to_string()))?;
        file.write_all(&data)
            .await
            .map_err(|e| PipelineError::Io(e.to_string()))?;
        written += data.len() as u64;
    }

    file.flush()
        .await
        .map_err(|e| PipelineError::Io(e.to_string()))?;
    Ok(written)
}

// ═══════════════════════════════════════════════════════════════════════════
// Magnet Download via aria2c RPC
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) async fn stream_magnet_download(
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

/// Check if aria2c RPC daemon is running and responsive.
/// Returns Ok(version_string) if healthy, Err(message) if not.
pub async fn check_aria2c_health() -> Result<String, String> {
    let client = reqwest::Client::new();
    let rpc_url = "http://127.0.0.1:6800/jsonrpc";

    let rpc_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "nt-health-check",
        "method": "aria2.getVersion",
        "params": [],
    });

    match tokio::time::timeout(
        Duration::from_secs(3),
        client.post(rpc_url).json(&rpc_body).send(),
    )
    .await
    {
        Ok(Ok(resp)) => {
            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("parse error: {}", e))?;
            if let Some(error) = body.get("error") {
                return Err(format!(
                    "aria2c error: {}",
                    error
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown")
                ));
            }
            let version = body["result"]["version"]
                .as_str()
                .unwrap_or("unknown");
            Ok(format!("aria2c v{}", version))
        }
        Ok(Err(e)) => Err(format!("connection failed: {}", e)),
        Err(_) => Err("timeout (3s)".into()),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HTTP to FIFO (zero disk I/O)
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) async fn stream_http_to_fifo(
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
