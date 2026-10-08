//! 函数记忆 — 基于 Grok Build 模式
//!
//! ⛔ 2026-10-06/07：工作流 / 子任务两类记忆经实测为死代码，已移除。

use std::collections::HashMap;

/// 函数记忆（工具/函数的使用模式）——
///
/// ⛔ 2026-10-07：原文件另有 `WorkflowMemory` / `WorkflowStep` / `SubtaskMemory`
///   三个类型 + `TripleMemoryStore` 的 `workflows`/`subtasks` 字段 + 4 个方法，
///   实测**全仓零消费者、零构造点** ⇒ 已删。
///   ⇒ 模块顶注释说的「三类记忆」**不再成立**，改为「一类记忆」。
#[derive(Clone, Debug)]
pub struct FunctionMemory {
    pub function_name: String,
    pub call_count: u32,
    pub success_count: u32,
    pub avg_latency_ms: u64,
    pub common_inputs: Vec<String>,
    pub common_outputs: Vec<String>,
}

/// 记忆存储（⛔ 原名「三类记忆存储」，实际只存 `FunctionMemory` 一类）
#[derive(Debug)]
pub struct TripleMemoryStore {
    // ⛔ 已删 `workflows` / `subtasks`（2026-10-07）：其元素类型
    //   `WorkflowMemory` / `SubtaskMemory` 全仓**零消费者**、**零构造点**
    //   （实测，含排除 `.worktrees/` 后的全仓统计）⇒ 恒为空容器。
    //   ⇒ 保留它们等于宣称「存了三类记忆」而实际只存一类。
    functions: HashMap<String, FunctionMemory>,
}

impl Default for TripleMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TripleMemoryStore {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// 记录函数调用
    pub(crate) fn _record_function_call(&mut self, name: &str, success: bool, latency_ms: u64) {
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

    /// 获取函数统计
    pub(crate) fn _function_stats(&self, name: &str) -> Option<(&str, u32, u32, f64)> {
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

    /// 总统计：⚠️ 原为 `(workflows, subtasks, functions)` 三元组，
    ///   但前两项对应的容器**恒为空**（死代码已删）⇒ 返回值里有两个恒为 0 的数字，
    ///   **看起来像统计，实际是假的**。
    /// ⇒ 改为只返回**真实的**函数记忆条数。
    pub fn stats(&self) -> usize {
        self.functions.len()
    }
}
