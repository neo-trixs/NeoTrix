//! PipelineCheckpointing — 管线检查点
//!
//! 中间结果存储 + 恢复 + 断点续传。
//! 支持从失败的作业中恢复，避免重新开始。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 检查点信息
#[derive(Debug, Clone)]
pub struct CheckpointInfo {
    /// 检查点 ID
    pub id: String,
    /// 作业 ID
    pub job_id: String,
    /// 阶段名
    pub stage: String,
    /// 阶段索引
    pub stage_index: u32,
    /// 中间结果路径
    pub artifact_path: String,
    /// 中间结果大小 (字节)
    pub artifact_size_bytes: u64,
    /// 创建时间
    pub created_at: Instant,
    /// 耗时
    pub duration_ms: f64,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 状态
    pub status: CheckpointStatus,
}

/// 检查点状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointStatus {
    Active,
    Archived,
    Deleted,
}

/// 检查点配置
#[derive(Debug, Clone)]
pub struct CheckpointConfig {
    /// 最大检查点数
    pub max_checkpoints: usize,
    /// 检查点保留时间
    pub retention_duration: Duration,
    /// 自动清理
    pub auto_cleanup: bool,
    /// 存储后端
    pub storage_backend: StorageBackend,
}

/// 存储后端
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageBackend {
    Local,
    S3,
    GCS,
    Azure,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            max_checkpoints: 100,
            retention_duration: Duration::from_secs(7 * 24 * 3600), // 7 天
            auto_cleanup: true,
            storage_backend: StorageBackend::Local,
        }
    }
}

/// 管线检查点管理器
pub struct PipelineCheckpointing {
    /// 检查点存储
    checkpoints: HashMap<String, Vec<CheckpointInfo>>,
    /// 配置
    config: CheckpointConfig,
    /// 统计信息
    stats: CheckpointStats,
}

impl PipelineCheckpointing {
    pub fn new(config: CheckpointConfig) -> Self {
        Self {
            checkpoints: HashMap::new(),
            config,
            stats: CheckpointStats::default(),
        }
    }

    /// 创建检查点
    pub fn create_checkpoint(&mut self, job_id: &str, stage: &str, stage_index: u32, artifact_path: &str, artifact_size_bytes: u64, duration_ms: f64, metadata: HashMap<String, String>) -> CheckpointInfo {
        let id = format!("cp-{}", uuid::Uuid::new_v4());

        let checkpoint = CheckpointInfo {
            id: id.clone(),
            job_id: job_id.to_string(),
            stage: stage.to_string(),
            stage_index,
            artifact_path: artifact_path.to_string(),
            artifact_size_bytes,
            created_at: Instant::now(),
            duration_ms,
            metadata,
            status: CheckpointStatus::Active,
        };

        self.checkpoints
            .entry(job_id.to_string())
            .or_insert_with(Vec::new)
            .push(checkpoint.clone());

        self.stats.total_checkpoints += 1;
        self.stats.total_size_bytes += artifact_size_bytes;

        // 自动清理
        if self.config.auto_cleanup {
            self.cleanup_checkpoints(job_id);
        }

        checkpoint
    }

    /// 获取最新检查点
    pub fn get_latest_checkpoint(&self, job_id: &str) -> Option<&CheckpointInfo> {
        self.checkpoints.get(job_id)
            .and_then(|cps| cps.iter().max_by_key(|cp| cp.stage_index))
    }

    /// 获取指定阶段的检查点
    pub fn get_stage_checkpoint(&self, job_id: &str, stage: &str) -> Option<&CheckpointInfo> {
        self.checkpoints.get(job_id)
            .and_then(|cps| cps.iter().find(|cp| cp.stage == stage && cp.status == CheckpointStatus::Active))
    }

    /// 获取所有检查点
    pub fn get_all_checkpoints(&self, job_id: &str) -> Vec<&CheckpointInfo> {
        self.checkpoints.get(job_id)
            .map(|cps| cps.iter().collect())
            .unwrap_or_default()
    }

    /// 清理过期检查点
    fn cleanup_checkpoints(&mut self, job_id: &str) {
        if let Some(cps) = self.checkpoints.get_mut(job_id) {
            cps.retain(|cp| cp.created_at.elapsed() < self.config.retention_duration);

            if cps.len() > self.config.max_checkpoints {
                cps.sort_by_key(|cp| cp.stage_index);
                let to_remove = cps.len() - self.config.max_checkpoints;
                cps.drain(..to_remove);
            }
        }
    }

    /// 删除检查点
    pub fn delete_checkpoint(&mut self, job_id: &str, checkpoint_id: &str) -> bool {
        if let Some(cps) = self.checkpoints.get_mut(job_id) {
            if let Some(pos) = cps.iter().position(|cp| cp.id == checkpoint_id) {
                let cp = cps.remove(pos);
                self.stats.total_size_bytes -= cp.artifact_size_bytes;
                return true;
            }
        }
        false
    }

    /// 获取统计信息
    pub fn stats(&self) -> CheckpointStats {
        self.stats.clone()
    }
}

impl Default for PipelineCheckpointing {
    fn default() -> Self {
        Self::new(CheckpointConfig::default())
    }
}

/// 检查点统计
#[derive(Debug, Clone, Default)]
pub struct CheckpointStats {
    pub total_checkpoints: u32,
    pub total_size_bytes: u64,
}

impl CheckpointStats {
    pub fn total_size_gb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_checkpoint() {
        let mut manager = PipelineCheckpointing::default();
        let cp = manager.create_checkpoint(
            "job-1",
            "storyboard",
            0,
            "/tmp/storyboard.json",
            1024,
            5000.0,
            HashMap::new(),
        );
        assert_eq!(cp.stage, "storyboard");
    }

    #[test]
    fn test_get_latest_checkpoint() {
        let mut manager = PipelineCheckpointing::default();
        manager.create_checkpoint("job-1", "storyboard", 0, "/tmp/storyboard.json", 1024, 5000.0, HashMap::new());
        manager.create_checkpoint("job-1", "image_gen", 1, "/tmp/images/", 10240, 30000.0, HashMap::new());

        let latest = manager.get_latest_checkpoint("job-1");
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().stage, "image_gen");
    }
}
