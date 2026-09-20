//! # JIT-Harness 生成 (JIT-Agent pattern)
//!
//! 按任务动态合成执行 harness:
//! memory_module + planning_module + action_module + tool_module

use serde::{Deserialize, Serialize};

/// JIT Harness 协议
pub trait HarnessProtocol: Send + Sync {
    fn memory_module(&self) -> &str;
    fn planning_module(&self) -> &str;
    fn action_module(&self) -> &str;
    fn tool_module(&self) -> &str;
    fn name(&self) -> &str;
}

/// 默认 harness 实现
pub struct DefaultHarness {
    pub name: String,
    pub memory: String,
    pub planning: String,
    pub action: String,
    pub tool: String,
}

impl HarnessProtocol for DefaultHarness {
    fn memory_module(&self) -> &str { &self.memory }
    fn planning_module(&self) -> &str { &self.planning }
    fn action_module(&self) -> &str { &self.action }
    fn tool_module(&self) -> &str { &self.tool }
    fn name(&self) -> &str { &self.name }
}

/// Harness 合成器
pub struct HarnessSynthesizer {
    /// 历史 harness 性能档案
    archive: Vec<HarnessRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessRecord {
    pub task_type: String,
    pub harness_name: String,
    pub success_rate: f64,
    pub avg_latency_ms: u64,
}

impl HarnessSynthesizer {
    pub fn new() -> Self {
        Self { archive: vec![] }
    }
    
    /// 为特定任务类型合成最优 harness
    pub fn synthesize(&self, task_type: &str) -> Box<dyn HarnessProtocol> {
        // 查找历史记录
        let best = self.archive.iter()
            .filter(|r| r.task_type == task_type)
            .max_by(|a, b| a.success_rate.partial_cmp(&b.success_rate).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some(record) = best {
            Box::new(DefaultHarness {
                name: record.harness_name.clone(),
                memory: "episodic".into(),
                planning: "reactive".into(),
                action: "parallel".into(),
                tool: "mcp".into(),
            })
        } else {
            // 默认 harness
            Box::new(DefaultHarness {
                name: format!("default_{}", task_type),
                memory: "working".into(),
                planning: "sequential".into(),
                action: "sequential".into(),
                tool: "basic".into(),
            })
        }
    }
    
    /// 记录 harness 执行结果
    pub fn record(&mut self, task_type: &str, harness_name: &str, success: bool, latency_ms: u64) {
        if let Some(record) = self.archive.iter_mut()
            .find(|r| r.task_type == task_type && r.harness_name == harness_name) {
            // 更新统计
            let total = record.success_rate * 100.0;
            let new_total = if success { total + 1.0 } else { total };
            record.success_rate = new_total / 101.0;
            record.avg_latency_ms = (record.avg_latency_ms + latency_ms) / 2;
        } else {
            self.archive.push(HarnessRecord {
                task_type: task_type.into(),
                harness_name: harness_name.into(),
                success_rate: if success { 1.0 } else { 0.0 },
                avg_latency_ms: latency_ms,
            });
        }
    }
}

impl Default for HarnessSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_synthesize_default() {
        let synth = HarnessSynthesizer::new();
        let harness = synth.synthesize("unknown_task");
        assert!(harness.name().contains("unknown_task"));
    }
}
