//! Pareto 子系统 — HDA 归因 + 大阵对比 + 摘要 (纯搬移自门面)。

use super::nt_harness::EvalHarness;
use super::nt_types::{DatasetSpec, GalaxyComparison, ModelQualityCurve};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ────────────────────────────────────────────────────────────────
// P2: HdaAttribution (吸收 harness.dev blog: Model Trained Detects When Models Think)
// Harness-Driven Analysis: 解释 score 提升时, 强制归因给具体组件 (Open/Tuned/Guard)。
// 消除"综合提升"式空洞结论 — attribution 必须单一且带置信度。
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HdaComponent {
    Open,
    Tuned,
    Guard,
}

impl HdaComponent {
    pub fn label(&self) -> &'static str {
        match self {
            HdaComponent::Open => "open-model",
            HdaComponent::Tuned => "tuned-model",
            HdaComponent::Guard => "guard-rail",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HdaAttribution {
    pub component: HdaComponent,
    pub delta_score: f64,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HdaAttributionReport {
    pub attributions: Vec<HdaAttribution>,
    pub total_delta: f64,
}

impl HdaAttributionReport {
    /// 按 delta 排序取 top_n, 供进化 loop 定向强化 (隔离每组件增益)。
    pub fn top(&self, n: usize) -> Vec<HdaAttribution> {
        let mut v = self.attributions.clone();
        v.sort_by(|a, b| b.delta_score.partial_cmp(&a.delta_score).unwrap_or(std::cmp::Ordering::Equal));
        v.truncate(n);
        v
    }

    pub fn leading(&self) -> Option<&HdaAttribution> {
        self.attributions
            .iter()
            .max_by(|a, b| a.delta_score.partial_cmp(&b.delta_score).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// 检验归因完备性: 总和接近 total_delta (容差 1e-6), 否则判定空洞归因。
    pub fn is_complete(&self) -> bool {
        let sum: f64 = self.attributions.iter().map(|a| a.delta_score).sum();
        (sum - self.total_delta).abs() < 1e-6
    }
}

pub fn hda_attribution(tuned: f64, open: f64, guard: f64, delta: f64) -> HdaAttributionReport {
    HdaAttributionReport {
        attributions: vec![
            HdaAttribution {
                component: HdaComponent::Tuned,
                delta_score: tuned,
                confidence: if tuned > 0.0 { 0.8 } else { 0.2 },
                evidence: vec!["tuned-model pass-rate delta".into()],
            },
            HdaAttribution {
                component: HdaComponent::Open,
                delta_score: open,
                confidence: if open > 0.0 { 0.7 } else { 0.3 },
                evidence: vec!["open-model baseline shift".into()],
            },
            HdaAttribution {
                component: HdaComponent::Guard,
                delta_score: guard,
                confidence: if guard > 0.0 { 0.9 } else { 0.1 },
                evidence: vec!["guard-rail rejection delta".into()],
            },
        ],
        total_delta: delta,
    }
}

/// SelfTest 包装件 (T2 注册): HDA 归因纯函数健康检测。
/// 无需 provider, 轻量可进 Lightweight registry。
#[derive(Default)]
pub struct HdaAttributionSelfTest;

impl crate::l6_meta::healing::nt_core_self_test::SelfTest for HdaAttributionSelfTest {
    fn name(&self) -> &str {
        "nt_mind_eval_harness_hda_attribution"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let report = hda_attribution(0.3, 0.1, 0.05, 0.45);
        if !report.is_complete() {
            return Err(vec!["hda attribution must be complete".into()]);
        }
        if report.top(2).is_empty() {
            return Err(vec!["top attribution must not be empty".into()]);
        }
        Ok(())
    }
}

impl EvalHarness {
    /// 大阵 vs 基线对比: 同模型不同 effort_tier
    pub(crate) fn compare_galaxy_vs_baseline(
        &self,
        _curves: &[ModelQualityCurve],
        _dataset: &DatasetSpec,
    ) -> HashMap<String, GalaxyComparison> {
        // 这里简化：实际需对比同模型在 effort_tier=Low/Max 下的曲线
        // 需要数据集包含 effort_tier 标注
        HashMap::new()
    }

    pub(crate) fn generate_summary(
        &self,
        curves: &[ModelQualityCurve],
        audc: &HashMap<String, f64>,
        qnc: &HashMap<String, f64>,
        peak: &HashMap<String, f64>,
    ) -> String {
        let mut lines = vec![format!(
            "Eval Summary ({} models, {} budgets)",
            curves.len(),
            self.budget_grid.len()
        )];
        for curve in curves {
            lines.push(format!(
                "  {}: AUDC={:.3} QNC={:.3} Peak={:.3}",
                curve.model_name,
                audc.get(&curve.model_name).unwrap_or(&0.0),
                qnc.get(&curve.model_name).unwrap_or(&1.0),
                peak.get(&curve.model_name).unwrap_or(&0.0)
            ));
        }
        lines.join("\n")
    }
}
