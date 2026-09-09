//! 检查点持久化模块
//!
//! 跨阶段检查点保存与恢复
//! 支持断点续传、状态快照

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 检查点定义
// ============================================================================

/// 检查点状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CheckpointStatus {
    /// 创建中
    Creating,
    /// 已保存
    Saved,
    /// 已加载
    Loaded,
    /// 已过期
    Expired,
}

/// 检查点元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMeta {
    /// 检查点ID
    pub id: String,
    /// 工作流ID
    pub workflow_id: String,
    /// 阶段名称
    pub stage_name: String,
    /// 状态
    pub status: CheckpointStatus,
    /// 创建时间
    pub created_at: u64,
    /// 过期时间
    pub expires_at: Option<u64>,
    /// 文件大小 (字节)
    pub file_size: u64,
    /// 描述
    pub description: Option<String>,
}

/// 检查点数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointData {
    /// 元数据
    pub meta: CheckpointMeta,
    /// 阶段状态
    pub stage_state: HashMap<String, serde_json::Value>,
    /// 输出文件路径
    pub output_files: Vec<String>,
    /// 中间结果
    pub intermediate_results: HashMap<String, serde_json::Value>,
}

/// 检查点存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStorageConfig {
    /// 存储路径
    pub storage_path: String,
    /// 最大检查点数
    pub max_checkpoints: usize,
    /// 过期时间 (秒)
    pub expiration_secs: u64,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 是否启用加密
    pub enable_encryption: bool,
}

/// 检查点操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointResult {
    /// 是否成功
    pub success: bool,
    /// 检查点ID
    pub checkpoint_id: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 操作耗时 (毫秒)
    pub operation_time_ms: u64,
}

// ============================================================================
// 检查点持久化管理器
// ============================================================================

/// 检查点持久化管理器
/// 实现跨阶段检查点保存与恢复
pub struct CheckpointPersistence {
    /// 配置
    config: CheckpointStorageConfig,
    /// 检查点索引
    index: HashMap<String, CheckpointMeta>,
    /// 当前检查点
    current: Option<CheckpointData>,
}

impl CheckpointPersistence {
    /// 创建管理器
    pub fn new(storage_path: &str) -> Self {
        Self {
            config: CheckpointStorageConfig {
                storage_path: storage_path.to_string(),
                max_checkpoints: 100,
                expiration_secs: 86400, // 24小时
                enable_compression: true,
                enable_encryption: false,
            },
            index: HashMap::new(),
            current: None,
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: CheckpointStorageConfig) -> Self {
        Self {
            config,
            index: HashMap::new(),
            current: None,
        }
    }
    
    /// 保存检查点
    pub fn save_checkpoint(
        &mut self,
        workflow_id: &str,
        stage_name: &str,
        state: HashMap<String, serde_json::Value>,
        output_files: Vec<String>,
    ) -> CheckpointResult {
        let start = std::time::Instant::now();
        
        let checkpoint_id = format!("{}_{}_{}", workflow_id, stage_name, current_timestamp());
        
        let meta = CheckpointMeta {
            id: checkpoint_id.clone(),
            workflow_id: workflow_id.to_string(),
            stage_name: stage_name.to_string(),
            status: CheckpointStatus::Saved,
            created_at: current_timestamp(),
            expires_at: Some(current_timestamp() + self.config.expiration_secs),
            file_size: 0, // TODO: 计算实际大小
            description: None,
        };
        
        let data = CheckpointData {
            meta: meta.clone(),
            stage_state: state,
            output_files,
            intermediate_results: HashMap::new(),
        };
        
        // TODO: 实际保存到文件系统
        self.index.insert(checkpoint_id.clone(), meta);
        self.current = Some(data);
        
        CheckpointResult {
            success: true,
            checkpoint_id: Some(checkpoint_id),
            error: None,
            operation_time_ms: start.elapsed().as_millis() as u64,
        }
    }
    
    /// 加载检查点
    pub fn load_checkpoint(&mut self, checkpoint_id: &str) -> CheckpointResult {
        let start = std::time::Instant::now();
        
        if let Some(meta) = self.index.get(checkpoint_id) {
            // TODO: 实际从文件系统加载
            let data = CheckpointData {
                meta: meta.clone(),
                stage_state: HashMap::new(),
                output_files: vec![],
                intermediate_results: HashMap::new(),
            };
            
            self.current = Some(data);
            
            CheckpointResult {
                success: true,
                checkpoint_id: Some(checkpoint_id.to_string()),
                error: None,
                operation_time_ms: start.elapsed().as_millis() as u64,
            }
        } else {
            CheckpointResult {
                success: false,
                checkpoint_id: None,
                error: Some(format!("检查点 {} 不存在", checkpoint_id)),
                operation_time_ms: start.elapsed().as_millis() as u64,
            }
        }
    }
    
    /// 列出检查点
    pub fn list_checkpoints(&self, workflow_id: &str) -> Vec<&CheckpointMeta> {
        self.index.values()
            .filter(|m| m.workflow_id == workflow_id)
            .collect()
    }
    
    /// 删除检查点
    pub fn delete_checkpoint(&mut self, checkpoint_id: &str) -> CheckpointResult {
        let start = std::time::Instant::now();
        
        if self.index.remove(checkpoint_id).is_some() {
            // TODO: 实际删除文件
            CheckpointResult {
                success: true,
                checkpoint_id: Some(checkpoint_id.to_string()),
                error: None,
                operation_time_ms: start.elapsed().as_millis() as u64,
            }
        } else {
            CheckpointResult {
                success: false,
                checkpoint_id: None,
                error: Some(format!("检查点 {} 不存在", checkpoint_id)),
                operation_time_ms: start.elapsed().as_millis() as u64,
            }
        }
    }
    
    /// 清理过期检查点
    pub fn cleanup_expired(&mut self) -> usize {
        let now = current_timestamp();
        let expired: Vec<String> = self.index.iter()
            .filter(|(_, meta)| {
                meta.expires_at.map_or(false, |exp| exp < now)
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        let count = expired.len();
        for id in &expired {
            self.index.remove(id);
            // TODO: 实际删除文件
        }
        
        count
    }
    
    /// 获取当前检查点
    pub fn current(&self) -> Option<&CheckpointData> {
        self.current.as_ref()
    }
    
    /// 获取统计信息
    pub fn statistics(&self) -> CheckpointStats {
        let total_checkpoints = self.index.len();
        let total_size: u64 = self.index.values().map(|m| m.file_size).sum();
        let workflows: std::collections::HashSet<String> = self.index.values()
            .map(|m| m.workflow_id.clone())
            .collect();
        
        CheckpointStats {
            total_checkpoints,
            total_size_bytes: total_size,
            unique_workflows: workflows.len(),
        }
    }
}

/// 检查点统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStats {
    /// 总检查点数
    pub total_checkpoints: usize,
    /// 总大小 (字节)
    pub total_size_bytes: u64,
    /// 唯一工作流数
    pub unique_workflows: usize,
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_checkpoint_persistence() {
        let mut persistence = CheckpointPersistence::new("/tmp/checkpoints");
        
        // 保存检查点
        let mut state = HashMap::new();
        state.insert("step".to_string(), serde_json::json!("planning"));
        
        let result = persistence.save_checkpoint(
            "workflow_001",
            "planning",
            state,
            vec!["/output/plan.json".to_string()],
        );
        
        assert!(result.success);
        assert!(result.checkpoint_id.is_some());
        
        // 加载检查点
        let checkpoint_id = result.checkpoint_id.unwrap();
        let load_result = persistence.load_checkpoint(&checkpoint_id);
        assert!(load_result.success);
        
        // 列出检查点
        let checkpoints = persistence.list_checkpoints("workflow_001");
        assert_eq!(checkpoints.len(), 1);
    }
}