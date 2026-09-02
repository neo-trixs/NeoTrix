//! 并行任务管理模块 (通用)
//!
//! 管理 GPU 显存、批量调度、指数退避重试
//! 适用于：所有 AI 推理和生成场景

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 任务定义
// ============================================================================

/// 任务状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// 等待中
    Pending,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 完成
    Completed,
    /// 失败
    Failed,
    /// 取消
    Cancelled,
}

/// 任务优先级
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// 低
    Low = 0,
    /// 中
    Medium = 1,
    /// 高
    High = 2,
    /// 紧急
    Critical = 3,
}

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务类型
    pub task_type: String,
    /// 优先级
    pub priority: TaskPriority,
    /// 状态
    pub status: TaskStatus,
    /// GPU 显存需求 (MB)
    pub gpu_memory_mb: u64,
    /// 预计执行时间 (秒)
    pub estimated_duration_secs: u32,
    /// 依赖的任务ID列表
    pub dependencies: Vec<String>,
    /// 任务参数
    pub params: HashMap<String, serde_json::Value>,
    /// 最大重试次数
    pub max_retries: u32,
    /// 当前重试次数
    pub current_retries: u32,
    /// 超时时间 (秒)
    pub timeout_secs: u32,
}

/// GPU 设备状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUDevice {
    /// 设备ID
    pub device_id: u32,
    /// 设备名称
    pub name: String,
    /// 总显存 (MB)
    pub total_memory_mb: u64,
    /// 已用显存 (MB)
    pub used_memory_mb: u64,
    /// 利用率 (0.0-1.0)
    pub utilization: f32,
    /// 温度 (摄氏度)
    pub temperature: f32,
    /// 是否可用
    pub available: bool,
}

impl GPUDevice {
    /// 可用显存
    pub fn available_memory_mb(&self) -> u64 {
        self.total_memory_mb.saturating_sub(self.used_memory_mb)
    }
    
    /// 检查是否满足任务需求
    pub fn can_run_task(&self, task: &Task) -> bool {
        self.available && self.available_memory_mb() >= task.gpu_memory_mb
    }
}

/// 任务调度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// 最大并行任务数
    pub max_parallel_tasks: u32,
    /// 最大 GPU 显存使用率
    pub max_gpu_utilization: f32,
    /// 重试间隔基数 (秒)
    pub retry_interval_base_secs: u32,
    /// 重试间隔最大倍数
    pub retry_max_multiplier: u32,
    /// 是否启用负载均衡
    pub enable_load_balancing: bool,
    /// 是否启用抢占式调度
    pub enable_preemptive_scheduling: bool,
    /// 调度算法
    pub scheduling_algorithm: SchedulingAlgorithm,
}

/// 调度算法
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SchedulingAlgorithm {
    /// 先来先服务
    FCFS,
    /// 最短作业优先
    SJF,
    /// 优先级调度
    Priority,
    /// 轮转调度
    RoundRobin,
    /// 多级反馈队列
    MLFQ,
}

/// 任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// 任务ID
    pub task_id: String,
    /// 是否成功
    pub success: bool,
    /// 输出数据
    pub output: Option<serde_json::Value>,
    /// 执行耗时 (毫秒)
    pub execution_time_ms: u64,
    /// GPU 显存使用峰值 (MB)
    pub peak_gpu_memory_mb: u64,
    /// 重试次数
    pub retries: u32,
    /// 错误信息
    pub error: Option<String>,
}

// ============================================================================
// 并行任务管理器
// ============================================================================

/// 并行任务管理器
/// 管理 GPU 资源和任务调度
pub struct ParallelTaskManager {
    /// 调度配置
    config: SchedulerConfig,
    /// GPU 设备列表
    devices: Vec<GPUDevice>,
    /// 任务队列
    task_queue: Vec<Task>,
    /// 运行中的任务
    running_tasks: HashMap<String, Task>,
    /// 完成的任务
    completed_tasks: Vec<TaskResult>,
}

impl ParallelTaskManager {
    /// 创建管理器
    pub fn new() -> Self {
        Self {
            config: SchedulerConfig {
                max_parallel_tasks: 4,
                max_gpu_utilization: 0.9,
                retry_interval_base_secs: 5,
                retry_max_multiplier: 10,
                enable_load_balancing: true,
                enable_preemptive_scheduling: false,
                scheduling_algorithm: SchedulingAlgorithm::Priority,
            },
            devices: vec![],
            task_queue: vec![],
            running_tasks: HashMap::new(),
            completed_tasks: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: SchedulerConfig) -> Self {
        Self {
            config,
            devices: vec![],
            task_queue: vec![],
            running_tasks: HashMap::new(),
            completed_tasks: vec![],
        }
    }
    
    /// 注册 GPU 设备
    pub fn register_device(&mut self, device: GPUDevice) {
        self.devices.push(device);
    }
    
    /// 提交任务
    pub fn submit_task(&mut self, task: Task) {
        self.task_queue.push(task);
        self.task_queue.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// 调度下一个任务
    pub fn schedule_next(&mut self) -> Option<String> {
        // 检查并行限制
        if self.running_tasks.len() as u32 >= self.config.max_parallel_tasks {
            return None;
        }
        
        // 查找可运行的任务
        for task in &self.task_queue {
            // 检查依赖是否满足
            let deps_met = task.dependencies.iter()
                .all(|dep| self.completed_tasks.iter().any(|r| &r.task_id == dep && r.success));
            
            if !deps_met {
                continue;
            }
            
            // 查找可用 GPU
            if let Some(device) = self.devices.iter_mut()
                .find(|d| d.can_run_task(task)) 
            {
                // 分配显存
                device.used_memory_mb += task.gpu_memory_mb;
                
                // 从队列移除并加入运行中
                if let Some(pos) = self.task_queue.iter().position(|t| t.id == task.id) {
                    let mut task = self.task_queue.remove(pos);
                    task.status = TaskStatus::Running;
                    let task_id = task.id.clone();
                    self.running_tasks.insert(task_id.clone(), task);
                    return Some(task_id);
                }
            }
        }
        
        None
    }
    
    /// 完成任务
    pub fn complete_task(&mut self, task_id: &str, result: TaskResult) {
        // 释放 GPU 显存
        if let Some(task) = self.running_tasks.remove(task_id) {
            if let Some(device) = self.devices.iter_mut().find(|d| {
                d.used_memory_mb >= task.gpu_memory_mb
            }) {
                device.used_memory_mb = device.used_memory_mb.saturating_sub(task.gpu_memory_mb);
            }
        }
        
        // 失败重试
        if !result.success {
            if let Some(mut task) = self.running_tasks.remove(task_id) {
                task.current_retries += 1;
                if task.current_retries < task.max_retries {
                    task.status = TaskStatus::Pending;
                    // 指数退避重试
                    let delay = self.config.retry_interval_base_secs
                        * (2u32.pow(task.current_retries).min(self.config.retry_max_multiplier));
                    // TODO: 实际实现延迟调度
                    self.task_queue.push(task);
                } else {
                    self.completed_tasks.push(result);
                    return;
                }
            }
        } else {
            self.completed_tasks.push(result);
        }
    }
    
    /// 获取统计信息
    pub fn statistics(&self) -> SchedulerStats {
        let queued = self.task_queue.len();
        let running = self.running_tasks.len();
        let completed = self.completed_tasks.iter().filter(|r| r.success).count();
        let failed = self.completed_tasks.iter().filter(|r| !r.success).count();
        let total_memory_mb: u64 = self.devices.iter().map(|d| d.total_memory_mb).sum();
        let used_memory_mb: u64 = self.devices.iter().map(|d| d.used_memory_mb).sum();
        
        SchedulerStats {
            queued_tasks: queued,
            running_tasks: running,
            completed_tasks: completed,
            failed_tasks: failed,
            total_gpu_memory_mb: total_memory_mb,
            used_gpu_memory_mb: used_memory_mb,
            gpu_utilization: if total_memory_mb > 0 {
                used_memory_mb as f32 / total_memory_mb as f32
            } else {
                0.0
            },
        }
    }
    
    /// 计算指数退避延迟
    pub fn calculate_backoff_delay(&self, retry_count: u32) -> u32 {
        self.config.retry_interval_base_secs
            * (2u32.pow(retry_count).min(self.config.retry_max_multiplier))
    }
}

/// 调度统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// 排队任务数
    pub queued_tasks: usize,
    /// 运行任务数
    pub running_tasks: usize,
    /// 完成任务数
    pub completed_tasks: usize,
    /// 失败任务数
    pub failed_tasks: usize,
    /// 总 GPU 显存
    pub total_gpu_memory_mb: u64,
    /// 已用 GPU 显存
    pub used_gpu_memory_mb: u64,
    /// GPU 利用率
    pub gpu_utilization: f32,
}

// ============================================================================
// 向后兼容别名
// ============================================================================

/// 任务调度器 (向后兼容别名)
pub type TaskScheduler = ParallelTaskManager;

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_scheduler() {
        let mut scheduler = ParallelTaskManager::new();
        
        // 注册 GPU
        scheduler.register_device(GPUDevice {
            device_id: 0,
            name: "RTX 4090".to_string(),
            total_memory_mb: 24000,
            used_memory_mb: 0,
            utilization: 0.0,
            temperature: 35.0,
            available: true,
        });
        
        // 提交任务
        scheduler.submit_task(Task {
            id: "task_001".to_string(),
            name: "测试任务".to_string(),
            task_type: "inference".to_string(),
            priority: TaskPriority::High,
            status: TaskStatus::Pending,
            gpu_memory_mb: 4000,
            estimated_duration_secs: 60,
            dependencies: vec![],
            params: HashMap::new(),
            max_retries: 3,
            current_retries: 0,
            timeout_secs: 300,
        });
        
        // 调度
        let task_id = scheduler.schedule_next();
        assert!(task_id.is_some());
        
        // 完成任务
        scheduler.complete_task(&task_id.unwrap(), TaskResult {
            task_id: "task_001".to_string(),
            success: true,
            output: None,
            execution_time_ms: 50000,
            peak_gpu_memory_mb: 3500,
            retries: 0,
            error: None,
        });
        
        let stats = scheduler.statistics();
        assert_eq!(stats.completed_tasks, 1);
    }
    
    #[test]
    fn test_backoff_delay() {
        let scheduler = ParallelTaskManager::new();
        
        assert_eq!(scheduler.calculate_backoff_delay(0), 5);
        assert_eq!(scheduler.calculate_backoff_delay(1), 10);
        assert_eq!(scheduler.calculate_backoff_delay(2), 20);
        assert_eq!(scheduler.calculate_backoff_delay(3), 40);
    }
}