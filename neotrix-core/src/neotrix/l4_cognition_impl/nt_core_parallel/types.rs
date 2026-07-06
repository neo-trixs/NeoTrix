//! 并行模块基础类型

use serde::{Deserialize, Serialize};

pub type TaskId = u64;
pub type AgentId = String;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub agent_id: AgentId,
    pub input: Vec<f64>,      // 矩阵化输入
    pub state: TaskState,
    pub output: Option<Vec<f64>>,  // 矩阵化输出
    pub priority: i32,
    pub created_at: i64,
}

impl Task {
    pub fn new(agent_id: AgentId, input: Vec<f64>, priority: i32) -> Self {
        Self {
            id: rand::random(),
            agent_id,
            input,
            state: TaskState::Pending,
            output: None,
            priority,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// Agent 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub busy: bool,
    pub current_task: Option<TaskId>,
    pub capability: Vec<f64>,  // 能力向量
    pub throughput: f64,       // 吞吐量
}

impl Agent {
    pub fn new(id: AgentId) -> Self {
        Self {
            id,
            busy: false,
            current_task: None,
            capability: vec![1.0; 256],  // 默认能力向量
            throughput: 1.0,
        }
    }

    pub fn with_capability(mut self, cap: Vec<f64>) -> Self {
        self.capability = cap;
        self
    }

    pub fn assign_task(&mut self, task_id: TaskId) {
        self.busy = true;
        self.current_task = Some(task_id);
    }

    pub fn complete_task(&mut self) {
        self.busy = false;
        self.current_task = None;
    }
}

/// Agent Pool - 矩阵运算驱动调度
pub struct AgentPool {
    pub agents: Vec<Agent>,
    pub max_concurrent: usize,
}

impl AgentPool {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            agents: Vec::new(),
            max_concurrent,
        }
    }

    /// 注册 Agent
    pub fn register(&mut self, agent: Agent) {
        if self.agents.len() < self.max_concurrent {
            self.agents.push(agent);
        }
    }

    /// 分配任务 - 矩阵相似度驱动
    pub fn assign(&mut self, task: &Task) -> Option<AgentId> {
        // 找到最空闲的agent
        let idle: Vec<_> = self.agents.iter_mut()
            .filter(|a| !a.busy)
            .collect();

        if let Some(agent) = idle.into_iter().max_by_key(|a| (a.throughput * 1000.0) as i32) {
            let agent_id = agent.id.clone();
            agent.assign_task(task.id);
            return Some(agent_id);
        }
        None
    }

    /// 完成任务
    pub fn complete(&mut self, agent_id: &str) {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.complete_task();
        }
    }

    /// 获取空闲数
    pub fn idle_count(&self) -> usize {
        self.agents.iter().filter(|a| !a.busy).count()
    }
}

/// Todo List 任务定义 (用于自动并行执行)
#[derive(Debug, Clone)]
pub struct TodoTask {
    pub id: String,
    pub description: String,
    pub task_type: String,  // 任务类型标识
    pub priority: i32,
    pub dependencies: Vec<String>,  // 依赖的任务ID
    pub estimated_complexity: f64,  // 预估复杂度 (0.0-1.0)
}

impl TodoTask {
    pub fn new(id: String, description: String, task_type: String) -> Self {
        Self {
            id,
            description,
            task_type,
            priority: 0,
            dependencies: Vec::new(),
            estimated_complexity: 0.5,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn with_complexity(mut self, complexity: f64) -> Self {
        self.estimated_complexity = complexity.clamp(0.0, 1.0);
        self
    }
}

/// 任务分配策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationStrategy {
    /// 能力向量相似度优先 (默认)
    CapabilityFirst,
    /// 负载均衡优先
    LoadBalance,
    /// 吞吐量优先
    ThroughputFirst,
    /// 混合策略 (PARCO-inspired)
    Hybrid,
}

/// Agent消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    TaskAssigned { task_id: TaskId, agent_id: AgentId },
    TaskCompleted { task_id: TaskId, output: Vec<f64> },
    TaskFailed { task_id: TaskId, error: String },
    AgentBusy { agent_id: AgentId },
    AgentIdle { agent_id: AgentId },
}

/// Agent历史性能记录
#[derive(Debug, Clone)]
pub struct AgentPerformance {
    pub agent_id: AgentId,
    pub success_count: usize,
    pub failure_count: usize,
    pub avg_latency_ms: f64,
}
