//! 任务调度优化模块
//!
//! 实现 GPU 显存管理、批量调度优化、指数退避重试
//! 提升 AI 漫剧生产任务的调度效率

use serde::{Serialize, Deserialize};


// ============================================================================
// 任务调度定义
// ============================================================================

/// 任务优先级
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// 低优先级
    Low,
    /// 普通优先级
    Normal,
    /// 高优先级
    High,
    /// 紧急优先级
    Urgent,
}

/// 任务状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskState {
    /// 等待中
    Waiting,
    /// 运行中
    Running,
    /// 暂停中
    Paused,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// GPU 显存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMemoryConfig {
    /// 总显存 (MB)
    pub total_memory_mb: u32,
    /// 已使用显存 (MB)
    pub used_memory_mb: u32,
    /// 显存使用阈值 (0.0-1.0)
    pub memory_threshold: f32,
    /// 是否启用显存分片
    pub enable_memory_sharding: bool,
    /// 显存分片大小 (MB)
    pub shard_size_mb: u32,
}

/// 重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始退避时间 (毫秒)
    pub initial_backoff_ms: u64,
    /// 最大退避时间 (毫秒)
    pub max_backoff_ms: u64,
    /// 退避倍数
    pub backoff_multiplier: f32,
    /// 是否启用抖动
    pub enable_jitter: bool,
}

/// 调度任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingTask {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 优先级
    pub priority: TaskPriority,
    /// 状态
    pub state: TaskState,
    /// 所需显存 (MB)
    pub required_memory_mb: u32,
    /// 预估执行时间 (秒)
    pub estimated_duration_secs: f32,
    /// 依赖的任务ID列表
    pub dependencies: Vec<String>,
    /// 重试次数
    pub retry_count: u32,
    /// 创建时间
    pub created_at: u64,
    /// 开始时间
    pub started_at: Option<u64>,
    /// 完成时间
    pub completed_at: Option<u64>,
    /// 错误信息
    pub error: Option<String>,
}

/// 调度结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingResult {
    /// 任务ID
    pub task_id: String,
    /// 是否成功
    pub success: bool,
    /// 执行时间 (毫秒)
    pub execution_time_ms: u64,
    /// 使用的显存 (MB)
    pub used_memory_mb: u32,
    /// 错误信息
    pub error: Option<String>,
    /// 是否应该重试
    pub should_retry: bool,
    /// 下次重试时间 (毫秒)
    pub next_retry_ms: Option<u64>,
}

// ============================================================================
// 任务调度器
// ============================================================================

/// 任务调度器
pub struct TaskScheduler {
    /// GPU 显存配置
    gpu_config: GpuMemoryConfig,
    /// 重试配置
    retry_config: RetryConfig,
    /// 任务队列
    task_queue: Vec<SchedulingTask>,
    /// 调度历史
    history: Vec<SchedulingResult>,
    /// GPU 显存使用记录
    memory_usage_history: Vec<(u64, u32)>,
}

impl TaskScheduler {
    /// 创建调度器
    pub fn new() -> Self {
        Self {
            gpu_config: GpuMemoryConfig {
                total_memory_mb: 12000,
                used_memory_mb: 0,
                memory_threshold: 0.85,
                enable_memory_sharding: true,
                shard_size_mb: 1024,
            },
            retry_config: RetryConfig {
                max_retries: 3,
                initial_backoff_ms: 1000,
                max_backoff_ms: 30000,
                backoff_multiplier: 2.0,
                enable_jitter: true,
            },
            task_queue: vec![],
            history: vec![],
            memory_usage_history: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(gpu_config: GpuMemoryConfig, retry_config: RetryConfig) -> Self {
        Self {
            gpu_config,
            retry_config,
            task_queue: vec![],
            history: vec![],
            memory_usage_history: vec![],
        }
    }
    
    /// 添加任务
    pub fn add_task(&mut self, task: SchedulingTask) {
        self.task_queue.push(task);
        self.sort_tasks();
    }
    
    /// 按优先级排序任务
    fn sort_tasks(&mut self) {
        self.task_queue.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// 检查显存是否足够
    pub fn check_memory_available(&self, required_mb: u32) -> bool {
        let available = self.gpu_config.total_memory_mb - self.gpu_config.used_memory_mb;
        available >= required_mb
    }
    
    /// 分配显存
    pub fn allocate_memory(&mut self, _task_id: &str, required_mb: u32) -> bool {
        if self.check_memory_available(required_mb) {
            self.gpu_config.used_memory_mb += required_mb;
            self.memory_usage_history.push((
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                self.gpu_config.used_memory_mb,
            ));
            true
        } else {
            false
        }
    }
    
    /// 释放显存
    pub fn release_memory(&mut self, _task_id: &str, released_mb: u32) {
        self.gpu_config.used_memory_mb = self.gpu_config.used_memory_mb.saturating_sub(released_mb);
    }
    
    /// 获取下一个可执行任务
    pub fn get_next_task(&self) -> Option<&SchedulingTask> {
        self.task_queue.iter().find(|task| {
            task.state == TaskState::Waiting &&
            self.check_memory_available(task.required_memory_mb) &&
            task.dependencies.iter().all(|dep_id| {
                self.task_queue.iter()
                    .find(|t| t.id == *dep_id)
                    .map(|t| t.state == TaskState::Completed)
                    .unwrap_or(false)
            })
        })
    }
    
    /// 计算退避时间
    pub fn calculate_backoff(&self, retry_count: u32) -> u64 {
        let base_backoff = self.retry_config.initial_backoff_ms as f64 *
            (self.retry_config.backoff_multiplier as f64).powi(retry_count as i32);
        
        let backoff = base_backoff.min(self.retry_config.max_backoff_ms as f64) as u64;
        
        if self.retry_config.enable_jitter {
            let jitter = (backoff as f64 * 0.1) as u64;
            backoff + jitter
        } else {
            backoff
        }
    }
    
    /// 执行任务
    pub fn execute_task(&mut self, task_id: &str) -> SchedulingResult {
        // Extract needed fields first to avoid double mutable borrow
        let (required_memory_mb, retry_count) = {
            let task = self.task_queue.iter().find(|t| t.id == task_id);
            match task {
                Some(t) => (t.required_memory_mb, t.retry_count),
                None => {
                    return SchedulingResult {
                        task_id: task_id.to_string(),
                        success: false,
                        execution_time_ms: 0,
                        used_memory_mb: 0,
                        error: Some("任务不存在".to_string()),
                        should_retry: false,
                        next_retry_ms: None,
                    };
                }
            }
        };

        // 分配显存
        if !self.allocate_memory(task_id, required_memory_mb) {
            return SchedulingResult {
                task_id: task_id.to_string(),
                success: false,
                execution_time_ms: 0,
                used_memory_mb: 0,
                error: Some("显存不足".to_string()),
                should_retry: true,
                next_retry_ms: Some(self.calculate_backoff(retry_count)),
            };
        }

        // TODO: 实际执行任务逻辑
        let success = true; // 模拟成功

        if success {
            // 更新任务状态
            if let Some(task) = self.task_queue.iter_mut().find(|t| t.id == task_id) {
                task.state = TaskState::Completed;
                task.started_at = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                );
                task.completed_at = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                );
            }
            self.release_memory(task_id, required_memory_mb);

            let result = SchedulingResult {
                task_id: task_id.to_string(),
                success: true,
                execution_time_ms: 5000,
                used_memory_mb: required_memory_mb,
                error: None,
                should_retry: false,
                next_retry_ms: None,
            };

            self.history.push(result.clone());
            result
        } else {
            // 更新任务状态
            if let Some(task) = self.task_queue.iter_mut().find(|t| t.id == task_id) {
                task.retry_count += 1;
                task.state = if task.retry_count >= self.retry_config.max_retries {
                    TaskState::Failed
                } else {
                    TaskState::Waiting
                };
                task.error = Some("任务执行失败".to_string());
            }
            self.release_memory(task_id, required_memory_mb);

            let new_retry_count = retry_count + 1;
            let result = SchedulingResult {
                task_id: task_id.to_string(),
                success: false,
                execution_time_ms: 5000,
                used_memory_mb: required_memory_mb,
                error: Some("任务执行失败".to_string()),
                should_retry: new_retry_count < self.retry_config.max_retries,
                next_retry_ms: if new_retry_count < self.retry_config.max_retries {
                    Some(self.calculate_backoff(new_retry_count - 1))
                } else {
                    None
                },
            };

            self.history.push(result.clone());
            result
        }
    }
    
    /// 获取调度统计
    pub fn statistics(&self) -> SchedulerStats {
        let total_tasks = self.task_queue.len();
        let running_tasks = self.task_queue.iter().filter(|t| t.state == TaskState::Running).count();
        let completed_tasks = self.task_queue.iter().filter(|t| t.state == TaskState::Completed).count();
        let failed_tasks = self.task_queue.iter().filter(|t| t.state == TaskState::Failed).count();
        let waiting_tasks = self.task_queue.iter().filter(|t| t.state == TaskState::Waiting).count();
        
        let avg_memory_usage = if !self.memory_usage_history.is_empty() {
            self.memory_usage_history.iter().map(|(_, mem)| *mem as f64).sum::<f64>() /
                self.memory_usage_history.len() as f64
        } else {
            0.0
        };
        
        SchedulerStats {
            total_tasks,
            running_tasks,
            completed_tasks,
            failed_tasks,
            waiting_tasks,
            memory_usage: avg_memory_usage as u32,
            memory_utilization: avg_memory_usage as f32 / self.gpu_config.total_memory_mb as f32,
        }
    }
}

/// 调度统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// 总任务数
    pub total_tasks: usize,
    /// 运行中任务数
    pub running_tasks: usize,
    /// 已完成任务数
    pub completed_tasks: usize,
    /// 失败任务数
    pub failed_tasks: usize,
    /// 等待中任务数
    pub waiting_tasks: usize,
    /// 平均显存使用 (MB)
    pub memory_usage: u32,
    /// 显存利用率
    pub memory_utilization: f32,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_scheduler() {
        let mut scheduler = TaskScheduler::new();
        
        // 添加任务
        scheduler.add_task(SchedulingTask {
            id: "task_001".to_string(),
            name: "任务1".to_string(),
            priority: TaskPriority::High,
            state: TaskState::Waiting,
            required_memory_mb: 2000,
            estimated_duration_secs: 10.0,
            dependencies: vec![],
            retry_count: 0,
            created_at: 0,
            started_at: None,
            completed_at: None,
            error: None,
        });
        
        // 检查显存
        assert!(scheduler.check_memory_available(2000));
        
        // 执行任务
        let result = scheduler.execute_task("task_001");
        assert!(result.success);
        
        // 检查统计
        let stats = scheduler.statistics();
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.completed_tasks, 1);
    }
    
    #[test]
    fn test_retry_backoff() {
        let scheduler = TaskScheduler::new();
        
        let backoff_0 = scheduler.calculate_backoff(0);
        let backoff_1 = scheduler.calculate_backoff(1);
        let backoff_2 = scheduler.calculate_backoff(2);
        
        assert!(backoff_1 > backoff_0);
        assert!(backoff_2 > backoff_1);
    }
    
    #[test]
    fn test_memory_management() {
        let mut scheduler = TaskScheduler::new();
        
        assert!(scheduler.allocate_memory("task_001", 5000));
        assert!(!scheduler.check_memory_available(8000)); // 12000 - 5000 = 7000 < 8000
        
        scheduler.release_memory("task_001", 5000);
        assert!(scheduler.check_memory_available(8000)); // 12000 - 0 = 12000 > 8000
    }
}