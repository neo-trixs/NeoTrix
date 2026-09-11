/// 核心下载引擎
/// 实现：分片下载、HTTP Range 请求、进度追踪、重试机制、代理支持、文件写入

use crate::models::{DownloadSession, DownloadProgress, DownloadSource, DownloadMetadata, DownloadStatus};
use crate::traits::DownloadStrategy;
use reqwest::{
    Client, Method, Response, Url,
    header::{HeaderMap, Range},
    Proxy,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::sync::Semaphore;
use tokio::time::{interval, timeout};

/// 下载引擎配置
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// 分片数量 (并发下载的片数)
    pub chunk_count: usize,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试基延迟 (秒)
    pub retry_base_secs: u64,
    /// 最大超时秒数
    pub timeout_secs: u64,
    /// 是否启用断点续传
    pub enable_resume: bool,
    /// 是否启用代理
    pub enable_proxy: bool,
    /// 进度报告间隔 (秒)
    pub progress_interval_secs: u64,
    /// 单片最大字节数 (防止内存爆炸, 默认 64MB)
    pub max_chunk_bytes: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            chunk_count: 16,
            max_retries: 5,
            retry_base_secs: 2,
            timeout_secs: 600,
            enable_resume: true,
            enable_proxy: true,
            progress_interval_secs: 1,
            max_chunk_bytes: 64 * 1024 * 1024, // 64MB per chunk
        }
    }
}

/// 下载引擎 - 核心实现
pub struct DownloadEngine {
    config: EngineConfig,
    client: Client,
}

impl DownloadEngine {
    /// 创建新的下载引擎
    pub fn new(config: EngineConfig) -> Self {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .connect_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(config.chunk_count + 4)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_nodelay(true)
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent("NeoTrix/1.0 (download-engine)");

        // 关键: 禁止自动解压, 避免 HuggingFace CDN 流式下载 error decoding
        builder = builder.no_gzip().no_brotli().no_deflate();

        // 注入代理
        if config.enable_proxy {
            if let Some(proxy_config) = crate::proxy::from_env() {
                if proxy_config.is_valid() {
                    if let Ok(proxy) = proxy_config.to_reqwest() {
                        builder = builder.proxy(proxy);
                    }
                }
            }
        }

        let client = builder.build().expect("Failed to create reqwest client");
        DownloadEngine { config, client }
    }

    /// 开始下载会话 — 主入口
    pub async fn download_session(&self, session: &mut DownloadSession) -> Result<(), DownloadError> {
        session.status = DownloadStatus::Downloading;
        session.updated_at = SystemTime::now();
        session.retry_count = 0;

        let result = self.download_loop(session).await;

        match result {
            Ok(_) => {
                session.status = DownloadStatus::Completed;
                session.progress.percent = 100.0;
                session.updated_at = SystemTime::now();
                // 最终进度推送
                if let Some(tx) = &session.progress_tx {
                    let _ = tx.try_send(session.progress.clone());
                }
                Ok(())
            }
            Err(e) => {
                session.status = DownloadStatus::Failed;
                session.updated_at = SystemTime::now();
                Err(e)
            }
        }
    }

    /// 下载循环 — 带重试
    async fn download_loop(&self, session: &mut DownloadSession) -> Result<(), DownloadError> {
        let mut attempt = 0;

        while attempt < self.config.max_retries {
            attempt += 1;
            session.retry_count = attempt;

            match self.download_to_file(session).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if attempt < self.config.max_retries {
                        let base = self.config.retry_base_secs * 2u64.pow(attempt - 1);
                        let jitter = (std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() % (base.max(1) as u128)) as u64;
                        let backoff = base.saturating_add(jitter % (base / 4 + 1));
                        eprintln!("[download] attempt {} failed: {}. retry in {}s", attempt, e, backoff);
                        tokio::time::sleep(Duration::from_secs(backoff)).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Err(DownloadError::Network(format!(
            "max retries ({}) exceeded",
            self.config.max_retries
        )))
    }

    /// 核心：下载到文件
    async fn download_to_file(&self, session: &mut DownloadSession) -> Result<(), DownloadError> {
        let mut url = Url::parse(&session.url).map_err(|e| DownloadError::Network(e.to_string()))?;

        // HuggingFace 镜像自动切换
        if crate::mirror::is_huggingface_url(&session.url) {
            let resolved = crate::mirror::resolve_mirror_url(&self.client, &session.url).await;
            url = Url::parse(&resolved).map_err(|e| DownloadError::Network(e.to_string()))?;
        }

        // 确保目标目录存在
        if let Some(parent) = session.path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                DownloadError::Io(format!("create dir {}: {}", parent.display(), e))
            })?;
        }

        // 检查 .done 标记: 如果存在且大小匹配, 跳过下载
        let done_marker = session.path.with_extension("done");
        if session.path.exists() && done_marker.exists() {
            if let Ok(meta) = fs::metadata(&session.path).await {
                if let Ok(done_content) = fs::read_to_string(&done_marker).await {
                    if let Some(done_size) = done_content.lines()
                        .find(|l| l.starts_with("size="))
                        .and_then(|l| l.strip_prefix("size="))
                        .and_then(|s| s.parse::<u64>().ok())
                    {
                        if meta.len() >= done_size {
                            eprintln!("[download] already complete (done marker exists)");
                            session.progress.percent = 100.0;
                            session.progress.downloaded = meta.len();
                            session.progress.total = done_size;
                            return Ok(());
                        }
                    }
                }
            }
        }

        // HEAD 获取文件大小
        let total_size = self.get_total_size(&url).await.unwrap_or(0);
        if total_size > 0 {
            session.progress.total = total_size;
        }

        // 磁盘预分配: 防止下载中途 ENOSPC
        if total_size > 0 && !session.path.exists() {
            if let Some(parent) = session.path.parent() {
                let _ = fs::create_dir_all(parent).await;
            }
            let _ = std::fs::File::options()
                .write(true).create(true).truncate(false)
                .open(&session.path)
                .and_then(|f| f.set_len(total_size));
        }

        // 检查已下载的部分 (断点续传)
        let existing_bytes = if self.config.enable_resume && session.path.exists() {
            fs::metadata(&session.path).await.map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        if existing_bytes > 0 {
            session.progress.downloaded = existing_bytes;
            eprintln!("[download] resuming from {} bytes", existing_bytes);
        }

        // 如果已下载完整，跳过
        if total_size > 0 && existing_bytes >= total_size {
            session.progress.percent = 100.0;
            return Ok(());
        }

        // 计算分片
        let remaining = total_size.saturating_sub(existing_bytes);
        let chunk_size = if remaining > 0 {
            // 每片大小 = remaining / chunk_count，但不超过 max_chunk_bytes
            let ideal = remaining / self.config.chunk_count as u64;
            ideal.min(self.config.max_chunk_bytes).max(1)
        } else {
            self.config.max_chunk_bytes
        };

        // 实际并发数
        let actual_chunks = if total_size > 0 {
            ((remaining - 1) / chunk_size + 1).min(self.config.chunk_count as u64) as usize
        } else {
            1 // 未知大小时单片下载
        };

        eprintln!(
            "[download] total={}MB remaining={}MB chunks={} chunk_size={}MB",
            total_size / 1024 / 1024,
            remaining / 1024 / 1024,
            actual_chunks,
            chunk_size / 1024 / 1024,
        );

        // 临时文件目录
        let tmp_dir = session.path.with_extension("");
        let tmp_dir_name = tmp_dir.file_name().unwrap_or_default();
        let tmp_base = session.path.parent().unwrap_or(Path::new(".")).join(format!(".dl_{}", tmp_dir_name.to_string_lossy()));
        fs::create_dir_all(&tmp_base).await.map_err(|e| {
            DownloadError::Io(format!("create tmp dir: {}", e))
        })?;

        // Semaphore 并发控制
        let semaphore = Arc::new(Semaphore::new(actual_chunks));
        let mut handles = Vec::with_capacity(actual_chunks);

        for i in 0..actual_chunks {
            let start = existing_bytes + (i as u64) * chunk_size;
            let end = if i == actual_chunks - 1 {
                if total_size > 0 { total_size - 1 } else { 0 }
            } else {
                existing_bytes + ((i + 1) as u64) * chunk_size - 1
            };

            let chunk_file = tmp_base.join(format!("chunk_{:04}.tmp", i));
            let url = url.clone();
            let client = self.client.clone();
            let timeout_secs = self.config.timeout_secs;
            let permit = semaphore.clone().acquire_owned().await
                .map_err(|e| DownloadError::Network(format!("semaphore: {}", e)))?;

            let handle = tokio::spawn(async move {
                let _permit = permit; // hold until done
                download_chunk_to_file(&client, &url, start, end, total_size, &chunk_file, timeout_secs).await
            });
            handles.push((i, handle, chunk_file));
        }

        // 等待所有片完成, 实时追踪速度
        let mut total_downloaded = existing_bytes;
        let loop_start = SystemTime::now();
        for (i, handle, _chunk_file) in handles {
            match handle.await {
                Ok(Ok(chunk_bytes)) => {
                    total_downloaded += chunk_bytes;
                    let elapsed = loop_start.elapsed().unwrap_or_default().as_secs_f64();
                    if total_size > 0 {
                        session.progress.percent = (total_downloaded as f32 / total_size as f32) * 100.0;
                        session.progress.total = total_size;
                    }
                    session.progress.downloaded = total_downloaded;
                    if elapsed > 0.5 {
                        session.progress.speed = (total_downloaded - existing_bytes) as f64 / elapsed;
                    }
                    if session.progress.speed > 0.0 && total_size > total_downloaded {
                        session.progress.eta = Some((total_size - total_downloaded) as f64 / session.progress.speed);
                    }
                    session.updated_at = SystemTime::now();
                    let speed_mib = session.progress.speed / 1048576.0;
                    let eta_str = session.progress.eta.map(|e| format!("{:.0}s", e)).unwrap_or_else(|| "?".into());
                    eprintln!(
                        "[download] chunk {} done: +{}MB {:.1}% {:.1}MiB/s ETA:{}",
                        i, chunk_bytes / 1024 / 1024, session.progress.percent, speed_mib, eta_str
                    );
                    // 通过 channel 推送进度给 UI
                    if let Some(tx) = &session.progress_tx {
                        let _ = tx.try_send(session.progress.clone());
                    }
                }
                Ok(Err(e)) => {
                    return Err(DownloadError::Network(format!("chunk {} failed: {}", i, e)));
                }
                Err(e) => {
                    return Err(DownloadError::Network(format!("chunk {} task failed: {}", i, e)));
                }
            }
        }

        // 合并临时文件到目标文件 (流式 8KB buffer, 不加载整片到内存)
        eprintln!("[download] merging {} chunks...", actual_chunks);
        let mut output = fs::File::create(&session.path).await.map_err(|e| {
            DownloadError::Io(format!("create {}: {}", session.path.display(), e))
        })?;
        let mut buf = vec![0u8; 8192];
        for i in 0..actual_chunks {
            let chunk_file = tmp_base.join(format!("chunk_{:04}.tmp", i));
            let mut chunk = fs::File::open(&chunk_file).await.map_err(|e| {
                DownloadError::Io(format!("open chunk {}: {}", chunk_file.display(), e))
            })?;
            loop {
                let n = chunk.read(&mut buf).await.map_err(|e| {
                    DownloadError::Io(format!("read chunk {}: {}", chunk_file.display(), e))
                })?;
                if n == 0 { break; }
                output.write_all(&buf[..n]).await.map_err(|e| {
                    DownloadError::Io(format!("write output: {}", e))
                })?;
            }
        }
        output.flush().await.map_err(|e| DownloadError::Io(format!("flush: {}", e)))?;
        drop(output);

        // 记录镜像速度画像 (adaptive URI 选择)
        let final_elapsed = loop_start.elapsed().unwrap_or_default().as_secs_f64();
        if let Some(host) = url.host_str() {
            if total_size > 0 && final_elapsed > 0.5 {
                let bps = (total_downloaded - existing_bytes) as f64 / final_elapsed;
                crate::mirror::record_mirror_speed(host, bps);
                eprintln!("[mirror] recorded speed for {}: {:.1} MiB/s", host, bps / 1048576.0);
            }
        }

        // 原子完成标记: 写 .done 文件防止半截文件被误用
        let done_marker = session.path.with_extension("done");
        let done_content = format!(
            "size={}\nurl={}\nid={}\ntimestamp={}\n",
            total_size, session.url, session.id,
            chrono_simple_now()
        );
        let _ = fs::write(&done_marker, done_content).await;

        // 清理临时文件
        let _ = fs::remove_dir_all(&tmp_base).await;

        eprintln!("[download] complete: {}", session.path.display());
        Ok(())
    }

    /// HEAD 请求获取文件大小
    async fn get_total_size(&self, url: &Url) -> Result<u64, DownloadError> {
        let response = timeout(
            Duration::from_secs(30),
            self.client.head(url.clone()).send(),
        ).await
        .map_err(|_| DownloadError::Network("HEAD timed out".into()))?;

        let response = response.map_err(|e| DownloadError::Network(e.to_string()))?;

        if let Some(len) = response.headers().get(reqwest::header::CONTENT_LENGTH) {
            let s = std::str::from_utf8(len.as_bytes()).map_err(|_| DownloadError::Network("bad Content-Length".into()))?;
            return s.parse().map_err(|_| DownloadError::Network("parse Content-Length failed".into()));
        }

        Ok(0)
    }

    /// 从 Content-Disposition header 提取文件名
    async fn detect_filename(&self, url: &Url, dest: &Path) -> PathBuf {
        if dest.file_name().is_some() && dest.file_stem().is_some_and(|s| !s.to_string_lossy().is_empty()) {
            return dest.to_path_buf();
        }
        // 尝试从 HEAD 的 Content-Disposition 获取
        if let Ok(resp) = self.client.head(url.clone()).send().await {
            if let Some(cd) = resp.headers().get("content-disposition") {
                if let Ok(cd_str) = cd.to_str() {
                    // filename*=UTF-8''encoded_name
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
        // fallback: 从 URL path 取最后一段
        if let Some(name) = url.path().rsplit('/').next() {
            if !name.is_empty() {
                return dest.with_file_name(name);
            }
        }
        dest.to_path_buf()
    }
}

/// 下载单个分片到临时文件
async fn download_chunk_to_file(
    client: &Client,
    url: &Url,
    start: u64,
    end: u64,
    total_size: u64,
    chunk_path: &Path,
    timeout_secs: u64,
) -> Result<u64, DownloadError> {
    // 如果已有部分数据且 total_size 已知，跳过已下载的
    let already = if chunk_path.exists() {
        fs::metadata(chunk_path).await.map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let actual_start = start + already;

    // 如果 end != 0 且 actual_start > end，这片已完成
    if end != 0 && actual_start > end {
        return Ok(already);
    }

    let mut req = client.request(Method::GET, url.clone());
    req = req.header("Accept-Encoding", "identity");

    // Range header
    if end != 0 {
        req = req.header("Range", format!("bytes={}-{}", actual_start, end));
    } else if actual_start > 0 {
        req = req.header("Range", format!("bytes={}-", actual_start));
    }

    let response = timeout(
        Duration::from_secs(timeout_secs),
        req.send(),
    ).await
    .map_err(|_| DownloadError::Network("request timed out".into()))?;

    let response = response.map_err(|e| DownloadError::Network(e.to_string()))?;

    let status = response.status();
    // 404/403/410 是永久性失败，不重试
    if status == 404 || status == 403 || status == 410 {
        return Err(DownloadError::Network(format!("permanent failure: HTTP {}", status)));
    }
    if !(status == 200 || status == 206) {
        return Err(DownloadError::Network(format!("HTTP {}", status)));
    }

    // 流式写入文件
    let mut file = if already > 0 {
        fs::OpenOptions::new().append(true).open(chunk_path).await.map_err(|e| {
            DownloadError::Io(format!("open chunk for append: {}", e))
        })?
    } else {
        fs::File::create(chunk_path).await.map_err(|e| {
            DownloadError::Io(format!("create chunk: {}", e))
        })?
    };

    let mut stream = response.bytes_stream();
    let mut downloaded = already;
    use futures_util::StreamExt;
    use tokio::io::BufWriter;

    let mut writer = BufWriter::with_capacity(256 * 1024, file); // 256KB buffer

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| DownloadError::Network(format!("stream error: {}", e)))?;
        writer.write_all(&chunk).await.map_err(|e| DownloadError::Io(format!("write chunk: {}", e)))?;
        downloaded += chunk.len() as u64;
    }

    writer.flush().await.map_err(|e| DownloadError::Io(format!("flush: {}", e)))?;

    Ok(downloaded)
}

/// 简单时间戳 (避免引入 chrono crate)
fn chrono_simple_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", now)
}

/// 下载错误类型
#[derive(Debug)]
pub enum DownloadError {
    Network(String),
    Io(String),
    Cancelled,
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::Network(msg) => write!(f, "network: {}", msg),
            DownloadError::Io(msg) => write!(f, "io: {}", msg),
            DownloadError::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::error::Error for DownloadError {}
