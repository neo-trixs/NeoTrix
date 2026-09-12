//! 批量生产工作流模块
//!
//! 管理多任务并行、进度追踪、断点续传
//! 支持工业化生产流水线

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 工作流定义
// ============================================================================

/// 生产任务状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// 待处理
    Pending,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已暂停
    Paused,
    /// 已取消
    Cancelled,
}

/// 生产任务类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    /// 剧本解析
    ScriptParsing,
    /// 角色生成
    CharacterGeneration,
    /// 场景生成
    SceneGeneration,
    /// 分镜生成
    StoryboardGeneration,
    /// 视频生成
    VideoGeneration,
    /// 配音合成
    VoiceSynthesis,
    /// 视频合成
    VideoComposition,
    /// 质量审核
    QualityReview,
}

/// 生产任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProductionTask {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务类型
    pub task_type: TaskType,
    /// 任务状态
    pub status: TaskStatus,
    /// 任务描述
    pub description: String,
    /// 开始时间
    pub started_at: Option<u64>,
    /// 结束时间
    pub completed_at: Option<u64>,
    /// 进度 (0.0-1.0)
    pub progress: f32,
    /// 依赖的任务ID列表
    pub dependencies: Vec<String>,
    /// 任务参数
    pub parameters: HashMap<String, serde_json::Value>,
    /// 任务结果
    pub result: Option<serde_json::Value>,
    /// 任务错误
    pub error: Option<String>,
    /// 重试次数
    pub retry_count: u32,
    /// 最大重试次数
    pub max_retries: u32,
}

/// 生产批次
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProductionBatch {
    /// 批次ID
    pub id: String,
    /// 批次名称
    pub name: String,
    /// 批次描述
    pub description: String,
    /// 任务列表
    pub tasks: Vec<ProductionTask>,
    /// 批次状态
    pub status: TaskStatus,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
}

/// 生产流水线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProductionPipeline {
    /// 流水线ID
    pub id: String,
    /// 流水线名称
    pub name: String,
    /// 批次列表
    pub batches: Vec<ProductionBatch>,
    /// 流水线状态
    pub status: TaskStatus,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
}

// ============================================================================
// 批量生产管理器
// ============================================================================

/// 批量生产管理器
pub struct BatchProductionManager {
    /// 所有流水线
    pipelines: HashMap<String, ProductionPipeline>,
    /// 任务依赖图
    #[allow(dead_code)]
    dependency_graph: HashMap<String, Vec<String>>,
    /// 任务结果缓存
    result_cache: HashMap<String, serde_json::Value>,
}

impl BatchProductionManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            dependency_graph: HashMap::new(),
            result_cache: HashMap::new(),
        }
    }
    
    /// 创建流水线
    pub fn create_pipeline(&mut self, pipeline: ProductionPipeline) {
        self.pipelines.insert(pipeline.id.clone(), pipeline);
    }
    
    /// 获取流水线
    pub(crate) fn _get_pipeline(&self, id: &str) -> Option<&ProductionPipeline> {
        self.pipelines.get(id)
    }
    
    /// 获取流水线可变引用
    pub(crate) fn _get_pipeline_mut(&mut self, id: &str) -> Option<&mut ProductionPipeline> {
        self.pipelines.get_mut(id)
    }
    
    /// 列出所有流水线
    pub(crate) fn _list_pipelines(&self) -> Vec<&ProductionPipeline> {
        self.pipelines.values().collect()
    }
    
    /// 启动批次
    pub(crate) fn _start_batch(&mut self, pipeline_id: &str, batch_id: &str) -> Result<(), String> {
        if let Some(pipeline) = self.pipelines.get_mut(pipeline_id) {
            if let Some(batch) = pipeline.batches.iter_mut().find(|b| b.id == batch_id) {
                batch.status = TaskStatus::Running;
                batch.updated_at = timestamp_now();
                Ok(())
            } else {
                Err(format!("Batch {} not found", batch_id))
            }
        } else {
            Err(format!("Pipeline {} not found", pipeline_id))
        }
    }
    
    /// 暂停批次
    pub(crate) fn _pause_batch(&mut self, pipeline_id: &str, batch_id: &str) -> Result<(), String> {
        if let Some(pipeline) = self.pipelines.get_mut(pipeline_id) {
            if let Some(batch) = pipeline.batches.iter_mut().find(|b| b.id == batch_id) {
                batch.status = TaskStatus::Paused;
                batch.updated_at = timestamp_now();
                Ok(())
            } else {
                Err(format!("Batch {} not found", batch_id))
            }
        } else {
            Err(format!("Pipeline {} not found", pipeline_id))
        }
    }
    
    /// 取消批次
    pub(crate) fn _cancel_batch(&mut self, pipeline_id: &str, batch_id: &str) -> Result<(), String> {
        if let Some(pipeline) = self.pipelines.get_mut(pipeline_id) {
            if let Some(batch) = pipeline.batches.iter_mut().find(|b| b.id == batch_id) {
                batch.status = TaskStatus::Cancelled;
                batch.updated_at = timestamp_now();
                Ok(())
            } else {
                Err(format!("Batch {} not found", batch_id))
            }
        } else {
            Err(format!("Pipeline {} not found", pipeline_id))
        }
    }
    
    /// 更新任务状态
    pub(crate) fn _update_task_status(
        &mut self,
        pipeline_id: &str,
        batch_id: &str,
        task_id: &str,
        status: TaskStatus,
        progress: Option<f32>,
        result: Option<serde_json::Value>,
        error: Option<String>,
    ) -> Result<(), String> {
        if let Some(pipeline) = self.pipelines.get_mut(pipeline_id) {
            if let Some(batch) = pipeline.batches.iter_mut().find(|b| b.id == batch_id) {
                if let Some(task) = batch.tasks.iter_mut().find(|t| t.id == task_id) {
                    task.status = status;
                    if let Some(p) = progress {
                        task.progress = p;
                    }
                    if let Some(r) = result {
                        task.result = Some(r.clone());
                        self.result_cache.insert(task_id.to_string(), r);
                    }
                    if let Some(e) = error {
                        task.error = Some(e);
                    }
                    task.completed_at = Some(timestamp_now());
                    batch.updated_at = timestamp_now();
                    Ok(())
                } else {
                    Err(format!("Task {} not found", task_id))
                }
            } else {
                Err(format!("Batch {} not found", batch_id))
            }
        } else {
            Err(format!("Pipeline {} not found", pipeline_id))
        }
    }
    
    /// 检查任务依赖
    pub fn check_dependencies(&self, pipeline_id: &str, batch_id: &str, task_id: &str) -> bool {
        if let Some(pipeline) = self.pipelines.get(pipeline_id) {
            if let Some(batch) = pipeline.batches.iter().find(|b| b.id == batch_id) {
                if let Some(task) = batch.tasks.iter().find(|t| t.id == task_id) {
                    task.dependencies.iter().all(|dep_id| {
                        batch.tasks.iter()
                            .find(|t| t.id == *dep_id)
                            .map(|t| t.status == TaskStatus::Completed)
                            .unwrap_or(false)
                    })
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }
    
    /// 获取批次进度
    pub(crate) fn _get_batch_progress(&self, pipeline_id: &str, batch_id: &str) -> Option<f32> {
        if let Some(pipeline) = self.pipelines.get(pipeline_id) {
            if let Some(batch) = pipeline.batches.iter().find(|b| b.id == batch_id) {
                if batch.tasks.is_empty() {
                    return Some(1.0);
                }
                let total_progress: f32 = batch.tasks.iter().map(|t| t.progress).sum();
                Some(total_progress / batch.tasks.len() as f32)
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// 保存断点
    pub fn save_checkpoint(&self, pipeline_id: &str) -> Result<Vec<u8>, String> {
        if let Some(pipeline) = self.pipelines.get(pipeline_id) {
            serde_json::to_vec(pipeline).map_err(|e| e.to_string())
        } else {
            Err(format!("Pipeline {} not found", pipeline_id))
        }
    }
    
    /// 恢复断点
    pub(crate) fn _restore_checkpoint(&mut self, data: &[u8]) -> Result<(), String> {
        let pipeline: ProductionPipeline = serde_json::from_slice(data).map_err(|e| e.to_string())?;
        self.pipelines.insert(pipeline.id.clone(), pipeline);
        Ok(())
    }
    
    /// 获取统计信息
    pub fn statistics(&self) -> ManagerStats {
        let mut total_tasks = 0;
        let mut completed_tasks = 0;
        let mut failed_tasks = 0;
        let mut running_tasks = 0;
        
        for pipeline in self.pipelines.values() {
            for batch in &pipeline.batches {
                for task in &batch.tasks {
                    total_tasks += 1;
                    match task.status {
                        TaskStatus::Completed => completed_tasks += 1,
                        TaskStatus::Failed => failed_tasks += 1,
                        TaskStatus::Running => running_tasks += 1,
                        _ => {}
                    }
                }
            }
        }
        
        ManagerStats {
            pipeline_count: self.pipelines.len(),
            total_tasks,
            completed_tasks,
            failed_tasks,
            running_tasks,
            pending_tasks: total_tasks - completed_tasks - failed_tasks - running_tasks,
        }
    }
}

/// 管理器统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ManagerStats {
    /// 流水线数量
    pub pipeline_count: usize,
    /// 总任务数
    pub total_tasks: usize,
    /// 已完成任务数
    pub completed_tasks: usize,
    /// 失败任务数
    pub failed_tasks: usize,
    /// 运行中任务数
    pub running_tasks: usize,
    /// 待处理任务数
    pub pending_tasks: usize,
}

/// 获取当前时间戳
fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_batch_production_manager() {
        let mut manager = BatchProductionManager::new();
        
        // 创建任务
        let task1 = ProductionTask {
            id: "task_001".to_string(),
            name: "剧本解析".to_string(),
            task_type: TaskType::ScriptParsing,
            status: TaskStatus::Pending,
            description: "解析剧本文本".to_string(),
            started_at: None,
            completed_at: None,
            progress: 0.0,
            dependencies: vec![],
            parameters: HashMap::new(),
            result: None,
            error: None,
            retry_count: 0,
            max_retries: 3,
        };
        
        let task2 = ProductionTask {
            id: "task_002".to_string(),
            name: "角色生成".to_string(),
            task_type: TaskType::CharacterGeneration,
            status: TaskStatus::Pending,
            description: "生成角色资产".to_string(),
            started_at: None,
            completed_at: None,
            progress: 0.0,
            dependencies: vec!["task_001".to_string()],
            parameters: HashMap::new(),
            result: None,
            error: None,
            retry_count: 0,
            max_retries: 3,
        };
        
        // 创建批次
        let batch = ProductionBatch {
            id: "batch_001".to_string(),
            name: "测试批次".to_string(),
            description: "测试".to_string(),
            tasks: vec![task1, task2],
            status: TaskStatus::Pending,
            created_at: 0,
            updated_at: 0,
        };
        
        // 创建流水线
        let pipeline = ProductionPipeline {
            id: "pipeline_001".to_string(),
            name: "测试流水线".to_string(),
            batches: vec![batch],
            status: TaskStatus::Pending,
            created_at: 0,
            updated_at: 0,
        };
        
        manager.create_pipeline(pipeline);
        
        // 验证
        let stats = manager.statistics();
        assert_eq!(stats.pipeline_count, 1);
        assert_eq!(stats.total_tasks, 2);
    }
    
    #[test]
    fn test_task_dependencies() {
        let mut manager = BatchProductionManager::new();
        
        let task1 = ProductionTask {
            id: "task_001".to_string(),
            name: "任务1".to_string(),
            task_type: TaskType::ScriptParsing,
            status: TaskStatus::Completed,
            description: "".to_string(),
            started_at: Some(0),
            completed_at: Some(1),
            progress: 1.0,
            dependencies: vec![],
            parameters: HashMap::new(),
            result: None,
            error: None,
            retry_count: 0,
            max_retries: 3,
        };
        
        let task2 = ProductionTask {
            id: "task_002".to_string(),
            name: "任务2".to_string(),
            task_type: TaskType::CharacterGeneration,
            status: TaskStatus::Pending,
            description: "".to_string(),
            started_at: None,
            completed_at: None,
            progress: 0.0,
            dependencies: vec!["task_001".to_string()],
            parameters: HashMap::new(),
            result: None,
            error: None,
            retry_count: 0,
            max_retries: 3,
        };
        
        let batch = ProductionBatch {
            id: "batch_001".to_string(),
            name: "测试批次".to_string(),
            description: "".to_string(),
            tasks: vec![task1, task2],
            status: TaskStatus::Pending,
            created_at: 0,
            updated_at: 0,
        };
        
        let pipeline = ProductionPipeline {
            id: "pipeline_001".to_string(),
            name: "测试流水线".to_string(),
            batches: vec![batch],
            status: TaskStatus::Pending,
            created_at: 0,
            updated_at: 0,
        };
        
        manager.create_pipeline(pipeline);
        
        // 检查依赖
        assert!(manager.check_dependencies("pipeline_001", "batch_001", "task_002"));
    }
}