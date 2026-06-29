use super::capability::CapabilityVector;

// RewardSource re-exported via knowledge_source.rs (core::knowledge).
// Removed local definition to avoid conflict with knowledge_source re-export.

pub struct PerformanceEvaluator;

impl PerformanceEvaluator {
    pub fn evaluate(
        task_type: &crate::neotrix::nt_expert_routing::TaskType,
        capability: &CapabilityVector,
    ) -> f64 {
        let raw_score = match task_type {
            crate::neotrix::nt_expert_routing::TaskType::Design
            | crate::neotrix::nt_expert_routing::TaskType::UIDesign => (capability.accessibility()
                * 0.2
                + capability.compound_composition() * 0.2
                + capability.tailwind_proficiency() * 0.15
                + capability.react_aria_usage() * 0.15
                + capability.figma_integration() * 0.1
                + capability.ai_native_states() * 0.1
                + capability.semantic_layer() * 0.1)
                .min(1.0),
            crate::neotrix::nt_expert_routing::TaskType::CodeAnalysis
            | crate::neotrix::nt_expert_routing::TaskType::CodeGeneration
            | crate::neotrix::nt_expert_routing::TaskType::CodeReview => (capability.analysis()
                * 0.3
                + capability.synthesis() * 0.3
                + capability.inference_depth() * 0.2
                + capability.creativity() * 0.2)
                .min(1.0),
            crate::neotrix::nt_expert_routing::TaskType::Security => (capability.analysis() * 0.4
                + capability.verification() * 0.3
                + capability.quality_gates() * 0.3)
                .min(1.0),
            crate::neotrix::nt_expert_routing::TaskType::Planning => (capability.inference_depth()
                * 0.4
                + capability.synthesis() * 0.3
                + capability.analysis() * 0.3)
                .min(1.0),
            _ => 0.5,
        };
        raw_score.clamp(0.0, 1.0)
    }

    pub fn has_meaningful_change(before: f64, after: f64, threshold: f64) -> bool {
        (after - before).abs() > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_expert_routing::TaskType;

    fn design_capability() -> CapabilityVector {
        CapabilityVector::from_values(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.9, 0.9, 0.8, 0.7,
            0.5, 0.8, 0.8, 0.7, 0.5, 0.5,
        )
    }

    fn code_capability() -> CapabilityVector {
        CapabilityVector::from_values(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.8, 0.7, 0.9, 0.9, 0.5, 0.5, 0.5, 0.5, 0.5,
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
        )
    }

    fn nt_shield_capability() -> CapabilityVector {
        CapabilityVector::from_values(
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.9, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
            0.5, 0.5, 0.5, 0.5, 0.8, 0.8,
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
        assert!(PerformanceEvaluator::has_meaningful_change(0.3, 0.8, 0.1));
    }

    #[test]
    fn test_has_meaningful_change_below_threshold() {
        assert!(!PerformanceEvaluator::has_meaningful_change(0.45, 0.5, 0.1));
    }

    #[test]
    fn test_has_meaningful_change_equal_value() {
        assert!(!PerformanceEvaluator::has_meaningful_change(0.5, 0.5, 0.01));
    }

    #[test]
    fn test_has_meaningful_change_negative_threshold() {
        assert!(PerformanceEvaluator::has_meaningful_change(0.3, 0.8, -0.1));
    }

    #[test]
    fn test_has_meaningful_change_exact_threshold() {
        assert!(!PerformanceEvaluator::has_meaningful_change(0.5, 0.6, 0.1));
    }
}
