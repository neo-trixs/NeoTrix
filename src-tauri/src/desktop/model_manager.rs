//! Model Manager — 本地模型缓存与下载管理
//!
//! 管理本地模型缓存目录，提供下载/验证/删除能力。
//! 与 provider_pool.rs（API 密钥管理）互补。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

// ========== Core Types ==========

/// 模型能力描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub chat: bool,
    pub completion: bool,
    pub embedding: bool,
    pub vision: bool,
    pub function_calling: bool,
    pub context_window: Option<u32>,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            chat: true,
            completion: true,
            embedding: false,
            vision: false,
            function_calling: false,
            context_window: None,
        }
    }
}

/// 模型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub size_bytes: u64,
    pub quantization: String,
    pub capabilities: ModelCapabilities,
    pub downloaded: bool,
    pub path: Option<PathBuf>,
    pub sha256: Option<String>,
    pub downloaded_at: Option<String>,
}

/// 下载状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Verifying,
    Completed,
    Failed(String),
    Cancelled,
}

/// 下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub task_id: String,
    pub model_id: String,
    pub status: DownloadStatus,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: Option<u64>,
}

/// 下载任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub model_id: String,
    pub url: String,
    pub destination: PathBuf,
    pub expected_sha256: Option<String>,
    pub status: DownloadStatus,
    pub created_at: String,
}

// ========== Model Manager ==========

/// 模型管理器
///
/// 管理本地模型缓存目录，提供：
/// - 模型列表（本地 + 远程仓库）
/// - 下载队列与进度追踪
/// - 模型验证与删除
pub struct ModelManager {
    cache_dir: PathBuf,
    models: RwLock<HashMap<String, ModelMetadata>>,
    download_queue: RwLock<VecDeque<DownloadTask>>,
    download_progress: RwLock<HashMap<String, DownloadProgress>>,
}

impl ModelManager {
    /// 创建模型管理器
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            models: RwLock::new(HashMap::new()),
            download_queue: RwLock::new(VecDeque::new()),
            download_progress: RwLock::new(HashMap::new()),
        }
    }

    /// 创建默认管理器（~/.neotrix/models）
    pub fn default_manager() -> Self {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".neotrix")
            .join("models");
        Self::new(cache_dir)
    }

    /// 初始化：扫描本地缓存目录
    pub async fn init(&self) -> Result<(), String> {
        tokio::fs::create_dir_all(&self.cache_dir)
            .await
            .map_err(|e| format!("创建缓存目录失败: {e}"))?;

        self.scan_local_models().await?;
        Ok(())
    }

    /// 扫描本地缓存目录
    async fn scan_local_models(&self) -> Result<(), String> {
        let mut models = self.models.write().await;

        let mut entries = tokio::fs::read_dir(&self.cache_dir)
            .await
            .map_err(|e| format!("读取缓存目录失败: {e}"))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("遍历目录失败: {e}"))?
        {
            let path = entry.path();
            if path.is_dir() {
                if let Some(model_id) = path.file_name().and_then(|n| n.to_str()) {
                    // 尝试读取 model.json 元数据
                    let meta_path = path.join("model.json");
                    let metadata = if meta_path.exists() {
                        let content = tokio::fs::read_to_string(&meta_path)
                            .await
                            .unwrap_or_default();
                        serde_json::from_str::<ModelMetadata>(&content).ok()
                    } else {
                        None
                    };

                    let metadata = metadata.unwrap_or_else(|| ModelMetadata {
                        id: model_id.to_string(),
                        name: model_id.to_string(),
                        provider: "unknown".to_string(),
                        size_bytes: 0,
                        quantization: "unknown".to_string(),
                        capabilities: ModelCapabilities::default(),
                        downloaded: true,
                        path: Some(path.clone()),
                        sha256: None,
                        downloaded_at: None,
                    });

                    models.insert(model_id.to_string(), metadata);
                }
            }
        }

        Ok(())
    }

    /// 列出所有模型
    pub async fn list_models(&self) -> Vec<ModelMetadata> {
        self.models.read().await.values().cloned().collect()
    }

    /// 获取模型元数据
    pub async fn get_model(&self, model_id: &str) -> Option<ModelMetadata> {
        self.models.read().await.get(model_id).cloned()
    }

    /// 下载模型
    pub async fn download_model(
        &self,
        model_id: &str,
        url: &str,
        expected_sha256: Option<String>,
    ) -> Result<DownloadTask, String> {
        let task_id = format!("dl-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let destination = self.cache_dir.join(model_id);

        let task = DownloadTask {
            id: task_id.clone(),
            model_id: model_id.to_string(),
            url: url.to_string(),
            destination,
            expected_sha256,
            status: DownloadStatus::Pending,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.download_queue
            .write()
            .await
            .push_back(task.clone());

        Ok(task)
    }

    /// 获取下载进度
    pub async fn get_progress(&self, task_id: &str) -> Option<DownloadProgress> {
        self.download_progress.read().await.get(task_id).cloned()
    }

    /// 取消下载
    pub async fn cancel_download(&self, task_id: &str) -> Result<(), String> {
        let mut queue = self.download_queue.write().await;
        queue.retain(|t| t.id != task_id);

        let mut progress = self.download_progress.write().await;
        if let Some(p) = progress.get_mut(task_id) {
            p.status = DownloadStatus::Cancelled;
        }

        Ok(())
    }

    /// 删除模型
    pub async fn delete_model(&self, model_id: &str) -> Result<(), String> {
        let path = self.cache_dir.join(model_id);
        if path.exists() {
            tokio::fs::remove_dir_all(&path)
                .await
                .map_err(|e| format!("删除模型失败: {e}"))?;
        }

        self.models.write().await.remove(model_id);
        Ok(())
    }

    /// 验证模型完整性
    pub async fn verify_model(&self, model_id: &str) -> Result<bool, String> {
        let models = self.models.read().await;
        let metadata = models.get(model_id).ok_or_else(|| {
            format!("模型不存在: {model_id}")
        })?;

        let path = metadata
            .path
            .as_ref()
            .ok_or_else(|| "模型路径未知".to_string())?;

        if !path.exists() {
            return Ok(false);
        }

        // 检查 model.json 是否存在
        let meta_path = path.join("model.json");
        if !meta_path.exists() {
            return Ok(false);
        }

        // 如果有 SHA256，验证文件完整性
        if let Some(expected) = &metadata.sha256 {
            let bin_files: Vec<_> = tokio::fs::read_dir(path)
                .await
                .map_err(|e| format!("读取模型目录失败: {e}"))?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "gguf" || ext == "bin" || ext == "onnx")
                        .unwrap_or(false)
                })
                .collect();

            for file in bin_files {
                let content = tokio::fs::read(file.path())
                    .await
                    .map_err(|e| format!("读取模型文件失败: {e}"))?;
                let hash = format!("{:x}", sha2::Sha256::digest(&content));
                if &hash != expected {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// 获取缓存目录
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// 获取缓存大小（字节）
    pub async fn cache_size(&self) -> u64 {
        self.models
            .read()
            .await
            .values()
            .map(|m| m.size_bytes)
            .sum()
    }
}

// ========== Default ==========

impl Default for ModelManager {
    fn default() -> Self {
        Self::default_manager()
    }
}
