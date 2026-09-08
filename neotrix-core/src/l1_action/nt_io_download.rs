//! 下载引擎 (自研，无外部依赖)
//!
//! Stub 模块 — 完整实现待迁移。

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub max_concurrent: usize,
    pub timeout_secs: u64,
    pub retry_count: u32,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            timeout_secs: 30,
            retry_count: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub url: String,
    pub dest: PathBuf,
}

#[derive(Debug)]
pub enum DownloadStatus {
    Pending,
    InProgress { bytes_downloaded: u64 },
    Completed,
    Failed(String),
}

#[allow(dead_code)]
pub struct DownloadEngine {
    config: DownloadConfig,
}

impl DownloadEngine {
    pub fn new(config: DownloadConfig) -> Self {
        Self { config }
    }

    pub fn download(&self, _task: &DownloadTask) -> Result<DownloadStatus, String> {
        // Stub implementation
        Ok(DownloadStatus::Completed)
    }

    pub fn download_all(&self, tasks: &[DownloadTask]) -> Vec<Result<DownloadStatus, String>> {
        tasks.iter().map(|t| self.download(t)).collect()
    }
}

impl Default for DownloadEngine {
    fn default() -> Self {
        Self::new(DownloadConfig::default())
    }
}
