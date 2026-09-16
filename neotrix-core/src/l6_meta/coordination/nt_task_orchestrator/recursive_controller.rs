//! ROMA Recursive Controller — 吸收自 ROMA (Recursive Open Meta-Agents)
//!
//! 递归控制循环: Atomizer → Planner → Executor → Aggregator
//! - Atomizer: 判断任务是否原子
//! - Planner: MECE 子任务图分解
//! - Executor: 并行原子执行
//! - Aggregator: 压缩+验证+返回父节点
//! 每层聚合压缩上下文，防止 context rot。

use serde::{Serialize, Deserialize};

/// 任务节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,
    pub description: String,
    pub is_atomic: bool,
    pub children: Vec<TaskNode>,
    pub result: Option<String>,
    pub depth: usize,
}

/// 递归控制器
pub struct RecursiveController {
    max_depth: usize,
    max_children: usize,
    compression_threshold: usize,
}

impl RecursiveController {
    pub fn new() -> Self {
        Self {
            max_depth: 10,
            max_children: 8,
            compression_threshold: 5,
        }
    }

    /// Atomizer: 判断任务是否原子（无需进一步分解）
    pub fn atomize(&self, task: &str) -> bool {
        // 简单启发: 任务描述短于50字符且不包含分解关键词
        let decomposition_keywords = ["and then", "after that", "first.*then", "multiple", "several"];
        let task_lower = task.to_lowercase();
        
        if task.len() < 50 {
            return true;
        }
        
        !decomposition_keywords.iter().any(|kw| task_lower.contains(kw))
    }

    /// Planner: 将非原子任务分解为 MECE 子任务
    pub fn plan(&self, task: &str, depth: usize) -> Vec<String> {
        if depth >= self.max_depth {
            return vec![task.to_string()];
        }

        // 简单分解: 按分句分割
        let subtasks: Vec<String> = task.split(|c| c == ';' || c == '\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .take(self.max_children)
            .collect();

        if subtasks.is_empty() {
            vec![task.to_string()]
        } else {
            subtasks
        }
    }

    /// Aggregator: 压缩子任务结果
    pub fn aggregate(&self, results: Vec<String>) -> String {
        if results.len() <= self.compression_threshold {
            // 少量结果: 直接合并
            results.join("\n")
        } else {
            // 大量结果: 压缩为摘要
            let total = results.len();
            let kept = self.compression_threshold;
            let summary = format!("Aggregated {} results, keeping {} key results", total, kept);
            let key_results: Vec<&str> = results.iter().take(kept).map(|s| s.as_str()).collect();
            format!("{}\nKey results:\n{}", summary, key_results.join("\n"))
        }
    }

    /// 递归执行
    pub fn execute_recursive(&self, task: &str) -> TaskNode {
        self.execute_node(task, 0)
    }

    fn execute_node(&self, task: &str, depth: usize) -> TaskNode {
        let node_id = format!("node_{}_{}", depth, task.len());
        let is_atomic = self.atomize(task);

        if is_atomic || depth >= self.max_depth {
            // 原子任务: 直接执行（返回模拟结果）
            TaskNode {
                id: node_id,
                description: task.to_string(),
                is_atomic: true,
                children: vec![],
                result: Some(format!("Executed: {}", task)),
                depth,
            }
        } else {
            // 非原子任务: 分解并递归
            let subtasks = self.plan(task, depth);
            let children: Vec<TaskNode> = subtasks.iter()
                .map(|st| self.execute_node(st, depth + 1))
                .collect();

            let child_results: Vec<String> = children.iter()
                .filter_map(|c| c.result.clone())
                .collect();

            let aggregated = self.aggregate(child_results);

            TaskNode {
                id: node_id,
                description: task.to_string(),
                is_atomic: false,
                children,
                result: Some(aggregated),
                depth,
            }
        }
    }

    /// 获取执行树的扁平化追踪
    pub fn trace(&self, node: &TaskNode) -> Vec<String> {
        let mut trace = Vec::new();
        self.trace_node(node, &mut trace);
        trace
    }

    fn trace_node(&self, node: &TaskNode, trace: &mut Vec<String>) {
        let indent = "  ".repeat(node.depth);
        trace.push(format!("{}[{}] {}", indent, node.id, node.description));
        if let Some(ref result) = node.result {
            trace.push(format!("{}  → {}", indent, result));
        }
        for child in &node.children {
            self.trace_node(child, trace);
        }
    }
}

impl Default for RecursiveController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomize() {
        let ctrl = RecursiveController::new();
        assert!(ctrl.atomize("Fix bug")); // short = atomic
        assert!(!ctrl.atomize("First build the UI; then implement auth; finally deploy to production")); // multi-step
    }

    #[test]
    fn test_recursive_execution() {
        let ctrl = RecursiveController::new();
        let tree = ctrl.execute_recursive("Task A; Task B; Task C");
        
        assert!(!tree.is_atomic);
        assert_eq!(tree.children.len(), 3);
        assert!(tree.result.is_some());
    }

    #[test]
    fn test_aggregation_compression() {
        let ctrl = RecursiveController::new();
        let results: Vec<String> = (0..10).map(|i| format!("Result {}", i)).collect();
        let aggregated = ctrl.aggregate(results);
        assert!(aggregated.contains("Aggregated 10 results"));
    }

    #[test]
    fn test_trace() {
        let ctrl = RecursiveController::new();
        let tree = ctrl.execute_recursive("A; B");
        let trace = ctrl.trace(&tree);
        assert!(trace.len() >= 2); // At least root + children
    }
}
