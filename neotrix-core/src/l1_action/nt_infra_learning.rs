//! L1 基础设施 — Router 自学习
//!
//! Router 从历史调用中学习最优路由策略
//! 基于: 成功率 + 延迟 + 用户反馈 → 动态权重调整

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Provider 学习记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderLearning {
    pub provider_id: String,
    pub total_calls: u64,
    pub successful: u64,
    pub failed: u64,
    pub avg_latency_ms: f64,
    pub user_feedback_score: f64,
    pub computed_weight: f64,
}

impl Default for ProviderLearning {
    fn default() -> Self {
        Self {
            provider_id: String::new(),
            total_calls: 0,
            successful: 0,
            failed: 0,
            avg_latency_ms: 0.0,
            user_feedback_score: 0.5,
            computed_weight: 0.5,
        }
    }
}

/// Router 学习引擎
pub struct RouterLearner {
    records: HashMap<String, ProviderLearning>,
    /// 权重配置
    success_weight: f64,
    latency_weight: f64,
    feedback_weight: f64,
}

impl Default for RouterLearner {
    fn default() -> Self { Self::new() }
}

impl RouterLearner {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            success_weight: 0.4,
            latency_weight: 0.3,
            feedback_weight: 0.3,
        }
    }

    /// 记录调用结果
    pub fn record_call(&mut self, provider_id: &str, success: bool, latency_ms: f64) {
        let entry = self.records.entry(provider_id.to_string())
            .or_insert_with(|| ProviderLearning {
                provider_id: provider_id.to_string(),
                ..Default::default()
            });
        entry.total_calls += 1;
        if success { entry.successful += 1; } else { entry.failed += 1; }
        let n = entry.total_calls as f64;
        entry.avg_latency_ms = (entry.avg_latency_ms * (n - 1.0) + latency_ms) / n;
        self.recompute_weight(provider_id);
    }

    /// 记录用户反馈
    pub fn record_feedback(&mut self, provider_id: &str, score: f64) {
        if let Some(entry) = self.records.get_mut(provider_id) {
            entry.user_feedback_score = (entry.user_feedback_score * 0.8 + score * 0.2).clamp(0.0, 1.0);
            self.recompute_weight(provider_id);
        }
    }

    /// 重新计算权重
    fn recompute_weight(&mut self, provider_id: &str) {
        if let Some(entry) = self.records.get_mut(provider_id) {
            let success_rate = if entry.total_calls > 0 {
                entry.successful as f64 / entry.total_calls as f64
            } else { 0.5 };
            // 延迟归一化: 假设 1000ms 为最差
            let latency_score = (1.0 - (entry.avg_latency_ms / 1000.0)).clamp(0.0, 1.0);
            entry.computed_weight = success_rate * self.success_weight
                + latency_score * self.latency_weight
                + entry.user_feedback_score * self.feedback_weight;
        }
    }

    /// 获取最优 provider
    pub fn best_provider(&self) -> Option<&str> {
        self.records.values()
            .max_by(|a, b| a.computed_weight.partial_cmp(&b.computed_weight)
                .unwrap_or(std::cmp::Ordering::Equal))
            .map(|r| r.provider_id.as_str())
    }

    /// 获取所有权重
    pub fn weights(&self) -> HashMap<String, f64> {
        self.records.iter()
            .map(|(k, v)| (k.clone(), v.computed_weight))
            .collect()
    }

    /// 获取学习记录
    pub fn get(&self, provider_id: &str) -> Option<&ProviderLearning> {
        self.records.get(provider_id)
    }

    pub(crate) fn _all_records(&self) -> &HashMap<String, ProviderLearning> {
        &self.records
    }
}

// 全局 Router 学习引擎
lazy_static::lazy_static! {
    static ref GLOBAL_LEARNER: std::sync::Mutex<RouterLearner> =
        std::sync::Mutex::new(RouterLearner::new());
}

pub fn learner_record_call(provider_id: &str, success: bool, latency_ms: f64) {
    GLOBAL_LEARNER.lock().unwrap().record_call(provider_id, success, latency_ms);
}

pub fn learner_record_feedback(provider_id: &str, score: f64) {
    GLOBAL_LEARNER.lock().unwrap().record_feedback(provider_id, score);
}

pub fn learner_best_provider() -> Option<String> {
    GLOBAL_LEARNER.lock().unwrap().best_provider().map(String::from)
}

pub fn learner_weights() -> HashMap<String, f64> {
    GLOBAL_LEARNER.lock().unwrap().weights()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_basic() {
        let mut l = RouterLearner::new();
        l.record_call("a", true, 100.0);
        l.record_call("a", true, 150.0);
        l.record_call("b", true, 50.0);
        // b has lower latency → higher weight
        assert_eq!(l.best_provider(), Some("b"));
    }

    #[test]
    fn test_learning_feedback() {
        let mut l = RouterLearner::new();
        l.record_call("a", true, 100.0);
        l.record_feedback("a", 0.9);
        let weights = l.weights();
        assert!(weights.get("a").unwrap() > &0.5);
    }
}
