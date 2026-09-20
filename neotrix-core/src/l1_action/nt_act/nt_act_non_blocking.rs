//! T2.8: 非阻塞子代理生成模块 (pi-crew A6)
//!
//! SubAgentSpawner: 异步任务委派 + 并行执行
//! TaskHandle: 结果检索

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Agent Archetype
// ============================================================================

/// 代理原型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentArchetype {
    pub name: String,
    pub role: String,
    pub tools: Vec<String>,
    pub model_preference: String,
}

// ============================================================================
// SpawnConfig
// ============================================================================

/// 生成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnConfig {
    /// 最大并行任务数
    pub max_parallelism: usize,
    /// 单任务超时 (ms)
    pub task_timeout_ms: u64,
    /// 任务原型名
    pub archetype: String,
    /// 额外上下文
    pub context: String,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            max_parallelism: 4,
            task_timeout_ms: 30_000,
            archetype: "default".to_string(),
            context: String::new(),
        }
    }
}

// ============================================================================
// TaskHandle
// ============================================================================

/// 任务句柄 — 用于结果检索
#[derive(Debug)]
pub struct TaskHandle {
    /// 任务唯一 ID
    pub task_id: String,
    /// 任务输入
    pub task: String,
    /// 使用的原型
    pub archetype: String,
    /// 是否已完成
    completed: bool,
    /// 存储的结果
    result: Option<AgentResult>,
}

impl TaskHandle {
    /// 创建新句柄
    pub fn new(task_id: String, task: String, archetype: String) -> Self {
        Self {
            task_id,
            task,
            archetype,
            completed: false,
            result: None,
        }
    }

    /// 是否已完成
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// 获取结果 (仅已完成)
    pub fn result(&self) -> Option<&AgentResult> {
        self.result.as_ref()
    }

    /// 设置结果 (内部使用)
    pub(crate) fn set_result(&mut self, result: AgentResult) {
        self.result = Some(result);
        self.completed = true;
    }
}

// ============================================================================
// AgentResult
// ============================================================================

/// 代理执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub task_id: String,
    pub success: bool,
    pub output: String,
    pub tokens_used: u64,
    pub cost: f64,
}

// ============================================================================
// SpawnResult
// ============================================================================

/// 生成结果 (立即返回)
#[derive(Debug)]
pub struct SpawnResult {
    pub task_id: String,
    pub handle: TaskHandle,
}

// ============================================================================
// SubAgentSpawner
// ============================================================================

/// 非阻塞子代理生成器
pub struct SubAgentSpawner {
    /// 代理原型注册表
    archetypes: HashMap<String, AgentArchetype>,
    /// 活跃任务追踪
    active_tasks: HashMap<String, TaskHandle>,
    /// 最大并行数
    max_parallelism: usize,
}

impl SubAgentSpawner {
    pub fn new(max_parallelism: usize) -> Self {
        Self {
            archetypes: HashMap::new(),
            active_tasks: HashMap::new(),
            max_parallelism,
        }
    }

    /// 注册代理原型
    pub fn register(&mut self, archetype: AgentArchetype) {
        self.archetypes.insert(archetype.name.clone(), archetype);
    }

    /// 非阻塞生成 (立即返回 TaskHandle)
    pub fn spawn(
        &mut self,
        archetype_name: &str,
        task: String,
        _config: &SpawnConfig,
    ) -> Result<SpawnResult, SpawnError> {
        if self.active_tasks.len() >= self.max_parallelism {
            return Err(SpawnError::MaxParallelismReached {
                current: self.active_tasks.len(),
                max: self.max_parallelism,
            });
        }

        if !self.archetypes.contains_key(archetype_name) {
            return Err(SpawnError::UnknownArchetype {
                name: archetype_name.to_string(),
            });
        }

        let task_id = self.next_task_id();
        let handle = TaskHandle::new(task_id.clone(), task, archetype_name.to_string());

        self.active_tasks.insert(task_id.clone(), handle);

        Ok(SpawnResult {
            task_id: task_id.clone(),
            handle: self.active_tasks.get(&task_id).unwrap().clone_task_handle(),
        })
    }

    /// 获取任务句柄
    pub fn get_handle(&self, task_id: &str) -> Option<&TaskHandle> {
        self.active_tasks.get(task_id)
    }

    /// 完成任务 (标记完成 + 存储结果)
    pub fn complete(&mut self, task_id: &str, result: AgentResult) -> Option<AgentResult> {
        if let Some(handle) = self.active_tasks.get_mut(task_id) {
            handle.set_result(result.clone());
            Some(result)
        } else {
            None
        }
    }

    /// 清理已完成任务
    pub fn cleanup_completed(&mut self) -> Vec<String> {
        let completed: Vec<String> = self
            .active_tasks
            .iter()
            .filter(|(_, h)| h.is_completed())
            .map(|(id, _)| id.clone())
            .collect();
        for id in &completed {
            self.active_tasks.remove(id);
        }
        completed
    }

    /// 当前活跃任务数
    pub fn active_count(&self) -> usize {
        self.active_tasks.len()
    }

    /// 是否已满
    pub fn is_full(&self) -> bool {
        self.active_tasks.len() >= self.max_parallelism
    }

    fn next_task_id(&self) -> String {
        format!("task-{}", self.active_tasks.len() + 1)
    }
}

impl TaskHandle {
    fn clone_task_handle(&self) -> TaskHandle {
        TaskHandle {
            task_id: self.task_id.clone(),
            task: self.task.clone(),
            archetype: self.archetype.clone(),
            completed: self.completed,
            result: self.result.clone(),
        }
    }
}

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpawnError {
    MaxParallelismReached { current: usize, max: usize },
    UnknownArchetype { name: String },
}

impl std::fmt::Display for SpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxParallelismReached { current, max } => {
                write!(f, "Max parallelism reached: {}/{}", current, max)
            }
            Self::UnknownArchetype { name } => {
                write!(f, "Unknown archetype: {}", name)
            }
        }
    }
}

impl std::error::Error for SpawnError {}

// ============================================================================
// ToolDelta
// ============================================================================

/// 工具增量: +tool 模式 (不替换现有工具列表)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDelta {
    pub additions: Vec<String>,
    pub removals: Vec<String>,
}

impl ToolDelta {
    /// 应用增量到工具列表
    pub fn apply(&self, current: &[String]) -> Vec<String> {
        let mut result: Vec<String> = current.to_vec();
        result.extend(self.additions.clone());
        result.retain(|t| !self.removals.contains(t));
        result
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawner_register_and_spawn() {
        let mut spawner = SubAgentSpawner::new(2);
        spawner.register(AgentArchetype {
            name: "coder".to_string(),
            role: "write code".to_string(),
            tools: vec!["bash".into()],
            model_preference: "fast".into(),
        });

        let config = SpawnConfig::default();
        let result = spawner.spawn("coder", "write tests".into(), &config);
        assert!(result.is_ok());
        assert_eq!(spawner.active_count(), 1);
    }

    #[test]
    fn test_max_parallelism() {
        let mut spawner = SubAgentSpawner::new(1);
        spawner.register(AgentArchetype {
            name: "a".to_string(),
            role: "r".to_string(),
            tools: vec![],
            model_preference: "m".into(),
        });

        let config = SpawnConfig::default();
        let _ = spawner.spawn("a", "t1".into(), &config);
        let result = spawner.spawn("a", "t2".into(), &config);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpawnError::MaxParallelismReached { .. } => {}
            _ => panic!("Expected MaxParallelismReached"),
        }
    }

    #[test]
    fn test_unknown_archetype() {
        let mut spawner = SubAgentSpawner::new(4);
        let config = SpawnConfig::default();
        let result = spawner.spawn("nonexistent", "t".into(), &config);
        assert!(matches!(result.unwrap_err(), SpawnError::UnknownArchetype { .. }));
    }

    #[test]
    fn test_tool_delta() {
        let delta = ToolDelta {
            additions: vec!["tool_c".into()],
            removals: vec!["tool_a".into()],
        };
        let current = vec!["tool_a".into(), "tool_b".into()];
        let result = delta.apply(&current);
        assert_eq!(result, vec!["tool_b".to_string(), "tool_c".to_string()]);
    }

    #[test]
    fn test_complete_task() {
        let mut spawner = SubAgentSpawner::new(4);
        spawner.register(AgentArchetype {
            name: "a".to_string(),
            role: "r".to_string(),
            tools: vec![],
            model_preference: "m".into(),
        });

        let config = SpawnConfig::default();
        let res = spawner.spawn("a", "t".into(), &config).unwrap();
        let task_id = res.task_id.clone();

        let result = AgentResult {
            task_id: task_id.clone(),
            success: true,
            output: "done".into(),
            tokens_used: 100,
            cost: 0.01,
        };
        spawner.complete(&task_id, result);

        let handle = spawner.get_handle(&task_id).unwrap();
        assert!(handle.is_completed());
    }
}
