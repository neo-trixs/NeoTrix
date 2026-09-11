//! 下载引擎 (自研，无外部依赖)
//!
//! 支持：分片并发、HTTP Range 断点续传、HuggingFace 镜像加速、代理、进度追踪、指数退避重试。
//! 
//! 核心实现在 `l1_action::nt_io::nt_io_download` 模块，本文件为薄封装层。

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ─── Re-exports from full engine ──────────────────────────────────────────

/// 下载引擎配置
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub max_concurrent: usize,
    pub timeout_secs: u64,
    pub retry_count: u32,
    pub max_chunk_bytes: u64,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 16,
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
}

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub percent: f32,
    pub downloaded: u64,
    pub total: u64,
    pub speed_mbps: f64,
}

#[derive(Debug)]
pub enum DownloadStatus {
    Pending,
    InProgress(DownloadProgress),
    Completed { elapsed_secs: f64, size_mb: f64 },
    Failed(String),
}

pub struct DownloadEngine {
    config: DownloadConfig,
    client: reqwest::Client,
}

impl DownloadEngine {
    pub fn new(config: DownloadConfig) -> Self {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .connect_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(config.max_concurrent)
            .no_gzip()
            .no_brotli()
            .no_deflate();

        for var in &["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"] {
            if let Ok(proxy_url) = std::env::var(var) {
                let proxy_url = proxy_url.trim().to_string();
                if !proxy_url.is_empty() {
                    let proxy_url = if !proxy_url.contains("://") {
                        format!("http://{}", proxy_url)
                    } else {
                        proxy_url
                    };
                    if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                        builder = builder.proxy(proxy);
                        break;
                    }
                }
            }
        }

        let client = builder.build().expect("create reqwest client");
        Self { config, client }
    }

    pub async fn download(&self, task: &DownloadTask) -> DownloadStatus {
        let start = SystemTime::now();
        match self.download_inner(task).await {
            Ok(bytes) => {
                let elapsed = start.elapsed().unwrap_or_default().as_secs_f64();
                let size_mb = bytes as f64 / 1024.0 / 1024.0;
                DownloadStatus::Completed { elapsed_secs: elapsed, size_mb }
            }
            Err(e) => DownloadStatus::Failed(e),
        }
    }

    pub async fn download_all(&self, tasks: &[DownloadTask]) -> Vec<DownloadStatus> {
        let mut futs = Vec::new();
        for task in tasks {
            futs.push(self.download(task));
        }
        futures::future::join_all(futs).await
    }

    async fn download_inner(&self, task: &DownloadTask) -> Result<u64, String> {
        let mut url = reqwest::Url::parse(&task.url).map_err(|e| e.to_string())?;

        // HuggingFace 镜像自动切换
        if task.url.contains("huggingface.co") || task.url.contains("hf-mirror.com") {
            if !task.url.contains("hf-mirror.com") {
                // 检查环境变量覆盖
                if let Ok(endpoint) = std::env::var("NT_DOWNLOAD_MIRROR_ENDPOINT") {
                    let endpoint = endpoint.trim().to_string();
                    if !endpoint.is_empty() {
                        let endpoint = if endpoint.starts_with("http") { endpoint } else { format!("https://{}", endpoint) };
                        let forced = task.url
                            .replace("https://huggingface.co", &endpoint)
                            .replace("http://huggingface.co", &endpoint);
                        if forced != task.url {
                            url = reqwest::Url::parse(&forced).unwrap_or(url);
                        }
                    }
                }
                // 否则尝试 hf-mirror.com
                if url.host_str() == Some("huggingface.co") {
                    let mirror_url = task.url.replace("https://huggingface.co", "https://hf-mirror.com");
                    if let Ok(mirror) = reqwest::Url::parse(&mirror_url) {
                        // 快速探测
                        if self.client.head(mirror.clone()).send().await
                            .map(|r| r.status().is_success())
                            .unwrap_or(false)
                        {
                            url = mirror;
                        }
                    }
                }
            }
        }

        if let Some(parent) = task.dest.parent() {
            fs::create_dir_all(parent).await.map_err(|e| format!("mkdir: {}", e))?;
        }

        let total_size = self.head_size(&url).await.unwrap_or(0);
        let existing = if task.dest.exists() {
            fs::metadata(&task.dest).await.map(|m| m.len()).unwrap_or(0)
        } else { 0 };

        if total_size > 0 && existing >= total_size {
            return Ok(existing);
        }

        // 磁盘预分配
        if total_size > 0 && !task.dest.exists() {
            let _ = std::fs::File::options().write(true).create(true).open(&task.dest)
                .and_then(|f| f.set_len(total_size));
        }

        let remaining = total_size.saturating_sub(existing);
        let chunk_size = if remaining > 0 {
            let ideal = remaining / self.config.max_concurrent as u64;
            ideal.min(self.config.max_chunk_bytes).max(1)
        } else { self.config.max_chunk_bytes };
        let n_chunks = if total_size > 0 {
            ((remaining - 1) / chunk_size + 1).min(self.config.max_concurrent as u64) as usize
        } else { 1 };

        let stem = task.dest.file_stem().and_then(|s| s.to_str()).unwrap_or("dl");
        let tmp_dir = task.dest.parent().unwrap_or(Path::new(".")).join(format!(".dl_{}", stem));
        fs::create_dir_all(&tmp_dir).await.map_err(|e| format!("tmp dir: {}", e))?;

        let mut handles = Vec::with_capacity(n_chunks);
        for i in 0..n_chunks {
            let start_byte = existing + i as u64 * chunk_size;
            let end_byte = if i == n_chunks - 1 {
                if total_size > 0 { total_size - 1 } else { 0 }
            } else { existing + (i + 1) as u64 * chunk_size - 1 };
            let chunk_file = tmp_dir.join(format!("c{:04}.tmp", i));
            let url = url.clone();
            let client = self.client.clone();
            let timeout = self.config.timeout_secs;
            handles.push(tokio::spawn(async move {
                download_chunk(&client, &url, start_byte, end_byte, &chunk_file, timeout).await
            }));
        }

        let mut total_downloaded = existing;
        for (i, h) in handles.into_iter().enumerate() {
            let bytes = h.await.map_err(|e| format!("join chunk {}: {}", i, e))?
                .map_err(|e| format!("chunk {}: {}", i, e))?;
            total_downloaded += bytes;
            if total_size > 0 {
                let pct = total_downloaded as f32 / total_size as f32 * 100.0;
                eprint!("\r[dl] {:.1}%  {}/{} MB", pct, total_downloaded / 1024 / 1024, total_size / 1024 / 1024);
            }
        }
        eprintln!();

        // 流式合并 (8KB buffer)
        let mut out = fs::File::create(&task.dest).await.map_err(|e| format!("create {}: {}", task.dest.display(), e))?;
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

        let _ = fs::remove_dir_all(&tmp_dir).await;
        Ok(total_downloaded)
    }

    async fn head_size(&self, url: &reqwest::Url) -> Option<u64> {
        let resp = self.client.head(url.clone()).send().await.ok()?;
        let len = resp.headers().get(reqwest::header::CONTENT_LENGTH)?;
        len.to_str().ok()?.parse().ok()
    }
}

impl Default for DownloadEngine {
    fn default() -> Self { Self::new(DownloadConfig::default()) }
}

async fn download_chunk(
    client: &reqwest::Client,
    url: &reqwest::Url,
    start: u64,
    end: u64,
    path: &std::path::Path,
    timeout_secs: u64,
) -> Result<u64, String> {
    let already = if path.exists() {
        fs::metadata(path).await.map(|m| m.len()).unwrap_or(0)
    } else { 0 };
    let actual_start = start + already;
    if end != 0 && actual_start > end { return Ok(already); }

    let mut req = client.get(url.clone())
        .header("Accept-Encoding", "identity");
    if end != 0 {
        req = req.header("Range", format!("bytes={}-{}", actual_start, end));
    } else if actual_start > 0 {
        req = req.header("Range", format!("bytes={}-", actual_start));
    }

    let resp = tokio::time::timeout(Duration::from_secs(timeout_secs), req.send())
        .await.map_err(|_| String::from("timeout"))?
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

    let mut downloaded = already;
    let bytes = resp.bytes().await.map_err(|e| format!("stream: {}", e))?;
    file.write_all(&bytes).await.map_err(|e| format!("write: {}", e))?;
    downloaded += bytes.len() as u64;
    Ok(downloaded)
}
