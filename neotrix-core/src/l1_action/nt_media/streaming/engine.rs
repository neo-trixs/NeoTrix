//! engine — 从 `streaming.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::mpsc;
use super::types::{AggregateProgress, DownloadConfig, DownloadProgressSnapshot, DownloadStatus, DownloadTask};
use super::dl_fs::{check_disk_space, detect_filename, is_done, make_dl_tmp_dir, merge_chunks, write_done_marker};
use super::parallel::http_chunk_download;
use super::{record_mirror_speed, resolve_mirror};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::SystemTime;
use super::super::router;
use crate::l1_action::nt_io::nt_io_http_factory;
use std::sync::atomic::AtomicU64;
use tokio::fs;

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
        progress_tx: Option<mpsc::Sender<DownloadProgressSnapshot>>,
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
                if let Err(e) = fs::create_dir_all(parent).await {
                    eprintln!("[dl] failed to create parent dir {}: {}", parent.display(), e);
                }
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
        progress_tx: Option<mpsc::Sender<DownloadProgressSnapshot>>,
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
