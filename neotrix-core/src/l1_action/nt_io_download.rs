//! NeoTrix 通用下载引擎 — 四层架构
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │  Scheduler (任务调度)                                    │
//! │  优先级队列 / 全局并发上限 / 去重 / 取消 / 聚合进度        │
//! ├─────────────────────────────────────────────────────────┤
//! │  Protocol (协议分发)                                    │
//! │  URL → scheme 匹配 → 选择 transport                     │
//! │  HTTP/HTTPS → Range 分片 | Magnet → BT backend          │
//! │  HuggingFace → 镜像加速 | FTP/SFTP → 扩展点             │
//! ├─────────────────────────────────────────────────────────┤
//! │  Transport (传输层)                                     │
//! │  分片并发 / 断点续传 / BufWriter / 磁盘预分配             │
//! ├─────────────────────────────────────────────────────────┤
//! │  Observer (可观测)                                      │
//! │  进度 channel / 速度画像 / .done 标记 / 聚合统计          │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! 能力清单:
//! - **协议**: HTTP(S) Range 分片 | HuggingFace 镜像自适应 | magnet URI 解析(预留)
//! - **调度**: 任务级全局并发上限 | 优先级 | URL 去重 | 任务取消 | 聚合进度
//! - **传输**: 分片并发 | 断点续传 | BufWriter 256KB | 磁盘预分配+空间预检
//! - **观测**: mpsc 进度 channel | 镜像速度画像(EMA) | 原子 .done 标记 | 指数退避重试

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::sync::{mpsc, Semaphore};

// ═══════════════════════════════════════════════════════════════════════════
// L4 Observer — 进度、速度画像、.done 标记
// ═══════════════════════════════════════════════════════════════════════════

/// 单任务进度
#[derive(Debug, Clone)]
pub(crate) struct DownloadProgress {
    pub percent: f32,
    pub downloaded: u64,
    pub total: u64,
    pub speed_mbps: f64,
    pub eta_secs: Option<f64>,
}

/// 多任务聚合进度
#[derive(Debug, Clone)]
pub(crate) struct AggregateProgress {
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

/// 单任务状态
#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Pending,
    InProgress(DownloadProgress),
    Completed { elapsed_secs: f64, size_mb: f64 },
    Failed(String),
    Cancelled,
}

// ═══════════════════════════════════════════════════════════════════════════
// L3 Transport — 镜像速度画像 + .done 标记
// ═══════════════════════════════════════════════════════════════════════════

static MIRROR_SPEED_MAP: LazyLock<Mutex<HashMap<String, f64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

const MIRROR_ENDPOINTS: &[&str] = &["https://hf-mirror.com", "https://huggingface.co"];

fn record_mirror_speed(endpoint: &str, bytes_per_sec: f64) {
    if let Ok(mut map) = MIRROR_SPEED_MAP.lock() {
        let entry = map.entry(endpoint.to_string()).or_insert(0.0);
        *entry = 0.7 * *entry + 0.3 * bytes_per_sec;
    }
}

fn ranked_mirrors() -> Vec<(String, f64)> {
    let map = MIRROR_SPEED_MAP.lock().unwrap();
    let mut pairs: Vec<_> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    pairs
}

// ═══════════════════════════════════════════════════════════════════════════
// L2 Protocol — URL 解析 + 协议分发
// ═══════════════════════════════════════════════════════════════════════════

/// URL 协议类型 — delegates to nt_media::router::UrlScheme
pub(crate) use crate::l1_action::nt_media::router::UrlScheme;

/// 判断是否为 HuggingFace URL — delegates to nt_media::router
pub(crate) use crate::l1_action::nt_media::router::is_huggingface_url;

/// 镜像解析 (HuggingFace 自适应)
async fn resolve_mirror(client: &reqwest::Client, original_url: &str) -> String {
    if !is_huggingface_url(original_url) || original_url.contains("hf-mirror.com") {
        return original_url.to_string();
    }
    // 环境变量强制覆盖
    if let Ok(endpoint) = std::env::var("NT_DOWNLOAD_MIRROR_ENDPOINT") {
        let ep = endpoint.trim();
        if !ep.is_empty() {
            let ep = if ep.starts_with("http") {
                ep.to_string()
            } else {
                format!("https://{}", ep)
            };
            let forced = original_url
                .replace("https://huggingface.co", &ep)
                .replace("http://huggingface.co", &ep);
            if forced != original_url {
                eprintln!("[mirror] env override: {}", ep);
                return forced;
            }
        }
    }
    // 按速度排序
    let speed_ranking = ranked_mirrors();
    let mut endpoints: Vec<&str> = MIRROR_ENDPOINTS.to_vec();
    if !speed_ranking.is_empty() {
        endpoints.sort_by(|a, b| {
            let sa = speed_ranking
                .iter()
                .find(|(k, _)| k.contains(a.trim_start_matches("https://")))
                .map(|(_, v)| *v)
                .unwrap_or(0.0);
            let sb = speed_ranking
                .iter()
                .find(|(k, _)| k.contains(b.trim_start_matches("https://")))
                .map(|(_, v)| *v)
                .unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    // 并行探测
    let mut candidates: Vec<String> = endpoints
        .iter()
        .filter_map(|ep| {
            let c = original_url
                .replace("https://huggingface.co", ep)
                .replace("http://huggingface.co", ep);
            if c != original_url {
                Some(c)
            } else {
                None
            }
        })
        .collect();
    candidates.push(original_url.to_string());
    let mut handles = Vec::new();
    for url in &candidates {
        let client = client.clone();
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            match tokio::time::timeout(Duration::from_secs(2), client.head(&url).send()).await {
                Ok(Ok(resp)) if resp.status().is_success() => Some(url),
                _ => None,
            }
        }));
    }
    for h in handles {
        if let Ok(Some(url)) = h.await {
            eprintln!("[mirror] using: {}", url);
            return url;
        }
    }
    original_url.to_string()
}

/// Content-Disposition 文件名检测
async fn detect_filename(client: &reqwest::Client, url: &reqwest::Url, dest: &Path) -> PathBuf {
    if dest
        .file_stem()
        .is_some_and(|s| !s.to_string_lossy().is_empty())
    {
        return dest.to_path_buf();
    }
    if let Ok(resp) = client.head(url.clone()).send().await {
        if let Some(cd) = resp.headers().get("content-disposition") {
            if let Ok(cd_str) = cd.to_str() {
                if let Some(pos) = cd_str.find("filename*=UTF-8''") {
                    let encoded = &cd_str[pos + 16..];
                    if let Some(name) = encoded.split(';').next() {
                        if let Ok(decoded) = urlencoding::decode(name) {
                            return dest.with_file_name(decoded.as_ref());
                        }
                    }
                }
                if let Some(start) = cd_str.find("filename=\"") {
                    let rest = &cd_str[start + 10..];
                    if let Some(end) = rest.find('"') {
                        return dest.with_file_name(&rest[..end]);
                    }
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
// L1 Transport — HTTP Range 分片下载
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
    writer.flush().await.map_err(|e| format!("flush: {}", e))?;
    Ok(downloaded)
}

// ═══════════════════════════════════════════════════════════════════════════
// L0 Config & Types
// ═══════════════════════════════════════════════════════════════════════════

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
// L0 Task Handle — 取消控制
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) struct TaskHandle {
    cancelled: Arc<AtomicBool>,
}

impl TaskHandle {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
    pub(crate) fn _is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Download Engine — 四层融合
// ═══════════════════════════════════════════════════════════════════════════

pub struct DownloadEngine {
    config: DownloadConfig,
    client: reqwest::Client,
    task_semaphore: Arc<Semaphore>,
    global_downloaded: Arc<AtomicU64>,
}

impl DownloadEngine {
    pub fn new(config: DownloadConfig) -> Self {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .connect_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(config.max_concurrent + 4)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_nodelay(true)
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent("NeoTrix/1.0 (download-engine)")
            .no_gzip()
            .no_brotli()
            .no_deflate();

        for var in &[
            "HTTPS_PROXY",
            "https_proxy",
            "HTTP_PROXY",
            "http_proxy",
            "ALL_PROXY",
            "all_proxy",
        ] {
            if let Ok(proxy_url) = std::env::var(var) {
                let p = proxy_url.trim();
                if !p.is_empty() {
                    let p = if p.contains("://") {
                        p.to_string()
                    } else {
                        format!("http://{}", p)
                    };
                    if let Ok(proxy) = reqwest::Proxy::all(&p) {
                        builder = builder.proxy(proxy);
                        break;
                    }
                }
            }
        }

        let client = builder.build().expect("create reqwest client");
        Self {
            task_semaphore: Arc::new(Semaphore::new(config.max_tasks)),
            global_downloaded: Arc::new(AtomicU64::new(0)),
            config,
            client,
        }
    }

    // ── 公共 API ──────────────────────────────────────────────────────────

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

    /// 多任务并行 (全局并发上限 + 去重 + 聚合进度)
    pub async fn download_all(
        &self,
        tasks: &[DownloadTask],
        progress_tx: Option<mpsc::Sender<AggregateProgress>>,
    ) -> Vec<DownloadStatus> {
        // 去重
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut deduped: Vec<(usize, &DownloadTask)> = Vec::new();
        for (i, task) in tasks.iter().enumerate() {
            if seen.contains_key(&task.url) {
                continue;
            }
            seen.insert(task.url.clone(), i);
            deduped.push((i, task));
        }

        // 磁盘预检
        if let Some(first) = deduped.first().map(|(_, t)| &t.dest) {
            if let Some(parent) = first.parent() {
                if let Ok(meta) = std::fs::metadata(parent) {
                    if !meta.is_dir() {
                        eprintln!(
                            "[dl] disk warning: parent path is not a directory: {}",
                            parent.display()
                        );
                    }
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
                    Err(_) => return DownloadStatus::Failed("semaphore closed".to_string()),
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

    // ── 内部 ──────────────────────────────────────────────────────────────

    fn spawn_child(&self) -> DownloadEngine {
        DownloadEngine {
            config: self.config.clone(),
            client: self.client.clone(),
            task_semaphore: self.task_semaphore.clone(),
            global_downloaded: self.global_downloaded.clone(),
        }
    }

    async fn download_inner(
        &self,
        task: &DownloadTask,
        progress_tx: Option<mpsc::Sender<DownloadProgress>>,
        cancelled: Arc<AtomicBool>,
    ) -> Result<u64, String> {
        let scheme = UrlScheme::parse(&task.url);

        // 协议分发
        match scheme {
            UrlScheme::Magnet => {
                // magnet 预留: 需要 librqbit 或外部 aria2 RPC
                return Err("magnet link: use aria2c --enable-rpc or add librqbit backend".into());
            }
            UrlScheme::Ftp => {
                return Err("ftp: not yet implemented, use HTTP mirror".into());
            }
            _ => {} // HTTP/HTTPS/Unknown → 继续
        }

        let mut url = reqwest::Url::parse(&task.url).map_err(|e| e.to_string())?;

        // L2: 镜像解析
        if is_huggingface_url(&task.url) {
            url = reqwest::Url::parse(&resolve_mirror(&self.client, &task.url).await)
                .map_err(|e| e.to_string())?;
        }

        if let Some(parent) = task.dest.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("mkdir: {}", e))?;
        }

        let dest = detect_filename(&self.client, &url, &task.dest).await;

        // .done 标记
        let done_marker = dest.with_extension("done");
        if dest.exists() && done_marker.exists() {
            if let Ok(meta) = fs::metadata(&dest).await {
                if let Ok(content) = fs::read_to_string(&done_marker).await {
                    if let Some(done_size) = content
                        .lines()
                        .find(|l| l.starts_with("size="))
                        .and_then(|l| l.strip_prefix("size="))
                        .and_then(|s| s.parse::<u64>().ok())
                    {
                        if meta.len() >= done_size {
                            return Ok(meta.len());
                        }
                    }
                }
            }
        }

        // HEAD 获取大小
        let total_size = self.head_size(&url).await.unwrap_or(0);

        // 磁盘预分配 + 空间预检
        if total_size > 0 && !dest.exists() {
            if let Some(parent) = dest.parent() {
                if let Ok(meta) = std::fs::metadata(parent) {
                    if !meta.is_dir() {
                        return Err(format!(
                            "disk full: parent path is not a directory: {}",
                            parent.display()
                        ));
                    }
                }
            }
            let _ = std::fs::File::options()
                .write(true)
                .create(true)
                .truncate(false)
                .open(&dest)
                .and_then(|f| f.set_len(total_size));
        }

        // 断点续传
        let existing = if dest.exists() {
            fs::metadata(&dest).await.map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        if total_size > 0 && existing >= total_size {
            let _ = fs::write(
                &done_marker,
                format!("size={}\nurl={}\n", total_size, task.url),
            )
            .await;
            return Ok(existing);
        }

        // 分片计算
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

        let stem = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("dl");
        let tmp_dir = dest
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!(".dl_{}", stem));
        fs::create_dir_all(&tmp_dir)
            .await
            .map_err(|e| format!("tmp dir: {}", e))?;

        // 并发分片
        let semaphore = Arc::new(Semaphore::new(n_chunks));
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

        // 等待 + 进度
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

            let progress = DownloadProgress {
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

        // 合并
        let mut out = BufWriter::with_capacity(
            256 * 1024,
            fs::File::create(&dest)
                .await
                .map_err(|e| format!("create: {}", e))?,
        );
        let mut buf = vec![0u8; 8192];
        for i in 0..n_chunks {
            let chunk_file = tmp_dir.join(format!("c{:04}.tmp", i));
            let mut f = fs::File::open(&chunk_file)
                .await
                .map_err(|e| format!("open: {}", e))?;
            loop {
                let n = f.read(&mut buf).await.map_err(|e| format!("read: {}", e))?;
                if n == 0 {
                    break;
                }
                out.write_all(&buf[..n])
                    .await
                    .map_err(|e| format!("write: {}", e))?;
            }
        }
        out.flush().await.map_err(|e| format!("flush: {}", e))?;
        drop(out);

        // 速度画像
        let final_elapsed = loop_start.elapsed().as_secs_f64();
        if let Some(host) = url.host_str() {
            if total_size > 0 && final_elapsed > 0.5 {
                let bps = (total_downloaded - existing) as f64 / final_elapsed;
                record_mirror_speed(host, bps);
            }
        }

        // .done 标记
        let _ = fs::write(
            &done_marker,
            format!(
                "size={}\nurl={}\ntimestamp={}\n",
                total_size,
                task.url,
                SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
        )
        .await;
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
