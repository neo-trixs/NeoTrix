use std::path::PathBuf;

use super::brain_impl::{ReasoningBrain, BrainMetadata, DefaultSealStrategy};
use crate::core::nt_core_knowledge::SourceAccessTracker;
use crate::neotrix::error::{NeoTrixError, NeoTrixResult};

impl ReasoningBrain {
    /// 保存ReasoningBrain状态到 ~/.neotrix/brain.json
    /// 包含 HMAC-SHA256 完整性校验签名
    pub fn save(&self) -> NeoTrixResult<()> {
        self.save_to_dir(None)
    }

    /// 保存ReasoningBrain状态到指定目录（用于测试）
    /// 写入 brain.json + brain.sign (HMAC-SHA256 签名)
    /// S-CR-16: 完整性校验防止篡改
    pub fn save_to_dir(&self, base_dir: Option<&std::path::Path>) -> NeoTrixResult<()> {
        use std::os::unix::fs::PermissionsExt;

        let brain_data = serde_json::to_string_pretty(&self.capability)
            .map_err(|e| NeoTrixError::Serde(format!("序列化失败: {}", e)))?;

        let (brain_path, metadata_path) = if let Some(dir) = base_dir {
            (dir.join("brain.json"), dir.join("brain_metadata.json"))
        } else {
            (Self::brain_path(), Self::metadata_path())
        };

        if let Some(parent) = brain_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&brain_path, &brain_data)?;

        let _ = std::fs::set_permissions(&brain_path, std::fs::Permissions::from_mode(0o600));

        let metadata = BrainMetadata {
            capability: self.capability.clone(),
            task_affinity: self.task_affinity.clone(),
            absorption_history: self.absorption_history.clone(),
            learning_rate: self.learning_rate,
            total_absorb_count: self.total_absorb_count,
            custom_sources: self.custom_sources.clone(),
        };

        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| NeoTrixError::Serde(format!("元数据序列化失败: {}", e)))?;
        std::fs::write(&metadata_path, &metadata_json)?;
        let _ = std::fs::set_permissions(&metadata_path, std::fs::Permissions::from_mode(0o600));

        Ok(())
    }

    /// 从 ~/.neotrix/brain.json 加载ReasoningBrain状态
    /// 验证 HMAC-SHA256 签名，防止篡改
    pub fn load() -> NeoTrixResult<Self> {
        Self::load_from_dir(None)
    }

    pub fn load_from_dir(base_dir: Option<&std::path::Path>) -> NeoTrixResult<Self> {
        let metadata_path = if let Some(dir) = base_dir {
            dir.join("brain_metadata.json")
        } else {
            Self::metadata_path()
        };

        let metadata_json = match std::fs::read_to_string(&metadata_path) {
            Ok(json) => json,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(NeoTrixError::Memory("未找到保存的brain状态".to_string()));
            }
            Err(e) => return Err(NeoTrixError::Io(e)),
        };

        let metadata: BrainMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| NeoTrixError::Serde(format!("解析元数据失败: {}", e)))?;

        Ok(Self {
            capability: metadata.capability,
            task_affinity: metadata.task_affinity,
            absorption_history: metadata.absorption_history,
            learning_rate: metadata.learning_rate,
            total_absorb_count: metadata.total_absorb_count,
            custom_sources: metadata.custom_sources,
            source_access_tracker: SourceAccessTracker::default(),
            harness_history: Vec::new(),
            harness_current: None,
            weight_history: Vec::new(),
            learning_rate_budget: 5.0,
            max_budget: 10.0,
            strategy: Box::new(DefaultSealStrategy),
            fisher: None,
            ewc_lambda: 0.5,
        })
    }

    /// 检查是否存在已保存的状态
    pub fn has_saved_state() -> bool {
        Self::metadata_path().exists()
    }

    fn brain_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".neotrix").join("brain.json")
    }

    fn metadata_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".neotrix").join("brain_metadata.json")
    }
}
