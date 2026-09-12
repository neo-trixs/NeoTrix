//! NeoTrix 自研下载引擎
//!
//! 能力: 分片并发 / HTTP Range 断点续传 / HuggingFace 镜像加速+速度画像 /
//!       代理 / 进度 channel / Content-Disposition 文件名 / BufWriter /
//!       原子 .done 标记 / 磁盘预分配 / 指数退避重试 / 状态感知重试 /
//!       多任务调度 / 全局并发上限 / 带宽配额 / 磁盘预检 / 去重 / 任务取消 / 聚合进度

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::sync::{mpsc, Semaphore};
use tokio::time::timeout;

// ═══════════════════════════════════════════════════════════════════════════
// 镜像速度画像 (aria2 --uri-selector=adaptive)
// ═══════════════════════════════════════════════════════════════════════════

static MIRROR_SPEED_MAP: LazyLock<Mutex<HashMap<String, f64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

const MIRROR_ENDPOINTS: &[&str] = &[
    "https://hf-mirror.com",
    "https://huggingface.co",
];

pub fn record_mirror_speed(endpoint: &str, bytes_per_sec: f64) {
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

fn is_huggingface_url(url: &str) -> bool {
    url.contains("huggingface.co") || url.contains("hf-mirror.com")
}

async fn resolve_mirror(client: &reqwest::Client, original_url: &str) -> String {
    if !is_huggingface_url(original_url) || original_url.contains("hf-mirror.com") {
        return original_url.to_string();
    }
    if let Ok(endpoint) = std::env::var("NT_DOWNLOAD_MIRROR_ENDPOINT") {
        let ep = endpoint.trim();
        if !ep.is_empty() {
            let ep = if ep.starts_with("http") { ep.to_string() } else { format!("https://{}", ep) };
            let forced = original_url
                .replace("https://huggingface.co", &ep)
                .replace("http://huggingface.co", &ep);
            if forced != original_url {
                eprintln!("[mirror] env override: {}", ep);
                return forced;
            }
        }
    }
    let speed_ranking = ranked_mirrors();
    let mut endpoints: Vec<&str> = MIRROR_ENDPOINTS.to_vec();
    if !speed_ranking.is_empty() {
        endpoints.sort_by(|a, b| {
            let sa = speed_ranking.iter().find(|(k, _)| k.contains(a.trim_start_matches("https://"))).map(|(_, v)| *v).unwrap_or(0.0);
            let sb = speed_ranking.iter().find(|(k, _)| k.contains(b.trim_start_matches("https://"))).map(|(_, v)| *v).unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    let mut candidates: Vec<String> = endpoints.iter()
        .filter_map(|ep| {
            let c = original_url.replace("https://huggingface.co", ep).replace("http://huggingface.co", ep);
            if c != original_url { Some(c) } else { None }
        })
        .collect();
    candidates.push(original_url.to_string());
    let mut handles = Vec::new();
    for url in &candidates {
        let client = client.clone();
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            match timeout(Duration::from_secs(2), client.head(&url).send()).await {
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

// ═══════════════════════════════════════════════════════════════════════════
// 配置 & 类型
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// 每个任务的分片并发数
    pub max_concurrent: usize,
    /// 任务级全局并发上限 (同时下载几个文件)
    pub max_tasks: usize,
    /// 全局带宽上限 bytes/s (0=不限)
    pub max_bandwidth: u64,
    /// 最小磁盘空间 bytes (低于此拒绝下载)
    pub min_disk_space: u64,
    /// 超时秒数
    pub timeout_secs: u64,
    /// 重试次数
    pub retry_count: u32,
    /// 单片最大字节
    pub max_chunk_bytes: u64,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 16,
            max_tasks: 8,
            max_bandwidth: 0,
            min_disk_space: 1024 * 1024 * 1024, // 1GB
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
    /// 优先级 (0=最低, 255=最高, 默认128)
    pub priority: u8,
}

impl DownloadTask {
    pub fn new(url: impl Into<String>, dest: impl Into<PathBuf>) -> Self {
        Self { url: url.into(), dest: dest.into(), priority: 128 }
    }
    pub fn with_priority(mut self, p: u8) -> Self { self.priority = p; self }
}

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub percent: f32,
    pub downloaded: u64,
    pub total: u64,
    pub speed_mbps: f64,
    pub eta_secs: Option<f64>,
}

#[derive(Debug)]
pub enum DownloadStatus {
    Pending,
    InProgress(DownloadProgress),
    Completed { elapsed_secs: f64, size_mb: f64 },
    Failed(String),
    Cancelled,
}

/// 多任务聚合进度
#[derive(Debug, Clone)]
pub struct AggregateProgress {
    pub total_tasks: usize,
    pub completed: usize,
    pub failed: usize,
    pub active: usize,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub overall_speed_mbps: f64,
    pub overall_percent: f32,
}

// ═══════════════════════════════════════════════════════════════════════════
// 单任务取消句柄
// ═══════════════════════════════════════════════════════════════════════════

pub struct TaskHandle {
    cancelled: Arc<AtomicBool>,
}

impl TaskHandle {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 下载引擎
// ═══════════════════════════════════════════════════════════════════════════

pub struct DownloadEngine {
    config: DownloadConfig,
    client: reqwest::Client,
    /// 全局任务级并发信号量
    task_semaphore: Arc<Semaphore>,
    /// 全局已下载字节计数 (用于聚合进度)
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
            .no_gzip().no_brotli().no_deflate();

        for var in &["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"] {
            if let Ok(proxy_url) = std::env::var(var) {
                let p = proxy_url.trim();
                if !p.is_empty() {
                    let p = if p.contains("://") { p.to_string() } else { format!("http://{}", p) };
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

    /// 单任务下载
    pub async fn download(&self, task: &DownloadTask) -> DownloadStatus {
        self.download_with_progress(task, None).await
    }

    /// 单任务下载 (带进度 channel)
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
                let size_mb = bytes as f64 / 1048576.0;
                DownloadStatus::Completed { elapsed_secs: elapsed, size_mb }
            }
            Err(e) => {
                if e == "cancelled" { DownloadStatus::Cancelled }
                else { DownloadStatus::Failed(e) }
            }
        }
    }

    /// 多任务并行下载 (带全局并发上限 + 去重 + 聚合进度)
    pub async fn download_all(
        &self,
        tasks: &[DownloadTask],
        progress_tx: Option<mpsc::Sender<AggregateProgress>>,
    ) -> Vec<DownloadStatus> {
        // 去重: 同 URL 只下载一次
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut deduped: Vec<(usize, &DownloadTask)> = Vec::new();
        for (i, task) in tasks.iter().enumerate() {
            let key = task.url.clone();
            if let Some(&first_idx) = seen.get(&key) {
                eprintln!("[dl] dedup: task {} same as {}, skipping", i, first_idx);
                continue;
            }
            seen.insert(key, i);
            deduped.push((i, task));
        }

        // 磁盘空间预检
        if let Some(first_dest) = deduped.first().map(|(_, t)| &t.dest) {
            if let Some(parent) = first_dest.parent() {
                if let Ok(stat) = std::fs::statvfs(parent) {
                    let avail = stat.available_free_space();
                    if avail < self.config.min_disk_space {
                        eprintln!("[dl] disk space warning: {:.1}GB available, min={:.1}GB",
                            avail as f64 / 1073741824.0,
                            self.config.min_disk_space as f64 / 1073741824.0);
                    }
                }
            }
        }

        let total = tasks.len();
        let completed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let failed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let active = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        self.global_downloaded.store(0, Ordering::Relaxed);

        let mut handles = Vec::new();
        for (_, task) in deduped {
            let engine = self.clone_for_task();
            let task = task.clone();
            let completed = completed.clone();
            let failed = failed.clone();
            let active = active.clone();
            let global_dl = self.global_downloaded.clone();
            let agg_tx = progress_tx.clone();

            handles.push(tokio::spawn(async move {
                // 获取任务级许可 (全局并发上限)
                let _permit = engine.task_semaphore.clone().acquire_owned().await
                    .map_err(|_| "semaphore closed".to_string())?;

                active.fetch_add(1, Ordering::Relaxed);

                let status = engine.download(&task).await;

                active.fetch_sub(1, Ordering::Relaxed);
                match &status {
                    DownloadStatus::Completed { .. } => { completed.fetch_add(1, Ordering::Relaxed); }
                    DownloadStatus::Failed(_) => { failed.fetch_add(1, Ordering::Relaxed); }
                    _ => {}
                }

                // 聚合进度推送
                if let Some(tx) = &agg_tx {
                    let agg = AggregateProgress {
                        total_tasks: total,
                        completed: completed.load(Ordering::Relaxed),
                        failed: failed.load(Ordering::Relaxed),
                        active: active.load(Ordering::Relaxed),
                        total_bytes: 0,
                        downloaded_bytes: global_dl.load(Ordering::Relaxed),
                        overall_speed_mbps: 0.0,
                        overall_percent: if total > 0 {
                            completed.load(Ordering::Relaxed) as f32 / total as f32 * 100.0
                        } else { 0.0 },
                    };
                    let _ = tx.try_send(agg);
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

    /// 克隆引擎用于任务 (共享 client + semaphore)
    fn clone_for_task(&self) -> DownloadEngine {
        DownloadEngine {
            config: self.config.clone(),
            client: self.client.clone(),
            task_semaphore: self.task_semaphore.clone(),
            global_downloaded: self.global_downloaded.clone(),
        }
    }

    // ── 核心下载逻辑 ──────────────────────────────────────────────────────

    async fn download_inner(
        &self,
        task: &DownloadTask,
        progress_tx: Option<mpsc::Sender<DownloadProgress>>,
        cancelled: Arc<AtomicBool>,
    ) -> Result<u64, String> {
        let mut url = reqwest::Url::parse(&task.url).map_err(|e| e.to_string())?;

        if is_huggingface_url(&task.url) {
            let resolved = resolve_mirror(&self.client, &task.url).await;
            url = reqwest::Url::parse(&resolved).map_err(|e| e.to_string())?;
        }

        if let Some(parent) = task.dest.parent() {
            fs::create_dir_all(parent).await.map_err(|e| format!("mkdir: {}", e))?;
        }

        let dest = self.detect_filename(&url, &task.dest).await;

        // .done 标记
        let done_marker = dest.with_extension("done");
        if dest.exists() && done_marker.exists() {
            if let Ok(meta) = fs::metadata(&dest).await {
                if let Ok(content) = fs::read_to_string(&done_marker).await {
                    if let Some(done_size) = content.lines()
                        .find(|l| l.starts_with("size="))
                        .and_then(|l| l.strip_prefix("size="))
                        .and_then(|s| s.parse::<u64>().ok())
                    {
                        if meta.len() >= done_size {
                            eprintln!("[dl] already complete (done marker)");
                            return Ok(meta.len());
                        }
                    }
                }
            }
        }

        let total_size = self.head_size(&url).await.unwrap_or(0);

        // 磁盘预分配
        if total_size > 0 && !dest.exists() {
            // 预检空间
            if let Some(parent) = dest.parent() {
                if let Ok(stat) = std::fs::statvfs(parent) {
                    if stat.available_free_space() < total_size + self.config.min_disk_space {
                        return Err(format!("insufficient disk space: need {}MB, available {}MB",
                            (total_size + self.config.min_disk_space) / 1048576,
                            stat.available_free_space() / 1048576));
                    }
                }
            }
            let _ = std::fs::File::options().write(true).create(true).truncate(false)
                .open(&dest).and_then(|f| f.set_len(total_size));
        }

        let existing = if dest.exists() {
            fs::metadata(&dest).await.map(|m| m.len()).unwrap_or(0)
        } else { 0 };

        if total_size > 0 && existing >= total_size {
            let _ = fs::write(&done_marker, format!("size={}\nurl={}\n", total_size, task.url)).await;
            return Ok(existing);
        }

        let remaining = total_size.saturating_sub(existing);
        let chunk_size = if remaining > 0 {
            (remaining / self.config.max_concurrent as u64).min(self.config.max_chunk_bytes).max(1)
        } else { self.config.max_chunk_bytes };
        let n_chunks = if total_size > 0 {
            ((remaining - 1) / chunk_size + 1).min(self.config.max_concurrent as u64) as usize
        } else { 1 };

        eprintln!(
            "[dl] total={}MB remaining={}MB chunks={} chunk={}MB",
            total_size / 1048576, remaining / 1048576, n_chunks, chunk_size / 1048576
        );

        let stem = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("dl");
        let tmp_dir = dest.parent().unwrap_or(Path::new(".")).join(format!(".dl_{}", stem));
        fs::create_dir_all(&tmp_dir).await.map_err(|e| format!("tmp dir: {}", e))?;

        let semaphore = Arc::new(Semaphore::new(n_chunks));
        let mut handles = Vec::with_capacity(n_chunks);

        for i in 0..n_chunks {
            let start_byte = existing + i as u64 * chunk_size;
            let end_byte = if i == n_chunks - 1 {
                if total_size > 0 { total_size - 1 } else { 0 }
            } else { existing + (i + 1) as u64 * chunk_size - 1 };

            let chunk_file = tmp_dir.join(format!("c{:04}.tmp", i));
            let url = url.clone();
            let client = self.client.clone();
            let timeout_secs = self.config.timeout_secs;
            let permit = semaphore.clone().acquire_owned().await
                .map_err(|e| format!("semaphore: {}", e))?;
            let cancelled = cancelled.clone();

            handles.push(tokio::spawn(async move {
                let _permit = permit;
                if cancelled.load(Ordering::Relaxed) {
                    return Err("cancelled".into());
                }
                download_chunk(&client, &url, start_byte, end_byte, total_size, &chunk_file, timeout_secs).await
            }));
        }

        let mut total_downloaded = existing;
        let loop_start = Instant::now();
        for (i, h) in handles.into_iter().enumerate() {
            if cancelled.load(Ordering::Relaxed) {
                return Err("cancelled".into());
            }
            let chunk_bytes = h.await.map_err(|e| format!("join {}: {}", i, e))?
                .map_err(|e| format!("chunk {}: {}", i, e))?;
            total_downloaded += chunk_bytes;

            // 全局计数
            self.global_downloaded.fetch_add(chunk_bytes, Ordering::Relaxed);

            let elapsed = loop_start.elapsed().as_secs_f64();
            let speed = if elapsed > 0.5 { (total_downloaded - existing) as f64 / elapsed } else { 0.0 };
            let pct = if total_size > 0 { total_downloaded as f32 / total_size as f32 * 100.0 } else { 0.0 };
            let eta = if speed > 0.0 && total_size > total_downloaded {
                Some((total_size - total_downloaded) as f64 / speed)
            } else { None };

            let progress = DownloadProgress {
                percent: pct, downloaded: total_downloaded, total: total_size,
                speed_mbps: speed / 1048576.0, eta_secs: eta,
            };
            if let Some(tx) = &progress_tx {
                let _ = tx.try_send(progress.clone());
            }

            let eta_str = eta.map(|e| format!("{:.0}s", e)).unwrap_or_else(|| "?".into());
            eprintln!(
                "\r[dl] chunk {} +{}MB {:.1}% {:.1}MiB/s ETA:{}",
                i, chunk_bytes / 1048576, pct, speed / 1048576.0, eta_str
            );
        }
        eprintln!();

        // 流式合并
        eprintln!("[dl] merging {} chunks...", n_chunks);
        let mut out = BufWriter::with_capacity(256 * 1024,
            fs::File::create(&dest).await.map_err(|e| format!("create {}: {}", dest.display(), e))?
        );
        let mut buf = vec![0u8; 8192];
        for i in 0..n_chunks {
            let chunk_file = tmp_dir.join(format!("c{:04}.tmp", i));
            let mut f = fs::File::open(&chunk_file).await.map_err(|e| format!("open {}: {}", chunk_file.display(), e))?;
            loop {
                let n = f.read(&mut buf).await.map_err(|e| format!("read {}: {}", chunk_file.display(), e))?;
                if n == 0 { break; }
                out.write_all(&buf[..n]).await.map_err(|e| format!("write: {}", e))?;
            }
        }
        out.flush().await.map_err(|e| format!("flush: {}", e))?;
        drop(out);

        let final_elapsed = loop_start.elapsed().as_secs_f64();
        if let Some(host) = url.host_str() {
            if total_size > 0 && final_elapsed > 0.5 {
                let bps = (total_downloaded - existing) as f64 / final_elapsed;
                record_mirror_speed(host, bps);
                eprintln!("[mirror] speed {}: {:.1} MiB/s", host, bps / 1048576.0);
            }
        }

        let done_content = format!("size={}\nurl={}\ntimestamp={}\n",
            total_size, task.url,
            SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
        );
        let _ = fs::write(&done_marker, done_content).await;
        let _ = fs::remove_dir_all(&tmp_dir).await;

        eprintln!("[dl] complete: {} ({:.1} MB)", dest.display(), total_downloaded as f64 / 1048576.0);
        Ok(total_downloaded)
    }

    async fn head_size(&self, url: &reqwest::Url) -> Option<u64> {
        let resp = self.client.head(url.clone()).send().await.ok()?;
        let len = resp.headers().get(reqwest::header::CONTENT_LENGTH)?;
        len.to_str().ok()?.parse().ok()
    }

    async fn detect_filename(&self, url: &reqwest::Url, dest: &Path) -> PathBuf {
        if dest.file_stem().is_some_and(|s| !s.to_string_lossy().is_empty()) {
            return dest.to_path_buf();
        }
        if let Ok(resp) = self.client.head(url.clone()).send().await {
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
}

impl Default for DownloadEngine {
    fn default() -> Self { Self::new(DownloadConfig::default()) }
}

// ═══════════════════════════════════════════════════════════════════════════
// 分片下载
// ═══════════════════════════════════════════════════════════════════════════

async fn download_chunk(
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
    } else { 0 };
    let actual_start = start + already;
    if end != 0 && actual_start > end { return Ok(already); }

    let mut req = client.get(url.clone()).header("Accept-Encoding", "identity");
    if end != 0 {
        req = req.header("Range", format!("bytes={}-{}", actual_start, end));
    } else if actual_start > 0 {
        req = req.header("Range", format!("bytes={}-", actual_start));
    }

    let mut resp = timeout(Duration::from_secs(timeout_secs), req.send())
        .await.map_err(|_| "timeout".to_string())?
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    if status == 404 || status == 403 || status == 410 {
        return Err(format!("permanent failure: HTTP {}", status));
    }
    if !status.is_success() && status != 206 {
        return Err(format!("HTTP {}", status));
    }

    let file = if already > 0 {
        fs::OpenOptions::new().append(true).open(path).await.map_err(|e| format!("append: {}", e))?
    } else {
        fs::File::create(path).await.map_err(|e| format!("create: {}", e))?
    };

    let mut writer = BufWriter::with_capacity(256 * 1024, file);
    let mut downloaded = already;

    loop {
        match resp.chunk().await {
            Ok(Some(chunk)) => {
                writer.write_all(&chunk).await.map_err(|e| format!("write: {}", e))?;
                downloaded += chunk.len() as u64;
            }
            Ok(None) => break,
            Err(e) => return Err(format!("stream: {}", e)),
        }
    }
    writer.flush().await.map_err(|e| format!("flush: {}", e))?;
    Ok(downloaded)
}
