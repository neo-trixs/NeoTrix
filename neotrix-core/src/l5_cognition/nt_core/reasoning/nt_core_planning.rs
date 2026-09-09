//! Planning Engine Enhanced — 规划引擎增强
//!
//! 吸收 LLM Agent Papers (规划/工具使用):
//! - 分层规划
//! - 任务分解
//! - 资源分配
//! - 执行监控
//! - 动态调整

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 规划引擎
#[allow(dead_code)]
pub struct PlanningEngine {
    plans: HashMap<String, Plan>,
    task_graph: TaskGraph,
    resource_manager: ResourceManager,
    config: PlanningConfig,
    stats: PlanningStats,
}

/// 规划配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningConfig {
    pub max_depth: u32,
    pub max_tasks: usize,
    pub enable_dynamic_planning: bool,
    pub enable_resource_allocation: bool,
    pub planning_horizon: u32,
}

impl Default for PlanningConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            max_tasks: 100,
            enable_dynamic_planning: true,
            enable_resource_allocation: true,
            planning_horizon: 10,
        }
    }
}

/// 计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub goal: String,
    pub sub_plans: Vec<Plan>,
    pub tasks: Vec<PlannedTask>,
    pub dependencies: Vec<Dependency>,
    pub estimated_duration: u64,
    pub priority: u32,
    pub status: PlanStatus,
}

/// 计划状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Failed,
}

/// 规划任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTask {
    pub id: String,
    pub name: String,
    pub task_type: String,
    pub description: String,
    pub required_resources: Vec<String>,
    pub estimated_time: u64,
    pub status: TaskStatus,
    pub result: Option<serde_json::Value>,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Blocked,
}

/// 依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub task_id: String,
    pub depends_on: String,
    pub dependency_type: String,
}

/// 任务图
pub struct TaskGraph {
    nodes: HashMap<String, TaskNode>,
    edges: Vec<TaskEdge>,
}

/// 任务节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,
    pub task: PlannedTask,
    pub in_degree: u32,
    pub out_degree: u32,
}

/// 任务边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
}

/// 资源管理器
pub struct ResourceManager {
    resources: HashMap<String, Resource>,
    allocations: Vec<ResourceAllocation>,
}

/// 资源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub resource_type: String,
    pub capacity: f64,
    pub available: f64,
    pub cost_per_unit: f64,
}

/// 资源分配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub resource_id: String,
    pub task_id: String,
    pub amount: f64,
    pub duration: u64,
}

/// 规划统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningStats {
    pub plans_created: u64,
    pub tasks_planned: u64,
    pub tasks_completed: u64,
    pub avg_planning_time: f64,
    pub plan_success_rate: f64,
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub plan_id: String,
    pub status: PlanStatus,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
    pub total_time: u64,
    pub resource_usage: HashMap<String, f64>,
}

impl PlanningEngine {
    /// 创建新的规划引擎
    pub fn new(config: PlanningConfig) -> Self {
        Self {
            plans: HashMap::new(),
            task_graph: TaskGraph {
                nodes: HashMap::new(),
                edges: Vec::new(),
            },
            resource_manager: ResourceManager {
                resources: HashMap::new(),
                allocations: Vec::new(),
            },
            config,
            stats: PlanningStats {
                plans_created: 0,
                tasks_planned: 0,
                tasks_completed: 0,
                avg_planning_time: 0.0,
                plan_success_rate: 0.0,
            },
        }
    }

    /// 创建计划
    pub fn create_plan(&mut self, name: &str, goal: &str) -> Plan {
        let plan = Plan {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            goal: goal.to_string(),
            sub_plans: Vec::new(),
            tasks: Vec::new(),
            dependencies: Vec::new(),
            estimated_duration: 0,
            priority: 1,
            status: PlanStatus::Draft,
        };

        self.plans.insert(plan.id.clone(), plan.clone());
        self.stats.plans_created += 1;
        plan
    }

    /// 添加任务到计划
    pub fn add_task_to_plan(&mut self, plan_id: &str, task: PlannedTask) -> Result<(), String> {
        if let Some(plan) = self.plans.get_mut(plan_id) {
            plan.tasks.push(task.clone());
            plan.estimated_duration += task.estimated_time;
            self.stats.tasks_planned += 1;

            // 添加到任务图
            let node = TaskNode {
                id: task.id.clone(),
                task,
                in_degree: 0,
                out_degree: 0,
            };
            self.task_graph.nodes.insert(node.id.clone(), node);

            Ok(())
        } else {
            Err(format!("Plan {} not found", plan_id))
        }
    }

    /// 添加依赖
    pub fn add_dependency(&mut self, plan_id: &str, dependency: Dependency) -> Result<(), String> {
        if let Some(plan) = self.plans.get_mut(plan_id) {
            plan.dependencies.push(dependency.clone());

            // 更新任务图
            let edge = TaskEdge {
                from: dependency.depends_on,
                to: dependency.task_id,
                edge_type: dependency.dependency_type,
            };
            self.task_graph.edges.push(edge);

            Ok(())
        } else {
            Err(format!("Plan {} not found", plan_id))
        }
    }

    /// 拓扑排序
    pub fn topological_sort(&self, plan_id: &str) -> Vec<String> {
        if let Some(plan) = self.plans.get(plan_id) {
            let mut in_degree: HashMap<String, u32> = HashMap::new();
            let mut queue = Vec::new();
            let mut sorted = Vec::new();

            // 初始化入度
            for task in &plan.tasks {
                in_degree.entry(task.id.clone()).or_insert(0);
            }

            // 计算入度
            for dep in &plan.dependencies {
                *in_degree.entry(dep.task_id.clone()).or_insert(0) += 1;
            }

            // 找到入度为 0 的节点
            for (id, &degree) in &in_degree {
                if degree == 0 {
                    queue.push(id.clone());
                }
            }

            // 拓扑排序
            while let Some(node) = queue.pop() {
                sorted.push(node.clone());

                for dep in &plan.dependencies {
                    if dep.depends_on == node {
                        if let Some(degree) = in_degree.get_mut(&dep.task_id) {
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push(dep.task_id.clone());
                            }
                        }
                    }
                }
            }

            sorted
        } else {
            Vec::new()
        }
    }

    /// 分配资源
    pub fn allocate_resource(&mut self, resource_id: &str, task_id: &str, amount: f64, duration: u64) -> Result<(), String> {
        if let Some(resource) = self.resource_manager.resources.get_mut(resource_id) {
            if resource.available >= amount {
                resource.available -= amount;

                self.resource_manager.allocations.push(ResourceAllocation {
                    resource_id: resource_id.to_string(),
                    task_id: task_id.to_string(),
                    amount,
                    duration,
                });

                Ok(())
            } else {
                Err("Insufficient resources".into())
            }
        } else {
            Err(format!("Resource {} not found", resource_id))
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> &PlanningStats {
        &self.stats
    }
}
