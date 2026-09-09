//! CostTracker — 成本追踪器
//!
//! 追踪 API 调用成本、计算 token 使用量、设置预算告警。
//! 支持按模型、功能、提供商分类追踪。

use std::collections::HashMap;
use std::time::Instant;

/// 成本条目
#[derive(Debug, Clone)]
pub struct CostEntry {
    /// 模型名
    pub model: String,
    /// 提供商
    pub provider: String,
    /// 功能标签
    pub feature: String,
    /// 输入 token 数
    pub input_tokens: u32,
    /// 输出 token 数
    pub output_tokens: u32,
    /// 成本 (USD)
    pub cost_usd: f64,
    /// 时间戳
    pub timestamp: Instant,
}

/// 预算告警阈值
#[derive(Debug, Clone)]
pub struct BudgetAlert {
    /// 告警名称
    pub name: String,
    /// 预算上限 (USD)
    pub limit_usd: f64,
    /// 当前使用量 (USD)
    pub current_usd: f64,
    /// 告警阈值 (百分比)
    pub threshold_percent: f64,
    /// 是否已触发
    pub triggered: bool,
}

/// 成本追踪器
pub struct CostTracker {
    /// 成本历史
    entries: Vec<CostEntry>,
    /// 最大记录数
    max_entries: usize,
    /// 预算告警
    alerts: Vec<BudgetAlert>,
    /// 按模型统计
    by_model: HashMap<String, f64>,
    /// 按功能统计
    by_feature: HashMap<String, f64>,
    /// 按提供商统计
    by_provider: HashMap<String, f64>,
    /// 总成本
    total_cost_usd: f64,
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 10_000,
            alerts: Vec::new(),
            by_model: HashMap::new(),
            by_feature: HashMap::new(),
            by_provider: HashMap::new(),
            total_cost_usd: 0.0,
        }
    }

    /// 记录成本
    pub fn record(&mut self, model: &str, provider: &str, feature: &str, input_tokens: u32, output_tokens: u32, cost_usd: f64) {
        let entry = CostEntry {
            model: model.to_string(),
            provider: provider.to_string(),
            feature: feature.to_string(),
            input_tokens,
            output_tokens,
            cost_usd,
            timestamp: Instant::now(),
        };

        self.entries.push(entry);
        self.total_cost_usd += cost_usd;
        *self.by_model.entry(model.to_string()).or_insert(0.0) += cost_usd;
        *self.by_feature.entry(feature.to_string()).or_insert(0.0) += cost_usd;
        *self.by_provider.entry(provider.to_string()).or_insert(0.0) += cost_usd;

        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }

        // 检查预算告警
        self.check_alerts();
    }

    /// 添加预算告警
    pub fn add_alert(&mut self, name: &str, limit_usd: f64, threshold_percent: f64) {
        self.alerts.push(BudgetAlert {
            name: name.to_string(),
            limit_usd,
            current_usd: 0.0,
            threshold_percent,
            triggered: false,
        });
    }

    /// 检查告警
    fn check_alerts(&mut self) {
        for alert in &mut self.alerts {
            alert.current_usd = self.total_cost_usd;
            if !alert.triggered && self.total_cost_usd >= alert.limit_usd * (alert.threshold_percent / 100.0) {
                alert.triggered = true;
                // TODO: 发送实际告警通知
            }
        }
    }

    /// 获取总成本
    pub fn total_cost(&self) -> f64 {
        self.total_cost_usd
    }

    /// 获取指定模型成本
    pub fn cost_by_model(&self, model: &str) -> f64 {
        self.by_model.get(model).copied().unwrap_or(0.0)
    }

    /// 获取指定功能成本
    pub fn cost_by_feature(&self, feature: &str) -> f64 {
        self.by_feature.get(feature).copied().unwrap_or(0.0)
    }

    /// 获取指定提供商成本
    pub fn cost_by_provider(&self, provider: &str) -> f64 {
        self.by_provider.get(provider).copied().unwrap_or(0.0)
    }

    /// 获取统计信息
    pub fn stats(&self) -> CostStats {
        let mut total_input_tokens = 0;
        let mut total_output_tokens = 0;

        for entry in &self.entries {
            total_input_tokens += entry.input_tokens;
            total_output_tokens += entry.output_tokens;
        }

        CostStats {
            total_entries: self.entries.len() as u32,
            total_cost_usd: self.total_cost_usd,
            total_input_tokens,
            total_output_tokens,
            alerts_triggered: self.alerts.iter().filter(|a| a.triggered).count() as u32,
        }
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// 成本统计
#[derive(Debug, Clone)]
pub struct CostStats {
    pub total_entries: u32,
    pub total_cost_usd: f64,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub alerts_triggered: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_cost() {
        let mut tracker = CostTracker::new();
        tracker.record("gpt-4", "openai", "chat", 100, 50, 0.01);
        assert_eq!(tracker.total_cost(), 0.01);
    }

    #[test]
    fn test_cost_by_model() {
        let mut tracker = CostTracker::new();
        tracker.record("gpt-4", "openai", "chat", 100, 50, 0.01);
        tracker.record("gpt-3.5-turbo", "openai", "chat", 100, 50, 0.001);
        assert_eq!(tracker.cost_by_model("gpt-4"), 0.01);
    }
}
