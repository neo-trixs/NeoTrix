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

        // Disk space pre-check（**只查，不预分配**）
        //
        // ⭐⭐ 2026-10-02 删除了原来的 `dest` 预分配（`File::…set_len(total_size)`），
        //    它是一个 **P0 正确性缺陷**，链条闭合：
        //      ① 预分配把 `dest` 撑成 `total_size` 字节零填充
        //      ② 下面的 `existing = metadata(dest).len()` 于是 **恒等于 total_size**
        //      ③ 紧跟的早退条件 `existing >= total_size` **必然成立**
        //      ④ ⇒ **一个字节都没下**就 `write_done_marker` + `return Ok(existing)`
        //      ⑤ `.done` 落盘 ⇒ `is_done()` 此后**永久**返回 `Some`
        //      ⑥ ⇒ `:293` 起的分块下载与 `merge_chunks` **永不可达**
        //    触发条件只是 HEAD 返回 `Content-Length`（`head_size()` 成功），
        //    即**绝大多数媒体文件**。
        //
        // ⭐ 为什么预分配本就多余：`merge_chunks`（`dl_fs.rs:127`）用
        //   `File::create(dest)` **整体覆写** `dest` ⇒ 预分配的内容必被丢弃。
        //   真正的「早失败」价值由上面的 `check_disk_space` 提供；
        //   真 ENOSPC 会在 `merge_chunks` 的 `create` 处以可读错误浮出。
        // ⛔ 不用 let-chain（本仓 edition < 2024， 编译不过）。
        if total_size > 0 && !dest.exists() {
            if let Err(e) = check_disk_space(&dest, total_size, self.config.min_disk_space) {
                return Err(e);
            }
        }

        // Resume: existing bytes
        let existing = if dest.exists() {
            fs::metadata(&dest).await.map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        // ⛔⛔ 2026-10-02 **删除**了这里的「长度够就算完成」早退。
        //    原码：`if total_size > 0 && existing >= total_size { write_done_marker(…); return Ok(existing); }`
        //    ⭐ 它本意是「续传时发现已经下完」；但在上面预分配被删之后，
        //    这个条件仍会在**上一次崩溃留下半截 dest** 时误判为完成
        //    —— 「文件够长」从来**不是**完成的证据，**`.done` 才是**。
        //    ⓰ 「已完成」的权威判据在函数开头 `:236` 的 `is_done(&dest)`，
        //       它读 `.done` marker ⇒ 那里已覆盖真正的完成态，**这里不需要第二份**。
        //    ⛔ 保留它就是保留一个「零字节下载报成功」的入口。

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

#[cfg(test)]
mod p0_regression_tests {
    use super::*;
    use crate::l1_action::nt_media::streaming::types::DownloadTask;

    /// ⭐⭐⭐ P0 回归（2026-10-02）：**HEAD 返回 Content-Length 时，
    /// 一次字节都不许下就报 `Completed`**。
    ///
    /// ## 原缺陷的链条（闭合）
    /// `dest` 预分配 `set_len(total_size)` → `existing = metadata.len()` 恒等于
    /// `total_size` → 早退条件 `existing >= total_size` 必然成立 →
    /// `write_done_marker` + `return Ok(existing)` ⇒ 零字节下载报成功，
    /// 且 `.done` 让 `is_done()` 此后永久返回 `Some` ⇒ 分块下载永不可达。
    ///
    /// ⭐ 判据用**「服务端实际发出的字节数」**而不是「落盘文件大小」：
    /// 前者是「有没有真的传输」的**唯一**可观测信号 —— 落盘大小可以被
    /// 预分配/零填充伪造，正是原缺陷能藏住的原因。
    #[tokio::test]
    async fn 有ContentLength时必须真的传输字节() {
        use axum::routing::get;
        use axum::Router;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        const N: usize = 4096;
        let served = Arc::new(AtomicUsize::new(0));
        let body: Vec<u8> = (0..N as u32).map(|i| (i % 251) as u8).collect();
        let served_c = Arc::clone(&served);
        let app = Router::new().route(
            "/blob",
            get(move || {
                let b = body.clone();
                let s = Arc::clone(&served_c);
                async move {
                    s.fetch_add(b.len(), Ordering::SeqCst);
                    b
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind loopback");
        let addr = listener.local_addr().expect("local_addr");
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let dir = tempfile::tempdir().expect("tmpdir");
        let dest = dir.path().join("blob.bin");
        let url = format!("http://{addr}/blob");
        let status = DownloadEngine::default()
            .download(&DownloadTask::new(url, &dest))
            .await;

        assert!(
            served.load(Ordering::SeqCst) >= N,
            "服务端只发出 {} 字节，期望 ≥ {N} ⇒ 客户端**零字节下载却报成功**（原 P0）",
            served.load(Ordering::SeqCst)
        );
        match status {
            DownloadStatus::Completed { .. } => {}
            other => panic!("期望 Completed，实际 {other:?}"),
        }
        // ⭐⭐ 落盘内容必须**真的是那份 blob** —— 这条比「大小相等」更强：
        // 零填充文件的前 N 字节是 0，与 blob 的 `(i % 251)` 必然不同。
        let bytes = std::fs::read(&dest).expect("read dest");
        // ⭐⭐ 这条断言在 parallel.rs 的 Range 修复**之前**是失败的
        //（实测落盘 65536 = 16 chunk × 4096，服务端忽略 Range 导致每块写全量）。
        // ⭐ 加回来是因为它才是「文件没被撑大」的**直接**判据；
        //   只断「传输 ≥ N」会漏掉「传输过量」这个方向的损坏。
        assert_eq!(
            bytes.len(), N,
            "落盘 {} 字节 ≠ Content-Length {N} ⇒ chunk 未按 Range 切片（服务端忽略 Range 时会成倍写长）",
            bytes.len()
        );
        assert!(
            bytes[..N].iter().enumerate().all(|(i, b)| *b == (i as u32 % 251) as u8),
            "落盘前 {N} 字节与 blob 不符 ⇒ 很可能是零填充而非真实传输（原 P0 的伪装形态）"
        );
    }
}
