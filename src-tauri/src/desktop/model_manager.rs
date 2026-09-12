use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: String,
    pub display_name: String,
    pub source: ModelSource,
    pub source_repo: Option<String>,
    pub source_revision: Option<String>,
    pub architecture: String,
    pub quantization: Option<String>,
    pub scale: u32,
    pub file_size: u64,
    pub sha256: Option<String>,
    pub download_url: Option<String>,
    pub capabilities: ModelCapabilities,
    pub context_length: Option<u32>,
    pub parameter_count: Option<String>,
    pub downloaded: bool,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelSource {
    HuggingFace,
    Ollama,
    ModelScope,
    Local,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub text: bool,
    pub vision: bool,
    pub audio: bool,
    pub function_calling: bool,
    pub streaming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub job_id: String,
    pub model_id: String,
    pub status: DownloadStatus,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading { progress: f32 },
    Paused,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub job_id: String,
    pub model_id: String,
    pub progress: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
}

pub struct ModelManager {
    cache_dir: PathBuf,
    models: HashMap<String, ModelMetadata>,
    download_queue: HashMap<String, DownloadTask>,
    max_concurrent_downloads: usize,
}

impl ModelManager {
    pub fn new() -> Self {
        let cache_dir = dirs::home_dir()
            .unwrap_or_default()
            .join(".neotrix")
            .join("models");
        
        Self {
            cache_dir,
            models: HashMap::new(),
            download_queue: HashMap::new(),
            max_concurrent_downloads: 3,
        }
    }

    /// 初始化：扫描本地模型
    pub async fn initialize(&mut self) -> Result<(), String> {
        fs::create_dir_all(&self.cache_dir)
            .await
            .map_err(|e| format!("Failed to create cache dir: {}", e))?;
        
        self.scan_local_models().await?;
        Ok(())
    }

    /// 扫描本地模型
    async fn scan_local_models(&mut self) -> Result<(), String> {
        let mut entries = fs::read_dir(&self.cache_dir)
            .await
            .map_err(|e| format!("Failed to read cache dir: {}", e))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| format!("Failed to read entry: {}", e))? 
        {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "gguf" || ext == "onnx") {
                let metadata = self.load_model_metadata(&path).await;
                if let Some(meta) = metadata {
                    self.models.insert(meta.id.clone(), meta);
                }
            }
        }
        
        Ok(())
    }

    /// 加载模型元数据
    async fn load_model_metadata(&self, path: &Path) -> Option<ModelMetadata> {
        let file_name = path.file_stem()?.to_str()?;
        let file_size = fs::metadata(path).await.ok()?.len();
        
        // 尝试读取 model.json
        let json_path = path.with_extension("json");
        if let Ok(content) = fs::read_to_string(&json_path).await {
            if let Ok(meta) = serde_json::from_str::<ModelMetadata>(&content) {
                return Some(meta);
            }
        }
        
        // 创建默认元数据
        Some(ModelMetadata {
            id: file_name.to_string(),
            display_name: file_name.replace('_', " "),
            source: ModelSource::Local,
            source_repo: None,
            source_revision: None,
            architecture: "unknown".to_string(),
            quantization: None,
            scale: 0,
            file_size,
            sha256: None,
            download_url: None,
            capabilities: ModelCapabilities {
                text: true,
                vision: false,
                audio: false,
                function_calling: false,
                streaming: true,
            },
            context_length: None,
            parameter_count: None,
            downloaded: true,
            path: Some(path.to_path_buf()),
        })
    }

    /// 列出所有可用模型
    pub fn list_models(&self) -> Vec<&ModelMetadata> {
        self.models.values().collect()
    }

    /// 获取模型详情
    pub fn get_model(&self, model_id: &str) -> Option<&ModelMetadata> {
        self.models.get(model_id)
    }

    /// 搜索模型
    pub fn search_models(&self, query: &str) -> Vec<&ModelMetadata> {
        self.models.values()
            .filter(|m| {
                m.display_name.to_lowercase().contains(&query.to_lowercase())
                    || m.id.to_lowercase().contains(&query.to_lowercase())
                    || m.architecture.to_lowercase().contains(&query.to_lowercase())
            })
            .collect()
    }

    /// 创建下载任务
    pub fn create_download_task(&mut self, model_id: String, download_url: String) -> Result<DownloadTask, String> {
        let job_id = format!("job_{}", uuid::Uuid::new_v4());
        
        let task = DownloadTask {
            job_id: job_id.clone(),
            model_id: model_id.clone(),
            status: DownloadStatus::Pending,
            total_bytes: 0,
            downloaded_bytes: 0,
            started_at: chrono::Utc::now(),
            completed_at: None,
            error: None,
        };
        
        self.download_queue.insert(job_id.clone(), task.clone());
        Ok(task)
    }

    /// 获取下载进度
    pub fn get_download_progress(&self, job_id: &str) -> Option<DownloadProgress> {
        self.download_queue.get(job_id).map(|task| {
            DownloadProgress {
                job_id: task.job_id.clone(),
                model_id: task.model_id.clone(),
                progress: if task.total_bytes > 0 {
                    task.downloaded_bytes as f32 / task.total_bytes as f32
                } else {
                    0.0
                },
                downloaded_bytes: task.downloaded_bytes,
                total_bytes: task.total_bytes,
                speed_bytes_per_sec: 0, // TODO: 计算速度
            }
        })
    }

    /// 暂停下载
    pub fn pause_download(&mut self, job_id: &str) -> Result<(), String> {
        if let Some(task) = self.download_queue.get_mut(job_id) {
            task.status = DownloadStatus::Paused;
            Ok(())
        } else {
            Err("Download task not found".into())
        }
    }

    /// 恢复下载
    pub fn resume_download(&mut self, job_id: &str) -> Result<(), String> {
        if let Some(task) = self.download_queue.get_mut(job_id) {
            task.status = DownloadStatus::Pending;
            Ok(())
        } else {
            Err("Download task not found".into())
        }
    }

    /// 取消下载
    pub fn cancel_download(&mut self, job_id: &str) -> Result<(), String> {
        self.download_queue.remove(job_id);
        Ok(())
    }

    /// 删除模型
    pub async fn delete_model(&mut self, model_id: &str) -> Result<(), String> {
        if let Some(meta) = self.models.remove(model_id) {
            if let Some(path) = meta.path {
                fs::remove_file(&path)
                    .await
                    .map_err(|e| format!("Failed to delete model file: {}", e))?;
            }
            Ok(())
        } else {
            Err("Model not found".into())
        }
    }

    /// 验证模型完整性
    pub async fn verify_model(&self, model_id: &str) -> Result<bool, String> {
        if let Some(meta) = self.models.get(model_id) {
            if let Some(path) = &meta.path {
                if path.exists() {
                    // TODO: 验证 SHA256
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// 获取缓存大小
    pub async fn get_cache_size(&self) -> Result<u64, String> {
        let mut total_size = 0;
        let mut entries = fs::read_dir(&self.cache_dir)
            .await
            .map_err(|e| format!("Failed to read cache dir: {}", e))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| format!("Failed to read entry: {}", e))?
        {
            if let Ok(metadata) = fs::metadata(entry.path()).await {
                total_size += metadata.len();
            }
        }
        
        Ok(total_size)
    }

    /// 清理缓存
    pub async fn clear_cache(&mut self) -> Result<(), String> {
        fs::remove_dir_all(&self.cache_dir)
            .await
            .map_err(|e| format!("Failed to clear cache: {}", e))?;
        
        fs::create_dir_all(&self.cache_dir)
            .await
            .map_err(|e| format!("Failed to recreate cache dir: {}", e))?;
        
        self.models.clear();
        self.download_queue.clear();
        
        Ok(())
    }
}