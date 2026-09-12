use super::capability::CapabilityVector;
use super::knowledge_source::AffectiveFeedback;

// RewardSource re-exported via knowledge_source.rs (core::knowledge).
// Removed local definition to avoid conflict with knowledge_source re-export.

pub struct PerformanceEvaluator;

// CUDA Agent 吸收接线 (cycle 1188): 外部执行反馈奖励信号。
// 对标 CUDA Agent 的 skill-augmented env + 自动验证/profiling 提供可靠奖励信号:
// 静态能力评估 (evaluate) 无经验反馈, RL 需要「执行后验证」信号。
// _combine_reward: 外部执行反馈 (verification/profiling, RewardSource::External)
// 与内部能力自评 (RewardSource::Internal) 加权融合 — external_weight 越高,
// 奖励越接地于真实执行结果 (CUDA Agent 核心主张: 自动验证 → 稳定 RL 训练)。
#[derive(Debug, Clone, Copy)]
pub struct ExecutionFeedback {
    pub verified: bool,        // 产物通过自动验证 (编译/测试/profiling gate)
    pub latency_ratio: f64,    // 相对基线延迟比 (0.5=快 2x, 1.0=持平, >1=更慢)
    pub quality: f64,          // 质量分 0..1 (如 benchmark 命中率)
}

impl ExecutionFeedback {
    pub fn new(verified: bool, latency_ratio: f64, quality: f64) -> Self {
        Self { verified, latency_ratio, quality }
    }
}

impl PerformanceEvaluator {
    pub fn evaluate(task_type: &crate::neotrix::nt_world_model::TaskType, capability: &CapabilityVector) -> f64 {
        let raw_score = match task_type {
            crate::neotrix::nt_world_model::TaskType::Design |
            crate::neotrix::nt_world_model::TaskType::UIDesign => {
                
                (capability.accessibility() * 0.2
                    + capability.compound_composition() * 0.2
                    + capability.tailwind_proficiency() * 0.15
                    + capability.react_aria_usage() * 0.15
                    + capability.figma_integration() * 0.1
                    + capability.ai_native_states() * 0.1
                    + capability.semantic_layer() * 0.1).min(1.0)
            }
            crate::neotrix::nt_world_model::TaskType::CodeAnalysis |
            crate::neotrix::nt_world_model::TaskType::CodeGeneration |
            crate::neotrix::nt_world_model::TaskType::CodeReview => {
                
                (capability.analysis() * 0.3
                    + capability.synthesis() * 0.3
                    + capability.inference_depth() * 0.2
                    + capability.creativity() * 0.2).min(1.0)
            }
            crate::neotrix::nt_world_model::TaskType::Security => {
                (capability.analysis() * 0.4
                    + capability.verification() * 0.3
                    + capability.quality_gates() * 0.3).min(1.0)
            }
            crate::neotrix::nt_world_model::TaskType::Planning => {
                (capability.inference_depth() * 0.4
                    + capability.synthesis() * 0.3
                    + capability.analysis() * 0.3).min(1.0)
            }
            _ => 0.5,
        };
        raw_score.clamp(0.0, 1.0)
    }

    pub(crate) fn _has_meaningful_change(before: f64, after: f64, threshold: f64) -> bool {
        (after - before).abs() > threshold
    }

    /// 确定性选择器 (all-agentic-architectures 吸收): 逃逸 LLM-as-Scorer flat-band 病。
    /// 纪律: 任何 LLM 打分面必须让 LLM 提交分类特征 (bool/enum), 而非连续浮点分 —
    /// 连续分总是坍缩到 ~0.7 中带 (flat-band), 无法区分信号; 分类承诺 + 代码组合
    /// 决定性信号, 信号不再饱和。
    /// - `commits`: LLM 对一组分类特征提交的 bool 承诺 (每特征一票)。
    /// - `weights`: 与 commits 对齐的票权, 长度需一致。
    /// - `threshold`: 加权支持率门限, ≥ 则采纳 (决定输出, 非概率)。
    ///
    /// 返回 (采纳, 支持率, 反对率): 决定性 bool + 可解释的边际。
    pub(crate) fn _deterministic_pick(
        commits: &[bool],
        weights: &[f64],
        threshold: f64,
    ) -> (bool, f64, f64) {
        debug_assert_eq!(commits.len(), weights.len(), "commits/weights 长度必须一致");
        if commits.is_empty() {
            return (false, 0.0, 0.0);
        }
        let total: f64 = weights.iter().map(|w| w.max(0.0)).sum();
        if total <= 0.0 {
            return (false, 0.0, 0.0);
        }
        let mut support = 0.0;
        for (c, w) in commits.iter().zip(weights.iter()) {
            if *c {
                support += w.max(0.0);
            }
        }
        let support_ratio = support / total;
        let oppose_ratio = 1.0 - support_ratio;
        (support_ratio >= threshold, support_ratio, oppose_ratio)
    }

    /// 融合外部执行反馈与内部能力评估 (CUDA Agent 奖励信号接线)。
    /// - external_weight: 0..1, 奖励中有多少比例来自外部验证/执行反馈。
    ///   为 0 → 纯内部自评 (无验证工具时的退化); 为 1 → 纯外部执行信号。
    /// - 外部信号: 验证通过 + 延迟比 + 质量分 合成的可执行奖励。
    /// - 内部信号: capability 静态评估 (evaluate)。
    pub(crate) fn _combine_reward(
        capability_score: f64,
        feedback: ExecutionFeedback,
        external_weight: f64,
    ) -> f64 {
        let w = external_weight.clamp(0.0, 1.0);
        let latency_bonus = (1.0 / feedback.latency_ratio.max(0.05)).clamp(0.5, 2.0);
        // 外部执行奖励: 未验证通过 → 惩罚 (cap 0.3, 防止未验证产物获得高奖励);
        // 验证通过 → 质量分 × 延迟奖励。
        let external_score = if !feedback.verified {
            feedback.quality * 0.3
        } else {
            (feedback.quality * latency_bonus).min(1.0)
        };
        let internal_score = capability_score.clamp(0.0, 1.0);
        let combined = external_score * w + internal_score * (1.0 - w);
        combined.clamp(0.0, 1.0)
    }

    /// _combine_reward 的情感扩展 (Q2, 设计: docs/1-DESIGN/affective-reward-context.md):
    /// external 通道内部再细分 — 执行验证信号 (verified/latency/quality) 与
    /// 情感引导信号 (AffectiveFeedback)。`affective == None` 时退化为 _combine_reward。
    ///
    /// 合成 (确定性, 防 LLM 打分 flat-band):
    /// - `affective_guide = valence * trust_k * engagement_k`
    ///   - `trust_k = 0.6 + 0.1 * stage` (Stranger 0.6 → Bond 1.0, 高信任加权)
    ///   - `engagement_k = if interactions >= 3 { 1.0 } else { 0.0 }` (冷启动抑制)
    ///   - `arousal` 修正: `valence < 0.5 && arousal > 0.5` (挫败) → guide *= 0.5
    /// - `signal_weight = min(signal_weight, 0.3)` — 情感幅度上限 30%
    /// - `combined = external_execution * (1 - sw) + guide * sw`, 再与内部自评加权
    pub fn combine_reward_with_affective(
        capability_score: f64,
        feedback: ExecutionFeedback,
        affective: Option<AffectiveFeedback>,
        external_weight: f64,
    ) -> f64 {
        let w = external_weight.clamp(0.0, 1.0);
        let latency_bonus = (1.0 / feedback.latency_ratio.max(0.05)).clamp(0.5, 2.0);
        let external_score = if !feedback.verified {
            feedback.quality * 0.3
        } else {
            (feedback.quality * latency_bonus).min(1.0)
        };
        let Some(a) = affective else {
            let internal_score = capability_score.clamp(0.0, 1.0);
            let combined = external_score * w + internal_score * (1.0 - w);
            return combined.clamp(0.0, 1.0);
        };
        let valence = a.valence.clamp(0.0, 1.0);
        let arousal = a.arousal.clamp(0.0, 1.0);
        let trust_k = (0.6 + 0.1 * a.stage.clamp(0, 4) as f64).clamp(0.6, 1.0);
        let engagement_k = if a.interactions >= 3 { 1.0 } else { 0.0 };
        let mut guide = valence * trust_k * engagement_k;
        if valence < 0.5 && arousal > 0.5 {
            guide *= 0.5;
        }
        let sw = if engagement_k == 0.0 {
            0.0
        } else {
            a.signal_weight.clamp(0.0, 1.0).min(0.3)
        };
        let external_guided = external_score * (1.0 - sw) + guide * sw;
        let internal_score = capability_score.clamp(0.0, 1.0);
        let combined = external_guided * w + internal_score * (1.0 - w);
        combined.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_world_model::TaskType;

    fn design_capability() -> CapabilityVector {
        CapabilityVector::from_values(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
            0.5, 0.5, 0.5, 0.5, 0.5,
            0.9, 0.9, 0.8, 0.7,
            0.5, 0.8, 0.8, 0.7, 0.5, 0.5,
        )
    }

    fn code_capability() -> CapabilityVector {
        CapabilityVector::from_values(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
            0.8, 0.7, 0.9, 0.9, 0.5,
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
        )
    }

    fn nt_shield_capability() -> CapabilityVector {
        CapabilityVector::from_values(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
            0.5, 0.5, 0.9, 0.5, 0.5,
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.8, 0.8,
        )
    }

    #[test]
    fn test_evaluate_design_scores_high_with_design_skills() {
        let cap = design_capability();
        let score = PerformanceEvaluator::evaluate(&TaskType::Design, &cap);
        assert!(score > 0.5);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_evaluate_ui_design_scores_high_with_design_skills() {
        let cap = design_capability();
        let score = PerformanceEvaluator::evaluate(&TaskType::UIDesign, &cap);
        assert!(score > 0.5);
    }

    #[test]
    fn test_evaluate_code_analysis_scores_high_with_code_skills() {
        let cap = code_capability();
        let score = PerformanceEvaluator::evaluate(&TaskType::CodeAnalysis, &cap);
        assert!(score > 0.5);
    }

    #[test]
    fn test_evaluate_code_generation_scores_high_with_code_skills() {
        let cap = code_capability();
        let score = PerformanceEvaluator::evaluate(&TaskType::CodeGeneration, &cap);
        assert!(score > 0.5);
    }

    #[test]
    fn test_evaluate_code_review_scores_high_with_code_skills() {
        let cap = code_capability();
        let score = PerformanceEvaluator::evaluate(&TaskType::CodeReview, &cap);
        assert!(score > 0.5);
    }

    #[test]
    fn test_evaluate_nt_shield_scores_high_with_nt_shield_skills() {
        let cap = nt_shield_capability();
        let score = PerformanceEvaluator::evaluate(&TaskType::Security, &cap);
        assert!(score > 0.5);
    }

    #[test]
    fn test_evaluate_planning_uses_inference_synthesis_analysis() {
        let cap = design_capability();
        let score = PerformanceEvaluator::evaluate(&TaskType::Planning, &cap);
        assert!(score >= 0.0);
    }

    #[test]
    fn test_evaluate_fallback_to_half() {
        let cap = CapabilityVector::default();
        let score = PerformanceEvaluator::evaluate(&TaskType::General, &cap);
        assert!((score - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_evaluate_clamps_output() {
        let mut cap = CapabilityVector::default();
        cap.set_analysis(10.0);
        let score = PerformanceEvaluator::evaluate(&TaskType::CodeAnalysis, &cap);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_evaluate_returns_zero_for_zero_capability() {
        let cap = CapabilityVector::default();
        let score = PerformanceEvaluator::evaluate(&TaskType::Design, &cap);
        assert!((score - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_evaluate_research_fallback() {
        let cap = CapabilityVector::default();
        let score = PerformanceEvaluator::evaluate(&TaskType::Research, &cap);
        assert!((score - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_has_meaningful_change_above_threshold() {
        assert!(PerformanceEvaluator::_has_meaningful_change(0.3, 0.8, 0.1));
    }

    #[test]
    fn test_has_meaningful_change_below_threshold() {
        assert!(!PerformanceEvaluator::_has_meaningful_change(0.45, 0.5, 0.1));
    }

    #[test]
    fn test_has_meaningful_change_equal_value() {
        assert!(!PerformanceEvaluator::_has_meaningful_change(0.5, 0.5, 0.01));
    }

    #[test]
    fn test_has_meaningful_change_negative_threshold() {
        assert!(PerformanceEvaluator::_has_meaningful_change(0.3, 0.8, -0.1));
    }

    #[test]
    fn test_has_meaningful_change_exact_threshold() {
        assert!(!PerformanceEvaluator::_has_meaningful_change(0.5, 0.6, 0.1));
    }

    // ========== CUDA Agent 接线测试 (cycle 1188: 外部执行反馈奖励) ==========

    #[test]
    fn test_combine_reward_verified_high_quality() {
        // 验证通过 + 高质量 → 外部权重越高奖励越高
        let feedback = ExecutionFeedback::new(true, 1.0, 0.9);
        let pure_internal = PerformanceEvaluator::_combine_reward(0.5, feedback, 0.0);
        let grounded = PerformanceEvaluator::_combine_reward(0.5, feedback, 1.0);
        assert!(grounded > pure_internal, "验证通过的高质量产物应获更高奖励");
        assert!((pure_internal - 0.5).abs() < 1e-9, "external_weight=0 应退化为内部自评");
    }

    #[test]
    fn test_combine_reward_unverified_penalized() {
        // 未验证通过 → 外部奖励被惩罚 (cap 0.3), 不应超过内部自评
        let feedback = ExecutionFeedback::new(false, 1.0, 0.9);
        let grounded = PerformanceEvaluator::_combine_reward(0.5, feedback, 1.0);
        assert!(grounded < 0.5, "未验证产物应受惩罚: {}", grounded);
        assert!(grounded <= 0.3, "未验证奖励 cap 0.3: {}", grounded);
    }

    #[test]
    fn test_combine_reward_latency_bonus() {
        // 快 2x (latency_ratio=0.5) 应比持平 (1.0) 奖励高
        let fast = ExecutionFeedback::new(true, 0.5, 0.8);
        let equal = ExecutionFeedback::new(true, 1.0, 0.8);
        let fast_r = PerformanceEvaluator::_combine_reward(0.5, fast, 1.0);
        let equal_r = PerformanceEvaluator::_combine_reward(0.5, equal, 1.0);
        assert!(fast_r > equal_r, "更快应获更高奖励: fast={} equal={}", fast_r, equal_r);
    }

    #[test]
    fn test_combine_reward_clamped() {
        let feedback = ExecutionFeedback::new(true, 0.05, 1.0); // latency_bonus 封顶 2.0
        let r = PerformanceEvaluator::_combine_reward(0.5, feedback, 1.0);
        assert!(r <= 1.0);
        assert!(r >= 0.0);
    }

    #[test]
    fn test_combine_reward_external_weight_blend() {
        // 外部权重 0.5 → 结果落在内部与外部之间
        let feedback = ExecutionFeedback::new(true, 1.0, 0.6);
        let internal_only = PerformanceEvaluator::_combine_reward(0.5, feedback, 0.0);
        let external_only = PerformanceEvaluator::_combine_reward(0.5, feedback, 1.0);
        let blended = PerformanceEvaluator::_combine_reward(0.5, feedback, 0.5);
        assert!((blended - (internal_only + external_only) / 2.0).abs() < 1e-9);
    }

    // ========== deterministic-picker 接线测试 (cycle 1191: all-agentic-architectures) ==========

    #[test]
    fn test_deterministic_pick_adopts_above_threshold() {
        // 2/3 分类承诺 → 支持率 0.667 ≥ 0.6 → 采纳
        let (adopt, support, oppose) = PerformanceEvaluator::_deterministic_pick(
            &[true, true, false],
            &[1.0, 1.0, 1.0],
            0.6,
        );
        assert!(adopt);
        assert!((support - 2.0 / 3.0).abs() < 1e-9);
        assert!((oppose - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_deterministic_pick_rejects_below_threshold() {
        let (adopt, support, _) = PerformanceEvaluator::_deterministic_pick(
            &[true, false, false],
            &[1.0, 1.0, 1.0],
            0.6,
        );
        assert!(!adopt);
        assert!((support - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_deterministic_pick_weighted_commit() {
        // 强票权 (3x) 的单票否决 2 票弱票: 3/(3+1+1)=0.6 ≥ 0.6 → 采纳
        let (adopt, support, _) = PerformanceEvaluator::_deterministic_pick(
            &[true, false, false],
            &[3.0, 1.0, 1.0],
            0.6,
        );
        assert!(adopt);
        assert!((support - 3.0 / 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_deterministic_pick_empty_is_reject() {
        let (adopt, support, oppose) = PerformanceEvaluator::_deterministic_pick(&[], &[], 0.5);
        assert!(!adopt);
        assert_eq!(support, 0.0);
        assert_eq!(oppose, 0.0);
    }

    #[test]
    fn test_deterministic_pick_zero_weight_is_reject() {
        let (adopt, _, _) = PerformanceEvaluator::_deterministic_pick(
            &[true, true],
            &[0.0, 0.0],
            0.0,
        );
        assert!(!adopt);
    }

    #[test]
    fn test_deterministic_pick_negative_weights_ignored() {
        // 负票权被忽略 (不拉低支持率), 符合 max(0) 语义
        let (adopt, support, _) = PerformanceEvaluator::_deterministic_pick(
            &[true, false],
            &[1.0, -5.0],
            0.5,
        );
        assert!(adopt);
        assert_eq!(support, 1.0);
    }

    #[test]
    fn test_deterministic_pick_threshold_boundary() {
        // 支持率恰等于阈值 → 采纳 (≥ 语义)
        let (adopt, _, _) = PerformanceEvaluator::_deterministic_pick(
            &[true, false],
            &[1.0, 1.0],
            0.5,
        );
        assert!(adopt);
    }

    fn fb(verified: bool, quality: f64) -> ExecutionFeedback {
        ExecutionFeedback::new(verified, 1.0, quality)
    }

    fn aff(valence: f64, stage: u8, interactions: u32) -> AffectiveFeedback {
        AffectiveFeedback {
            valence,
            arousal: 0.3,
            stage,
            interactions,
            signal_weight: 0.3,
        }
    }

    #[test]
    fn test_combine_reward_with_affective_boost() {
        // 高 valence + Bond(stage=4) + interactions≥3 → 情感引导抬高外部通道奖励
        let base = PerformanceEvaluator::_combine_reward(0.5, fb(true, 0.7), 1.0);
        let boosted = PerformanceEvaluator::combine_reward_with_affective(
            0.5, fb(true, 0.7), Some(aff(0.9, 4, 10)), 1.0,
        );
        assert!(boosted > base, "boosted={boosted} base={base}");
    }

    #[test]
    fn test_combine_reward_with_affective_cold_start() {
        // interactions<3 → engagement_k=0 → guide=0 → 与无情感信号等价
        let no_aff = PerformanceEvaluator::combine_reward_with_affective(
            0.5, fb(true, 0.7), None, 1.0,
        );
        let cold = PerformanceEvaluator::combine_reward_with_affective(
            0.5, fb(true, 0.7), Some(aff(0.9, 4, 2)), 1.0,
        );
        assert!((no_aff - cold).abs() < 1e-9, "cold={cold} no_aff={no_aff}");
    }

    #[test]
    fn test_combine_reward_with_affective_frustration() {
        // 低 valence + 高 arousal (挫败) → guide 砍半
        let calm = AffectiveFeedback { arousal: 0.3, ..aff(0.3, 4, 10) };
        let frustrated = AffectiveFeedback { arousal: 0.9, ..aff(0.3, 4, 10) };
        let a = PerformanceEvaluator::combine_reward_with_affective(
            0.5, fb(true, 0.7), Some(calm), 1.0,
        );
        let b = PerformanceEvaluator::combine_reward_with_affective(
            0.5, fb(true, 0.7), Some(frustrated), 1.0,
        );
        assert!(b < a, "frustrated={b} calm={a}");
    }

    #[test]
    fn test_combine_reward_with_affective_cap_30() {
        // signal_weight=1.0 仍被 cap 至 0.3 — 情感只做引导不做裁判
        let maxed = AffectiveFeedback { signal_weight: 1.0, ..aff(1.0, 4, 10) };
        let capped = AffectiveFeedback { signal_weight: 0.3, ..aff(1.0, 4, 10) };
        let a = PerformanceEvaluator::combine_reward_with_affective(
            0.5, fb(true, 0.7), Some(maxed), 1.0,
        );
        let b = PerformanceEvaluator::combine_reward_with_affective(
            0.5, fb(true, 0.7), Some(capped), 1.0,
        );
        assert!((a - b).abs() < 1e-9, "cap violated: maxed={a} capped={b}");
    }

    #[test]
    fn test_combine_reward_with_affective_none_regression() {
        // None 时与 _combine_reward 逐位一致 (回归)
        let plain = PerformanceEvaluator::_combine_reward(0.5, fb(true, 0.7), 0.5);
        let with_none = PerformanceEvaluator::combine_reward_with_affective(
            0.5, fb(true, 0.7), None, 0.5,
        );
        assert!((plain - with_none).abs() < 1e-12);
    }
}
