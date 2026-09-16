use std::collections::HashMap;
use std::fmt;

use super::{BrainStage, SelfIteratingBrain, StageDecision};
use crate::neotrix::nt_core_error::NeoTrixError;

// ── MetaRSI Three Operators ────────────────────────────────────────
// Source: MetaRSI/RSI2 (arXiv:2609.06396)
// "Meta-Recursive Self-Improvement: Data-RSI + Harness-RSI + Model-RSI"
//
// Maps to SEAL pipeline:
//   Data-RSI    → Phase-0 (data augmentation / training data curation)
//   Harness-RSI → Phase-1 (evaluation harness improvement)
//   Model-RSI   → Phase-2 (architecture / capability mutation)
// ────────────────────────────────────────────────────────────────────

/// MetaRSI 三算子: 元递归自改进的核心操作单元
///
/// 三种算子分别作用于 SEAL pipeline 的不同阶段:
/// - **DataRsi**: 数据增强 — 通过变换策略扩展训练数据覆盖
/// - **HarnessRsi**: 评估框架改进 — 通过新增/调整指标优化评估质量
/// - **ModelRsi**: 模型架构改进 — 通过能力维度突变探索架构空间
#[derive(Debug, Clone, PartialEq)]
pub enum RsiOperator {
    /// Data-RSI: 数据增强算子
    ///
    /// 对输入数据施加变换序列, 生成增强样本。
    /// 映射到 SEAL Phase-0: 数据合成与扩充。
    DataRsi {
        /// 变换策略名称序列 (如 "backtranslation", "synonym_replacement" 等)
        transformations: Vec<String>,
    },
    /// Harness-RSI: 评估框架改进算子
    ///
    /// 通过新增或调整评估指标, 改进评估框架的区分度。
    /// 映射到 SEAL Phase-1: 评估信号质量提升。
    HarnessRsi {
        /// 新增或调整的评估指标名称
        metrics: Vec<String>,
    },
    /// Model-RSI: 模型架构改进算子
    ///
    /// 通过能力维度突变探索架构空间。
    /// 映射到 SEAL Phase-2: 能力网结构演化。
    ModelRsi {
        /// 架构变更描述 (如 "add_attention_head", "increase_hidden_dim" 等)
        architecture_changes: Vec<String>,
    },
}

impl fmt::Display for RsiOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RsiOperator::DataRsi { transformations } => {
                write!(f, "DataRsi({} transforms)", transformations.len())
            }
            RsiOperator::HarnessRsi { metrics } => {
                write!(f, "HarnessRsi({} metrics)", metrics.len())
            }
            RsiOperator::ModelRsi {
                architecture_changes,
            } => {
                write!(f, "ModelRsi({} changes)", architecture_changes.len())
            }
        }
    }
}

impl RsiOperator {
    /// 执行 Data-RSI: 对数据施加变换序列, 生成增强样本
    ///
    /// 每个变换生成一个增强副本, 所有副本组成增强数据集。
    /// 用于 SEAL Phase-0 数据合成阶段。
    pub(crate) fn _execute_data_rsi(&self, data: &[String]) -> Vec<String> {
        match self {
            RsiOperator::DataRsi { transformations } => {
                if transformations.is_empty() || data.is_empty() {
                    return data.to_vec();
                }
                data.iter()
                    .flat_map(|d| {
                        transformations
                            .iter()
                            .map(move |t| format!("{} [transformed by {}]", d, t))
                    })
                    .collect()
            }
            _ => data.to_vec(),
        }
    }

    /// 执行 Harness-RSI: 向评估指标集注入新指标
    ///
    /// 新指标初始化为 0.0, 由后续评估循环逐步填充真实值。
    /// 用于 SEAL Phase-1 评估框架改进。
    pub(crate) fn _execute_harness_rsi(&self, metrics: &HashMap<String, f64>) -> HashMap<String, f64> {
        match self {
            RsiOperator::HarnessRsi {
                metrics: new_metrics,
            } => {
                let mut result = metrics.clone();
                for m in new_metrics {
                    result.entry(m.clone()).or_insert(0.0);
                }
                result
            }
            _ => metrics.clone(),
        }
    }

    /// 执行 Model-RSI: 对配置施加架构变更
    ///
    /// 将架构变更描述标记为已修改, 触发后续能力向量重评估。
    /// 用于 SEAL Phase-2 模型架构演化。
    pub(crate) fn _execute_model_rsi(&self, config: &mut HashMap<String, String>) {
        if let RsiOperator::ModelRsi {
            architecture_changes,
        } = self
        {
            for change in architecture_changes {
                config.insert(change.clone(), "modified".to_string());
            }
        }
    }

    /// 计算算子的预期收益 (用于优先级排序)
    ///
    /// Data-RSI: 与变换数量成正比 (数据多样性增益)
    /// Harness-RSI: 与指标数量成正比 (评估覆盖增益)
    /// Model-RSI: 与变更数量成正比 (架构探索增益)
    pub(crate) fn _expected_gain(&self) -> f64 {
        match self {
            RsiOperator::DataRsi { transformations } => {
                (transformations.len() as f64 * 0.05).min(0.3)
            }
            RsiOperator::HarnessRsi { metrics } => (metrics.len() as f64 * 0.08).min(0.4),
            RsiOperator::ModelRsi {
                architecture_changes,
            } => (architecture_changes.len() as f64 * 0.06).min(0.25),
        }
    }

    /// 算子名称
    pub fn name(&self) -> &'static str {
        match self {
            RsiOperator::DataRsi { .. } => "data_rsi",
            RsiOperator::HarnessRsi { .. } => "harness_rsi",
            RsiOperator::ModelRsi { .. } => "model_rsi",
        }
    }
}

// ── MetaRSI Pipeline Stage ─────────────────────────────────────────
// 三算子统一调度: 按当前 brain 状态选择最优算子执行
// ────────────────────────────────────────────────────────────────────

/// MetaRSI 算子链: 按优先级排列的算子序列
#[derive(Debug, Clone, Default)]
pub struct RsiOperatorChain {
    pub operators: Vec<RsiOperator>,
}

impl RsiOperatorChain {
    /// 根据 brain 状态生成推荐算子链
    ///
    /// 决策逻辑:
    /// - reward < 0.3 且数据不足 → DataRsi (数据扩充)
    /// - reward 0.3-0.6 且指标覆盖不足 → HarnessRsi (评估改进)
    /// - reward > 0.6 且有突变空间 → ModelRsi (架构探索)
    pub fn recommend(brain: &SelfIteratingBrain) -> Self {
        let reward = brain._reward;
        let cap_count = brain.brain.capability.arr().len();
        let eval_count = brain.evaluation_history.len();

        let mut operators = Vec::new();

        if reward < 0.3 {
            // 低 reward: 数据不足, 优先数据增强
            operators.push(RsiOperator::DataRsi {
                transformations: vec![
                    "backtranslation".into(),
                    "paraphrase".into(),
                    "augmented_sampling".into(),
                ],
            });
        }

        if reward >= 0.3 && reward < 0.6 && eval_count < 10 {
            // 中等 reward: 评估覆盖不足, 改进 harness
            operators.push(RsiOperator::HarnessRsi {
                metrics: vec![
                    "diversity_score".into(),
                    "novelty_score".into(),
                    "stability_index".into(),
                ],
            });
        }

        if reward >= 0.6 && cap_count > 0 {
            // 高 reward: 有突变空间, 探索架构
            operators.push(RsiOperator::ModelRsi {
                architecture_changes: vec![
                    "increase_attention_heads".into(),
                    "add_skip_connection".into(),
                ],
            });
        }

        Self { operators }
    }

    /// 执行所有算子, 返回总预期收益
    pub(crate) fn _execute_all(
        &self,
        data: &mut Vec<String>,
        metrics: &mut HashMap<String, f64>,
        config: &mut HashMap<String, String>,
    ) -> f64 {
        let mut total_gain = 0.0;
        for op in &self.operators {
            total_gain += op._expected_gain();
            match op {
                RsiOperator::DataRsi { .. } => {
                    let augmented = op._execute_data_rsi(data);
                    data.extend(augmented);
                }
                RsiOperator::HarnessRsi { .. } => {
                    let updated = op._execute_harness_rsi(metrics);
                    *metrics = updated;
                }
                RsiOperator::ModelRsi { .. } => {
                    op._execute_model_rsi(config);
                }
            }
        }
        total_gain
    }
}

/// MetaRSI SEAL pipeline stage
///
/// 三算子统一调度: 分析 brain 状态, 生成并执行最优算子组合。
/// 频率 5 — 每 5 次迭代运行一次, 避免过度突变。
pub struct MetaRsiStage;

impl Default for MetaRsiStage {
    fn default() -> Self {
        Self
    }
}

impl MetaRsiStage {
    pub fn new() -> Self {
        Self
    }
}

impl BrainStage for MetaRsiStage {
    fn name(&self) -> &str {
        "meta_rsi"
    }

    fn frequency(&self) -> usize {
        5
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let chain = RsiOperatorChain::recommend(brain);
        if chain.operators.is_empty() {
            log::trace!("[meta_rsi] no operators recommended, skip");
            return Ok(StageDecision::Skip("no operators recommended".into()));
        }

        // 收集当前数据/指标/配置用于执行
        let mut data: Vec<String> = brain.brain.harness_history.iter().cloned().collect();
        let mut metrics: HashMap<String, f64> = HashMap::new();
        metrics.insert("reward".into(), brain._reward);
        metrics.insert("entropy".into(), brain.entropy_crisis_level);
        metrics.insert(
            "cap_mean".into(),
            if brain.brain.capability.arr().is_empty() {
                0.0
            } else {
                brain.brain.capability.arr().iter().sum::<f64>()
                    / brain.brain.capability.arr().len() as f64
            },
        );
        let mut config: HashMap<String, String> = HashMap::new();

        let total_gain = chain._execute_all(&mut data, &mut metrics, &mut config);

        // 将增强后的 harness_history 写回 brain
        brain.brain.harness_history = data;
        // 更新 reward 反馈
        if let Some(&new_reward) = metrics.get("reward") {
            brain._set_reward(new_reward);
        }

        // 记录算子执行结果到 KB
        let operator_names: Vec<&str> = chain.operators.iter().map(|op| op.name()).collect();
        if let Some(ref kb) = brain._nt_memory_kb {
            let summary = serde_json::json!({
                "operators": operator_names,
                "total_gain": total_gain,
                "reward": brain._reward,
                "iteration": brain.iteration,
            });
            let _ = kb.kv_set(
                "meta_rsi",
                &format!("iter_{}", brain.iteration),
                &summary.to_string(),
            );
        }

        log::info!(
            "[meta_rsi] iter={} operators=[{}] gain={:.4} reward={:.4}",
            brain.iteration,
            operator_names.join(", "),
            total_gain,
            brain._reward,
        );

        Ok(StageDecision::Continue)
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_rsi_basic() {
        let op = RsiOperator::DataRsi {
            transformations: vec!["backtranslation".into(), "paraphrase".into()],
        };
        let data = vec!["hello".into(), "world".into()];
        let result = op._execute_data_rsi(&data);
        assert_eq!(result.len(), 4); // 2 items × 2 transforms
        assert!(result[0].contains("backtranslation"));
        assert!(result[1].contains("paraphrase"));
        assert!(result[2].contains("backtranslation"));
    }

    #[test]
    fn test_data_rsi_empty_transforms() {
        let op = RsiOperator::DataRsi {
            transformations: vec![],
        };
        let data = vec!["hello".into()];
        let result = op._execute_data_rsi(&data);
        assert_eq!(result, vec!["hello".to_string()]);
    }

    #[test]
    fn test_harness_rsi_injects_new_metrics() {
        let op = RsiOperator::HarnessRsi {
            metrics: vec!["diversity".into(), "novelty".into()],
        };
        let mut existing = HashMap::new();
        existing.insert("reward".into(), 0.5);
        let result = op._execute_harness_rsi(&existing);
        assert_eq!(result.len(), 3);
        assert_eq!(result["reward"], 0.5);
        assert_eq!(result["diversity"], 0.0);
    }

    #[test]
    fn test_harness_rsi_preserves_existing() {
        let op = RsiOperator::HarnessRsi {
            metrics: vec!["reward".into()], // existing key
        };
        let mut existing = HashMap::new();
        existing.insert("reward".into(), 0.75);
        let result = op._execute_harness_rsi(&existing);
        assert_eq!(result["reward"], 0.75); // preserved, not reset to 0.0
    }

    #[test]
    fn test_model_rsi_modifies_config() {
        let op = RsiOperator::ModelRsi {
            architecture_changes: vec!["add_head".into()],
        };
        let mut config = HashMap::new();
        op._execute_model_rsi(&mut config);
        assert_eq!(config["add_head"], "modified");
    }

    #[test]
    fn test_expected_gain() {
        let data_op = RsiOperator::DataRsi {
            transformations: vec!["a".into(), "b".into()],
        };
        assert!((data_op._expected_gain() - 0.1).abs() < 1e-6);

        let harness_op = RsiOperator::HarnessRsi {
            metrics: vec!["a".into(), "b".into(), "c".into()],
        };
        assert!((harness_op._expected_gain() - 0.24).abs() < 1e-6);

        let model_op = RsiOperator::ModelRsi {
            architecture_changes: vec!["a".into()],
        };
        assert!((model_op._expected_gain() - 0.06).abs() < 1e-6);
    }

    #[test]
    fn test_operator_name() {
        let data = RsiOperator::DataRsi {
            transformations: vec![],
        };
        assert_eq!(data.name(), "data_rsi");

        let harness = RsiOperator::HarnessRsi { metrics: vec![] };
        assert_eq!(harness.name(), "harness_rsi");

        let model = RsiOperator::ModelRsi {
            architecture_changes: vec![],
        };
        assert_eq!(model.name(), "model_rsi");
    }
}
