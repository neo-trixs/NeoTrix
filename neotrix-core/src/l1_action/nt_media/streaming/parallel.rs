//! parallel — 从 `streaming.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use super::types::{PipelineError, PipelineProgress, PipelineStatus};
use super::resilience::{RetryPolicy, StallDetector, TokenBucket};
use super::{ChunkDownloadStatus, ChunkState};
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::collections::VecDeque;
use tokio::sync::Mutex as TokioMutex;
use super::super::auth::AuthConfig;
use super::super::detect::MediaKind;
use tokio::fs;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicBool;
use super::pipeline::publish_download_event;
use futures::StreamExt;
use super::dl_fs::extract_domain;

pub(crate) struct ParallelDownloader {
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
    pub(crate) fn new(
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
            Arc::new(TokioMutex::new(TokenBucket::new(rate * 2.0, rate)))
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

    pub(crate) fn plan_chunks(total_size: u64, chunk_size: usize) -> Vec<ChunkState> {
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

            match http_range_request(
                &self.client,
                &self.url,
                range_start,
                range_end,
                self.auth.as_ref(),
                Duration::from_secs(60),
            )
            .await
            {
                Ok((resp, _content_length)) => {
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

    pub(crate) async fn run(
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
        publish_download_event(&PipelineProgress {
            url: self.url.clone(),
            status: PipelineStatus::Resolving,
            media_kind,
            output: self.output.clone(),
            elapsed: Duration::ZERO,
        });

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
                        publish_download_event(&PipelineProgress {
                            url: url.clone(),
                            status: PipelineStatus::Failed(e.to_string()),
                            media_kind,
                            output: output.clone(),
                            elapsed: started.elapsed(),
                        });
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
                            let oldest = match window.front() {
                                Some(v) => v,
                                None => continue,
                            };
                            let newest = match window.back() {
                                Some(v) => v,
                                None => continue,
                            };
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
                    publish_download_event(&PipelineProgress {
                        url: url.clone(),
                        status: PipelineStatus::Downloading {
                            downloaded: written,
                            total: Some(total_size),
                            speed_bps,
                        },
                        media_kind,
                        output: output.clone(),
                        elapsed: started.elapsed(),
                    });
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
pub(crate) async fn parallel_http_download(
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
// Range Support Probe
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) async fn probe_range_support(
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

/// Shared HTTP Range request helper. Sends a Range request and returns the response stream.
/// Used by both `http_chunk_download` and `ParallelDownloader::download_chunk`.
pub(crate) async fn http_range_request(
    client: &reqwest::Client,
    url: &str,
    start: u64,
    end: u64,
    auth: Option<&AuthConfig>,
    timeout: Duration,
) -> Result<(reqwest::Response, u64), PipelineError> {
    let mut req = client
        .get(url)
        .header("Accept-Encoding", "identity")
        .timeout(timeout);

    if end != 0 {
        req = req.header("Range", format!("bytes={}-{}", start, end));
    } else if start > 0 {
        req = req.header("Range", format!("bytes={}-", start));
    }

    if let Some(auth_cfg) = auth {
        let domain = extract_domain(url);
        req = auth_cfg.apply(req, &domain).await;
    }

    let resp = req
        .send()
        .await
        .map_err(|e| PipelineError::Network(e.to_string()))?;

    let status = resp.status().as_u16();
    if status == 404 || status == 403 || status == 410 {
        return Err(PipelineError::Http(status));
    }
    if !resp.status().is_success() && status != 206 {
        return Err(PipelineError::Http(status));
    }

    let content_length = resp.content_length().unwrap_or(0);
    Ok((resp, content_length))
}

pub(crate) async fn http_chunk_download(
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

    let (mut resp, _content_length) = http_range_request(
        client,
        url.as_str(),
        actual_start,
        end,
        None,
        Duration::from_secs(timeout_secs),
    )
    .await
    .map_err(|e| e.to_string())?;

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

    // 2026-10-02 修的**数据损坏 bug**：原实现把服务端发的**每一块**都写进文件，
    // 对 `end` **没有任何上限**。⇒ 只要服务端**忽略 `Range` 头**（回环/自建源/
    // 部分 CDN/反代都会），每个 chunk 请求都会拿到**整个响应体**
    // ⇒ N 个 chunk 各写全量 ⇒ 合并出 **N 倍长**的损坏文件。
    // 实测：`DownloadEngine` 默认 `max_concurrent=16`，一个 4096 字节的源
    // 落盘成 **65536**（= 16 × 4096）—— 由 P0 回归测试当场抓到。
    //
    // ✅ 修法按 HTTP 语义**取正确切片**，⛔ 不是「截断」（截断会静默产出错内容）：
    // · `206 Partial Content` ⇒ 服务端**遵守了** Range ⇒ 原样收。
    // · `200 OK` ⇒ 服务端**忽略了** Range、返回整个实体 ⇒ 需要的字节位于
    //   响应体内偏移 `actual_start` 处 ⇒ 跳过它，之后最多只收 `expected` 字节。
    let honored = resp.status() == reqwest::StatusCode::PARTIAL_CONTENT;
    let expected: Option<u64> = if honored {
        None
    } else if end != 0 {
        Some(end.saturating_sub(actual_start) + 1)
    } else {
        None
    };
    let mut to_skip: u64 = if honored { 0 } else { actual_start };
    let mut taken: u64 = 0;
    loop {
        match resp.chunk().await {
            Ok(Some(chunk)) => {
                let mut slice: &[u8] = &chunk;
                if to_skip > 0 {
                    let sk = to_skip.min(slice.len() as u64) as usize;
                    slice = &slice[sk..];
                    to_skip -= sk as u64;
                    if slice.is_empty() {
                        continue;
                    }
                }
                if let Some(cap) = expected {
                    if taken >= cap {
                        break;
                    }
                    let room = (cap - taken) as usize;
                    if slice.len() > room {
                        slice = &slice[..room];
                    }
                }
                writer
                    .write_all(slice)
                    .await
                    .map_err(|e| format!("write: {e}"))?;
                downloaded += slice.len() as u64;
                taken += slice.len() as u64;
            }
            Ok(None) => break,
            Err(e) => return Err(format!("stream: {e}")),
        }
    }
    // ⛔ 收少了 ⇒ 落盘将短于 Content-Length ⇒ **必须报错**，
    //    不能默默继续（否则会写出一个短了却标着完成的文件）。
    if let Some(cap) = expected {
        if taken < cap {
            return Err(format!(
                "range short read: 请求 [{actual_start}..={end}] 期望 {cap} 字节，实收 {taken}（服务端可能不支持 Range）"
            ));
        }
    }
    writer
        .flush()
        .await
        .map_err(|e| format!("flush: {}", e))?;
    Ok(downloaded)
}
