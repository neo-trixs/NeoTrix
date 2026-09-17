//! Meta-Cognitive 递归自改进层 — Metaⁿ 启发
//!
//! 递归元认知改进: 每一层将上一层的输出作为输入，
//! 通过固定Ω操作递归优化改进提案，直到收敛。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 改进提案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementProposal {
    pub target_module: String,
    pub improvement_type: ImprovementType,
    pub description: String,
    pub pre_process: String,
    pub helpers: Vec<String>,
    pub score_delta: f64,
    pub confidence: f64,
}

/// 改进类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImprovementType {
    MemoryOptimization,
    ReasoningEnhancement,
    SafetyStrengthening,
    PerformanceTuning,
    ArchitectureEvolution,
}

impl ImprovementType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MemoryOptimization => "memory_optimization",
            Self::ReasoningEnhancement => "reasoning_enhancement",
            Self::SafetyStrengthening => "safety_strengthening",
            Self::PerformanceTuning => "performance_tuning",
            Self::ArchitectureEvolution => "architecture_evolution",
        }
    }
}

/// 改进层 — 单层改进
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementLayer {
    pub input_summary: String,
    pub pre_process: String,
    pub helpers: Vec<String>,
    pub output: ImprovementProposal,
    pub depth: usize,
}

/// 层链 — 一次完整的递归改进路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerChain {
    pub layers: Vec<ImprovementLayer>,
    pub final_score: f64,
    pub converged: bool,
    pub total_depth: usize,
}

/// 改进上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementContext {
    pub current_scores: HashMap<String, f64>,
    pub recent_failures: Vec<String>,
    pub recent_successes: Vec<String>,
    pub system_health: f64,
}

/// 改进规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementRule {
    pub trigger: String,
    pub action: String,
    pub expected_delta: f64,
}

/// 递归自改进器 — Metaⁿ 启发
///
/// 核心思想: 每一层改进提案被"递归输入"到下一层，
/// 通过Ω元操作不断精化，直到改进收敛或达到深度上限。
pub struct RecursiveSelfImprover {
    pub archive: Vec<LayerChain>,
    pub current_depth: usize,
    pub max_depth: usize,
    pub convergence_threshold: f64,
    pub improvement_rules: Vec<ImprovementRule>,
}

impl RecursiveSelfImprover {
    pub fn new() -> Self {
        let improvement_rules = vec![
            ImprovementRule {
                trigger: "low_memory_score".into(),
                action: "optimize_encoding_and_retrieval".into(),
                expected_delta: 0.05,
            },
            ImprovementRule {
                trigger: "low_reasoning_score".into(),
                action: "enhance_causal_chain_depth".into(),
                expected_delta: 0.08,
            },
            ImprovementRule {
                trigger: "low_safety_score".into(),
                action: "add_constraint_validation".into(),
                expected_delta: 0.1,
            },
            ImprovementRule {
                trigger: "recent_failure_pattern".into(),
                action: "inject_failure_awareness".into(),
                expected_delta: 0.06,
            },
            ImprovementRule {
                trigger: "architecture_drift".into(),
                action: "rebalance_module_weights".into(),
                expected_delta: 0.04,
            },
        ];

        Self {
            archive: Vec::new(),
            current_depth: 0,
            max_depth: 5,
            convergence_threshold: 0.01,
            improvement_rules,
        }
    }

    /// Ω 操作: 固定的元操作，递归输入
    ///
    /// 对当前上下文应用元认知操作:
    /// 1. 分析当前评分差距
    /// 2. 匹配改进规则
    /// 3. 生成下一层改进提案
    pub fn omega(&self, context: &ImprovementContext) -> ImprovementProposal {
        // 找到最低分的模块
        let weakest = context
            .current_scores
            .iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, v)| (k.clone(), *v))
            .unwrap_or_else(|| ("unknown".into(), 0.0));

        // 找到匹配的改进规则
        let matching_rule = self
            .improvement_rules
            .iter()
            .find(|r| {
                let trigger_matches = match weakest.0.as_str() {
                    "memory" => r.trigger == "low_memory_score",
                    "reasoning" => r.trigger == "low_reasoning_score",
                    "safety" => r.trigger == "low_safety_score",
                    _ => r.trigger == "architecture_drift",
                };
                trigger_matches || r.trigger == "recent_failure_pattern"
            })
            .cloned()
            .unwrap_or_else(|| ImprovementRule {
                trigger: "default".into(),
                action: "general_optimization".into(),
                expected_delta: 0.03,
            });

        // 根据失败/成功模式调整
        let context_modifier = if !context.recent_failures.is_empty() {
            1.2 // 有失败经验，加大改进力度
        } else if !context.recent_successes.is_empty() {
            0.8 // 有成功经验，保守改进
        } else {
            1.0
        };

        // 健康度影响: 低健康度需要更激进的改进
        let health_modifier = if context.system_health < 0.5 {
            1.5
        } else if context.system_health > 0.8 {
            0.6
        } else {
            1.0
        };

        let score_delta =
            matching_rule.expected_delta * context_modifier * health_modifier;

        ImprovementProposal {
            target_module: weakest.0,
            improvement_type: match matching_rule.trigger.as_str() {
                "low_memory_score" => ImprovementType::MemoryOptimization,
                "low_reasoning_score" => ImprovementType::ReasoningEnhancement,
                "low_safety_score" => ImprovementType::SafetyStrengthening,
                "recent_failure_pattern" => ImprovementType::PerformanceTuning,
                _ => ImprovementType::ArchitectureEvolution,
            },
            description: matching_rule.action,
            pre_process: format!(
                "analyze_context(health={:.2}, failures={}, successes={})",
                context.system_health,
                context.recent_failures.len(),
                context.recent_successes.len()
            ),
            helpers: vec![
                "pattern_match".into(),
                "delta_estimation".into(),
                "safety_check".into(),
            ],
            score_delta: score_delta.clamp(-0.5, 0.5),
            confidence: (0.5 + context.system_health * 0.3 + 0.2 * (1.0 / (self.archive.len() as f64 + 1.0))).clamp(0.0, 1.0),
        }
    }

    /// 递归改进核心循环
    ///
    /// 每次迭代:
    /// 1. 应用Ω操作生成提案
    /// 2. 将提案作为下一层输入摘要
    /// 3. 检查收敛
    pub fn improve(&mut self, context: &ImprovementContext) -> &LayerChain {
        let mut layers: Vec<ImprovementLayer> = Vec::new();
        let mut current_summary = format!(
            "initial_context(health={:.2}, modules={})",
            context.system_health,
            context.current_scores.len()
        );

        let mut accumulated_score = 0.0;
        let mut converged = false;

        for depth in 0..self.max_depth {
            let proposal = self.omega(context);

            let layer = ImprovementLayer {
                input_summary: current_summary.clone(),
                pre_process: proposal.pre_process.clone(),
                helpers: proposal.helpers.clone(),
                output: proposal.clone(),
                depth,
            };

            accumulated_score += proposal.score_delta;
            layers.push(layer);

            // 检查收敛: 改进量小于阈值
            if proposal.score_delta.abs() < self.convergence_threshold {
                converged = true;
                break;
            }

            // 下一层输入: 当前提案的摘要
            current_summary = format!(
                "depth_{}: {} → {} (delta={:.4})",
                depth,
                proposal.target_module,
                proposal.description,
                proposal.score_delta
            );
        }

        let total_depth = layers.len();
        let chain = LayerChain {
            layers,
            final_score: accumulated_score,
            converged,
            total_depth,
        };

        self.archive.push(chain);
        self.archive.last().unwrap()
    }

    /// 检查层链是否收敛
    pub fn is_converged(&self, chain: &LayerChain) -> bool {
        if chain.layers.is_empty() {
            return true;
        }
        // 收敛条件: 达到最大深度或最后一层改进量极小
        chain.converged
            || chain.layers.last().map_or(true, |l| {
                l.output.score_delta.abs() < self.convergence_threshold
            })
    }

    /// 从历史中学习最优层链
    pub fn learn_from_archive(&self) -> Option<&LayerChain> {
        if self.archive.is_empty() {
            return None;
        }
        self.archive
            .iter()
            .max_by(|a, b| {
                a.final_score
                    .partial_cmp(&b.final_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// 获取当前最优改进提案
    pub fn best_proposal(&self) -> Option<&ImprovementProposal> {
        self.archive
            .iter()
            .flat_map(|chain| &chain.layers)
            .max_by(|a, b| {
                a.output
                    .score_delta
                    .partial_cmp(&b.output.score_delta)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|layer| &layer.output)
    }

    /// 记录改进结果
    pub fn record_result(&mut self, chain: LayerChain, actual_delta: f64) {
        // 根据实际效果更新规则的期望delta
        for rule in &mut self.improvement_rules {
            if chain.layers.iter().any(|l| l.output.description == rule.action) {
                // 指数移动平均: EMA = 0.7 * old + 0.3 * new
                rule.expected_delta =
                    0.7 * rule.expected_delta + 0.3 * actual_delta;
            }
        }

        // 保存到归档
        self.archive.push(chain);

        // 限制归档大小
        if self.archive.len() > 100 {
            self.archive.drain(0..50);
        }
    }

    /// 获取改进统计摘要
    pub fn summary(&self) -> ImprovementSummary {
        let total_chains = self.archive.len();
        let converged_chains = self.archive.iter().filter(|c| c.converged).count();
        let avg_depth = if total_chains > 0 {
            self.archive.iter().map(|c| c.total_depth).sum::<usize>() as f64
                / total_chains as f64
        } else {
            0.0
        };
        let avg_score = if total_chains > 0 {
            self.archive.iter().map(|c| c.final_score).sum::<f64>()
                / total_chains as f64
        } else {
            0.0
        };

        ImprovementSummary {
            total_chains,
            converged_chains,
            avg_depth,
            avg_score,
            max_depth: self.max_depth,
            convergence_threshold: self.convergence_threshold,
        }
    }
}

impl Default for RecursiveSelfImprover {
    fn default() -> Self {
        Self::new()
    }
}

/// 改进统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementSummary {
    pub total_chains: usize,
    pub converged_chains: usize,
    pub avg_depth: f64,
    pub avg_score: f64,
    pub max_depth: usize,
    pub convergence_threshold: f64,
}
