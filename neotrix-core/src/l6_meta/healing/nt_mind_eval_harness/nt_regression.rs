//! Regression 子系统 — 回归用例/门禁 + SmallScale 方法论 (纯搬移自门面)。

use super::nt_harness::EvalHarness;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 自动生成的回归/对抗测试用例 (E3 自改进闭环, T11)。
///
/// 由 `EvalHarness::generate_regression_test` 从候选行为变更摘要合成, 运行
/// (`run_regression_test`) 时对候选做确定性回归门禁 (无 provider 依赖):
///   - `forbidden_tokens`: 候选不得包含的已知回归反模式 (引入即拒)。
///   - `required_categories`: 候选须至少命中其一 (源自 harness 既有评测数据集
///     的 category 维度); 若 harness 无任何数据集则为空 → 门禁自动放行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionCase {
    pub id: String,
    pub candidate: String,
    pub forbidden_tokens: Vec<String>,
    pub required_categories: Vec<String>,
}

/// 回归测试运行结果。
#[derive(Debug, Clone, Default)]
pub struct RegressionResult {
    pub passed: bool,
    pub reasons: Vec<String>,
}

impl EvalHarness {
    /// 从候选行为变更摘要合成一个回归/对抗测试用例 (E3 闭环, T11)。
    ///
    /// 确定性, 无 provider 依赖: 候选的回归门禁由两个真实不变量构成 —
    ///   1. 禁止不变量: 候选不得引入已知退化反模式 (TODO/unimplemented/unsafe/…)。
    ///   2. 覆盖不变量: 候选须命中 harness 既有评测数据集的至少一个 `category`
    ///      维度, 否则视为无回归覆盖的孤儿变更 (Dark Forest: 不接线的变更即拒)。
    /// harness 无任何数据集时, 覆盖不变量为空 → 门禁对该维自动放行。
    pub fn generate_regression_test(&self, candidate: &str) -> RegressionCase {
        let candidate = candidate.trim().to_string();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        candidate.hash(&mut hasher);
        let id = format!("reg-{:016x}", hasher.finish());

        // 禁止不变量: 真实退化反模式清单 (非占位)。
        let forbidden_tokens = vec![
            "TODO".to_string(),
            "todo!".to_string(),
            "unimplemented".to_string(),
            "unimplemented!".to_string(),
            "panic!".to_string(),
            "unreachable!".to_string(),
            "unsafe".to_string(),
        ];

        // 覆盖不变量: 取自 harness 既有评测数据集的 category 维度。
        let required_categories: Vec<String> = self
            .datasets
            .iter()
            .flat_map(|d| d.queries.iter().map(|q| q.category.clone()))
            .filter(|c| !c.is_empty())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        RegressionCase {
            id,
            candidate,
            forbidden_tokens,
            required_categories,
        }
    }

    /// 运行回归测试用例 (确定性, 同步)。
    pub fn run_regression_test(&self, case: &RegressionCase) -> RegressionResult {
        let mut reasons: Vec<String> = Vec::new();
        let cand = case.candidate.trim();

        if cand.is_empty() {
            reasons.push("candidate is empty (no behavior change)".into());
        }

        for tok in &case.forbidden_tokens {
            if cand.contains(tok) {
                reasons.push(format!(
                    "candidate regresses: contains forbidden token '{}'",
                    tok
                ));
            }
        }

        if !case.required_categories.is_empty() && !cand.is_empty() {
            let covered = case
                .required_categories
                .iter()
                .any(|c| cand.contains(c.as_str()));
            if !covered {
                reasons.push(format!(
                    "candidate has no regression coverage: must reference one of {:?}",
                    case.required_categories
                ));
            }
        }

        RegressionResult {
            passed: reasons.is_empty(),
            reasons,
        }
    }
}

// ────────────────────────────────────────────────────────────────
// P10: SmallScaleMethod (吸收 arXiv 2608.11859v1 "Small-Scale Experiments:
// Are We There Yet?")
// 小模型实验被超参数敏感度混淆, 敏感度随规模增大而衰减;
// scaling laws 只在 fully-tuned frontier 涌现; 超参数损失曲面
// 随规模增大而降维。方法论: hyperparameter-first 的 holistic 视角。
// ────────────────────────────────────────────────────────────────

/// 超参数敏感度观测点 (scale: 模型规模, 如参数量或训练 FLOPs)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HyperparamSensitivity {
    pub scale: f64,
    pub loss_variance: f64,
    pub dimensions: usize,
}

/// 小规模实验警告: 规模低于前沿时结果被超参数敏感度混淆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmallScaleWarning {
    pub model_scale: f64,
    pub sensitivity: f64,
    pub needs_full_tuning: bool,
    pub reason: String,
}

/// Small-Scale Method: 以超参数优先的 holistic 评测方法论
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SmallScaleMethod {
    pub sensitivity_decay_rate: f64, // loss_variance ∝ scale^(-rate)
    pub dimension_decay_rate: f64,   // effective dims ∝ scale^(-rate)
    pub tuning_budget: usize,        // max hyperparameter configs to run
    pub full_tuning_frontier: bool,
}

impl Default for SmallScaleMethod {
    fn default() -> Self {
        Self {
            sensitivity_decay_rate: 1.0,
            dimension_decay_rate: 0.5,
            tuning_budget: 32,
            full_tuning_frontier: false,
        }
    }
}

impl SmallScaleMethod {
    pub fn new(sensitivity_decay_rate: f64, dimension_decay_rate: f64, tuning_budget: usize) -> Self {
        Self {
            sensitivity_decay_rate,
            dimension_decay_rate,
            tuning_budget,
            full_tuning_frontier: false,
        }
    }

    /// 超参数敏感度随规模衰减: loss_variance ∝ scale^(-rate)
    pub(crate) fn _sensitivity_at_scale(&self, scale: f64) -> f64 {
        scale.max(1.0).powf(-self.sensitivity_decay_rate)
    }

    /// 有效超参数维度随规模衰减 (下限 1 维)
    pub(crate) fn _effective_dimensions(&self, scale: f64) -> usize {
        (self.tuning_budget as f64 * scale.max(1.0).powf(-self.dimension_decay_rate))
            .max(1.0)
            .round() as usize
    }

    /// 小规模警告: scale < 1e6 时需全量调参, 结果不可作为 scaling law 证据
    pub(crate) fn _warn_small_scale(&self, model_scale: f64) -> SmallScaleWarning {
        let sensitivity = self._sensitivity_at_scale(model_scale);
        let needs_full_tuning = model_scale < 1e6;
        SmallScaleWarning {
            model_scale,
            sensitivity,
            needs_full_tuning,
            reason: if needs_full_tuning {
                "hyperparameter sensitivity high; scaling laws only emerge on fully-tuned frontier".into()
            } else {
                "hyperparameter sensitivity low; frontier tuning reliable at this scale".into()
            },
        }
    }

    /// 是否到达 fully-tuned frontier: 敏感度低于阈值
    pub(crate) fn _tuned_frontier_reached(&self, sensitivity: f64, threshold: f64) -> bool {
        sensitivity <= threshold
    }

    /// 调参优先级 rank: 高方差 + 高维度 = 更需优先调参
    pub(crate) fn _hyperparam_rank(&self, loss_variance: f64, dims: usize) -> f64 {
        loss_variance * (dims as f64) / 100.0
    }
}

impl crate::l6_meta::healing::nt_core_self_test::SelfTest for SmallScaleMethod {
    fn name(&self) -> &str {
        "nt_mind_eval_harness_small_scale"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let method = SmallScaleMethod::new(1.0, 0.5, 32);
        let big = method._sensitivity_at_scale(1e9);
        let small = method._sensitivity_at_scale(1e3);
        if big >= small {
            return Err(vec!["sensitivity must decrease with scale".into()]);
        }
        if method._effective_dimensions(1e3) >= method._effective_dimensions(10.0) {
            return Err(vec!["effective dimensions must decrease with scale".into()]);
        }
        if !method._warn_small_scale(1e5).needs_full_tuning {
            return Err(vec!["small scale must flag needs_full_tuning".into()]);
        }
        Ok(())
    }
}
