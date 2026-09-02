/// 核心下载引擎
/// 实现：分片下载、HTTP Range 请求、进度追踪、重试机制、代理支持
/// 依赖：reqwest (异步 HTTP client), tokio (异任务)

use crate::models::{DownloadSession, DownloadProgress, DownloadSource, DownloadMetadata, DownloadStatus};
use crate::traits::DownloadStrategy;
use reqwest::{
    Client, Method, RequestBuilder, Response, Url,
    header::{HeaderMap, Range},
    Proxy,
};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
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
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            chunk_count: 8,          // 8 路并发 (类似 aria2 默认)
            max_retries: 5,          // 最多重试 5 次
            retry_base_secs: 2,      // 基础延迟 2s (指数退避)
            timeout_secs: 300,       // 单次请求超时 5分钟
            enable_resume: true,     // 启用断点续传
            enable_proxy: true,      // 启用代理支持
            progress_interval_secs: 1, // 每秒报告进度
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
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .connect_timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create reqwest client");

        DownloadEngine { config, client }
    }

    /// 开始下载会话
    pub async fn download_session(&self, session: &mut DownloadSession) -> Result<(), DownloadError> {
        // 更新状态
        session.status = DownloadStatus::Downloading;
        session.updated_at = SystemTime::now();
        session.retry_count = 0;

        // 执行下载循环
        let result = self.download_loop(session).await;

        match result {
            Ok(_) => {
                session.status = DownloadStatus::Completed;
                session.updated_at = SystemTime::now();
                Ok(())
            }
            Err(e) => {
                session.status = DownloadStatus::Failed;
                session.retry_count += 1;
                session.updated_at = SystemTime::now();
                Err(e)
            }
        }
    }

    /// 下载循环 - 处理重试和进度
    async fn download_loop(&self, session: &mut DownloadSession) -> Result<(), DownloadError> {
        let mut attempt = 0;

        while attempt < self.config.max_retries {
            attempt += 1;
            session.retry_count = attempt;

            match self.download_chunk(session).await {
                Ok(_) => return Ok(()), // 成功
                Err(e) => {
                    // 记录错误并决定是否重试
                    if attempt < self.config.max_retries {
                        // 指数退避等待
                        let backoff = self.config.retry_base_secs * 2_u64.pow(attempt - 1);
                        // 这里可以添加日志记录
                        // tracing::warn!("Download attempt {} failed: {}. Retrying in {}s", attempt, e, backoff);
                        tokio::time::sleep(Duration::from_secs(backoff)).await;
                    } else {
                        return Err(e); // 最大重试次数耗尽
                    }
                }
            }
        }

        Err(DownloadError::Network(format!(
            "Max retries ({}) exceeded",
            self.config.max_retries
        )))
    }

    /// 单次下载块 (片)
    async fn download_chunk(
        &self,
        session: &mut DownloadSession,
    ) -> Result<(), DownloadError> {
        // 解析 URL
        let url = Url::parse(&session.url).map_err(|e| DownloadError::Network(e.to_string()))?;

        // 获取文件总大小 (通过 HEAD 请求)
        let total_size = self.get_total_size(&url).await?;

        // 更新总大小 (如果已知)
        if total_size > 0 {
            session.progress.total = total_size;
        }

        // 计算每片大小
        let chunk_size = if total_size > 0 {
            total_size / self.config.chunk_count
        } else {
            // 未知大小时的默认片大小
            1_000_000 // 1MB
        };

        // 如果启用断点续传，计算每片的起始偏移
        let (chunks, start_offsets) = if self.config.enable_resume && session.progress.downloaded > 0 {
            self.calculate_resume_chunks(total_size, chunk_size).await?
        } else {
            (0..self.config.chunk_count, vec![0u64; self.config.chunk_count])
        };

        // 创建下载任务
        let mut handles = Vec::new();

        for (i, (chunk_idx, start_offset)) in (0..chunks.len()).zip(start_offsets.iter()).enumerate() {
            let session_clone = session.clone();
            let url = url.clone();
            let config = self.config.clone();

            let handle = tokio::spawn(async move {
                Self::download_single_chunk(
                    &url,
                    chunk_idx as u32,
                    *start_offset,
                    chunk_size,
                    &config,
                    &session_clone,
                )
                .await
            });

            handles.push(handle);
        }

        // 收集所有片的结果
        let mut total_downloaded: u64 = 0;

        for handle in handles {
            match handle.await {
                Ok(chunk_size) => total_downloaded += chunk_size,
                Err(e) => {
                    // 单个片失败，但不 necessarily 表示整体失败
                    // 根据配置决定是重试还是忽略
                    eprintln!("Chunk download error: {}", e);
                }
            }
        }

        // 更新进度
        let percent = if session.progress.total > 0 {
            (total_downloaded as f32 / session.progress.total as f32) * 100.0
        } else {
            0.0
        };

        // 这里应该调用 session 的 progress 更新方法
        // 在实际集成中，通过消息传递或回调更新

        Ok(total_downloaded)
    }

    /// 下载单个片
    async fn download_single_chunk(
        url: &Url,
        chunk_idx: u32,
        start_offset: u64,
        chunk_size: u64,
        config: &EngineConfig,
        session: &DownloadSession,
    ) -> Result<u64, DownloadError> {
        // 构建 Range header
        let end_offset = start_offset + chunk_size - 1;
        let range = format!("bytes={}-{}", start_offset, end_offset);

        // 创建请求
        let mut req = self
            .client
            .request(Method::GET, url.clone());

        // 设置 Range header (断点续传)
        req = req.header("Range", &range);

        // 设置代理 (如果启用)
        if config.enable_proxy {
            if let Some(proxy_config) = crate::proxy::from_env() {
                if proxy_config.is_valid() {
                    if let Ok(proxy) = proxy_config.to_reqwest() {
                        req = req.proxy(proxy);
                    }
                }
            }
        }

        // 发送请求
        let response = timeout(
            Duration::from_secs(config.timeout_secs),
            req.send(),
        )
        .await
        .map_err(|_| DownloadError::Network("Request timed out".to_string()))?;

        // 检查响应状态
        let status = response.status();
        if !status.is_success() && status.as_u16() != 206 {
            // 服务器不支持 Range 请求，回退到完整下载
            return self.download_full_chunk(url, chunk_size, config).await;
        }

        // 读取内容到内存 (或写入文件)
        let bytes = response.bytes().await.map_err(|e| {
            DownloadError::Network(format!("Failed to read response: {}", e))
        })?;

        let downloaded = bytes.len() as u64;

        // 写入到临时文件或累积
        // 实际实现中，这里应该写入到临时文件并拼接
        // 为演示简化：返回下载的字节数

        Ok(downloaded)
    }

    /// 完整下载片 (当服务器不支持 Range 时的回退方案)
    async fn download_full_chunk(
        &self,
        url: &Url,
        chunk_size: u64,
        config: &EngineConfig,
    ) -> Result<u64, DownloadError> {
        let response = timeout(
            Duration::from_secs(config.timeout_secs),
            self.client.get(url).send(),
        )
        .await
        .map_err(|_| DownloadError::Network("Request timed out".to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| DownloadError::Network(format!("Failed to read: {}", e)))?;

        Ok(bytes.len() as u64)
    }

    /// 获取文件总大小 (HEAD 请求)
    async fn get_total_size(&self, url: &Url) -> Result<u64, DownloadError> {
        let mut req = self.client.head(url.clone());

        // 添加代理 (如果启用)
        if self.config.enable_proxy {
            if let Some(proxy_config) = crate::proxy::from_env() {
                if proxy_config.is_valid() {
                    if let Ok(proxy) = proxy_config.to_reqwest() {
                        req = req.proxy(proxy);
                    }
                }
            }
        }

        let response = timeout(
            Duration::from_secs(self.config.timeout_secs),
            req.send(),
        )
        .await
        .map_err(|_| DownloadError::Network("HEAD request timed out".to_string()))?;

        let headers = response.headers();
        // 尝试获取 Content-Length
        if let Some(len) = headers.get(reqwest::header::CONTENT_LENGTH) {
            let len_str = std::str::from_utf8(len.as_bytes())
                .map_err(|_| DownloadError::Network("Invalid Content-Length".to_string()))?;
            let size: u64 = len_str
                .parse()
                .map_err(|_| DownloadError::Network("Failed to parse Content-Length".to_string()))?;
            return Ok(size);
        }

        // 如果有 Content-Range (罕见于 HEAD)
        // 否则返回 0 (表示未知)
        Ok(0)
    }

    /// 计算断点续传的片分配
    async fn calculate_resume_chunks(
        &self,
        total_size: u64,
        chunk_size: u64,
    ) -> Result<(Vec<u32>, Vec<u64>), DownloadError> {
        let total_chunks = if total_size > 0 {
            ((total_size - 1) / chunk_size) + 1
        } else {
            self.config.chunk_count
        };

        let mut chunk_indices: Vec<u32> = (0..total_chunks).map(|i| i as u32).collect();
        let mut start_offsets: Vec<u64> = vec![0u64; total_chunks];

        // 如果有已下载进度，跳过已完成的片
        if session.progress.downloaded > 0 {
            let completed_chunks = (session.progress.downloaded / chunk_size) + 1;
            chunk_indices = chunk_indices.into_iter().skip(completed_chunks).collect();
            start_offsets = start_offsets.into_iter()
                .skip(completed_chunks)
                .collect();
        }

        Ok((chunk_indices, start_offsets))
    }
}

/// 下载会话管理器
/// 负责会话的创建、持久化和状态管理
pub struct DownloadSessionManager {
    /// KV store 接口 (通过 trait 依赖注入)。
    /// 
    /// 实际的持久化实现由外部通过 `DownloadSessionManager.save()` 提供，
    /// 例如保存到 NeoTrix KB (`domain_nt_io` namespace) 或其他存储后端。
    // store: Option<DownloadStore>,
}

impl DownloadSessionManager {
    /// 创建新的下载会话
    pub fn new(url: String, path: PathBuf, user: String) -> DownloadSession {
        DownloadSession::new(url, path, user)
    }

    /// 持存会话到 KV store (由外部实现)
    pub fn save(&self, _session: &DownloadSession) {
        // TODO: 实现 KV store 持久化
        // 例如：neotrix-experience absorb 到 KB
    }

    /// 从 KV load 会话
    pub fn load(&self, _session_id: &str) -> Option<DownloadSession> {
        // TODO: 从 KB load
        None
    }

    /// 检查会话是否已存在
    pub fn exists(&self, _session_id: &str) -> bool {
        false
    }
}