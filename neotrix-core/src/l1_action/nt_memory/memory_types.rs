//! 三类记忆 — 基于 Grok Build 模式
//!
//! Workflow/Subtask/Function 三种记忆类型。

use std::collections::HashMap;

/// 工作流记忆（完整的任务执行流程）
#[derive(Clone, Debug)]
pub(crate) struct WorkflowMemory {
    pub id: String,
    pub task: String,
    pub steps: Vec<WorkflowStep>,
    pub success: bool,
    pub total_tokens: u32,
    pub created_at: i64,
}

/// 工作流步骤
#[derive(Clone, Debug)]
pub struct WorkflowStep {
    pub action: String,
    pub tool: String,
    pub input_summary: String,
    pub output_summary: String,
    pub success: bool,
}

/// 子任务记忆（单个子任务的执行记录）
#[derive(Clone, Debug)]
pub(crate) struct SubtaskMemory {
    pub id: String,
    pub parent_workflow: String,
    pub task: String,
    pub result: String,
    pub success: bool,
    pub tokens_used: u32,
    pub created_at: i64,
}

/// 函数记忆（工具/函数的使用模式）
#[derive(Clone, Debug)]
pub(crate) struct FunctionMemory {
    pub function_name: String,
    pub call_count: u32,
    pub success_count: u32,
    pub avg_latency_ms: u64,
    pub common_inputs: Vec<String>,
    pub common_outputs: Vec<String>,
}

/// 三类记忆存储
#[derive(Debug)]
pub(crate) struct TripleMemoryStore {
    workflows: Vec<WorkflowMemory>,
    subtasks: Vec<SubtaskMemory>,
    functions: HashMap<String, FunctionMemory>,
}

impl TripleMemoryStore {
    pub fn new() -> Self {
        Self {
            workflows: Vec::new(),
            subtasks: Vec::new(),
            functions: HashMap::new(),
        }
    }

    /// 记录工作流
    pub fn record_workflow(&mut self, workflow: WorkflowMemory) {
        self.workflows.push(workflow);
    }

    /// 记录子任务
    pub fn record_subtask(&mut self, subtask: SubtaskMemory) {
        self.subtasks.push(subtask);
    }

    /// 记录函数调用
    pub fn record_function_call(&mut self, name: &str, success: bool, latency_ms: u64) {
        let func = self
            .functions
            .entry(name.to_string())
            .or_insert_with(|| FunctionMemory {
                function_name: name.to_string(),
                call_count: 0,
                success_count: 0,
                avg_latency_ms: 0,
                common_inputs: Vec::new(),
                common_outputs: Vec::new(),
            });
        func.call_count += 1;
        if success {
            func.success_count += 1;
        }
        func.avg_latency_ms = (func.avg_latency_ms * (func.call_count - 1) as u64 + latency_ms)
            / func.call_count as u64;
    }

    /// 搜索工作流
    pub fn search_workflows(&self, query: &str) -> Vec<&WorkflowMemory> {
        self.workflows
            .iter()
            .filter(|w| w.task.contains(query))
            .collect()
    }

    /// 获取函数统计
    pub fn function_stats(&self, name: &str) -> Option<(&str, u32, u32, f64)> {
        self.functions.get(name).map(|f| {
            let success_rate = if f.call_count > 0 {
                f.success_count as f64 / f.call_count as f64
            } else {
                0.0
            };
            (
                f.function_name.as_str(),
                f.call_count,
                f.success_count,
                success_rate,
            )
        })
    }

    /// 总统计
    pub fn stats(&self) -> (usize, usize, usize) {
        (
            self.workflows.len(),
            self.subtasks.len(),
            self.functions.len(),
        )
    }
}
