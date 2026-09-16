use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    pub format: ModelFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelFormat {
    GGUF,
    ONNX,
    Safetensors,
    GGJ,
}

impl std::fmt::Display for ModelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelFormat::GGUF => write!(f, "gguf"),
            ModelFormat::ONNX => write!(f, "onnx"),
            ModelFormat::Safetensors => write!(f, "safetensors"),
            ModelFormat::GGJ => write!(f, "ggj"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelSource {
    HuggingFace,
    Ollama,
    ModelScope,
    Local,
    LMStudio,
    oMLX,
    LocalGGUF,
    VLLM,
    OpenResearch,
    Custom(String),
}

impl ModelSource {
    pub fn display_name(&self) -> String {
        match self {
            ModelSource::HuggingFace => "Hugging Face".into(),
            ModelSource::Ollama => "Ollama".into(),
            ModelSource::ModelScope => "Model Scope".into(),
            ModelSource::Local => "Local".into(),
            ModelSource::LMStudio => "LM Studio".into(),
            ModelSource::oMLX => "oMLX".into(),
            ModelSource::LocalGGUF => "Local GGUF".into(),
            ModelSource::VLLM => "vLLM".into(),
            ModelSource::OpenResearch => "OpenResearch".into(),
            ModelSource::Custom(name) => format!("Custom: {name}"),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ModelSource::HuggingFace => "huggingface",
            ModelSource::Ollama => "ollama",
            ModelSource::ModelScope => "modelscope",
            ModelSource::Local => "local",
            ModelSource::LMStudio => "lmstudio",
            ModelSource::oMLX => "omlx",
            ModelSource::LocalGGUF => "localgguf",
            ModelSource::VLLM => "vllm",
            ModelSource::OpenResearch => "openresearch",
            ModelSource::Custom(_) => "custom",
        }
    }
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
    pub source: ModelSource,
    pub format: ModelFormat,
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
    pub source: ModelSource,
    pub format: ModelFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelValidationResult {
    pub model_id: String,
    pub sha256_valid: bool,
    pub model_json_valid: bool,
    pub format_valid: bool,
    pub file_size_matches: bool,
    pub overall_valid: bool,
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

    /// 扫描本地模型（支持所有格式）
    async fn scan_local_models(&mut self) -> Result<(), String> {
        let valid_extensions = ["gguf", "onnx", "safetensors", "ggj"];
        let mut entries = fs::read_dir(&self.cache_dir)
            .await
            .map_err(|e| format!("Failed to read cache dir: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read entry: {}", e))?
        {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());
            if ext.map_or(false, |e| valid_extensions.contains(&e)) {
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
        let format = Self::detect_format(path);

        // 尝试读取 model.json
        let json_path = path.with_extension("json");
        if let Ok(content) = fs::read_to_string(&json_path).await {
            if let Ok(mut meta) = serde_json::from_str::<ModelMetadata>(&content) {
                meta.format = format;
                return Some(meta);
            }
        }

        // 尝试读取 model.safetensors.json
        let st_json_path = path.with_extension("safetensors.json");
        if let Ok(content) = fs::read_to_string(&st_json_path).await {
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
            format,
        })
    }

    /// 检测模型格式
    fn detect_format(path: &Path) -> ModelFormat {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext.to_lowercase().as_str() {
            "gguf" => ModelFormat::GGUF,
            "onnx" => ModelFormat::ONNX,
            "safetensors" => ModelFormat::Safetensors,
            "ggj" => ModelFormat::GGJ,
            _ => {
                let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
                if name.contains("safetensors") {
                    ModelFormat::Safetensors
                } else if name.contains("gguf") {
                    ModelFormat::GGUF
                } else if name.contains("onnx") {
                    ModelFormat::ONNX
                } else {
                    ModelFormat::GGUF
                }
            }
        }
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
        self.models
            .values()
            .filter(|m| {
                m.display_name
                    .to_lowercase()
                    .contains(&query.to_lowercase())
                    || m.id.to_lowercase().contains(&query.to_lowercase())
                    || m.architecture
                        .to_lowercase()
                        .contains(&query.to_lowercase())
            })
            .collect()
    }

    /// 列出所有支持的源
    pub fn list_sources() -> Vec<&'static str> {
        vec![
            "Hugging Face",
            "Ollama",
            "LM Studio",
            "oMLX",
            "Local GGUF",
            "vLLM",
            "OpenResearch",
            "Model Scope",
        ]
    }

    /// 创建下载任务
    pub fn create_download_task(
        &mut self,
        model_id: String,
        download_url: String,
        source: ModelSource,
        format: ModelFormat,
    ) -> Result<DownloadTask, String> {
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
            source,
            format,
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
                speed_bytes_per_sec: 0,
                source: task.source.clone(),
                format: task.format.clone(),
            }
        })
    }

    /// 获取所有下载任务
    pub fn list_downloads(&self) -> Vec<&DownloadTask> {
        self.download_queue.values().collect()
    }

    /// 统一下载进度快照
    pub fn unified_progress_snapshot(&self) -> Vec<DownloadProgress> {
        self.download_queue
            .values()
            .map(|task| DownloadProgress {
                job_id: task.job_id.clone(),
                model_id: task.model_id.clone(),
                progress: if task.total_bytes > 0 {
                    task.downloaded_bytes as f32 / task.total_bytes as f32
                } else {
                    0.0
                },
                downloaded_bytes: task.downloaded_bytes,
                total_bytes: task.total_bytes,
                speed_bytes_per_sec: 0,
                source: task.source.clone(),
                format: task.format.clone(),
            })
            .collect()
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

    /// 计算文件 SHA256
    pub async fn compute_sha256(&self, path: &Path) -> Result<String, String> {
        let content = tokio::fs::read(path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;
        let hash = Sha256::digest(&content);
        Ok(format!("{:x}", hash))
    }

    /// 验证模型完整性 (SHA256 + model.json)
    pub async fn verify_model(&self, model_id: &str) -> Result<ModelValidationResult, String> {
        let meta = self.models.get(model_id)
            .ok_or_else(|| format!("Model '{}' not found", model_id))?;

        let mut sha256_valid = false;
        let mut model_json_valid = false;
        let mut format_valid = false;
        let mut file_size_matches = false;

        if let Some(path) = &meta.path {
            if path.exists() {
                // SHA256 验证
                if let Some(expected_sha256) = &meta.sha256 {
                    let actual = self.compute_sha256(path).await?;
                    sha256_valid = actual == *expected_sha256;
                } else {
                    sha256_valid = true; // 无预期哈希，跳过
                }

                // model.json 验证
                let json_path = path.with_extension("json");
                if json_path.exists() {
                    if let Ok(content) = tokio::fs::read_to_string(&json_path).await {
                        if serde_json::from_str::<ModelMetadata>(&content).is_ok() {
                            model_json_valid = true;
                        }
                    }
                } else {
                    model_json_valid = true; // 无 model.json，跳过
                }

                // 格式验证
                let detected = Self::detect_format(path);
                format_valid = detected == meta.format;

                // 文件大小验证
                if let Ok(metadata) = tokio::fs::metadata(path).await {
                    file_size_matches = metadata.len() == meta.file_size;
                }
            }
        }

        let overall_valid = sha256_valid && model_json_valid && format_valid && file_size_matches;

        Ok(ModelValidationResult {
            model_id: model_id.to_string(),
            sha256_valid,
            model_json_valid,
            format_valid,
            file_size_matches,
            overall_valid,
        })
    }

    /// 从 OpenResearch 源下载模型
    pub async fn download_from_openresearch(
        &mut self,
        model_id: String,
        model_name: String,
    ) -> Result<DownloadTask, String> {
        let url = format!(
            "https://openresearch.ai/api/models/{}/download",
            model_name
        );
        self.create_download_task(model_id, url, ModelSource::OpenResearch, ModelFormat::GGUF)
    }

    /// 从 LMStudio 源下载模型
    pub async fn download_from_lmstudio(
        &mut self,
        model_id: String,
        model_name: String,
    ) -> Result<DownloadTask, String> {
        let url = format!(
            "http://localhost:1234/api/llm/models/{}/download",
            model_name
        );
        self.create_download_task(model_id, url, ModelSource::LMStudio, ModelFormat::GGUF)
    }

    /// 从 vLLM 源下载模型
    pub async fn download_from_vllm(
        &mut self,
        model_id: String,
        model_name: String,
        base_url: String,
    ) -> Result<DownloadTask, String> {
        let url = format!("{}/api/download/{}", base_url, model_name);
        self.create_download_task(model_id, url, ModelSource::VLLM, ModelFormat::ONNX)
    }

    /// 从 oMLX 源下载模型
    pub async fn download_from_omlx(
        &mut self,
        model_id: String,
        model_name: String,
    ) -> Result<DownloadTask, String> {
        let url = format!(
            "https://olmx.ai/api/models/{}/download",
            model_name
        );
        self.create_download_task(model_id, url, ModelSource::oMLX, ModelFormat::GGUF)
    }

    /// 从 LocalGGUF 源下载模型
    pub async fn download_from_localgguf(
        &mut self,
        model_id: String,
        model_url: String,
    ) -> Result<DownloadTask, String> {
        self.create_download_task(model_id, model_url, ModelSource::LocalGGUF, ModelFormat::GGUF)
    }

    /// 获取缓存大小
    pub async fn get_cache_size(&self) -> Result<u64, String> {
        let mut total_size = 0;
        let mut entries = fs::read_dir(&self.cache_dir)
            .await
            .map_err(|e| format!("Failed to read cache dir: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
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
