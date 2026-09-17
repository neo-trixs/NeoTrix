#![forbid(unsafe_code)]

//! # Cost Tracker — Token 成本追踪
//!
//! 基于 Azure APIM 的 token metrics 模式。
//! 追踪每个 provider/model 的 token 使用量和成本。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 模型定价（每 1K token）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model_id: String,
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
    pub cached_cost_per_1k: Option<f64>,
}

impl ModelPricing {
    pub fn new(model_id: &str, input: f64, output: f64) -> Self {
        Self {
            model_id: model_id.to_string(),
            input_cost_per_1k: input,
            output_cost_per_1k: output,
            cached_cost_per_1k: None,
        }
    }
}

/// 单次请求的 token 使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub cached_tokens: Option<u32>,
    pub reasoning_tokens: Option<u32>,
    pub audio_tokens: Option<u32>,
}

/// 单次请求的成本记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    pub request_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub usage: TokenUsage,
    pub input_cost: f64,
    pub output_cost: f64,
    pub total_cost: f64,
    pub timestamp: String,
}

/// Provider 级别的成本汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCostSummary {
    pub provider_id: String,
    pub total_requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost: f64,
    pub avg_latency_ms: f64,
}

/// 全局成本追踪器
pub struct CostTracker {
    records: Vec<CostRecord>,
    provider_summaries: HashMap<String, ProviderCostSummary>,
    model_pricing: HashMap<String, ModelPricing>,
    budget_cap: Option<f64>,
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            provider_summaries: HashMap::new(),
            model_pricing: HashMap::new(),
            budget_cap: None,
        }
    }

    /// 设置预算上限
    pub fn set_budget_cap(&mut self, cap: f64) {
        self.budget_cap = Some(cap);
    }

    /// 注册模型定价
    pub fn register_pricing(&mut self, pricing: ModelPricing) {
        self.model_pricing.insert(pricing.model_id.clone(), pricing);
    }

    /// 记录一次请求的成本
    pub fn record(&mut self, record: CostRecord) {
        // 更新 provider 汇总
        let summary = self
            .provider_summaries
            .entry(record.provider_id.clone())
            .or_insert_with(|| ProviderCostSummary {
                provider_id: record.provider_id.clone(),
                total_requests: 0,
                total_input_tokens: 0,
                total_output_tokens: 0,
                total_cost: 0.0,
                avg_latency_ms: 0.0,
            });

        summary.total_requests += 1;
        summary.total_input_tokens += record.usage.prompt_tokens as u64;
        summary.total_output_tokens += record.usage.completion_tokens as u64;
        summary.total_cost += record.total_cost;

        self.records.push(record);
    }

    /// 计算请求成本
    pub fn calculate_cost(&self, model_id: &str, usage: &TokenUsage) -> (f64, f64) {
        if let Some(pricing) = self.model_pricing.get(model_id) {
            let input_cost = (usage.prompt_tokens as f64 / 1000.0) * pricing.input_cost_per_1k;
            let output_cost = (usage.completion_tokens as f64 / 1000.0) * pricing.output_cost_per_1k;
            (input_cost, output_cost)
        } else {
            // 默认定价
            let input_cost = usage.prompt_tokens as f64 * 0.00001;
            let output_cost = usage.completion_tokens as f64 * 0.00003;
            (input_cost, output_cost)
        }
    }

    /// 检查是否超出预算
    pub fn is_over_budget(&self) -> bool {
        if let Some(cap) = self.budget_cap {
            let total: f64 = self.provider_summaries.values().map(|s| s.total_cost).sum();
            total >= cap
        } else {
            false
        }
    }

    /// 获取总成本
    pub fn total_cost(&self) -> f64 {
        self.provider_summaries.values().map(|s| s.total_cost).sum()
    }

    /// 获取所有 provider 汇总
    pub fn summaries(&self) -> Vec<&ProviderCostSummary> {
        self.provider_summaries.values().collect()
    }

    /// 获取最近 N 条记录
    pub fn recent_records(&self, n: usize) -> &[CostRecord] {
        let start = self.records.len().saturating_sub(n);
        &self.records[start..]
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_cost() {
        let mut tracker = CostTracker::new();
        tracker.register_pricing(ModelPricing::new("gpt-4", 0.03, 0.06));

        let usage = TokenUsage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            cached_tokens: None,
            reasoning_tokens: None,
            audio_tokens: None,
        };

        let (input, output) = tracker.calculate_cost("gpt-4", &usage);
        assert!((input - 0.03).abs() < 0.001);
        assert!((output - 0.03).abs() < 0.001);
    }

    #[test]
    fn test_budget_cap() {
        let mut tracker = CostTracker::new();
        tracker.set_budget_cap(1.0);

        let record = CostRecord {
            request_id: "r1".into(),
            provider_id: "openai".into(),
            model_id: "gpt-4".into(),
            usage: TokenUsage {
                prompt_tokens: 1000,
                completion_tokens: 500,
                cached_tokens: None,
                reasoning_tokens: None,
                audio_tokens: None,
            },
            input_cost: 0.03,
            output_cost: 0.03,
            total_cost: 0.06,
            timestamp: "2026-01-01".into(),
        };

        tracker.record(record);
        assert!(!tracker.is_over_budget());
    }
}