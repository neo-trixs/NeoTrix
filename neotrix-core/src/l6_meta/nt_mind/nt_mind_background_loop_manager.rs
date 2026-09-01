//! Background Loop Manager — 背景循环管理器
//!
//! 吸收 KB 经验:
//! - 自主进化循环管理
//! - 进程监控
//! - 资源控制
//! - 错误恢复
//! - 优雅停止

use serde::{Deserialize, Serialize};

/// 背景循环管理器
pub struct BackgroundLoopManager {
    loops: Vec<BackgroundLoop>,
    config: LoopConfig,
    stats: LoopStats,
}

/// 循环配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    pub max_concurrent_loops: usize,
    pub default_interval: u64,
    pub enable_resource_limits: bool,
    pub max_memory_mb: u64,
    pub max_cpu_percent: f64,
    pub graceful_shutdown_timeout: u64,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_concurrent_loops: 3,
            default_interval: 60,
            enable_resource_limits: true,
            max_memory_mb: 512,
            max_cpu_percent: 50.0,
            graceful_shutdown_timeout: 30,
        }
    }
}

/// 背景循环
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundLoop {
    pub loop_id: String,
    pub name: String,
    pub loop_type: LoopType,
    pub status: LoopStatus,
    pub interval: u64,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run: Option<chrono::DateTime<chrono::Utc>>,
    pub resource_usage: ResourceUsage,
}

/// 循环类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoopType {
    Evolution,      // 进化循环
    HealthCheck,    // 健康检查
    KnowledgeSync,  // 知识同步
    BuildMonitor,   // 构建监控
    Custom,
}

/// 循环状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoopStatus {
    Running,
    Paused,
    Stopped,
    Error,
}

/// 资源使用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_mb: f64,
    pub cpu_percent: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// 循环统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStats {
    pub total_loops: u64,
    pub running_loops: u64,
    pub total_runs: u64,
    pub successful_runs: u64,
    pub failed_runs: u64,
    pub avg_duration: f64,
}

/// 循环事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopEvent {
    pub event_id: String,
    pub loop_id: String,
    pub event_type: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl BackgroundLoopManager {
    /// 创建新的背景循环管理器
    pub fn new() -> Self {
        Self {
            loops: Vec::new(),
            config: LoopConfig::default(),
            stats: LoopStats {
                total_loops: 0,
                running_loops: 0,
                total_runs: 0,
                successful_runs: 0,
                failed_runs: 0,
                avg_duration: 0.0,
            },
        }
    }

    /// 创建新的背景循环
    pub fn create_loop(&mut self, name: &str, loop_type: LoopType, interval: u64) -> String {
        let loop_id = uuid::Uuid::new_v4().to_string();

        let loop_obj = BackgroundLoop {
            loop_id: loop_id.clone(),
            name: name.to_string(),
            loop_type,
            status: LoopStatus::Stopped,
            interval,
            last_run: None,
            next_run: None,
            resource_usage: ResourceUsage {
                memory_mb: 0.0,
                cpu_percent: 0.0,
                last_updated: chrono::Utc::now(),
            },
        };

        self.loops.push(loop_obj);
        self.stats.total_loops += 1;

        loop_id
    }

    /// 启动循环
    pub fn start_loop(&mut self, loop_id: &str) -> bool {
        if let Some(loop_obj) = self.loops.iter_mut().find(|l| l.loop_id == loop_id) {
            if self.stats.running_loops >= self.config.max_concurrent_loops as u64 {
                return false;
            }

            loop_obj.status = LoopStatus::Running;
            loop_obj.next_run = Some(chrono::Utc::now() + chrono::Duration::seconds(loop_obj.interval as i64));
            self.stats.running_loops += 1;
            true
        } else {
            false
        }
    }

    /// 停止循环
    pub fn stop_loop(&mut self, loop_id: &str) -> bool {
        if let Some(loop_obj) = self.loops.iter_mut().find(|l| l.loop_id == loop_id) {
            loop_obj.status = LoopStatus::Stopped;
            loop_obj.next_run = None;
            self.stats.running_loops -= 1;
            true
        } else {
            false
        }
    }

    /// 暂停循环
    pub fn pause_loop(&mut self, loop_id: &str) -> bool {
        if let Some(loop_obj) = self.loops.iter_mut().find(|l| l.loop_id == loop_id) {
            loop_obj.status = LoopStatus::Paused;
            true
        } else {
            false
        }
    }

    /// 检查资源使用
    pub fn check_resources(&self, loop_id: &str) -> Option<ResourceUsage> {
        self.loops.iter()
            .find(|l| l.loop_id == loop_id)
            .map(|l| l.resource_usage.clone())
    }

    /// 获取所有循环
    pub fn loops(&self) -> &[BackgroundLoop] {
        &self.loops
    }

    /// 获取统计信息
    pub fn stats(&self) -> &LoopStats {
        &self.stats
    }

    /// 清理已停止的循环
    pub fn cleanup_stopped(&mut self) {
        self.loops.retain(|l| l.status != LoopStatus::Stopped);
    }
}
