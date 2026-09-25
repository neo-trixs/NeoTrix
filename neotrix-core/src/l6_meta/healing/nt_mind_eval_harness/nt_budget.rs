//! Budget 子系统 — 预算网格 + 插值 + AUDC/QNC/Peak/Pareto 指标 (纯搬移自门面)。

use super::nt_harness::EvalHarness;
use super::nt_types::{EvalPoint, ModelQualityCurve, ParetoPoint};
use std::collections::HashMap;

/// 评测预算网格 (R2-Bench 16 点 + 我们努力分层对齐)
pub const DEFAULT_BUDGET_GRID: &[u32] = &[
    0,     // 直接回答 (EffortTier::Low, thinking_budget=0)
    512,   // 极低
    1024,  // Low-Medium 边界
    2048,  // Medium
    4096,  // High
    8192,  // XHigh
    16384, // Max
    32768, // Max+ (unlimited)
];

impl EvalHarness {
    /// 线性插值质量曲线到标准预算网格
    pub(crate) fn interpolate_quality(points: &[EvalPoint], grid: &[u32]) -> Vec<f64> {
        // 按 budget 分组取平均
        let mut budget_to_quality: HashMap<u32, Vec<f64>> = HashMap::new();
        for p in points {
            budget_to_quality
                .entry(p.budget)
                .or_default()
                .push(p.quality_score);
        }
        let avg_quality: HashMap<u32, f64> = budget_to_quality
            .into_iter()
            .map(|(b, qs)| (b, qs.iter().sum::<f64>() / qs.len() as f64))
            .collect();

        // 线性插值
        grid.iter()
            .map(|&target_budget| {
                if let Some(&q) = avg_quality.get(&target_budget) {
                    return q;
                }
                let mut lower = None;
                let mut upper = None;
                for &b in avg_quality.keys() {
                    if b <= target_budget && lower.is_none_or(|l| b > l) {
                        lower = Some(b);
                    }
                    if b >= target_budget && upper.is_none_or(|u| b < u) {
                        upper = Some(b);
                    }
                }
                match (lower, upper) {
                    (Some(l), Some(u)) if l != u => {
                        let ql = avg_quality[&l];
                        let qu = avg_quality[&u];
                        let t = (target_budget - l) as f64 / (u - l) as f64;
                        ql + t * (qu - ql)
                    }
                    (Some(l), None) => avg_quality[&l],
                    (None, Some(u)) => avg_quality[&u],
                    _ => 0.0,
                }
            })
            .collect()
    }

    /// 计算 AUDC, QNC, Peak Quality, Pareto 前沿
    pub(crate) fn compute_metrics(
        &self,
        curves: &[ModelQualityCurve],
    ) -> (
        HashMap<String, f64>,
        HashMap<String, f64>,
        HashMap<String, f64>,
        Vec<ParetoPoint>,
    ) {
        let mut audc_scores = HashMap::new();
        let mut qnc_scores = HashMap::new();
        let mut peak_quality = HashMap::new();
        let mut all_points = Vec::new();

        for curve in curves {
            let qualities = &curve.interpolated_quality;
            let costs: Vec<f64> = self
                .budget_grid
                .iter()
                .map(|&b| b as f64 * 0.000001) // 简化：假设 $1/1M tokens，实际应用 model 定价
                .collect();

            // AUDC: 梯形积分 quality vs cost
            let mut audc = 0.0;
            for i in 1..qualities.len() {
                let q_avg = (qualities[i - 1] + qualities[i]) / 2.0;
                let c_delta = costs[i] - costs[i - 1];
                audc += q_avg * c_delta;
            }
            // 归一化到 [0,1] (除以 max cost)
            let max_cost = costs.last().copied().unwrap_or(1.0);
            audc_scores.insert(curve.model_name.clone(), audc / max_cost);

            // Peak Quality
            let peak = qualities.iter().copied().fold(0.0, f64::max);
            peak_quality.insert(curve.model_name.clone(), peak);

            // 收集 Pareto 候选
            for (i, &q) in qualities.iter().enumerate() {
                all_points.push(ParetoPoint {
                    model_name: curve.model_name.clone(),
                    budget: self.budget_grid[i],
                    quality: q,
                    cost_usd: costs[i],
                });
            }
        }

        // QNC: 相对最佳单模型的成本归一化
        let best_peak = peak_quality.values().copied().fold(0.0, f64::max);
        for curve in curves {
            let min_cost_for_best = all_points
                .iter()
                .filter(|p| {
                    p.model_name == curve.model_name && (p.quality - best_peak).abs() < 0.01
                })
                .map(|p| p.cost_usd)
                .min_by(|a, b| a.total_cmp(b))
                .unwrap_or(f64::INFINITY);
            let best_single_cost = all_points
                .iter()
                .filter(|p| (p.quality - best_peak).abs() < 0.01)
                .map(|p| p.cost_usd)
                .min_by(|a, b| a.total_cmp(b))
                .unwrap_or(1.0);
            qnc_scores.insert(
                curve.model_name.clone(),
                if best_single_cost > 0.0 {
                    min_cost_for_best / best_single_cost
                } else {
                    1.0
                },
            );
        }

        // Pareto 前沿: 无其他点同时 quality >= 且 cost <=
        let mut pareto = Vec::new();
        for p in &all_points {
            let dominated = all_points.iter().any(|o| {
                o.quality >= p.quality
                    && o.cost_usd <= p.cost_usd
                    && (o.quality > p.quality || o.cost_usd < p.cost_usd)
            });
            if !dominated {
                pareto.push(p.clone());
            }
        }
        pareto.sort_by(|a, b| a.cost_usd.total_cmp(&b.cost_usd));

        (audc_scores, qnc_scores, peak_quality, pareto)
    }
}
