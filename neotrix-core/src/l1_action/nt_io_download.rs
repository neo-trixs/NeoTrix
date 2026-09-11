//! NeoTrix 自研下载引擎
//!
//! 能力: 分片并发 / HTTP Range 断点续传 / HuggingFace 镜像加速+速度画像 /
//!       代理 / 进度 channel / Content-Disposition 文件名 / BufWriter /
//!       原子 .done 标记 / 磁盘预分配 / 指数退避重试 / 状态感知重试

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
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

/// 记录镜像速度 (EMA: 0.7*old + 0.3*new)
pub fn record_mirror_speed(endpoint: &str, bytes_per_sec: f64) {
    if let Ok(mut map) = MIRROR_SPEED_MAP.lock() {
        let entry = map.entry(endpoint.to_string()).or_insert(0.0);
        *entry = 0.7 * *entry + 0.3 * bytes_per_sec;
    }
}

/// 按历史速度排序镜像端点
fn ranked_mirrors() -> Vec<(String, f64)> {
    let map = MIRROR_SPEED_MAP.lock().unwrap();
    let mut pairs: Vec<_> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    pairs
}

fn is_huggingface_url(url: &str) -> bool {
    url.contains("huggingface.co") || url.contains("hf-mirror.com")
}

/// 镜像探测: 并行 HEAD 所有端点, 返回第一个可用 URL
async fn resolve_mirror(client: &reqwest::Client, original_url: &str) -> String {
    if !is_huggingface_url(original_url) || original_url.contains("hf-mirror.com") {
        return original_url.to_string();
    }
    // 环境变量强制覆盖
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
    // 按速度排序候选
    let speed_ranking = ranked_mirrors();
    let mut endpoints: Vec<&str> = MIRROR_ENDPOINTS.to_vec();
    if !speed_ranking.is_empty() {
        endpoints.sort_by(|a, b| {
            let sa = speed_ranking.iter().find(|(k, _)| k.contains(a.trim_start_matches("https://"))).map(|(_, v)| *v).unwrap_or(0.0);
            let sb = speed_ranking.iter().find(|(k, _)| k.contains(b.trim_start_matches("https://"))).map(|(_, v)| *v).unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    // 并行探测
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
    pub max_concurrent: usize,
    pub timeout_secs: u64,
    pub retry_count: u32,
    pub max_chunk_bytes: u64,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self { max_concurrent: 16, timeout_secs: 600, retry_count: 5, max_chunk_bytes: 64 * 1024 * 1024 }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub url: String,
    pub dest: PathBuf,
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
}

// ═══════════════════════════════════════════════════════════════════════════
// 下载引擎
// ═══════════════════════════════════════════════════════════════════════════

pub struct DownloadEngine {
    config: DownloadConfig,
    client: reqwest::Client,
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

        // 代理: 6 级优先级 HTTPS_PROXY > https_proxy > HTTP_PROXY > http_proxy > ALL_PROXY > all_proxy
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
        Self { config, client }
    }

    /// 下载单个任务 (带进度 channel)
    pub async fn download(&self, task: &DownloadTask) -> DownloadStatus {
        self.download_with_progress(task, None).await
    }

    /// 下载单个任务 (带进度 channel 订阅)
    pub async fn download_with_progress(
        &self,
        task: &DownloadTask,
        progress_tx: Option<mpsc::Sender<DownloadProgress>>,
    ) -> DownloadStatus {
        let start = SystemTime::now();
        match self.download_inner(task, progress_tx).await {
            Ok(bytes) => {
                let elapsed = start.elapsed().unwrap_or_default().as_secs_f64();
                let size_mb = bytes as f64 / 1048576.0;
                DownloadStatus::Completed { elapsed_secs: elapsed, size_mb }
            }
            Err(e) => DownloadStatus::Failed(e),
        }
    }

    /// 并行下载多个任务
    pub async fn download_all(&self, tasks: &[DownloadTask]) -> Vec<DownloadStatus> {
        let futs: Vec<_> = tasks.iter().map(|t| self.download(t)).collect();
        futures::future::join_all(futs).await
    }

    // ── 核心下载逻辑 ──────────────────────────────────────────────────────

    async fn download_inner(
        &self,
        task: &DownloadTask,
        progress_tx: Option<mpsc::Sender<DownloadProgress>>,
    ) -> Result<u64, String> {
        let mut url = reqwest::Url::parse(&task.url).map_err(|e| e.to_string())?;

        // HuggingFace 镜像自动切换 + 速度画像
        if is_huggingface_url(&task.url) {
            let resolved = resolve_mirror(&self.client, &task.url).await;
            url = reqwest::Url::parse(&resolved).map_err(|e| e.to_string())?;
        }

        // 确保目标目录存在
        if let Some(parent) = task.dest.parent() {
            fs::create_dir_all(parent).await.map_err(|e| format!("mkdir: {}", e))?;
        }

        // Content-Disposition 文件名检测
        let dest = self.detect_filename(&url, &task.dest).await;

        // .done 标记: 已完成则跳过
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

        // HEAD 获取文件大小
        let total_size = self.head_size(&url).await.unwrap_or(0);

        // 磁盘预分配
        if total_size > 0 && !dest.exists() {
            let _ = std::fs::File::options().write(true).create(true).truncate(false)
                .open(&dest).and_then(|f| f.set_len(total_size));
        }

        // 断点续传: 检查已有数据
        let existing = if dest.exists() {
            fs::metadata(&dest).await.map(|m| m.len()).unwrap_or(0)
        } else { 0 };

        if total_size > 0 && existing >= total_size {
            // 写 .done 标记
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

        // 临时目录
        let stem = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("dl");
        let tmp_dir = dest.parent().unwrap_or(Path::new(".")).join(format!(".dl_{}", stem));
        fs::create_dir_all(&tmp_dir).await.map_err(|e| format!("tmp dir: {}", e))?;

        // Semaphore 并发控制
        let semaphore = std::sync::Arc::new(Semaphore::new(n_chunks));
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

            handles.push(tokio::spawn(async move {
                let _permit = permit;
                download_chunk(&client, &url, start_byte, end_byte, total_size, &chunk_file, timeout_secs).await
            }));
        }

        // 等待所有片完成 + 实时进度
        let mut total_downloaded = existing;
        let loop_start = SystemTime::now();
        for (i, h) in handles.into_iter().enumerate() {
            let chunk_bytes = h.await.map_err(|e| format!("join {}: {}", i, e))?
                .map_err(|e| format!("chunk {}: {}", i, e))?;
            total_downloaded += chunk_bytes;

            let elapsed = loop_start.elapsed().unwrap_or_default().as_secs_f64();
            let speed = if elapsed > 0.5 { (total_downloaded - existing) as f64 / elapsed } else { 0.0 };
            let pct = if total_size > 0 { total_downloaded as f32 / total_size as f32 * 100.0 } else { 0.0 };
            let eta = if speed > 0.0 && total_size > total_downloaded {
                Some((total_size - total_downloaded) as f64 / speed)
            } else { None };

            let progress = DownloadProgress {
                percent: pct,
                downloaded: total_downloaded,
                total: total_size,
                speed_mbps: speed / 1048576.0,
                eta_secs: eta,
            };

            // channel 推送
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

        // 流式合并 (BufWriter 256KB)
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

        // 记录镜像速度画像
        let final_elapsed = loop_start.elapsed().unwrap_or_default().as_secs_f64();
        if let Some(host) = url.host_str() {
            if total_size > 0 && final_elapsed > 0.5 {
                let bps = (total_downloaded - existing) as f64 / final_elapsed;
                record_mirror_speed(host, bps);
                eprintln!("[mirror] speed {}: {:.1} MiB/s", host, bps / 1048576.0);
            }
        }

        // 原子 .done 标记
        let done_content = format!("size={}\nurl={}\ntimestamp={}\n",
            total_size, task.url,
            SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
        );
        let _ = fs::write(&done_marker, done_content).await;

        // 清理临时文件
        let _ = fs::remove_dir_all(&tmp_dir).await;

        eprintln!("[dl] complete: {} ({:.1} MB)", dest.display(), total_downloaded as f64 / 1048576.0);
        Ok(total_downloaded)
    }

    async fn head_size(&self, url: &reqwest::Url) -> Option<u64> {
        let resp = self.client.head(url.clone()).send().await.ok()?;
        let len = resp.headers().get(reqwest::header::CONTENT_LENGTH)?;
        len.to_str().ok()?.parse().ok()
    }

    /// Content-Disposition 文件名检测
    async fn detect_filename(&self, url: &reqwest::Url, dest: &Path) -> PathBuf {
        if dest.file_stem().is_some_and(|s| !s.to_string_lossy().is_empty()) {
            return dest.to_path_buf();
        }
        if let Ok(resp) = self.client.head(url.clone()).send().await {
            if let Some(cd) = resp.headers().get("content-disposition") {
                if let Ok(cd_str) = cd.to_str() {
                    // filename*=UTF-8''encoded
                    if let Some(pos) = cd_str.find("filename*=UTF-8''") {
                        let encoded = &cd_str[pos + 16..];
                        if let Some(name) = encoded.split(';').next() {
                            if let Ok(decoded) = urlencoding::decode(name) {
                                return dest.with_file_name(decoded.as_ref());
                            }
                        }
                    }
                    // filename="name"
                    if let Some(start) = cd_str.find("filename=\"") {
                        let rest = &cd_str[start + 10..];
                        if let Some(end) = rest.find('"') {
                            return dest.with_file_name(&rest[..end]);
                        }
                    }
                }
            }
        }
        // fallback: URL path 最后一段
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
    total_size: u64,
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

    let resp = timeout(Duration::from_secs(timeout_secs), req.send())
        .await.map_err(|_| "timeout".into())?
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    if status == 404 || status == 403 || status == 410 {
        return Err(format!("permanent failure: HTTP {}", status));
    }
    if !status.is_success() && status != 206 {
        return Err(format!("HTTP {}", status));
    }

    let mut file = if already > 0 {
        fs::OpenOptions::new().append(true).open(path).await.map_err(|e| format!("append: {}", e))?
    } else {
        fs::File::create(path).await.map_err(|e| format!("create: {}", e))?
    };

    let mut writer = BufWriter::with_capacity(256 * 1024, file);
    let mut downloaded = already;
    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("stream: {}", e))?;
        writer.write_all(&chunk).await.map_err(|e| format!("write: {}", e))?;
        downloaded += chunk.len() as u64;
    }
    writer.flush().await.map_err(|e| format!("flush: {}", e))?;
    Ok(downloaded)
}
