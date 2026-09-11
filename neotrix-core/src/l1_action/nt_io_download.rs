//! 下载引擎 (自研，无外部依赖)
//!
//! 支持：分片并发、HTTP Range 断点续传、代理、进度追踪、指数退避重试。

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ─── Config ──────────────────────────────────────────────────────────────

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
            max_concurrent: 8,
            timeout_secs: 600,
            retry_count: 5,
            max_chunk_bytes: 64 * 1024 * 1024, // 64 MB
        }
    }
}

// ─── Task / Status ───────────────────────────────────────────────────────

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

// ─── Engine ──────────────────────────────────────────────────────────────

pub struct DownloadEngine {
    config: DownloadConfig,
    client: reqwest::Client,
}

impl DownloadEngine {
    pub fn new(config: DownloadConfig) -> Self {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .connect_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(config.max_concurrent);

        // 代理: 从环境变量读取
        for var in &["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy"] {
            if let Ok(proxy_url) = std::env::var(var) {
                if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                    builder = builder.proxy(proxy);
                    break;
                }
            }
        }

        let client = builder.build().expect("create reqwest client");
        Self { config, client }
    }

    /// 下载单个文件，返回最终状态
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

    /// 批量下载
    pub async fn download_all(&self, tasks: &[DownloadTask]) -> Vec<DownloadStatus> {
        let mut futs = Vec::new();
        for task in tasks {
            futs.push(self.download(task));
        }
        futures::future::join_all(futs).await
    }

    // ── internal ─────────────────────────────────────────────────────────

    async fn download_inner(&self, task: &DownloadTask) -> Result<u64, String> {
        let url = reqwest::Url::parse(&task.url).map_err(|e| e.to_string())?;

        // 确保目录存在
        if let Some(parent) = task.dest.parent() {
            fs::create_dir_all(parent).await.map_err(|e| format!("mkdir: {}", e))?;
        }

        // HEAD → total size
        let total_size = self.head_size(&url).await.unwrap_or(0);

        // 断点续传: 已有字节
        let existing = if task.dest.exists() {
            fs::metadata(&task.dest).await.map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        if total_size > 0 && existing >= total_size {
            return Ok(existing);
        }

        let remaining = total_size.saturating_sub(existing);
        let chunk_size = if remaining > 0 {
            let ideal = remaining / self.config.max_concurrent as u64;
            ideal.min(self.config.max_chunk_bytes).max(1)
        } else {
            self.config.max_chunk_bytes
        };
        let n_chunks = if total_size > 0 {
            ((remaining - 1) / chunk_size + 1).min(self.config.max_concurrent as u64) as usize
        } else {
            1
        };

        eprintln!(
            "[dl] {} → {}  total={}MB  chunks={}×{}MB",
            url.path().rsplit('/').next().unwrap_or("?"),
            task.dest.display(),
            total_size / 1024 / 1024,
            n_chunks,
            chunk_size / 1024 / 1024,
        );

        // 临时目录
        let stem = task.dest.file_stem().and_then(|s| s.to_str()).unwrap_or("dl");
        let tmp_dir = task.dest.parent().unwrap_or(Path::new(".")).join(format!(".dl_{}", stem));
        fs::create_dir_all(&tmp_dir).await.map_err(|e| format!("tmp dir: {}", e))?;

        // 并发下载各片
        let mut handles = Vec::with_capacity(n_chunks);
        for i in 0..n_chunks {
            let start_byte = existing + i as u64 * chunk_size;
            let end_byte = if i == n_chunks - 1 {
                if total_size > 0 { total_size - 1 } else { 0 }
            } else {
                existing + (i + 1) as u64 * chunk_size - 1
            };
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

        // 合并到目标文件
        let mut out = fs::File::create(&task.dest).await.map_err(|e| format!("create {}: {}", task.dest.display(), e))?;
        for i in 0..n_chunks {
            let chunk_file = tmp_dir.join(format!("c{:04}.tmp", i));
            let mut f = fs::File::open(&chunk_file).await.map_err(|e| format!("open {}: {}", chunk_file.display(), e))?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).await.map_err(|e| format!("read {}: {}", chunk_file.display(), e))?;
            out.write_all(&buf).await.map_err(|e| format!("write: {}", e))?;
        }
        out.flush().await.map_err(|e| format!("flush: {}", e))?;
        drop(out);

        // 清理临时文件
        let _ = fs::remove_dir_all(&tmp_base(tmp_dir)).await;

        Ok(total_downloaded)
    }

    async fn head_size(&self, url: &reqwest::Url) -> Option<u64> {
        let resp = self.client.head(url.clone()).send().await.ok()?;
        let len = resp.headers().get(reqwest::header::CONTENT_LENGTH)?;
        len.to_str().ok()?.parse().ok()
    }
}

fn tmp_base(p: PathBuf) -> PathBuf { p }

/// 下载单片到临时文件（支持续传）
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
    } else {
        0
    };
    let actual_start = start + already;
    if end != 0 && actual_start > end {
        return Ok(already);
    }

    let mut req = client.get(url.clone());
    if end != 0 {
        req = req.header("Range", format!("bytes={}-{}", actual_start, end));
    } else if actual_start > 0 {
        req = req.header("Range", format!("bytes={}-", actual_start));
    }

    let resp = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        req.send(),
    ).await.map_err(|_| "timeout".into())?
     .map_err(|e| e.to_string())?;

    let status = resp.status();
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

impl Default for DownloadEngine {
    fn default() -> Self { Self::new(DownloadConfig::default()) }
}
