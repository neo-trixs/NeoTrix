use super::core::KnowledgeSource;
use super::model_router::RouterTier;
use crate::neotrix::nt_expert_routing::TaskType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainStats {
    pub total_absorbed: u64,
    pub unique_sources: Vec<KnowledgeSource>,
    pub latest_absorption: Option<u64>,
    pub capability_sum: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationResult {
    pub iteration: u64,
    pub task_type: TaskType,
    pub score_before: f64,
    pub score_after: f64,
    pub improved: bool,
    pub absorbed_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainReport {
    pub iteration: u64,
    pub total_absorbed: u64,
    pub capability_sum: f64,
    pub recent_improvement: usize,
}

// ====== 路由统计 ======

/// 路由统计仪表盘
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterStats {
    pub total_routed: u64,
    pub tier_counts: HashMap<String, u64>,
    pub tier_costs: HashMap<String, f64>,
    pub tier_avg_duration_ms: HashMap<String, f64>,
    pub total_cost: f64,
    pub estimated_savings: f64,
    pub tier_distribution_pct: HashMap<String, f64>,
    pub last_24h_count: u64,
}

impl RouterStats {
    pub fn from_cost_log(cost_log: &[super::reasoning_engine::CostRecord]) -> Self {
        let total = cost_log.len() as u64;
        let mut tier_counts: HashMap<String, u64> = HashMap::new();
        let mut tier_costs: HashMap<String, f64> = HashMap::new();
        let mut tier_durations: HashMap<String, Vec<u64>> = HashMap::new();
        let mut total_cost = 0.0;

        let now = chrono::Utc::now().timestamp();
        let last_24h = cost_log
            .iter()
            .filter(|c| c.timestamp > now - 86400)
            .count() as u64;

        for record in cost_log {
            *tier_counts.entry(record.tier.clone()).or_insert(0) += 1;
            *tier_costs.entry(record.tier.clone()).or_insert(0.0) += record.cost_estimate_usd;
            tier_durations
                .entry(record.tier.clone())
                .or_default()
                .push(record.duration_ms);
            total_cost += record.cost_estimate_usd;
        }

        let tier_avg_duration_ms = tier_durations
            .into_iter()
            .map(|(t, ds)| {
                let avg = if ds.is_empty() {
                    0.0
                } else {
                    ds.iter().sum::<u64>() as f64 / ds.len() as f64
                };
                (t, avg)
            })
            .collect();

        let distr = tier_counts
            .iter()
            .map(|(t, c)| (t.clone(), *c as f64 / total.max(1) as f64 * 100.0))
            .collect();

        // 估算节约：如果全用 T4 的费用 vs 实际费用
        let if_all_t4 = total as f64 * RouterTier::T4.cost_multiplier() * 0.02;
        let estimated_savings = (if_all_t4 - total_cost).max(0.0);

        Self {
            total_routed: total,
            tier_counts,
            tier_costs,
            tier_avg_duration_ms,
            total_cost,
            estimated_savings,
            tier_distribution_pct: distr,
            last_24h_count: last_24h,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!(self)
    }
}
