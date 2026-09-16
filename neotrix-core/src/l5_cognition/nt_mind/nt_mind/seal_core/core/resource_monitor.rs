//! Resource Monitor — 资源监控器
//! 跟踪每个任务的 token 消耗、延迟、API 调用次数

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub api_calls: u32,
    pub latency_ms: u64,
    pub errors: u32,
}

pub struct ResourceMonitor {
    task_usage: HashMap<String, ResourceUsage>,
    global_usage: ResourceUsage,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            task_usage: HashMap::new(),
            global_usage: ResourceUsage::default(),
        }
    }

    pub fn record_tokens(&mut self, task_id: &str, input: u64, output: u64) {
        let usage = self.task_usage.entry(task_id.to_string()).or_default();
        usage.tokens_in += input;
        usage.tokens_out += output;
        self.global_usage.tokens_in += input;
        self.global_usage.tokens_out += output;
    }

    pub fn record_api_call(&mut self, task_id: &str, latency_ms: u64, success: bool) {
        let usage = self.task_usage.entry(task_id.to_string()).or_default();
        usage.api_calls += 1;
        usage.latency_ms += latency_ms;
        if !success { usage.errors += 1; }
        
        self.global_usage.api_calls += 1;
        self.global_usage.latency_ms += latency_ms;
        if !success { self.global_usage.errors += 1; }
    }

    pub fn task_usage(&self, task_id: &str) -> Option<&ResourceUsage> {
        self.task_usage.get(task_id)
    }

    pub fn global_usage(&self) -> &ResourceUsage {
        &self.global_usage
    }

    pub fn cost_estimate(&self, task_id: &str, cost_per_1k_tokens: f64) -> f64 {
        let usage = self.task_usage.get(task_id).unwrap_or(&self.global_usage);
        (usage.tokens_in + usage.tokens_out) as f64 / 1000.0 * cost_per_1k_tokens
    }

    pub fn efficiency_score(&self, task_id: &str) -> f64 {
        if let Some(usage) = self.task_usage.get(task_id) {
            if usage.api_calls == 0 { return 0.0; }
            let success_rate = 1.0 - (usage.errors as f64 / usage.api_calls as f64);
            let token_efficiency = 1.0 / (1.0 + (usage.tokens_in + usage.tokens_out) as f64 / 100000.0);
            success_rate * token_efficiency
        } else {
            0.0
        }
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_basic() {
        let mut mon = ResourceMonitor::new();
        mon.record_tokens("task1", 500, 200);
        mon.record_api_call("task1", 100, true);
        mon.record_api_call("task1", 150, false);
        
        let usage = mon.task_usage("task1").unwrap();
        assert_eq!(usage.tokens_in, 500);
        assert_eq!(usage.api_calls, 2);
        assert_eq!(usage.errors, 1);
        
        let cost = mon.cost_estimate("task1", 0.01);
        assert!(cost > 0.0);
    }
}
