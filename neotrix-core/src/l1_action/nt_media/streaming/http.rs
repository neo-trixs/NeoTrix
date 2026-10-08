//! http — 从 `streaming.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use super::types::{PipelineError, PipelineProgress, PipelineStatus};
use super::parallel::probe_range_support;
use super::dl_fs::extract_domain;
use super::parallel::ParallelDownloader;
use super::pipeline::publish_download_event;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use sha2::{Digest, Sha256};
use super::super::auth::AuthConfig;
use super::super::detect::MediaKind;
use tokio::fs;
use std::sync::atomic::AtomicU64;
use futures::StreamExt;

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
/// Delegates to streaming implementation to avoid loading entire file into memory.
pub async fn compute_sha256(path: &Path) -> Result<String, PipelineError> {
    super::super::persistence::compute_sha256_streaming(path)
        .await
        .map_err(PipelineError::Io)
}

// ═══════════════════════════════════════════════════════════════════════════
// HTTP Streaming Download (enhanced: parallel + stall + retry + verify +
//   mirror resolution + temp-file merge + .done markers + disk pre-check)
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) async fn stream_http_download(
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
    persistence: Option<Arc<super::super::persistence::DownloadStore>>,
    concurrency: usize,
    expected_sha256: Option<&str>,
    stall_timeout: Duration,
    max_retries: u32,
) -> Result<(), PipelineError> {
    let started = Instant::now();

    let record_id = if let Some(ref store) = persistence {
        let record = super::super::persistence::DownloadRecord::new(
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
            if let Some(sha) = expected_sha256 {
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
            publish_download_event(&PipelineProgress {
                url: url.to_string(),
                status: PipelineStatus::Complete {
                    total_bytes: final_bytes,
                    elapsed: started.elapsed(),
                },
                media_kind,
                output: output.to_path_buf(),
                elapsed: started.elapsed(),
            });
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
        publish_download_event(&PipelineProgress {
            url: url.to_string(),
            status: PipelineStatus::Resolving,
            media_kind,
            output: output.to_path_buf(),
            elapsed: Duration::ZERO,
        });

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
                publish_download_event(&PipelineProgress {
                    url: url.to_string(),
                    status: PipelineStatus::Cancelled,
                    media_kind,
                    output: output.to_path_buf(),
                    elapsed: started.elapsed(),
                });
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

                        let progress = PipelineProgress {
                            url: url.to_string(),
                            status,
                            media_kind,
                            output: output.to_path_buf(),
                            elapsed: started.elapsed(),
                        };
                        publish_download_event(&progress);
                        let _ = progress_tx.send(progress).await;

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
                    publish_download_event(&PipelineProgress {
                        url: url.to_string(),
                        status: PipelineStatus::Failed(e.to_string()),
                        media_kind,
                        output: output.to_path_buf(),
                        elapsed: started.elapsed(),
                    });
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
            publish_download_event(&PipelineProgress {
                url: url.to_string(),
                status: PipelineStatus::Failed("SHA-256 integrity check failed".into()),
                media_kind,
                output: output.to_path_buf(),
                elapsed: started.elapsed(),
            });
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
    publish_download_event(&PipelineProgress {
        url: url.to_string(),
        status: PipelineStatus::Complete {
            total_bytes: final_bytes,
            elapsed: started.elapsed(),
        },
        media_kind,
        output: output.to_path_buf(),
        elapsed: started.elapsed(),
    });

    Ok(())
}
