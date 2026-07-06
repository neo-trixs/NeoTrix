use std::collections::HashMap;

use crate::core::nt_core_prm::AgentTrajectory;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearningTipType {
    Strategy,
    Recovery,
    Optimization,
}

#[derive(Debug, Clone)]
pub struct LearningTip {
    pub tip_type: LearningTipType,
    pub source_step_idx: usize,
    pub pattern: String,
    pub recommendation: String,
    pub confidence: f64,
    pub provenance: String,
}

#[derive(Debug, Clone)]
pub struct LearningReport {
    pub tips: Vec<LearningTip>,
    pub trajectory_id: String,
    pub total_steps: usize,
    pub success: bool,
}

pub struct TrajectoryLearner;

impl TrajectoryLearner {
    pub fn analyze(trajectory: &AgentTrajectory) -> LearningReport {
        let mut tips = Vec::new();
        let success = trajectory.completed;
        tips.extend(Self::extract_strategy_tips(trajectory));
        tips.extend(Self::extract_recovery_tips(trajectory));
        tips.extend(Self::extract_optimization_tips(trajectory));
        LearningReport {
            tips,
            trajectory_id: format!("traj-{}", trajectory.trajectory_id),
            total_steps: trajectory.steps.len(),
            success,
        }
    }

    fn extract_strategy_tips(trajectory: &AgentTrajectory) -> Vec<LearningTip> {
        let mut tips = Vec::new();
        for (i, w) in trajectory.steps.windows(2).enumerate() {
            if w[0].success && w[1].success {
                let pattern = format!("{:?}->{:?} transition", w[0].specialist, w[1].specialist);
                tips.push(LearningTip {
                    tip_type: LearningTipType::Strategy,
                    source_step_idx: i,
                    pattern,
                    recommendation: format!("Use {} after {} for similar tasks", w[1].action, w[0].action),
                    confidence: 0.7,
                    provenance: format!("step {}->{}", w[0].step_idx, w[1].step_idx),
                });
            }
        }
        tips
    }

    fn extract_recovery_tips(trajectory: &AgentTrajectory) -> Vec<LearningTip> {
        let mut tips = Vec::new();
        for (i, w) in trajectory.steps.windows(2).enumerate() {
            if !w[0].success && w[1].success {
                tips.push(LearningTip {
                    tip_type: LearningTipType::Recovery,
                    source_step_idx: i,
                    pattern: format!("Recovered from {} failure via {}", w[0].action, w[1].action),
                    recommendation: format!("When {} fails, try {} instead", w[0].action, w[1].action),
                    confidence: 0.8,
                    provenance: format!("recovery step {}->{}", w[0].step_idx, w[1].step_idx),
                });
            }
        }
        tips
    }

    fn extract_optimization_tips(trajectory: &AgentTrajectory) -> Vec<LearningTip> {
        let mut tips = Vec::new();
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for step in &trajectory.steps {
            *counts.entry(&step.action).or_insert(0) += 1;
        }
        for (action, count) in &counts {
            if *count > 1 {
                tips.push(LearningTip {
                    tip_type: LearningTipType::Optimization,
                    source_step_idx: 0,
                    pattern: format!("{} repeated {} times", action, count),
                    recommendation: format!("Cache or deduplicate {} calls", action),
                    confidence: 0.6,
                    provenance: format!("duplicate: {} x{}", action, count),
                });
            }
        }
        tips
    }

    pub fn generate_skill_doc(trajectory: &AgentTrajectory, tips: &[LearningTip]) -> String {
        let mut content = String::from("# Learned Skill\n\n");
        content.push_str(&format!("## Source\nTrajectory: {}\n\n", trajectory.trajectory_id));
        if tips.is_empty() {
            content.push_str("No actionable learnings extracted.\n");
            return content;
        }
        content.push_str("## Strategy Tips\n");
        for tip in tips.iter().filter(|t| t.tip_type == LearningTipType::Strategy) {
            content.push_str(&format!("- {}: {}\n", tip.pattern, tip.recommendation));
        }
        content.push_str("\n## Recovery Tips\n");
        for tip in tips.iter().filter(|t| t.tip_type == LearningTipType::Recovery) {
            content.push_str(&format!("- {}: {}\n", tip.pattern, tip.recommendation));
        }
        content.push_str("\n## Optimization Tips\n");
        for tip in tips.iter().filter(|t| t.tip_type == LearningTipType::Optimization) {
            content.push_str(&format!("- {}: {}\n", tip.pattern, tip.recommendation));
        }
        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hex::ReasoningHexagram;
    use crate::core::nt_core_traits::SpecialistType;
    use crate::core::nt_core_prm::TrajectoryStep;

    fn mk_step(idx: usize, sp: SpecialistType, mode: u8, inp: &str, out: &str, ok: bool, r: Option<f64>) -> TrajectoryStep {
        TrajectoryStep {
            step_idx: idx, specialist: sp,
            e8_mode: ReasoningHexagram::new(mode),
            action: "act".into(), input: inp.into(), output: out.into(),
            duration_ms: Some(100), success: ok, external_reward: r,
        }
    }

    #[test]
    fn test_empty_trajectory_gives_empty_tips() {
        let t = AgentTrajectory::new(1, "empty".into());
        let report = TrajectoryLearner::analyze(&t);
        assert!(report.tips.is_empty());
        assert!(!report.success);
    }

    #[test]
    fn test_strategy_tip_on_consecutive_success() {
        let mut t = AgentTrajectory::new(1, "t".into());
        t.push(mk_step(0, SpecialistType::PatternMatcher, 1, "a", "b", true, None));
        t.push(mk_step(1, SpecialistType::AnomalyDetector, 1, "c", "d", true, None));
        let report = TrajectoryLearner::analyze(&t);
        let s: Vec<_> = report.tips.iter().filter(|t| t.tip_type == LearningTipType::Strategy).collect();
        assert!(!s.is_empty(), "Should have strategy tips");
    }

    #[test]
    fn test_recovery_tip_on_failure_then_success() {
        let mut t = AgentTrajectory::new(1, "t".into());
        t.push(mk_step(0, SpecialistType::Planner, 1, "x", "y", false, None));
        t.push(mk_step(1, SpecialistType::Planner, 1, "z", "w", true, None));
        let report = TrajectoryLearner::analyze(&t);
        let r: Vec<_> = report.tips.iter().filter(|t| t.tip_type == LearningTipType::Recovery).collect();
        assert!(!r.is_empty(), "Should have recovery tips");
    }

    #[test]
    fn test_optimization_tip_on_duplicate_action() {
        let mut t = AgentTrajectory::new(1, "t".into());
        t.push(mk_step(0, SpecialistType::PatternMatcher, 1, "a", "b", true, None));
        t.push(mk_step(1, SpecialistType::PatternMatcher, 1, "c", "d", true, None));
        let report = TrajectoryLearner::analyze(&t);
        let o: Vec<_> = report.tips.iter().filter(|t| t.tip_type == LearningTipType::Optimization).collect();
        assert!(!o.is_empty(), "Should have optimization tips");
    }

    #[test]
    fn test_generate_skill_doc_includes_tips() {
        let mut t = AgentTrajectory::new(1, "t".into());
        t.push(mk_step(0, SpecialistType::PatternMatcher, 1, "a", "b", true, None));
        t.push(mk_step(1, SpecialistType::AnomalyDetector, 1, "c", "d", true, None));
        let report = TrajectoryLearner::analyze(&t);
        let doc = TrajectoryLearner::generate_skill_doc(&t, &report.tips);
        assert!(doc.contains("Strategy Tips"));
        assert!(doc.contains("Recovery Tips"));
        assert!(doc.contains("Optimization Tips"));
    }

    #[test]
    fn test_empty_trajectory_generates_no_skill_doc() {
        let t = AgentTrajectory::new(1, "empty".into());
        let report = TrajectoryLearner::analyze(&t);
        let doc = TrajectoryLearner::generate_skill_doc(&t, &report.tips);
        assert!(doc.contains("No actionable learnings"));
    }
}
