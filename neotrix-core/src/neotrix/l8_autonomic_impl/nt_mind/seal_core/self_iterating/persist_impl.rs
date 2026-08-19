use super::brain_impl::{ReasoningBrain, BrainMetadata, DefaultSealStrategy};
use crate::core::nt_core_knowledge::SourceAccessTracker;
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};

impl ReasoningBrain {
    /// 保存ReasoningBrain状态 (KB 直写, 双 key: brain=capability, brain_metadata=元数据)
    pub fn save(&self) -> NeoTrixResult<()> {
        self.save_to_dir(None)
    }

    /// 保存ReasoningBrain状态到指定目录（用于测试 / 显式目录）
    /// 写入 brain.json + brain_metadata.json (含 HMAC 完整性语义)。
    /// 生产路径 (None) → KB kv_store state.brain + state.brain_metadata (Phase 2c 迁移),
    ///   dual-write 保留 legacy 文件形状供旧 reader (entry/server/doctor) 兼容。
    pub fn save_to_dir(&self, base_dir: Option<&std::path::Path>) -> NeoTrixResult<()> {
        let brain_data = serde_json::to_string_pretty(&self.capability)
            .map_err(|e| NeoTrixError::Serde(format!("序列化失败: {}", e)))?;
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

        match base_dir {
            Some(dir) => {
                use std::os::unix::fs::PermissionsExt;
                let brain_path = dir.join("brain.json");
                let metadata_path = dir.join("brain_metadata.json");
                if let Some(parent) = brain_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                // 原子写：crash 中途不截断原文件 (R-P16 对应代码侧)
                neotrix_types::fs_util::atomic_write(&brain_path, brain_data.as_bytes())?;
                let _ = std::fs::set_permissions(&brain_path, std::fs::Permissions::from_mode(0o600));
                neotrix_types::fs_util::atomic_write(&metadata_path, metadata_json.as_bytes())?;
                let _ = std::fs::set_permissions(&metadata_path, std::fs::Permissions::from_mode(0o600));
                Ok(())
            }
            None => {
                crate::core::nt_core_state::save("brain", &brain_data)
                    .map_err(NeoTrixError::Io)?;
                crate::core::nt_core_state::save("brain_metadata", &metadata_json)
                    .map_err(NeoTrixError::Io)?;
                Ok(())
            }
        }
    }

    /// 从 KB 加载ReasoningBrain状态 (legacy brain_metadata.json 作 fallback)
    pub fn load() -> NeoTrixResult<Self> {
        Self::load_from_dir(None)
    }

    pub fn load_from_dir(base_dir: Option<&std::path::Path>) -> NeoTrixResult<Self> {
        let metadata_json = match base_dir {
            Some(dir) => {
                let metadata_path = dir.join("brain_metadata.json");
                match std::fs::read_to_string(&metadata_path) {
                    Ok(json) => json,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        return Err(NeoTrixError::Memory("未找到保存的brain状态".to_string()));
                    }
                    Err(e) => return Err(NeoTrixError::Io(e.to_string())),
                }
            }
            None => match crate::core::nt_core_state::load("brain_metadata") {
                Some(json) => json,
                None => return Err(NeoTrixError::Memory("未找到保存的brain状态".to_string())),
            },
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
        crate::core::nt_core_state::load("brain_metadata").is_some()
    }
}
